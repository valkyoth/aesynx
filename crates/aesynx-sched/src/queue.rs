use core::{cell::Cell, marker::PhantomData};

use aesynx_abi::{CoreId, TaskId};

use crate::{Task, TaskState};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueueStatus {
    pub capacity: usize,
    pub len: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub struct LocalRunQueue<const CAPACITY: usize> {
    owner_core: CoreId,
    slots: [Option<Task>; CAPACITY],
    head: usize,
    len: usize,
    _not_sync: PhantomData<Cell<()>>,
}

impl<const CAPACITY: usize> LocalRunQueue<CAPACITY> {
    pub const fn new(owner_core: CoreId) -> Result<Self, TaskQueueError> {
        if CAPACITY == 0 {
            return Err(TaskQueueError::QueueCapacityZero);
        }

        Ok(Self {
            owner_core,
            slots: [const { None }; CAPACITY],
            head: 0,
            len: 0,
            _not_sync: PhantomData,
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

    pub fn push(&mut self, task: Task) -> Result<(), TaskRejected> {
        if let Err(error) = validate_task_id(task.id()) {
            return Err(TaskRejected { error, task });
        }
        if task.owner_core() != self.owner_core {
            return Err(TaskRejected {
                error: TaskQueueError::WrongCore,
                task,
            });
        }
        if task.state() != TaskState::Runnable {
            return Err(TaskRejected {
                error: TaskQueueError::TaskNotRunnable,
                task,
            });
        }
        if self.contains(task.id()) {
            return Err(TaskRejected {
                error: TaskQueueError::DuplicateTask,
                task,
            });
        }
        if self.len == CAPACITY {
            return Err(TaskRejected {
                error: TaskQueueError::QueueFull,
                task,
            });
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
            if let Some(task) = self.slots[index].as_ref()
                && task.id() == id
            {
                return true;
            }
            offset += 1;
        }

        false
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct WaitQueue<const CAPACITY: usize> {
    reason: WaitReason,
    slots: [Option<Task>; CAPACITY],
    head: usize,
    len: usize,
    _not_sync: PhantomData<Cell<()>>,
}

impl<const CAPACITY: usize> WaitQueue<CAPACITY> {
    pub const fn new(reason: WaitReason) -> Result<Self, TaskQueueError> {
        if CAPACITY == 0 {
            return Err(TaskQueueError::QueueCapacityZero);
        }

        Ok(Self {
            reason,
            slots: [const { None }; CAPACITY],
            head: 0,
            len: 0,
            _not_sync: PhantomData,
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

    pub fn push(&mut self, task: Task) -> Result<(), TaskRejected> {
        if let Err(error) = validate_task_id(task.id()) {
            return Err(TaskRejected { error, task });
        }
        if self.reason.task_state() != task.state() {
            return Err(TaskRejected {
                error: TaskQueueError::WaitReasonMismatch,
                task,
            });
        }
        if self.contains(task.id()) {
            return Err(TaskRejected {
                error: TaskQueueError::DuplicateTask,
                task,
            });
        }
        if self.len == CAPACITY {
            return Err(TaskRejected {
                error: TaskQueueError::QueueFull,
                task,
            });
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
        if task.transition(TaskState::Runnable).is_err() {
            self.slots[self.head] = Some(task);
            return Err(TaskQueueError::InvalidWakeTransition);
        }
        self.head = (self.head + 1) % CAPACITY;
        self.len -= 1;
        Ok(task)
    }

    #[must_use]
    pub fn contains(&self, id: TaskId) -> bool {
        let mut offset = 0usize;
        while offset < self.len {
            let index = (self.head + offset) % CAPACITY;
            if let Some(task) = self.slots[index].as_ref()
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

#[derive(Debug, Eq, PartialEq)]
pub struct TaskRejected {
    error: TaskQueueError,
    task: Task,
}

impl TaskRejected {
    #[must_use]
    pub const fn error(&self) -> TaskQueueError {
        self.error
    }

    #[must_use]
    pub const fn task(&self) -> &Task {
        &self.task
    }

    #[must_use]
    pub fn into_task(self) -> Task {
        self.task
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

#[cfg(test)]
impl<const CAPACITY: usize> WaitQueue<CAPACITY> {
    pub(crate) fn inject_head_for_test(&mut self, task: Task) -> Result<(), TaskQueueError> {
        if self.len != 0 {
            return Err(TaskQueueError::QueueFull);
        }

        self.slots[self.head] = Some(task);
        self.len = 1;
        Ok(())
    }
}
