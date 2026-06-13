use aesynx_abi::CapId;

const SLOT_BITS: u32 = 32;
const SLOT_MASK: u64 = u32::MAX as u64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapSlotIndex(u32);

impl CapSlotIndex {
    #[must_use]
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapGeneration(u32);

impl CapGeneration {
    pub const fn new(raw: u32) -> Result<Self, CapIdError> {
        if raw == 0 {
            return Err(CapIdError::ZeroGeneration);
        }

        Ok(Self(raw))
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapIdParts {
    slot: CapSlotIndex,
    generation: CapGeneration,
}

impl CapIdParts {
    #[must_use]
    pub const fn new(slot: CapSlotIndex, generation: CapGeneration) -> Self {
        Self { slot, generation }
    }

    pub const fn from_cap_id(id: CapId) -> Result<Self, CapIdError> {
        let generation = match CapGeneration::new((id.get() >> SLOT_BITS) as u32) {
            Ok(generation) => generation,
            Err(error) => return Err(error),
        };
        let slot = CapSlotIndex::new((id.get() & SLOT_MASK) as u32);

        Ok(Self { slot, generation })
    }

    #[must_use]
    pub const fn cap_id(self) -> CapId {
        CapId::new(((self.generation.get() as u64) << SLOT_BITS) | self.slot.get() as u64)
    }

    #[must_use]
    pub const fn slot(self) -> CapSlotIndex {
        self.slot
    }

    #[must_use]
    pub const fn generation(self) -> CapGeneration {
        self.generation
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapIdError {
    ZeroGeneration,
}

#[cfg(test)]
mod tests {
    use aesynx_abi::CapId;

    use super::{CapGeneration, CapIdError, CapIdParts, CapSlotIndex};

    #[test]
    fn cap_id_layout_round_trips_slot_and_generation() {
        let generation = CapGeneration::new(7);
        assert_eq!(generation.map(CapGeneration::get), Ok(7));

        if let Ok(generation) = generation {
            let parts = CapIdParts::new(CapSlotIndex::new(0x00ab_cdef), generation);
            let encoded = parts.cap_id();

            assert_eq!(encoded.get(), 0x0000_0007_00ab_cdef);
            assert_eq!(CapIdParts::from_cap_id(encoded), Ok(parts));
        }
    }

    #[test]
    fn cap_id_layout_rejects_zero_generation() {
        assert_eq!(CapGeneration::new(0), Err(CapIdError::ZeroGeneration));
        assert_eq!(
            CapIdParts::from_cap_id(CapId::new(0x0000_0000_ffff_ffff)),
            Err(CapIdError::ZeroGeneration)
        );
    }

    #[test]
    fn cap_id_layout_allows_zero_slot() {
        let generation = CapGeneration::new(1);

        if let Ok(generation) = generation {
            let parts = CapIdParts::new(CapSlotIndex::new(0), generation);

            assert_eq!(parts.cap_id().get(), 0x0000_0001_0000_0000);
            assert_eq!(CapIdParts::from_cap_id(parts.cap_id()), Ok(parts));
        }
    }
}
