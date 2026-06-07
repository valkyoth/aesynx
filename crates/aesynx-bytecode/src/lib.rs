#![no_std]
#![deny(unsafe_code)]

use aesynx_cap::CapPerms;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Instruction {
    LoadCap {
        dst: u8,
        cap_slot: u16,
    },
    CheckPerm {
        reg: u8,
        perms: CapPerms,
    },
    ReadU64 {
        dst: u8,
        base: u8,
        offset: UncheckedOffset,
    },
    WriteU64 {
        base: u8,
        offset: UncheckedOffset,
        src: u8,
    },
    SendMsg {
        endpoint: u8,
        payload: u8,
    },
    BranchIf {
        reg: u8,
        target: UncheckedTarget,
    },
    Return {
        reg: u8,
    },
    Yield,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UncheckedTarget(u16);

impl UncheckedTarget {
    #[must_use]
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UncheckedOffset(u16);

impl UncheckedOffset {
    #[must_use]
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Fuel {
    remaining: u64,
}

pub const MAX_FUEL: u64 = 1_000_000;

impl Fuel {
    pub const fn new(remaining: u64) -> Result<Self, FuelError> {
        if remaining > MAX_FUEL {
            return Err(FuelError::ExceedsLimit);
        }

        Ok(Self { remaining })
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
    ExceedsLimit,
    Exhausted,
}

#[cfg(test)]
mod tests {
    use super::{Fuel, FuelError, MAX_FUEL};

    #[test]
    fn consume_decrements_fuel_until_exhausted() {
        let mut fuel = match Fuel::new(2) {
            Ok(fuel) => fuel,
            Err(error) => return assert_eq!(error, FuelError::ExceedsLimit),
        };

        assert_eq!(fuel.consume(), Ok(()));
        assert_eq!(fuel.remaining(), 1);
        assert_eq!(fuel.consume(), Ok(()));
        assert_eq!(fuel.remaining(), 0);
        assert_eq!(fuel.consume(), Err(FuelError::Exhausted));
    }

    #[test]
    fn fuel_rejects_values_above_limit() {
        assert_eq!(Fuel::new(MAX_FUEL + 1), Err(FuelError::ExceedsLimit));
    }
}
