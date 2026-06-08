use aesynx_abi::{ObjectId, PrincipalId, VirtAddr};

use crate::{CapKind, CapPerms, CapValidationError, DeriveError, DeriveRequest};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapAuditEvent {
    pub action: CapAuditAction,
    pub object_id: ObjectId,
    pub source_owner: PrincipalId,
    pub target_owner: PrincipalId,
    pub perms: CapPerms,
    pub generation: u32,
    pub revocation_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapAuditAction {
    Derive,
    Grant,
}

pub trait CapAuditLog {
    fn record(&mut self, event: CapAuditEvent) -> Result<(), DeriveError>;
}

#[derive(Debug, Eq, PartialEq)]
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
    #[must_use]
    #[allow(dead_code)]
    pub(crate) const fn new_root(
        object_id: ObjectId,
        kind: CapKind,
        owner: PrincipalId,
        perms: CapPerms,
        generation: u32,
        revocation_epoch: u64,
    ) -> Self {
        Self {
            object_id,
            base: None,
            len: None,
            perms,
            owner,
            generation,
            revocation_epoch,
            kind,
        }
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
    pub const fn perms(&self) -> CapPerms {
        self.perms
    }

    #[must_use]
    pub const fn owner(&self) -> PrincipalId {
        self.owner
    }

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

    pub fn derive_with_audit(
        self,
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
            })
            .map_err(|_| DeriveError::AuditRejected)?;

        Ok(child)
    }

    pub fn grant_with_audit(
        self,
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
            })
            .map_err(|_| DeriveError::AuditRejected)?;

        Ok(child)
    }

    fn derive_inner(self, request: DeriveRequest) -> Result<Self, DeriveError> {
        if !self.perms.contains(CapPerms::DERIVE) {
            return Err(DeriveError::MissingDerivePermission);
        }

        if matches!(request.len, Some(0)) {
            return Err(DeriveError::RangeEscalates);
        }

        if !self.perms.contains(request.perms) {
            return Err(DeriveError::PermissionsEscalate);
        }

        if !range_is_subset(self.base, self.len, request.base, request.len) {
            return Err(DeriveError::RangeEscalates);
        }

        Ok(Self {
            object_id: self.object_id,
            base: request.base,
            len: request.len,
            perms: request.perms,
            owner: request.owner,
            generation: self.generation,
            revocation_epoch: self.revocation_epoch,
            kind: self.kind,
        })
    }

    fn grant_inner(self, target_owner: PrincipalId) -> Result<Self, DeriveError> {
        if !self.perms.contains(CapPerms::GRANT) {
            return Err(DeriveError::MissingGrantPermission);
        }

        Ok(Self {
            perms: self.perms.without(CapPerms::GRANT),
            owner: target_owner,
            ..self
        })
    }
}

#[cfg(test)]
struct TestCapabilitySpec {
    object_id: ObjectId,
    base: Option<VirtAddr>,
    len: Option<u64>,
    perms: CapPerms,
    owner: PrincipalId,
    generation: u32,
    revocation_epoch: u64,
    kind: CapKind,
}

#[cfg(test)]
impl Capability {
    const fn new_for_test(spec: TestCapabilitySpec) -> Self {
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
        (None, None, Some(_), Some(_)) => true,
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
mod tests {
    use aesynx_abi::{ObjectId, PrincipalId, VirtAddr};

    use crate::{
        CapAuditAction, CapAuditEvent, CapAuditLog, CapKind, CapPerms, CapValidationError,
        Capability, DeriveError, DeriveRequest,
    };

    use super::TestCapabilitySpec;

    fn parent_cap(perms: CapPerms) -> Capability {
        Capability::new_for_test(TestCapabilitySpec {
            object_id: ObjectId::new(7),
            base: Some(VirtAddr::new(100)),
            len: Some(50),
            perms,
            owner: PrincipalId::new(1),
            generation: 3,
            revocation_epoch: 9,
            kind: CapKind::Memory,
        })
    }

    #[derive(Default)]
    struct TestAudit {
        last_event: Option<CapAuditEvent>,
    }

    impl CapAuditLog for TestAudit {
        fn record(&mut self, event: CapAuditEvent) -> Result<(), DeriveError> {
            self.last_event = Some(event);
            Ok(())
        }
    }

    fn audited_derive(
        parent: Capability,
        request: DeriveRequest,
    ) -> Result<Capability, DeriveError> {
        let mut audit = TestAudit::default();

        parent.derive_with_audit(request, &mut audit)
    }

    fn audited_grant(
        parent: Capability,
        target_owner: PrincipalId,
    ) -> Result<Capability, DeriveError> {
        let mut audit = TestAudit::default();

        parent.grant_with_audit(target_owner, &mut audit)
    }

    #[test]
    fn derive_reduces_authority_and_changes_owner() {
        let parent = parent_cap(
            CapPerms::READ
                .union(CapPerms::WRITE)
                .union(CapPerms::DERIVE),
        );
        let request = DeriveRequest {
            perms: CapPerms::READ,
            owner: PrincipalId::new(2),
            base: Some(VirtAddr::new(120)),
            len: Some(10),
        };

        let expected = Capability::new_for_test(TestCapabilitySpec {
            object_id: parent.object_id(),
            base: Some(VirtAddr::new(120)),
            len: Some(10),
            perms: CapPerms::READ,
            owner: PrincipalId::new(2),
            generation: parent.generation(),
            revocation_epoch: parent.revocation_epoch(),
            kind: parent.kind(),
        });

        assert_eq!(audited_derive(parent, request), Ok(expected));
    }

    #[test]
    fn derive_with_audit_records_chain_of_custody() {
        let parent = parent_cap(CapPerms::READ.union(CapPerms::DERIVE));
        let request = DeriveRequest {
            perms: CapPerms::READ,
            owner: PrincipalId::new(2),
            base: Some(VirtAddr::new(120)),
            len: Some(10),
        };
        let mut audit = TestAudit::default();

        assert_eq!(
            parent
                .derive_with_audit(request, &mut audit)
                .map(|cap| cap.owner()),
            Ok(PrincipalId::new(2))
        );
        assert_eq!(
            audit.last_event.map(|event| event.action),
            Some(CapAuditAction::Derive)
        );
    }

    #[test]
    fn derive_requires_derive_permission() {
        let parent = parent_cap(CapPerms::READ);
        let request = DeriveRequest {
            perms: CapPerms::READ,
            owner: PrincipalId::new(2),
            base: Some(VirtAddr::new(120)),
            len: Some(10),
        };

        assert_eq!(
            audited_derive(parent, request),
            Err(DeriveError::MissingDerivePermission)
        );
    }

    #[test]
    fn derive_rejects_permission_escalation() {
        let parent = parent_cap(CapPerms::READ.union(CapPerms::DERIVE));
        let request = DeriveRequest {
            perms: CapPerms::READ.union(CapPerms::WRITE),
            owner: PrincipalId::new(2),
            base: Some(VirtAddr::new(120)),
            len: Some(10),
        };

        assert_eq!(
            audited_derive(parent, request),
            Err(DeriveError::PermissionsEscalate)
        );
    }

    #[test]
    fn derive_rejects_range_expansion() {
        let parent = parent_cap(CapPerms::READ.union(CapPerms::DERIVE));
        let request = DeriveRequest {
            perms: CapPerms::READ,
            owner: PrincipalId::new(2),
            base: Some(VirtAddr::new(120)),
            len: Some(40),
        };

        assert_eq!(
            audited_derive(parent, request),
            Err(DeriveError::RangeEscalates)
        );
    }

    #[test]
    fn derive_rejects_zero_length_range() {
        let parent = parent_cap(CapPerms::READ.union(CapPerms::DERIVE));
        let request = DeriveRequest {
            perms: CapPerms::READ,
            owner: PrincipalId::new(2),
            base: Some(VirtAddr::new(120)),
            len: Some(0),
        };

        assert_eq!(
            audited_derive(parent, request),
            Err(DeriveError::RangeEscalates)
        );
    }

    #[test]
    fn grant_requires_grant_permission() {
        let parent = parent_cap(CapPerms::READ);

        assert_eq!(
            audited_grant(parent, PrincipalId::new(2)),
            Err(DeriveError::MissingGrantPermission)
        );
    }

    #[test]
    fn grant_transfers_authority_without_regrant_by_default() {
        let parent = parent_cap(CapPerms::READ.union(CapPerms::GRANT));
        let expected = Capability::new_for_test(TestCapabilitySpec {
            object_id: parent.object_id(),
            base: parent.base(),
            len: parent.range_len(),
            perms: CapPerms::READ,
            owner: PrincipalId::new(2),
            generation: parent.generation(),
            revocation_epoch: parent.revocation_epoch(),
            kind: parent.kind(),
        });

        assert_eq!(audited_grant(parent, PrincipalId::new(2)), Ok(expected));
    }

    #[test]
    fn grant_with_audit_records_chain_of_custody() {
        let parent = parent_cap(CapPerms::READ.union(CapPerms::GRANT));
        let mut audit = TestAudit::default();

        assert_eq!(
            parent
                .grant_with_audit(PrincipalId::new(2), &mut audit)
                .map(|cap| cap.owner()),
            Ok(PrincipalId::new(2))
        );
        assert_eq!(
            audit.last_event.map(|event| event.action),
            Some(CapAuditAction::Grant)
        );
    }

    #[test]
    fn live_validation_rejects_stale_generation_and_epoch() {
        let parent = parent_cap(CapPerms::READ);

        assert_eq!(parent.validate_live(3, 9), Ok(()));
        assert_eq!(
            parent.validate_live(2, 9),
            Err(CapValidationError::StaleGeneration)
        );
        assert_eq!(parent.validate_live(3, 8), Err(CapValidationError::Revoked));
    }
}
