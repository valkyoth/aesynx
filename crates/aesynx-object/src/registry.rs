use aesynx_abi::{CoreId, ObjectId};
use aesynx_cap::{CapKind, CapPerms, Capability};

use crate::{KernelObject, ObjectRecord, ObjectType};

const INITIAL_OBJECT_GENERATION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectCreate {
    id: ObjectId,
    object_type: ObjectType,
    owner_core: CoreId,
    revocation_epoch: u64,
}

impl ObjectCreate {
    #[must_use]
    pub const fn new(id: ObjectId, object_type: ObjectType, owner_core: CoreId) -> Self {
        Self {
            id,
            object_type,
            owner_core,
            revocation_epoch: 0,
        }
    }

    #[must_use]
    pub const fn with_revocation_epoch(mut self, revocation_epoch: u64) -> Self {
        self.revocation_epoch = revocation_epoch;
        self
    }

    #[must_use]
    pub const fn memory(id: ObjectId, owner_core: CoreId) -> Self {
        Self::new(id, ObjectType::MemoryRegion, owner_core)
    }

    #[must_use]
    pub const fn endpoint(id: ObjectId, owner_core: CoreId) -> Self {
        Self::new(id, ObjectType::Endpoint, owner_core)
    }

    #[must_use]
    pub const fn queue(id: ObjectId, owner_core: CoreId) -> Self {
        Self::new(id, ObjectType::Queue, owner_core)
    }

    #[must_use]
    pub const fn task_placeholder(id: ObjectId, owner_core: CoreId) -> Self {
        Self::new(id, ObjectType::Task, owner_core)
    }
}

#[derive(Clone, Copy)]
enum ObjectSlot {
    Empty { next_generation: u32 },
    Live(ObjectRecord),
}

impl ObjectSlot {
    const EMPTY: Self = Self::Empty {
        next_generation: INITIAL_OBJECT_GENERATION,
    };
}

pub struct ObjectRegistry<const CAPACITY: usize> {
    slots: [ObjectSlot; CAPACITY],
}

impl<const CAPACITY: usize> ObjectRegistry<CAPACITY> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            slots: [const { ObjectSlot::EMPTY }; CAPACITY],
        }
    }

    pub fn create(&mut self, request: ObjectCreate) -> Result<ObjectRecord, ObjectRegistryError> {
        validate_id(request.id)?;
        self.ensure_id_unused(request.id)?;
        let (slot, generation) = self
            .vacant_slot()
            .ok_or(ObjectRegistryError::RegistryFull)?;
        let record = ObjectRecord::new(
            request.id,
            request.object_type,
            request.owner_core,
            generation,
            request.revocation_epoch,
        );

        self.slots[slot] = ObjectSlot::Live(record);
        Ok(record)
    }

    pub fn delete(&mut self, id: ObjectId) -> Result<ObjectRecord, ObjectRegistryError> {
        validate_id(id)?;
        let slot = self.live_slot(id)?;
        let ObjectSlot::Live(record) = self.slots[slot] else {
            return Err(ObjectRegistryError::ObjectNotFound);
        };
        let next_generation = record
            .generation()
            .checked_add(1)
            .ok_or(ObjectRegistryError::GenerationExhausted)?;

        self.slots[slot] = ObjectSlot::Empty { next_generation };
        Ok(record)
    }

    pub fn get(&self, id: ObjectId) -> Result<ObjectRecord, ObjectRegistryError> {
        validate_id(id)?;
        let slot = self.live_slot(id)?;
        match self.slots[slot] {
            ObjectSlot::Live(record) => Ok(record),
            ObjectSlot::Empty { .. } => Err(ObjectRegistryError::ObjectNotFound),
        }
    }

    pub fn list(&self, out: &mut [ObjectRecord]) -> Result<usize, ObjectRegistryError> {
        let count = self.len();
        if out.len() < count {
            return Err(ObjectRegistryError::OutputTooSmall);
        }

        let mut written = 0usize;
        for slot in self.slots {
            if let ObjectSlot::Live(record) = slot {
                out[written] = record;
                written += 1;
            }
        }

        Ok(written)
    }

    pub fn resolve_capability(
        &self,
        capability: &Capability,
        required: CapPerms,
    ) -> Result<ObjectRecord, ObjectRegistryError> {
        if !capability.allows(required) {
            return Err(ObjectRegistryError::MissingPermission);
        }
        let record = self.get(capability.object_id())?;
        if !cap_kind_matches(record, capability.kind()) {
            return Err(ObjectRegistryError::WrongCapabilityKind);
        }
        if capability.generation() != record.generation() {
            return Err(ObjectRegistryError::StaleObjectGeneration);
        }
        if !capability.matches_revocation_epoch(record.revocation_epoch()) {
            return Err(ObjectRegistryError::Revoked);
        }

        Ok(record)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.slots
            .iter()
            .filter(|slot| matches!(slot, ObjectSlot::Live(_)))
            .count()
    }

    #[must_use]
    pub const fn capacity(&self) -> usize {
        CAPACITY
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn ensure_id_unused(&self, id: ObjectId) -> Result<(), ObjectRegistryError> {
        for slot in &self.slots {
            match slot {
                ObjectSlot::Live(record) if record.object_id() == id => {
                    return Err(ObjectRegistryError::DuplicateObject);
                }
                ObjectSlot::Empty { .. } | ObjectSlot::Live(_) => {}
            }
        }
        Ok(())
    }

    fn vacant_slot(&self) -> Option<(usize, u32)> {
        for (index, slot) in self.slots.iter().enumerate() {
            if let ObjectSlot::Empty { next_generation } = slot {
                return Some((index, *next_generation));
            }
        }

        None
    }

    fn live_slot(&self, id: ObjectId) -> Result<usize, ObjectRegistryError> {
        for (index, slot) in self.slots.iter().enumerate() {
            match slot {
                ObjectSlot::Live(record) if record.object_id() == id => return Ok(index),
                ObjectSlot::Empty { .. } | ObjectSlot::Live(_) => {}
            }
        }
        Err(ObjectRegistryError::ObjectNotFound)
    }
}

impl<const CAPACITY: usize> Default for ObjectRegistry<CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

fn validate_id(id: ObjectId) -> Result<(), ObjectRegistryError> {
    if id.get() == 0 {
        return Err(ObjectRegistryError::InvalidObjectId);
    }
    Ok(())
}

fn cap_kind_matches(record: ObjectRecord, cap_kind: CapKind) -> bool {
    record.cap_kind() == cap_kind
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectRegistryError {
    InvalidObjectId,
    DuplicateObject,
    RegistryFull,
    ObjectNotFound,
    OutputTooSmall,
    MissingPermission,
    WrongCapabilityKind,
    StaleObjectGeneration,
    Revoked,
    /// The object was deliberately left live because recycling its slot would
    /// wrap the generation counter and could make stale capabilities valid
    /// again. Operators should treat this as a registry-retirement signal.
    GenerationExhausted,
}
