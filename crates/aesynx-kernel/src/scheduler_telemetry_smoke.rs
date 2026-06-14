use aesynx_abi::{ROOT_CORE, TaskId};
use aesynx_sched::{
    ExecutorError, LocalExecutor, Priority, SchedError, Task, TaskQueueError, TaskRejected,
    TimeBudget,
};
use aesynx_telemetry::{
    CoreTelemetry, SchedulerDecisionReason, SchedulerTelemetry, TaskTelemetry, TelemetryError,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SchedulerTelemetrySmokeStatus {
    pub decisions: usize,
    pub task_a_runs: u64,
    pub task_b_runs: u64,
    pub core_run_queue_len: u64,
    pub first_reason_round_robin: bool,
    pub last_reason_round_robin: bool,
    pub trace_ok: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerTelemetrySmokeError {
    Priority(SchedError),
    Budget(SchedError),
    Queue(TaskQueueError),
    Executor(ExecutorError),
    Telemetry(TelemetryError),
    UnexpectedTrace,
}

pub fn run() -> Result<SchedulerTelemetrySmokeStatus, SchedulerTelemetrySmokeError> {
    let mut executor =
        LocalExecutor::<4, 1>::new(ROOT_CORE).map_err(SchedulerTelemetrySmokeError::Executor)?;
    let mut decisions =
        SchedulerTelemetry::<4>::new().map_err(SchedulerTelemetrySmokeError::Telemetry)?;
    let core = CoreTelemetry::default();
    let mut task_a_telemetry = TaskTelemetry::default();
    let mut task_b_telemetry = TaskTelemetry::default();
    let task_a = TaskId::new(1);
    let task_b = TaskId::new(2);

    executor
        .spawn(task(task_a.get())?)
        .map_err(rejected_queue_error)?;
    executor
        .spawn(task(task_b.get())?)
        .map_err(rejected_queue_error)?;

    let mut sequence = [TaskId::new(0); 4];
    sequence[0] = dispatch_and_count(
        &mut executor,
        &mut decisions,
        &mut task_a_telemetry,
        &mut task_b_telemetry,
        task_a,
        task_b,
    )?;
    executor
        .yield_current()
        .map_err(SchedulerTelemetrySmokeError::Executor)?;

    sequence[1] = dispatch_and_count(
        &mut executor,
        &mut decisions,
        &mut task_a_telemetry,
        &mut task_b_telemetry,
        task_a,
        task_b,
    )?;
    executor
        .yield_current()
        .map_err(SchedulerTelemetrySmokeError::Executor)?;

    sequence[2] = dispatch_and_count(
        &mut executor,
        &mut decisions,
        &mut task_a_telemetry,
        &mut task_b_telemetry,
        task_a,
        task_b,
    )?;
    executor
        .yield_current()
        .map_err(SchedulerTelemetrySmokeError::Executor)?;

    sequence[3] = dispatch_and_count(
        &mut executor,
        &mut decisions,
        &mut task_a_telemetry,
        &mut task_b_telemetry,
        task_a,
        task_b,
    )?;
    executor
        .yield_current()
        .map_err(SchedulerTelemetrySmokeError::Executor)?;

    let executor_status = executor.status();
    core.set_run_queue_len(executor_status.run_queue_len as u64);
    let core_snapshot = core.snapshot();
    let task_a_snapshot = task_a_telemetry.snapshot();
    let task_b_snapshot = task_b_telemetry.snapshot();
    let summary = decisions.summary();
    let first_reason_round_robin = decisions
        .get(0)
        .map(|record| record.reason() == SchedulerDecisionReason::RoundRobinRunnable)
        .unwrap_or(false);
    let last_reason_round_robin =
        summary.last_reason == Some(SchedulerDecisionReason::RoundRobinRunnable);
    let trace_ok = sequence == [task_a, task_b, task_a, task_b]
        && decisions.len() == 4
        && task_a_snapshot.scheduled_runs == 2
        && task_b_snapshot.scheduled_runs == 2
        && first_reason_round_robin
        && last_reason_round_robin;

    let status = SchedulerTelemetrySmokeStatus {
        decisions: summary.decisions,
        task_a_runs: task_a_snapshot.scheduled_runs,
        task_b_runs: task_b_snapshot.scheduled_runs,
        core_run_queue_len: core_snapshot.run_queue_len,
        first_reason_round_robin,
        last_reason_round_robin,
        trace_ok,
    };
    validate_status(status)?;
    Ok(status)
}

fn dispatch_and_count<const RUN_CAPACITY: usize, const TIMER_CAPACITY: usize>(
    executor: &mut LocalExecutor<RUN_CAPACITY, TIMER_CAPACITY>,
    decisions: &mut SchedulerTelemetry<4>,
    task_a_telemetry: &mut TaskTelemetry,
    task_b_telemetry: &mut TaskTelemetry,
    task_a: TaskId,
    task_b: TaskId,
) -> Result<TaskId, SchedulerTelemetrySmokeError> {
    let dispatched = executor
        .dispatch_next_with_telemetry(decisions)
        .map_err(SchedulerTelemetrySmokeError::Executor)?;
    if dispatched == task_a {
        task_a_telemetry
            .inc_scheduled_runs()
            .map_err(SchedulerTelemetrySmokeError::Telemetry)?;
    } else if dispatched == task_b {
        task_b_telemetry
            .inc_scheduled_runs()
            .map_err(SchedulerTelemetrySmokeError::Telemetry)?;
    } else {
        return Err(SchedulerTelemetrySmokeError::UnexpectedTrace);
    }
    Ok(dispatched)
}

fn task(id: u64) -> Result<Task, SchedulerTelemetrySmokeError> {
    let priority = Priority::new(1).map_err(SchedulerTelemetrySmokeError::Priority)?;
    let budget = TimeBudget::new(10).map_err(SchedulerTelemetrySmokeError::Budget)?;
    Ok(Task::new(TaskId::new(id), ROOT_CORE, priority, budget))
}

fn rejected_queue_error(rejected: TaskRejected) -> SchedulerTelemetrySmokeError {
    SchedulerTelemetrySmokeError::Queue(rejected.error())
}

fn validate_status(
    status: SchedulerTelemetrySmokeStatus,
) -> Result<(), SchedulerTelemetrySmokeError> {
    if status.decisions != 4
        || status.task_a_runs != 2
        || status.task_b_runs != 2
        || !status.first_reason_round_robin
        || !status.last_reason_round_robin
        || !status.trace_ok
    {
        return Err(SchedulerTelemetrySmokeError::UnexpectedTrace);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::run;

    #[test]
    fn scheduler_telemetry_smoke_records_round_robin_trace() {
        let result = run();
        assert!(result.is_ok());
        let status = match result {
            Ok(status) => status,
            Err(_error) => return,
        };

        assert_eq!(status.decisions, 4);
        assert_eq!(status.task_a_runs, 2);
        assert_eq!(status.task_b_runs, 2);
        assert!(status.first_reason_round_robin);
        assert!(status.last_reason_round_robin);
        assert!(status.trace_ok);
    }
}
