#![no_std]
#![deny(unsafe_code)]

use aesynx_abi::CoreId;

pub trait ClockSource {
    fn now_ticks(&self) -> Result<u64, ClockError>;
    fn monotonic_ns(&self) -> Result<u64, ClockError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockError {
    Unavailable,
    Uninitialized,
    Overflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimerTick {
    core: CoreId,
    tick: u64,
    timestamp: u64,
}

impl TimerTick {
    #[must_use]
    #[allow(dead_code)]
    pub(crate) const fn new(core: CoreId, tick: u64, timestamp: u64) -> Self {
        Self {
            core,
            tick,
            timestamp,
        }
    }

    #[must_use]
    pub const fn core(self) -> CoreId {
        self.core
    }

    #[must_use]
    pub const fn tick(self) -> u64 {
        self.tick
    }

    #[must_use]
    pub const fn timestamp(self) -> u64 {
        self.timestamp
    }
}

#[cfg(test)]
mod tests {
    use aesynx_abi::CoreId;

    use super::TimerTick;

    #[test]
    fn timer_tick_exposes_kernel_stamped_values() {
        let tick = TimerTick::new(CoreId::new(1), 2, 3);

        assert_eq!(tick.core(), CoreId::new(1));
        assert_eq!(tick.tick(), 2);
        assert_eq!(tick.timestamp(), 3);
    }
}
