use aesynx_abi::CoreId;

pub trait LiveCoreSet {
    fn contains(&self, core: CoreId) -> bool;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidatedCoreId(CoreId);

impl ValidatedCoreId {
    pub fn new(core: CoreId, live_cores: &impl LiveCoreSet) -> Result<Self, CoreValidationError> {
        if !live_cores.contains(core) {
            return Err(CoreValidationError::UnknownCore);
        }

        Ok(Self(core))
    }

    #[must_use]
    pub const fn get(self) -> CoreId {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreValidationError {
    UnknownCore,
}
