#![no_std]
#![deny(unsafe_code)]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Instruction {
    LoadCap { dst: u8, cap_slot: u16 },
    CheckPerm { reg: u8, perms: u32 },
    ReadU64 { dst: u8, base: u8, offset: u16 },
    WriteU64 { base: u8, offset: u16, src: u8 },
    SendMsg { endpoint: u8, payload: u8 },
    BranchIf { reg: u8, target: u16 },
    Return { reg: u8 },
    Yield,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Fuel {
    remaining: u64,
}

impl Fuel {
    #[must_use]
    pub const fn new(remaining: u64) -> Self {
        Self { remaining }
    }

    #[must_use]
    pub const fn remaining(self) -> u64 {
        self.remaining
    }

    pub fn consume(&mut self) -> Result<(), FuelError> {
        if self.remaining == 0 {
            return Err(FuelError::Exhausted);
        }
        self.remaining -= 1;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FuelError {
    Exhausted,
}

#[cfg(test)]
mod tests {
    use super::{Fuel, FuelError};

    #[test]
    fn consume_decrements_fuel_until_exhausted() {
        let mut fuel = Fuel::new(2);

        assert_eq!(fuel.consume(), Ok(()));
        assert_eq!(fuel.remaining(), 1);
        assert_eq!(fuel.consume(), Ok(()));
        assert_eq!(fuel.remaining(), 0);
        assert_eq!(fuel.consume(), Err(FuelError::Exhausted));
    }
}
