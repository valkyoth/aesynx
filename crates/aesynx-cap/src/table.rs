use core::fmt;

use aesynx_abi::{CapId, ObjectId, PrincipalId};

use crate::{
    CapAuditLog, CapIdError, CapIdParts, CapKind, CapPerms, CapSlotIndex, Capability, DeriveError,
    DeriveRequest, RevocationError, ensure_revoke_authority,
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
        let slot = slot_usize(parts.slot())?;
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
        audit: &mut impl CapAuditLog,
    ) -> Result<CapId, CapTableError> {
        let source = self.get(source_id)?;
        let child = source.derive_live_with_audit(
            request,
            source.generation(),
            source.revocation_epoch(),
            audit,
        )?;
        let slot = self.vacant_slot().ok_or(CapTableError::TableFull)?;
        let slot_index = slot_index(slot)?;
        let id = cap_id_for_slot(slot_index, self.slots[slot].generation)?;

        self.slots[slot].cap = Some(child);

        Ok(id)
    }

    pub fn grant_with_audit(
        &mut self,
        source_id: CapId,
        target_owner: PrincipalId,
        audit: &mut impl CapAuditLog,
    ) -> Result<CapId, CapTableError> {
        let source = self.get(source_id)?;
        let child = source.grant_live_with_audit(
            target_owner,
            source.generation(),
            source.revocation_epoch(),
            audit,
        )?;
        let slot = self.vacant_slot().ok_or(CapTableError::TableFull)?;
        let slot_index = slot_index(slot)?;
        let id = cap_id_for_slot(slot_index, self.slots[slot].generation)?;

        self.slots[slot].cap = Some(child);

        Ok(id)
    }

    pub fn revoke(&mut self, authority_id: CapId, target_id: CapId) -> Result<u32, CapTableError> {
        let authority = self.get(authority_id)?;
        let target_object = self.get(target_id)?.object_id();
        ensure_revoke_authority(authority, target_object)?;
        self.validate_revoke_can_commit(target_object)?;

        let mut revoked = 0u32;
        let mut index = 0usize;
        while index < SLOTS {
            if self.slots[index]
                .cap
                .as_ref()
                .is_some_and(|cap| cap.object_id() == target_object)
            {
                self.slots[index].cap = None;
                self.slots[index].generation += 1;
                revoked += 1;
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
    MissingPermission,
    Revoke(RevocationError),
    SlotOutOfRange,
    StaleId,
    TableFull,
}

impl From<CapIdError> for CapTableError {
    fn from(error: CapIdError) -> Self {
        Self::Id(error)
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

fn slot_usize(slot: CapSlotIndex) -> Result<usize, CapTableError> {
    let value = slot.get();
    if value as u64 > usize::MAX as u64 {
        return Err(CapTableError::SlotOutOfRange);
    }

    Ok(value as usize)
}

#[cfg(test)]
mod tests;
