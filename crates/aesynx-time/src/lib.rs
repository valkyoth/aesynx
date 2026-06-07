#![no_std]
#![deny(unsafe_code)]

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
    pub core: aesynx_abi::CoreId,
    pub tick: u64,
    pub timestamp: u64,
}
