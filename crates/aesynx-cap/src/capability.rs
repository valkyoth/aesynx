use core::fmt;

use aesynx_abi::{CapId, ObjectId, PrincipalId, VirtAddr};

use crate::{
    CapGeneration, CapIdError, CapIdParts, CapKind, CapPerms, CapSlotIndex, CapValidationError,
    DeriveError, DeriveRequest,
};

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct CapAuditEvent {
    pub action: CapAuditAction,
    pub object_id: ObjectId,
    pub source_owner: PrincipalId,
    pub target_owner: PrincipalId,
    pub perms: CapPerms,
    pub generation: u32,
    pub revocation_epoch: u64,
    pub affected_slots: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapAuditAction {
    Derive,
    Grant,
    Mint,
    Revoke,
}

impl CapAuditAction {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Derive => "derive",
            Self::Grant => "grant",
            Self::Mint => "mint",
            Self::Revoke => "revoke",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapAuditError {
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RedactedCapAuditEvent {
    pub action: CapAuditAction,
    pub permission_bits: u32,
    pub cross_owner: bool,
    pub affected_slots: u32,
}

/// Audit sink for capability authority movement.
///
/// Implementations must not reenter the capability table they are auditing.
/// Table mutation paths validate commit feasibility before calling `record`,
/// but the actual mutation may happen after the audit event is accepted.
pub trait CapAuditLog {
    fn record(&mut self, event: CapAuditEvent) -> Result<(), CapAuditError>;
}

impl CapAuditEvent {
    #[must_use]
    pub const fn redacted(self) -> RedactedCapAuditEvent {
        RedactedCapAuditEvent {
            action: self.action,
            permission_bits: self.perms.bits(),
            cross_owner: self.source_owner.get() != self.target_owner.get(),
            affected_slots: self.affected_slots,
        }
    }
}

impl fmt::Debug for CapAuditEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CapAuditEvent")
            .field("redacted", &self.redacted())
            .finish_non_exhaustive()
    }
}

/// Capability token for one object and owner.
///
/// The `generation` field is part of live-token validation. Object stores that
/// create or recycle object generations must fail instead of wrapping the
/// `u32` generation counter; wrapped generations can let stale capabilities
/// pass `validate_live`. The current `aesynx-object` registry enforces that
/// with checked generation increments. If object-generation state becomes a
/// public capability-layer input, it must move behind a checked non-wrapping
/// newtype instead of accepting raw integers from untrusted callers.
///
/// `base = None` and `len = None` denotes unscoped whole-object authority. A
/// bounded derivation request must carry an `ObjectBoundedRange`, which is the
/// type-level handoff that an object, memory, or syscall layer already checked
/// the requested range against the real backing extent before the cap layer
/// reduces authority.
#[derive(Eq, PartialEq)]
pub struct Capability {
    object_id: ObjectId,
    base: Option<VirtAddr>,
    len: Option<u64>,
    perms: CapPerms,
    owner: PrincipalId,
    generation: u32,
    revocation_epoch: u64,
    kind: CapKind,
}

impl Capability {
    pub(crate) const fn new_root(
        object_id: ObjectId,
        kind: CapKind,
        owner: PrincipalId,
        perms: CapPerms,
        generation: u32,
        revocation_epoch: u64,
    ) -> Result<Self, CapIdError> {
        match CapGeneration::new(generation) {
            Ok(_) => {}
            Err(error) => return Err(error),
        }

        Ok(Self {
            object_id,
            base: None,
            len: None,
            perms,
            owner,
            generation,
            revocation_epoch,
            kind,
        })
    }

    #[must_use]
    pub const fn object_id(&self) -> ObjectId {
        self.object_id
    }

    #[must_use]
    pub const fn base(&self) -> Option<VirtAddr> {
        self.base
    }

    #[must_use]
    pub const fn range_len(&self) -> Option<u64> {
        self.len
    }

    #[must_use]
    pub const fn has_bounded_range(&self) -> bool {
        matches!((self.base, self.len), (Some(_), Some(_)))
    }

    #[must_use]
    pub const fn perms(&self) -> CapPerms {
        self.perms
    }

    #[must_use]
    pub const fn owner(&self) -> PrincipalId {
        self.owner
    }

    /// Returns the object generation this capability was minted against.
    ///
    /// The authority that owns object-generation state must prevent `u32`
    /// wraparound before minting capabilities for a reused object identity.
    #[must_use]
    pub const fn generation(&self) -> u32 {
        self.generation
    }

    #[must_use]
    pub const fn revocation_epoch(&self) -> u64 {
        self.revocation_epoch
    }

    #[must_use]
    pub const fn kind(&self) -> CapKind {
        self.kind
    }

    pub const fn id_for_slot(&self, slot: CapSlotIndex) -> Result<CapId, CapIdError> {
        let generation = match CapGeneration::new(self.generation) {
            Ok(generation) => generation,
            Err(error) => return Err(error),
        };

        Ok(CapIdParts::new(slot, generation).cap_id())
    }

    #[must_use]
    pub const fn allows(&self, required: CapPerms) -> bool {
        self.perms.contains(required)
    }

    #[must_use]
    pub const fn matches_revocation_epoch(&self, current_epoch: u64) -> bool {
        self.revocation_epoch == current_epoch
    }

    pub const fn validate_live(
        &self,
        current_generation: u32,
        current_epoch: u64,
    ) -> Result<(), CapValidationError> {
        if self.generation != current_generation {
            return Err(CapValidationError::StaleGeneration);
        }

        if self.revocation_epoch != current_epoch {
            return Err(CapValidationError::Revoked);
        }

        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn derive_with_audit(
        self,
        request: DeriveRequest,
        audit: &mut impl CapAuditLog,
    ) -> Result<Self, DeriveError> {
        self.derive_prevalidated_with_audit(request, audit)
    }

    pub fn derive_live_with_audit(
        &self,
        request: DeriveRequest,
        current_generation: u32,
        current_epoch: u64,
        audit: &mut impl CapAuditLog,
    ) -> Result<Self, DeriveError> {
        self.validate_live_for_derivation(current_generation, current_epoch)?;
        self.derive_prevalidated_with_audit(request, audit)
    }

    fn derive_prevalidated_with_audit(
        &self,
        request: DeriveRequest,
        audit: &mut impl CapAuditLog,
    ) -> Result<Self, DeriveError> {
        let source_owner = self.owner;
        let child = self.derive_inner(request)?;
        audit
            .record(CapAuditEvent {
                action: CapAuditAction::Derive,
                object_id: child.object_id,
                source_owner,
                target_owner: child.owner,
                perms: child.perms,
                generation: child.generation,
                revocation_epoch: child.revocation_epoch,
                affected_slots: 1,
            })
            .map_err(|_| DeriveError::AuditRejected)?;

        Ok(child)
    }

    #[cfg(test)]
    pub(crate) fn grant_with_audit(
        self,
        target_owner: PrincipalId,
        audit: &mut impl CapAuditLog,
    ) -> Result<Self, DeriveError> {
        self.grant_prevalidated_with_audit(target_owner, audit)
    }

    pub fn grant_live_with_audit(
        &self,
        target_owner: PrincipalId,
        current_generation: u32,
        current_epoch: u64,
        audit: &mut impl CapAuditLog,
    ) -> Result<Self, DeriveError> {
        self.validate_live_for_derivation(current_generation, current_epoch)?;
        self.grant_prevalidated_with_audit(target_owner, audit)
    }

    fn grant_prevalidated_with_audit(
        &self,
        target_owner: PrincipalId,
        audit: &mut impl CapAuditLog,
    ) -> Result<Self, DeriveError> {
        let source_owner = self.owner;
        let child = self.grant_inner(target_owner)?;
        audit
            .record(CapAuditEvent {
                action: CapAuditAction::Grant,
                object_id: child.object_id,
                source_owner,
                target_owner: child.owner,
                perms: child.perms,
                generation: child.generation,
                revocation_epoch: child.revocation_epoch,
                affected_slots: 1,
            })
            .map_err(|_| DeriveError::AuditRejected)?;

        Ok(child)
    }

    const fn validate_live_for_derivation(
        &self,
        current_generation: u32,
        current_epoch: u64,
    ) -> Result<(), DeriveError> {
        match self.validate_live(current_generation, current_epoch) {
            Ok(()) => Ok(()),
            Err(CapValidationError::Revoked) => Err(DeriveError::ParentRevoked),
            Err(CapValidationError::StaleGeneration) => Err(DeriveError::ParentStaleGeneration),
        }
    }

    fn derive_inner(&self, request: DeriveRequest) -> Result<Self, DeriveError> {
        if !self.perms.contains(CapPerms::DERIVE) {
            return Err(DeriveError::MissingDerivePermission);
        }

        let cross_owner = request.owner() != self.owner;
        if cross_owner && !self.perms.contains(CapPerms::GRANT) {
            return Err(DeriveError::MissingGrantPermission);
        }

        if matches!(request.range_len(), Some(0)) {
            return Err(DeriveError::RangeEscalates);
        }

        if !self.perms.contains(request.perms()) {
            return Err(DeriveError::PermissionsEscalate);
        }

        if !range_is_subset(self.base, self.len, request.base(), request.range_len()) {
            return Err(DeriveError::RangeEscalates);
        }

        let child_perms = delegated_perms(request.perms(), cross_owner);

        Ok(Self {
            object_id: self.object_id,
            base: request.base(),
            len: request.range_len(),
            perms: child_perms,
            owner: request.owner(),
            generation: self.generation,
            revocation_epoch: self.revocation_epoch,
            kind: self.kind,
        })
    }

    fn grant_inner(&self, target_owner: PrincipalId) -> Result<Self, DeriveError> {
        if !self.perms.contains(CapPerms::GRANT) {
            return Err(DeriveError::MissingGrantPermission);
        }
        let cross_owner = target_owner != self.owner;

        // Same-owner grants intentionally retain REVOKE/ADMIN: the owner is
        // creating another handle for its own authority, not delegating to a
        // different principal. Cross-owner grants strip those permissions in
        // `delegated_perms`.
        Ok(Self {
            perms: delegated_perms(self.perms.without(CapPerms::GRANT), cross_owner),
            owner: target_owner,
            object_id: self.object_id,
            base: self.base,
            len: self.len,
            generation: self.generation,
            revocation_epoch: self.revocation_epoch,
            kind: self.kind,
        })
    }
}

const fn delegated_perms(perms: CapPerms, cross_owner: bool) -> CapPerms {
    if cross_owner {
        perms.without(
            CapPerms::GRANT
                .union(CapPerms::REVOKE)
                .union(CapPerms::ADMIN),
        )
    } else {
        perms
    }
}

impl fmt::Debug for Capability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Capability")
            .field("object_id", &"<redacted>")
            .field("has_range", &(self.base.is_some() && self.len.is_some()))
            .field("owner", &"<redacted>")
            .field("kind", &self.kind)
            .finish()
    }
}

#[cfg(test)]
pub(crate) struct TestCapabilitySpec {
    pub(crate) object_id: ObjectId,
    pub(crate) base: Option<VirtAddr>,
    pub(crate) len: Option<u64>,
    pub(crate) perms: CapPerms,
    pub(crate) owner: PrincipalId,
    pub(crate) generation: u32,
    pub(crate) revocation_epoch: u64,
    pub(crate) kind: CapKind,
}

#[cfg(test)]
impl Capability {
    pub(crate) const fn new_for_test(spec: TestCapabilitySpec) -> Self {
        Self {
            object_id: spec.object_id,
            base: spec.base,
            len: spec.len,
            perms: spec.perms,
            owner: spec.owner,
            generation: spec.generation,
            revocation_epoch: spec.revocation_epoch,
            kind: spec.kind,
        }
    }
}

fn range_is_subset(
    parent_base: Option<VirtAddr>,
    parent_len: Option<u64>,
    child_base: Option<VirtAddr>,
    child_len: Option<u64>,
) -> bool {
    match (parent_base, parent_len, child_base, child_len) {
        (None, None, None, None) => true,
        (None, None, Some(child_base), Some(child_len)) => {
            child_base.get().checked_add(child_len).is_some()
        }
        (Some(_), Some(_), None, None) => false,
        (Some(parent_base), Some(parent_len), Some(child_base), Some(child_len)) => {
            bounded_range_contains(parent_base.get(), parent_len, child_base.get(), child_len)
        }
        _ => false,
    }
}

fn bounded_range_contains(
    parent_base: u64,
    parent_len: u64,
    child_base: u64,
    child_len: u64,
) -> bool {
    let Some(parent_end) = parent_base.checked_add(parent_len) else {
        return false;
    };
    let Some(child_end) = child_base.checked_add(child_len) else {
        return false;
    };

    child_base >= parent_base && child_end <= parent_end
}

#[cfg(test)]
mod tests;
