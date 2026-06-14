use aesynx_abi::{CoreId, TaskId};

use crate::{Task, TaskState};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueueStatus {
    pub capacity: usize,
    pub len: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalRunQueue<const CAPACITY: usize> {
    owner_core: CoreId,
    slots: [Option<Task>; CAPACITY],
    head: usize,
    len: usize,
}

impl<const CAPACITY: usize> LocalRunQueue<CAPACITY> {
    pub const fn new(owner_core: CoreId) -> Result<Self, TaskQueueError> {
        if CAPACITY == 0 {
            return Err(TaskQueueError::QueueCapacityZero);
        }

        Ok(Self {
            owner_core,
            slots: [None; CAPACITY],
            head: 0,
            len: 0,
        })
    }

    #[must_use]
    pub const fn owner_core(&self) -> CoreId {
        self.owner_core
    }

    #[must_use]
    pub const fn status(&self) -> QueueStatus {
        QueueStatus {
            capacity: CAPACITY,
            len: self.len,
        }
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push(&mut self, task: Task) -> Result<(), TaskQueueError> {
        validate_task_id(task.id())?;
        if task.owner_core() != self.owner_core {
            return Err(TaskQueueError::WrongCore);
        }
        if task.state() != TaskState::Runnable {
            return Err(TaskQueueError::TaskNotRunnable);
        }
        if self.contains(task.id()) {
            return Err(TaskQueueError::DuplicateTask);
        }
        if self.len == CAPACITY {
            return Err(TaskQueueError::QueueFull);
        }

        let tail = (self.head + self.len) % CAPACITY;
        self.slots[tail] = Some(task);
        self.len += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Result<Task, TaskQueueError> {
        if self.len == 0 {
            return Err(TaskQueueError::QueueEmpty);
        }

        let Some(task) = self.slots[self.head].take() else {
            return Err(TaskQueueError::CorruptQueue);
        };
        self.head = (self.head + 1) % CAPACITY;
        self.len -= 1;
        Ok(task)
    }

    #[must_use]
    pub fn contains(&self, id: TaskId) -> bool {
        let mut offset = 0usize;
        while offset < self.len {
            let index = (self.head + offset) % CAPACITY;
            if let Some(task) = self.slots[index]
                && task.id() == id
            {
                return true;
            }
            offset += 1;
        }

        false
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WaitQueue<const CAPACITY: usize> {
    reason: WaitReason,
    slots: [Option<Task>; CAPACITY],
    head: usize,
    len: usize,
}

impl<const CAPACITY: usize> WaitQueue<CAPACITY> {
    pub const fn new(reason: WaitReason) -> Result<Self, TaskQueueError> {
        if CAPACITY == 0 {
            return Err(TaskQueueError::QueueCapacityZero);
        }

        Ok(Self {
            reason,
            slots: [None; CAPACITY],
            head: 0,
            len: 0,
        })
    }

    #[must_use]
    pub const fn reason(&self) -> WaitReason {
        self.reason
    }

    #[must_use]
    pub const fn status(&self) -> QueueStatus {
        QueueStatus {
            capacity: CAPACITY,
            len: self.len,
        }
    }

    pub fn push(&mut self, task: Task) -> Result<(), TaskQueueError> {
        validate_task_id(task.id())?;
        if self.reason.task_state() != task.state() {
            return Err(TaskQueueError::WaitReasonMismatch);
        }
        if self.contains(task.id()) {
            return Err(TaskQueueError::DuplicateTask);
        }
        if self.len == CAPACITY {
            return Err(TaskQueueError::QueueFull);
        }

        let tail = (self.head + self.len) % CAPACITY;
        self.slots[tail] = Some(task);
        self.len += 1;
        Ok(())
    }

    pub fn wake_one(&mut self) -> Result<Task, TaskQueueError> {
        if self.len == 0 {
            return Err(TaskQueueError::QueueEmpty);
        }

        let Some(mut task) = self.slots[self.head].take() else {
            return Err(TaskQueueError::CorruptQueue);
        };
        task.transition(TaskState::Runnable)
            .map_err(|_| TaskQueueError::InvalidWakeTransition)?;
        self.head = (self.head + 1) % CAPACITY;
        self.len -= 1;
        Ok(task)
    }

    #[must_use]
    pub fn contains(&self, id: TaskId) -> bool {
        let mut offset = 0usize;
        while offset < self.len {
            let index = (self.head + offset) % CAPACITY;
            if let Some(task) = self.slots[index]
                && task.id() == id
            {
                return true;
            }
            offset += 1;
        }

        false
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WaitReason {
    Message,
    Timer,
    Object,
}

impl WaitReason {
    const fn task_state(self) -> TaskState {
        match self {
            Self::Message => TaskState::WaitingOnMessage,
            Self::Timer => TaskState::WaitingOnTimer,
            Self::Object => TaskState::WaitingOnObject,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskQueueError {
    QueueCapacityZero,
    QueueFull,
    QueueEmpty,
    TaskIdZero,
    WrongCore,
    TaskNotRunnable,
    DuplicateTask,
    WaitReasonMismatch,
    InvalidWakeTransition,
    CorruptQueue,
}

const fn validate_task_id(id: TaskId) -> Result<(), TaskQueueError> {
    if id.get() == 0 {
        return Err(TaskQueueError::TaskIdZero);
    }

    Ok(())
}
