#![no_std]
#![deny(unsafe_code)]

use aesynx_abi::{CoreId, TaskId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Task {
    pub id: TaskId,
    pub owner_core: CoreId,
    pub state: TaskState,
    pub priority: Priority,
    pub budget: TimeBudget,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskState {
    Runnable,
    Running,
    WaitingOnMessage,
    WaitingOnTimer,
    WaitingOnObject,
    Suspended,
    Dead,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Priority(u8);

impl Priority {
    #[must_use]
    pub const fn new(value: u8) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimeBudget {
    pub ticks: u64,
}
