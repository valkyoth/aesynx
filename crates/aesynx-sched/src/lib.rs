#![no_std]
#![deny(unsafe_code)]

use aesynx_abi::{CoreId, TaskId};

pub const MAX_PRIORITY: u8 = 127;
pub const MAX_TASK_BUDGET_TICKS: u64 = 1_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Task {
    id: TaskId,
    owner_core: CoreId,
    state: TaskState,
    priority: Priority,
    budget: TimeBudget,
}

impl Task {
    #[must_use]
    pub const fn new(
        id: TaskId,
        owner_core: CoreId,
        priority: Priority,
        budget: TimeBudget,
    ) -> Self {
        Self {
            id,
            owner_core,
            state: TaskState::Runnable,
            priority,
            budget,
        }
    }

    #[must_use]
    pub const fn id(self) -> TaskId {
        self.id
    }

    #[must_use]
    pub const fn owner_core(self) -> CoreId {
        self.owner_core
    }

    #[must_use]
    pub const fn state(self) -> TaskState {
        self.state
    }

    #[must_use]
    pub const fn priority(self) -> Priority {
        self.priority
    }

    #[must_use]
    pub const fn budget(self) -> TimeBudget {
        self.budget
    }

    pub const fn transition(&mut self, next: TaskState) -> Result<(), SchedError> {
        if !task_transition_allowed(self.state, next) {
            return Err(SchedError::InvalidStateTransition);
        }

        self.state = next;
        Ok(())
    }
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
    InvalidStateTransition,
}

const fn task_transition_allowed(current: TaskState, next: TaskState) -> bool {
    matches!(
        (current, next),
        (TaskState::Runnable, TaskState::Running)
            | (TaskState::Running, TaskState::Runnable)
            | (TaskState::Running, TaskState::WaitingOnMessage)
            | (TaskState::Running, TaskState::WaitingOnTimer)
            | (TaskState::Running, TaskState::WaitingOnObject)
            | (TaskState::Running, TaskState::Suspended)
            | (TaskState::WaitingOnMessage, TaskState::Runnable)
            | (TaskState::WaitingOnTimer, TaskState::Runnable)
            | (TaskState::WaitingOnObject, TaskState::Runnable)
            | (TaskState::Suspended, TaskState::Runnable)
            | (_, TaskState::Dead)
    )
}

#[cfg(test)]
mod tests {
    use aesynx_abi::{CoreId, TaskId};

    use super::{
        MAX_PRIORITY, MAX_TASK_BUDGET_TICKS, Priority, SchedError, Task, TaskState, TimeBudget,
    };

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

    #[test]
    fn task_state_transitions_are_checked() {
        let priority = match Priority::new(1) {
            Ok(priority) => priority,
            Err(error) => return assert_eq!(error, SchedError::PriorityOutOfRange),
        };
        let budget = match TimeBudget::new(10) {
            Ok(budget) => budget,
            Err(error) => return assert_eq!(error, SchedError::BudgetExceedsLimit),
        };
        let mut task = Task::new(TaskId::new(1), CoreId::new(0), priority, budget);

        assert_eq!(
            task.transition(TaskState::WaitingOnMessage),
            Err(SchedError::InvalidStateTransition)
        );
        assert_eq!(task.transition(TaskState::Running), Ok(()));
        assert_eq!(task.transition(TaskState::WaitingOnMessage), Ok(()));
        assert_eq!(task.transition(TaskState::Runnable), Ok(()));
        assert_eq!(task.transition(TaskState::Dead), Ok(()));
    }

    #[test]
    fn task_identity_and_scheduling_fields_are_read_only() {
        let priority = match Priority::new(1) {
            Ok(priority) => priority,
            Err(error) => return assert_eq!(error, SchedError::PriorityOutOfRange),
        };
        let budget = match TimeBudget::new(10) {
            Ok(budget) => budget,
            Err(error) => return assert_eq!(error, SchedError::BudgetExceedsLimit),
        };
        let task = Task::new(TaskId::new(1), CoreId::new(0), priority, budget);

        assert_eq!(task.id(), TaskId::new(1));
        assert_eq!(task.owner_core(), CoreId::new(0));
        assert_eq!(task.priority(), priority);
        assert_eq!(task.budget(), budget);
    }
}
