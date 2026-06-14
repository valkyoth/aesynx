use crate::PolicyError;

pub const MAX_CONFIDENCE: u16 = 10_000;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Confidence(u16);

impl Confidence {
    pub const ZERO: Self = Self(0);

    pub const fn new(value: u16) -> Result<Self, PolicyError> {
        if value > MAX_CONFIDENCE {
            return Err(PolicyError::ConfidenceOutOfRange);
        }

        Ok(Self(value))
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}
