use aesynx_abi::{ROOT_CORE, TaskId};
use aesynx_sched::{
    ExecutorError, LocalExecutor, Priority, SchedError, Task, TaskQueueError, TaskRejected,
    TimeBudget,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CooperativeSchedSmokeStatus {
    pub task_a_steps: u64,
    pub task_b_steps: u64,
    pub dispatched: u64,
    pub yielded: u64,
    pub slept: u64,
    pub woke: u64,
    pub final_run_queue_len: usize,
    pub final_timer_wait_len: usize,
    pub round_robin_ok: bool,
    pub sleep_wake_ok: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub enum CooperativeSchedSmokeError {
    Priority(SchedError),
    Budget(SchedError),
    Queue(TaskQueueError),
    Executor(ExecutorError),
    UnexpectedState,
}

pub fn run() -> Result<CooperativeSchedSmokeStatus, CooperativeSchedSmokeError> {
    let mut executor =
        LocalExecutor::<4, 2>::new(ROOT_CORE).map_err(CooperativeSchedSmokeError::Executor)?;
    let task_a = TaskId::new(1);
    let task_b = TaskId::new(2);

    executor
        .spawn(task(task_a.get())?)
        .map_err(rejected_queue_error)?;
    executor
        .spawn(task(task_b.get())?)
        .map_err(rejected_queue_error)?;

    let mut sequence = [TaskId::new(0); 4];
    sequence[0] = executor
        .dispatch_next()
        .map_err(CooperativeSchedSmokeError::Executor)?;
    executor
        .yield_current()
        .map_err(CooperativeSchedSmokeError::Executor)?;

    sequence[1] = executor
        .dispatch_next()
        .map_err(CooperativeSchedSmokeError::Executor)?;
    executor
        .sleep_current_on_timer()
        .map_err(CooperativeSchedSmokeError::Executor)?;
    let woken = executor
        .wake_one_timer()
        .map_err(CooperativeSchedSmokeError::Executor)?;

    sequence[2] = executor
        .dispatch_next()
        .map_err(CooperativeSchedSmokeError::Executor)?;
    executor
        .yield_current()
        .map_err(CooperativeSchedSmokeError::Executor)?;

    sequence[3] = executor
        .dispatch_next()
        .map_err(CooperativeSchedSmokeError::Executor)?;
    executor
        .yield_current()
        .map_err(CooperativeSchedSmokeError::Executor)?;

    let status = executor.status();
    let task_a_steps = count_steps(sequence, task_a);
    let task_b_steps = count_steps(sequence, task_b);
    let round_robin_ok =
        sequence == [task_a, task_b, task_a, task_b] && task_a_steps == 2 && task_b_steps == 2;
    let sleep_wake_ok = woken == task_b
        && status.slept == 1
        && status.woke == 1
        && status.timer_wait_len == 0
        && status.current_task.is_none();

    let smoke = CooperativeSchedSmokeStatus {
        task_a_steps,
        task_b_steps,
        dispatched: status.dispatched,
        yielded: status.yielded,
        slept: status.slept,
        woke: status.woke,
        final_run_queue_len: status.run_queue_len,
        final_timer_wait_len: status.timer_wait_len,
        round_robin_ok,
        sleep_wake_ok,
    };
    validate_status(smoke)?;
    Ok(smoke)
}

fn task(id: u64) -> Result<Task, CooperativeSchedSmokeError> {
    let priority = Priority::new(1).map_err(CooperativeSchedSmokeError::Priority)?;
    let budget = TimeBudget::new(10).map_err(CooperativeSchedSmokeError::Budget)?;
    Ok(Task::new(TaskId::new(id), ROOT_CORE, priority, budget))
}

fn count_steps(sequence: [TaskId; 4], id: TaskId) -> u64 {
    let mut count = 0u64;
    for entry in sequence {
        if entry == id {
            count += 1;
        }
    }
    count
}

fn rejected_queue_error(rejected: TaskRejected) -> CooperativeSchedSmokeError {
    CooperativeSchedSmokeError::Queue(rejected.error())
}

fn validate_status(status: CooperativeSchedSmokeStatus) -> Result<(), CooperativeSchedSmokeError> {
    if status.task_a_steps != 2
        || status.task_b_steps != 2
        || status.dispatched != 4
        || status.yielded != 3
        || status.slept != 1
        || status.woke != 1
        || status.final_run_queue_len != 2
        || status.final_timer_wait_len != 0
        || !status.round_robin_ok
        || !status.sleep_wake_ok
    {
        return Err(CooperativeSchedSmokeError::UnexpectedState);
    }

    Ok(())
}
