#![no_std]
#![deny(unsafe_code)]

use aesynx_abi::{CoreId, TaskId};

pub const MAX_PRIORITY: u8 = 127;
pub const MAX_TASK_BUDGET_TICKS: u64 = 1_000_000;

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
    pub const fn new(value: u8) -> Result<Self, SchedError> {
        if value > MAX_PRIORITY {
            return Err(SchedError::PriorityOutOfRange);
        }

        Ok(Self(value))
    }

    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimeBudget {
    ticks: u64,
}

impl TimeBudget {
    pub const fn new(ticks: u64) -> Result<Self, SchedError> {
        if ticks > MAX_TASK_BUDGET_TICKS {
            return Err(SchedError::BudgetExceedsLimit);
        }

        Ok(Self { ticks })
    }

    #[must_use]
    pub const fn ticks(self) -> u64 {
        self.ticks
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedError {
    PriorityOutOfRange,
    BudgetExceedsLimit,
}

#[cfg(test)]
mod tests {
    use super::{MAX_PRIORITY, MAX_TASK_BUDGET_TICKS, Priority, SchedError, TimeBudget};

    #[test]
    fn priority_rejects_user_values_above_limit() {
        assert_eq!(
            Priority::new(MAX_PRIORITY + 1),
            Err(SchedError::PriorityOutOfRange)
        );
    }

    #[test]
    fn time_budget_rejects_overlong_budget() {
        assert_eq!(
            TimeBudget::new(MAX_TASK_BUDGET_TICKS + 1),
            Err(SchedError::BudgetExceedsLimit)
        );
    }

    #[test]
    fn bounded_scheduler_values_expose_raw_values() {
        assert_eq!(Priority::new(1).map(Priority::get), Ok(1));
        assert_eq!(TimeBudget::new(10).map(TimeBudget::ticks), Ok(10));
    }
}
