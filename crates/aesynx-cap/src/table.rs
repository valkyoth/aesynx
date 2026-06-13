use core::fmt;

use aesynx_abi::{CapId, ObjectId, PrincipalId};

use crate::{
    CapAuditAction, CapAuditEvent, CapAuditLog, CapIdError, CapIdParts, CapKind, CapPerms,
    CapSlotIndex, Capability, DeriveError, DeriveRequest, RevocationError, ensure_revoke_authority,
};

const INITIAL_SLOT_GENERATION: u32 = 1;

pub struct CapabilityTable<const SLOTS: usize> {
    slots: [CapabilitySlot; SLOTS],
}

struct CapabilitySlot {
    cap: Option<Capability>,
    generation: u32,
}

impl CapabilitySlot {
    const EMPTY: Self = Self {
        cap: None,
        generation: INITIAL_SLOT_GENERATION,
    };

    const fn is_occupied(&self) -> bool {
        self.cap.is_some()
    }
}

impl<const SLOTS: usize> CapabilityTable<SLOTS> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            slots: [const { CapabilitySlot::EMPTY }; SLOTS],
        }
    }

    pub fn insert_root(
        &mut self,
        object_id: ObjectId,
        kind: CapKind,
        owner: PrincipalId,
        perms: CapPerms,
        object_generation: u32,
        revocation_epoch: u64,
    ) -> Result<CapId, CapTableError> {
        let slot = self.vacant_slot().ok_or(CapTableError::TableFull)?;
        let slot_index = slot_index(slot)?;
        let cap = Capability::new_root(
            object_id,
            kind,
            owner,
            perms,
            object_generation,
            revocation_epoch,
        );
        let id = cap_id_for_slot(slot_index, self.slots[slot].generation)?;

        self.slots[slot].cap = Some(cap);

        Ok(id)
    }

    pub fn check(&self, id: CapId, required: CapPerms) -> Result<&Capability, CapTableError> {
        let cap = self.get(id)?;
        if !cap.allows(required) {
            return Err(CapTableError::MissingPermission);
        }

        Ok(cap)
    }

    pub fn get(&self, id: CapId) -> Result<&Capability, CapTableError> {
        let parts = CapIdParts::from_cap_id(id)?;
        let slot = slot_usize(parts.slot());
        let entry = self.slots.get(slot).ok_or(CapTableError::SlotOutOfRange)?;
        if entry.generation != parts.generation().get() {
            return Err(CapTableError::StaleId);
        }

        entry.cap.as_ref().ok_or(CapTableError::EmptySlot)
    }

    pub fn derive_with_audit(
        &mut self,
        source_id: CapId,
        request: DeriveRequest,
        live: &impl LiveAuthorityView,
        audit: &mut impl CapAuditLog,
    ) -> Result<CapId, CapTableError> {
        let slot = self.vacant_slot().ok_or(CapTableError::TableFull)?;
        let slot_index = slot_index(slot)?;
        let id = cap_id_for_slot(slot_index, self.slots[slot].generation)?;
        let source = self.get(source_id)?;
        let current = live.live_authority(source.object_id())?;
        let child = source.derive_live_with_audit(
            request,
            current.generation(),
            current.revocation_epoch(),
            audit,
        )?;

        self.slots[slot].cap = Some(child);

        Ok(id)
    }

    pub fn grant_with_audit(
        &mut self,
        source_id: CapId,
        target_owner: PrincipalId,
        live: &impl LiveAuthorityView,
        audit: &mut impl CapAuditLog,
    ) -> Result<CapId, CapTableError> {
        let slot = self.vacant_slot().ok_or(CapTableError::TableFull)?;
        let slot_index = slot_index(slot)?;
        let id = cap_id_for_slot(slot_index, self.slots[slot].generation)?;
        let source = self.get(source_id)?;
        let current = live.live_authority(source.object_id())?;
        let child = source.grant_live_with_audit(
            target_owner,
            current.generation(),
            current.revocation_epoch(),
            audit,
        )?;

        self.slots[slot].cap = Some(child);

        Ok(id)
    }

    /// Revokes every in-table capability for the target object's authority
    /// epoch, including the authority capability used for the revoke.
    ///
    /// This is deliberate total-revoke semantics for the v0.20 table model:
    /// callers that need post-revoke authority must re-mint it from a future
    /// object registry or epoch store. Slot generations fail instead of
    /// wrapping; a persistent table must rebuild or retire slots before
    /// `u32::MAX` is reached.
    ///
    /// Callers must treat `authority_id`, `target_id`, and every other in-table
    /// `CapId` for the same object as consumed by a successful revoke. A later
    /// `StaleId` from one of those IDs is the expected self-revocation result,
    /// not a recovery signal.
    pub fn revoke_with_audit(
        &mut self,
        authority_id: CapId,
        target_id: CapId,
        live: &impl LiveAuthorityView,
        audit: &mut impl CapAuditLog,
    ) -> Result<u32, CapTableError> {
        let authority = self.get(authority_id)?;
        let current = live.live_authority(authority.object_id())?;
        authority.validate_live(current.generation(), current.revocation_epoch())?;
        let source_owner = authority.owner();
        let target = self.get(target_id)?;
        let target_object = target.object_id();
        let target_owner = target.owner();
        let target_generation = target.generation();
        let target_revocation_epoch = target.revocation_epoch();
        ensure_revoke_authority(authority, target_object)?;
        self.validate_revoke_can_commit(target_object)?;
        let revoked = self.count_revoke_targets(target_object)?;

        audit
            .record(CapAuditEvent {
                action: CapAuditAction::Revoke,
                object_id: target_object,
                source_owner,
                target_owner,
                perms: CapPerms::REVOKE,
                generation: target_generation,
                revocation_epoch: target_revocation_epoch,
                affected_slots: revoked,
            })
            .map_err(|_| CapTableError::AuditRejected)?;

        let mut index = 0usize;
        while index < SLOTS {
            if self.slots[index]
                .cap
                .as_ref()
                .is_some_and(|cap| cap.object_id() == target_object)
            {
                self.slots[index].cap = None;
                self.slots[index].generation += 1;
            }
            index += 1;
        }

        Ok(revoked)
    }

    #[must_use]
    pub fn occupied_slots(&self) -> usize {
        let mut occupied = 0usize;
        let mut index = 0usize;
        while index < SLOTS {
            if self.slots[index].is_occupied() {
                occupied += 1;
            }
            index += 1;
        }

        occupied
    }

    #[must_use]
    pub const fn capacity(&self) -> usize {
        SLOTS
    }

    fn vacant_slot(&self) -> Option<usize> {
        let mut index = 0usize;
        while index < SLOTS {
            if !self.slots[index].is_occupied() {
                return Some(index);
            }
            index += 1;
        }

        None
    }

    fn validate_revoke_can_commit(&self, object_id: ObjectId) -> Result<(), CapTableError> {
        let mut index = 0usize;
        while index < SLOTS {
            if self.slots[index]
                .cap
                .as_ref()
                .is_some_and(|cap| cap.object_id() == object_id)
                && self.slots[index].generation == u32::MAX
            {
                return Err(CapTableError::GenerationOverflow);
            }
            index += 1;
        }

        Ok(())
    }

    fn count_revoke_targets(&self, object_id: ObjectId) -> Result<u32, CapTableError> {
        let mut revoked = 0u32;
        let mut index = 0usize;
        while index < SLOTS {
            if self.slots[index]
                .cap
                .as_ref()
                .is_some_and(|cap| cap.object_id() == object_id)
            {
                revoked = revoked
                    .checked_add(1)
                    .ok_or(CapTableError::RevokeCountOverflow)?;
            }
            index += 1;
        }

        Ok(revoked)
    }
}

impl<const SLOTS: usize> Default for CapabilityTable<SLOTS> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const SLOTS: usize> fmt::Debug for CapabilityTable<SLOTS> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CapabilityTable")
            .field("capacity", &SLOTS)
            .field("occupied_slots", &self.occupied_slots())
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapTableError {
    AuditRejected,
    EmptySlot,
    GenerationOverflow,
    Id(CapIdError),
    LiveAuthority(LiveAuthorityError),
    MissingPermission,
    Revoke(RevocationError),
    RevokeCountOverflow,
    SlotOutOfRange,
    StaleId,
    TableFull,
}

impl From<CapIdError> for CapTableError {
    fn from(error: CapIdError) -> Self {
        Self::Id(error)
    }
}

impl From<LiveAuthorityError> for CapTableError {
    fn from(error: LiveAuthorityError) -> Self {
        Self::LiveAuthority(error)
    }
}

impl From<crate::CapValidationError> for CapTableError {
    fn from(error: crate::CapValidationError) -> Self {
        Self::Revoke(RevocationError::from(error))
    }
}

impl From<DeriveError> for CapTableError {
    fn from(error: DeriveError) -> Self {
        match error {
            DeriveError::AuditRejected => Self::AuditRejected,
            DeriveError::MissingDerivePermission => Self::MissingPermission,
            DeriveError::MissingGrantPermission => Self::MissingPermission,
            DeriveError::ParentRevoked => Self::StaleId,
            DeriveError::ParentStaleGeneration => Self::StaleId,
            DeriveError::PermissionsEscalate => Self::MissingPermission,
            DeriveError::RangeEscalates => Self::MissingPermission,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LiveAuthorityState {
    generation: u32,
    revocation_epoch: u64,
}

impl LiveAuthorityState {
    #[must_use]
    pub const fn new(generation: u32, revocation_epoch: u64) -> Self {
        Self {
            generation,
            revocation_epoch,
        }
    }

    #[must_use]
    pub const fn generation(self) -> u32 {
        self.generation
    }

    #[must_use]
    pub const fn revocation_epoch(self) -> u64 {
        self.revocation_epoch
    }
}

pub trait LiveAuthorityView {
    fn live_authority(&self, object_id: ObjectId)
    -> Result<LiveAuthorityState, LiveAuthorityError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LiveAuthorityError {
    ObjectNotFound,
}

impl From<RevocationError> for CapTableError {
    fn from(error: RevocationError) -> Self {
        Self::Revoke(error)
    }
}

fn cap_id_for_slot(slot: CapSlotIndex, generation: u32) -> Result<CapId, CapTableError> {
    let generation = crate::CapGeneration::new(generation)?;

    Ok(CapIdParts::new(slot, generation).cap_id())
}

fn slot_index(index: usize) -> Result<CapSlotIndex, CapTableError> {
    if index > u32::MAX as usize {
        return Err(CapTableError::SlotOutOfRange);
    }

    Ok(CapSlotIndex::new(index as u32))
}

fn slot_usize(slot: CapSlotIndex) -> usize {
    slot.get() as usize
}

#[cfg(test)]
mod tests;
