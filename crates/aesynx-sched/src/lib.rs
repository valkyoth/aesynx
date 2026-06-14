#![no_std]
#![deny(unsafe_code)]

mod executor;
mod queue;
mod task;

pub use executor::{ExecutorError, ExecutorStatus, LocalExecutor};
pub use queue::{LocalRunQueue, QueueStatus, TaskQueueError, TaskRejected, WaitQueue, WaitReason};
pub use task::{
    MAX_PRIORITY, MAX_TASK_BUDGET_TICKS, Priority, SchedError, Task, TaskState, TimeBudget,
};

#[cfg(test)]
mod executor_tests;
#[cfg(test)]
mod tests;
