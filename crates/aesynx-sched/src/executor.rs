use aesynx_abi::{CoreId, TaskId};

use crate::{
    LocalRunQueue, SchedError, Task, TaskQueueError, TaskRejected, TaskState, WaitQueue, WaitReason,
};

#[derive(Debug, Eq, PartialEq)]
pub struct LocalExecutor<const RUN_CAPACITY: usize, const TIMER_CAPACITY: usize> {
    run_queue: LocalRunQueue<RUN_CAPACITY>,
    timer_wait: WaitQueue<TIMER_CAPACITY>,
    current: Option<Task>,
    dispatched: u64,
    yielded: u64,
    slept: u64,
    woke: u64,
}

impl<const RUN_CAPACITY: usize, const TIMER_CAPACITY: usize>
    LocalExecutor<RUN_CAPACITY, TIMER_CAPACITY>
{
    pub fn new(owner_core: CoreId) -> Result<Self, ExecutorError> {
        Ok(Self {
            run_queue: LocalRunQueue::new(owner_core).map_err(ExecutorError::Queue)?,
            timer_wait: WaitQueue::new(WaitReason::Timer).map_err(ExecutorError::Queue)?,
            current: None,
            dispatched: 0,
            yielded: 0,
            slept: 0,
            woke: 0,
        })
    }

    pub fn spawn(&mut self, task: Task) -> Result<(), TaskRejected> {
        self.run_queue.push(task)
    }

    pub fn dispatch_next(&mut self) -> Result<TaskId, ExecutorError> {
        if self.current.is_some() {
            return Err(ExecutorError::TaskAlreadyRunning);
        }

        let mut task = self.run_queue.pop().map_err(ExecutorError::Queue)?;
        let id = task.id();
        if let Err(error) = task.transition(TaskState::Running) {
            self.restore_runnable_task(task)?;
            return Err(ExecutorError::Transition(error));
        }

        self.current = Some(task);
        self.dispatched = self
            .dispatched
            .checked_add(1)
            .ok_or(ExecutorError::CounterOverflow)?;
        Ok(id)
    }

    pub fn yield_current(&mut self) -> Result<(), ExecutorError> {
        let Some(mut task) = self.current.take() else {
            return Err(ExecutorError::NoCurrentTask);
        };

        if let Err(error) = task.transition(TaskState::Runnable) {
            self.current = Some(task);
            return Err(ExecutorError::Transition(error));
        }

        match self.run_queue.push(task) {
            Ok(()) => {}
            Err(rejected) => {
                let error = rejected.error();
                self.restore_current_from_runnable(rejected.into_task())?;
                return Err(ExecutorError::Queue(error));
            }
        }

        self.yielded = self
            .yielded
            .checked_add(1)
            .ok_or(ExecutorError::CounterOverflow)?;
        Ok(())
    }

    pub fn sleep_current_on_timer(&mut self) -> Result<(), ExecutorError> {
        let Some(mut task) = self.current.take() else {
            return Err(ExecutorError::NoCurrentTask);
        };

        let task_id = task.id();
        if self.timer_wait.contains(task_id) {
            self.current = Some(task);
            return Err(ExecutorError::Queue(TaskQueueError::DuplicateTask));
        }
        if self.timer_wait.status().len == self.timer_wait.status().capacity {
            self.current = Some(task);
            return Err(ExecutorError::Queue(TaskQueueError::QueueFull));
        }

        if let Err(error) = task.transition(TaskState::WaitingOnTimer) {
            self.current = Some(task);
            return Err(ExecutorError::Transition(error));
        }

        match self.timer_wait.push(task) {
            Ok(()) => {}
            Err(rejected) => {
                let error = rejected.error();
                self.restore_current_from_timer_wait(rejected.into_task())?;
                return Err(ExecutorError::Queue(error));
            }
        }

        self.slept = self
            .slept
            .checked_add(1)
            .ok_or(ExecutorError::CounterOverflow)?;
        Ok(())
    }

    pub fn wake_one_timer(&mut self) -> Result<TaskId, ExecutorError> {
        let task_id = self
            .timer_wait
            .front_task_id()
            .ok_or(ExecutorError::Queue(TaskQueueError::QueueEmpty))?;

        if self.run_queue.contains(task_id) {
            return Err(ExecutorError::Queue(TaskQueueError::DuplicateTask));
        }
        if self.run_queue.status().len == self.run_queue.status().capacity {
            return Err(ExecutorError::Queue(TaskQueueError::QueueFull));
        }

        let task = self.timer_wait.wake_one().map_err(ExecutorError::Queue)?;
        let id = task.id();
        match self.run_queue.push(task) {
            Ok(()) => {}
            Err(rejected) => {
                let error = rejected.error();
                self.restore_timer_wait_from_runnable(rejected.into_task())?;
                return Err(ExecutorError::Queue(error));
            }
        }

        self.woke = self
            .woke
            .checked_add(1)
            .ok_or(ExecutorError::CounterOverflow)?;
        Ok(id)
    }

    #[must_use]
    pub fn status(&self) -> ExecutorStatus {
        ExecutorStatus {
            run_queue_len: self.run_queue.status().len,
            timer_wait_len: self.timer_wait.status().len,
            current_task: self.current.as_ref().map(Task::id),
            dispatched: self.dispatched,
            yielded: self.yielded,
            slept: self.slept,
            woke: self.woke,
        }
    }

    fn restore_runnable_task(&mut self, task: Task) -> Result<(), ExecutorError> {
        self.run_queue
            .push(task)
            .map_err(|rejected| ExecutorError::RestoreFailed(rejected.error()))
    }

    fn restore_current_from_runnable(&mut self, mut task: Task) -> Result<(), ExecutorError> {
        task.transition(TaskState::Running)
            .map_err(ExecutorError::Transition)?;
        self.current = Some(task);
        Ok(())
    }

    fn restore_current_from_timer_wait(&mut self, mut task: Task) -> Result<(), ExecutorError> {
        task.transition(TaskState::Runnable)
            .map_err(ExecutorError::Transition)?;
        task.transition(TaskState::Running)
            .map_err(ExecutorError::Transition)?;
        self.current = Some(task);
        Ok(())
    }

    fn restore_timer_wait_from_runnable(&mut self, mut task: Task) -> Result<(), ExecutorError> {
        task.transition(TaskState::Running)
            .map_err(ExecutorError::Transition)?;
        task.transition(TaskState::WaitingOnTimer)
            .map_err(ExecutorError::Transition)?;
        self.timer_wait
            .push(task)
            .map_err(|rejected| ExecutorError::RestoreFailed(rejected.error()))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutorStatus {
    pub run_queue_len: usize,
    pub timer_wait_len: usize,
    pub current_task: Option<TaskId>,
    pub dispatched: u64,
    pub yielded: u64,
    pub slept: u64,
    pub woke: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutorError {
    Queue(TaskQueueError),
    Transition(SchedError),
    TaskAlreadyRunning,
    NoCurrentTask,
    CounterOverflow,
    RestoreFailed(TaskQueueError),
}
