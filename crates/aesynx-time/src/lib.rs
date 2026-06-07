#![no_std]
#![deny(unsafe_code)]

pub trait ClockSource {
    fn now_ticks(&self) -> u64;
    fn monotonic_ns(&self) -> u64;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimerTick {
    pub core: aesynx_abi::CoreId,
    pub tick: u64,
    pub timestamp: u64,
}
