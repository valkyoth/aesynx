use core::fmt;

use aesynx_abi::{CoreId, TaskId};

pub const MAX_PRIORITY: u8 = 127;
pub const MAX_TASK_BUDGET_TICKS: u64 = 1_000_000;

#[derive(Eq, PartialEq)]
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
    pub const fn id(&self) -> TaskId {
        self.id
    }

    #[must_use]
    pub const fn owner_core(&self) -> CoreId {
        self.owner_core
    }

    #[must_use]
    pub const fn state(&self) -> TaskState {
        self.state
    }

    #[must_use]
    pub const fn priority(&self) -> Priority {
        self.priority
    }

    #[must_use]
    pub const fn budget(&self) -> TimeBudget {
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

impl fmt::Debug for Task {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Task")
            .field("id", &"<redacted>")
            .field("owner_core", &self.owner_core)
            .field("state", &self.state)
            .field("priority", &self.priority)
            .field("budget", &self.budget)
            .finish()
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

/// Scheduling priority.
///
/// Higher values run first: `Priority(127)` has higher urgency than
/// `Priority(0)`. Value 0 is minimum priority and `MAX_PRIORITY` is maximum.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Priority(u8);

impl Priority {
    pub const MIN: Self = Self(0);

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
    pub const ZERO: Self = Self { ticks: 0 };

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
    if matches!(current, TaskState::Dead) {
        return false;
    }

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
