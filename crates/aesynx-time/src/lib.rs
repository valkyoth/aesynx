#![no_std]
#![deny(unsafe_code)]

use core::sync::atomic::{AtomicU64, Ordering};

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

pub struct TickCounter {
    ticks: AtomicU64,
}

impl TickCounter {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            ticks: AtomicU64::new(0),
        }
    }

    pub fn record_tick(&self, core: CoreId, timestamp: u64) -> Result<TimerTick, ClockError> {
        let next = self
            .ticks
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |value| {
                value.checked_add(1)
            })
            .map_err(|_| ClockError::Overflow)?
            + 1;
        Ok(TimerTick::new(core, next, timestamp))
    }

    #[must_use]
    pub fn ticks(&self) -> u64 {
        self.ticks.load(Ordering::Acquire)
    }
}

impl Default for TickCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use aesynx_abi::CoreId;

    use super::{TickCounter, TimerTick};

    #[test]
    fn timer_tick_exposes_kernel_stamped_values() {
        let tick = TimerTick::new(CoreId::new(1), 2, 3);

        assert_eq!(tick.core(), CoreId::new(1));
        assert_eq!(tick.tick(), 2);
        assert_eq!(tick.timestamp(), 3);
    }

    #[test]
    fn tick_counter_records_monotonic_ticks() {
        let counter = TickCounter::new();

        let first = counter.record_tick(CoreId::new(0), 10);
        let second = counter.record_tick(CoreId::new(0), 20);

        assert_eq!(first.map(TimerTick::tick), Ok(1));
        assert_eq!(second.map(TimerTick::tick), Ok(2));
        assert_eq!(counter.ticks(), 2);
    }
}
