use aesynx_abi::{CoreId, ROOT_CORE, TaskId};
use aesynx_sched::{
    LocalRunQueue, Priority, SchedError, Task, TaskQueueError, TaskState, TimeBudget, WaitQueue,
    WaitReason,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TaskSmokeStatus {
    pub created_tasks: usize,
    pub runnable_before: usize,
    pub runnable_after: usize,
    pub message_wait_before: usize,
    pub message_wait_after: usize,
    pub timer_wait_before: usize,
    pub timer_wait_after: usize,
    pub fifo_ok: bool,
    pub wake_ok: bool,
    pub wrong_core_denied: bool,
    pub zero_id_denied: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskSmokeError {
    Priority(SchedError),
    Budget(SchedError),
    Transition(SchedError),
    Queue(TaskQueueError),
    UnexpectedState,
}

pub fn run() -> Result<TaskSmokeStatus, TaskSmokeError> {
    let mut run_queue = LocalRunQueue::<4>::new(ROOT_CORE).map_err(TaskSmokeError::Queue)?;
    let mut message_wait =
        WaitQueue::<2>::new(WaitReason::Message).map_err(TaskSmokeError::Queue)?;
    let mut timer_wait = WaitQueue::<2>::new(WaitReason::Timer).map_err(TaskSmokeError::Queue)?;

    let first = task(1, ROOT_CORE)?;
    let second = task(2, ROOT_CORE)?;
    let wrong_core = task(3, CoreId::new(1))?;
    let zero_id = task(0, ROOT_CORE)?;

    run_queue.push(first).map_err(TaskSmokeError::Queue)?;
    run_queue.push(second).map_err(TaskSmokeError::Queue)?;
    let runnable_before = run_queue.status().len;
    let wrong_core_denied = run_queue.push(wrong_core) == Err(TaskQueueError::WrongCore);
    let zero_id_denied = run_queue.push(zero_id) == Err(TaskQueueError::TaskIdZero);

    let popped_first = run_queue.pop().map_err(TaskSmokeError::Queue)?;
    let popped_second = run_queue.pop().map_err(TaskSmokeError::Queue)?;
    let fifo_ok = popped_first.id() == TaskId::new(1) && popped_second.id() == TaskId::new(2);
    run_queue
        .push(popped_first)
        .map_err(TaskSmokeError::Queue)?;
    run_queue
        .push(popped_second)
        .map_err(TaskSmokeError::Queue)?;

    let mut waits_on_message = run_queue.pop().map_err(TaskSmokeError::Queue)?;
    waits_on_message
        .transition(TaskState::Running)
        .map_err(TaskSmokeError::Transition)?;
    waits_on_message
        .transition(TaskState::WaitingOnMessage)
        .map_err(TaskSmokeError::Transition)?;
    message_wait
        .push(waits_on_message)
        .map_err(TaskSmokeError::Queue)?;

    let mut waits_on_timer = run_queue.pop().map_err(TaskSmokeError::Queue)?;
    waits_on_timer
        .transition(TaskState::Running)
        .map_err(TaskSmokeError::Transition)?;
    waits_on_timer
        .transition(TaskState::WaitingOnTimer)
        .map_err(TaskSmokeError::Transition)?;
    timer_wait
        .push(waits_on_timer)
        .map_err(TaskSmokeError::Queue)?;

    let message_wait_before = message_wait.status().len;
    let timer_wait_before = timer_wait.status().len;
    let mut woken = message_wait.wake_one().map_err(TaskSmokeError::Queue)?;
    let wake_ok = woken.state() == TaskState::Runnable && woken.id() == TaskId::new(1);
    woken
        .transition(TaskState::Running)
        .map_err(TaskSmokeError::Transition)?;
    woken
        .transition(TaskState::Runnable)
        .map_err(TaskSmokeError::Transition)?;
    run_queue.push(woken).map_err(TaskSmokeError::Queue)?;

    let status = TaskSmokeStatus {
        created_tasks: 4,
        runnable_before,
        runnable_after: run_queue.status().len,
        message_wait_before,
        message_wait_after: message_wait.status().len,
        timer_wait_before,
        timer_wait_after: timer_wait.status().len,
        fifo_ok,
        wake_ok,
        wrong_core_denied,
        zero_id_denied,
    };

    if status.created_tasks != 4
        || status.runnable_before != 2
        || status.runnable_after != 1
        || status.message_wait_before != 1
        || status.message_wait_after != 0
        || status.timer_wait_before != 1
        || status.timer_wait_after != 1
        || !status.fifo_ok
        || !status.wake_ok
        || !status.wrong_core_denied
        || !status.zero_id_denied
    {
        return Err(TaskSmokeError::UnexpectedState);
    }

    Ok(status)
}

fn task(id: u64, core: CoreId) -> Result<Task, TaskSmokeError> {
    let priority = Priority::new(1).map_err(TaskSmokeError::Priority)?;
    let budget = TimeBudget::new(10).map_err(TaskSmokeError::Budget)?;
    Ok(Task::new(TaskId::new(id), core, priority, budget))
}
