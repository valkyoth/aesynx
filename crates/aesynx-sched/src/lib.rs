#![no_std]
#![deny(unsafe_code)]

mod queue;
mod task;

pub use queue::{LocalRunQueue, QueueStatus, TaskQueueError, WaitQueue, WaitReason};
pub use task::{
    MAX_PRIORITY, MAX_TASK_BUDGET_TICKS, Priority, SchedError, Task, TaskState, TimeBudget,
};

#[cfg(test)]
mod tests;
