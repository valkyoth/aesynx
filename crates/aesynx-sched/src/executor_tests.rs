use aesynx_abi::{CoreId, TaskId};
use aesynx_telemetry::{SchedulerDecisionReason, SchedulerTelemetry, TelemetryError};

use crate::{ExecutorError, LocalExecutor, Priority, Task, TaskQueueError, TimeBudget};

fn task(id: u64, core: u32) -> Task {
    let priority = match Priority::new(1) {
        Ok(priority) => priority,
        Err(_) => Priority::MIN,
    };
    let budget = match TimeBudget::new(10) {
        Ok(budget) => budget,
        Err(_) => TimeBudget::ZERO,
    };
    Task::new(TaskId::new(id), CoreId::new(core), priority, budget)
}

#[test]
fn cooperative_executor_round_robins_yielded_tasks() {
    let mut executor = match LocalExecutor::<3, 1>::new(CoreId::new(0)) {
        Ok(executor) => executor,
        Err(error) => return assert_eq!(error, ExecutorError::Queue(TaskQueueError::QueueEmpty)),
    };

    assert_eq!(executor.spawn(task(1, 0)), Ok(()));
    assert_eq!(executor.spawn(task(2, 0)), Ok(()));

    assert_eq!(executor.dispatch_next(), Ok(TaskId::new(1)));
    assert_eq!(executor.yield_current(), Ok(()));
    assert_eq!(executor.dispatch_next(), Ok(TaskId::new(2)));
    assert_eq!(executor.yield_current(), Ok(()));
    assert_eq!(executor.dispatch_next(), Ok(TaskId::new(1)));

    let status = executor.status();
    assert_eq!(status.current_task, Some(TaskId::new(1)));
    assert_eq!(status.dispatched, 3);
    assert_eq!(status.yielded, 2);
    assert_eq!(status.run_queue_len, 1);
}

#[test]
fn cooperative_executor_sleeps_and_wakes_timer_waiters() {
    let mut executor = match LocalExecutor::<3, 1>::new(CoreId::new(0)) {
        Ok(executor) => executor,
        Err(error) => return assert_eq!(error, ExecutorError::Queue(TaskQueueError::QueueEmpty)),
    };

    assert_eq!(executor.spawn(task(1, 0)), Ok(()));
    assert_eq!(executor.spawn(task(2, 0)), Ok(()));
    assert_eq!(executor.dispatch_next(), Ok(TaskId::new(1)));
    assert_eq!(executor.sleep_current_on_timer(), Ok(()));

    let sleeping = executor.status();
    assert_eq!(sleeping.current_task, None);
    assert_eq!(sleeping.timer_wait_len, 1);
    assert_eq!(sleeping.slept, 1);

    assert_eq!(executor.dispatch_next(), Ok(TaskId::new(2)));
    assert_eq!(executor.yield_current(), Ok(()));
    assert_eq!(executor.wake_one_timer(), Ok(TaskId::new(1)));
    assert_eq!(executor.dispatch_next(), Ok(TaskId::new(2)));
    assert_eq!(executor.yield_current(), Ok(()));
    assert_eq!(executor.dispatch_next(), Ok(TaskId::new(1)));

    let status = executor.status();
    assert_eq!(status.current_task, Some(TaskId::new(1)));
    assert_eq!(status.timer_wait_len, 0);
    assert_eq!(status.slept, 1);
    assert_eq!(status.woke, 1);
}

#[test]
fn cooperative_executor_records_round_robin_decision_telemetry() {
    let mut executor = match LocalExecutor::<3, 1>::new(CoreId::new(0)) {
        Ok(executor) => executor,
        Err(error) => return assert_eq!(error, ExecutorError::Queue(TaskQueueError::QueueEmpty)),
    };
    let mut telemetry = match SchedulerTelemetry::<3>::new() {
        Ok(telemetry) => telemetry,
        Err(error) => return assert_eq!(error, TelemetryError::TelemetryCapacityZero),
    };

    assert_eq!(executor.spawn(task(1, 0)), Ok(()));
    assert_eq!(executor.spawn(task(2, 0)), Ok(()));
    assert_eq!(
        executor.dispatch_next_with_telemetry(&mut telemetry),
        Ok(TaskId::new(1))
    );
    assert_eq!(executor.yield_current(), Ok(()));
    assert_eq!(
        executor.dispatch_next_with_telemetry(&mut telemetry),
        Ok(TaskId::new(2))
    );

    assert_eq!(telemetry.len(), 2);
    assert_eq!(
        telemetry.get(0).map(|record| record.reason()),
        Some(SchedulerDecisionReason::RoundRobinRunnable)
    );
    assert_eq!(
        telemetry.get(0).map(|record| record.selected_task()),
        Some(TaskId::new(1))
    );
    assert_eq!(
        telemetry.get(1).map(|record| record.selected_task()),
        Some(TaskId::new(2))
    );
    assert_eq!(
        telemetry.get(0).map(|record| record.runnable_before()),
        Some(2)
    );
}

#[test]
fn cooperative_executor_full_telemetry_buffer_does_not_dispatch() {
    let mut executor = match LocalExecutor::<2, 1>::new(CoreId::new(0)) {
        Ok(executor) => executor,
        Err(error) => return assert_eq!(error, ExecutorError::Queue(TaskQueueError::QueueEmpty)),
    };
    let mut telemetry = match SchedulerTelemetry::<1>::new() {
        Ok(telemetry) => telemetry,
        Err(error) => return assert_eq!(error, TelemetryError::TelemetryCapacityZero),
    };

    assert_eq!(executor.spawn(task(1, 0)), Ok(()));
    assert_eq!(executor.spawn(task(2, 0)), Ok(()));
    assert_eq!(
        executor.dispatch_next_with_telemetry(&mut telemetry),
        Ok(TaskId::new(1))
    );
    assert_eq!(executor.yield_current(), Ok(()));
    let before = executor.status();

    assert_eq!(
        executor.dispatch_next_with_telemetry(&mut telemetry),
        Err(ExecutorError::Telemetry(
            TelemetryError::TelemetryBufferFull
        ))
    );
    assert_eq!(executor.status(), before);
}

#[test]
fn cooperative_executor_rejects_nested_dispatch_without_mutation() {
    let mut executor = match LocalExecutor::<2, 1>::new(CoreId::new(0)) {
        Ok(executor) => executor,
        Err(error) => return assert_eq!(error, ExecutorError::Queue(TaskQueueError::QueueEmpty)),
    };

    assert_eq!(executor.spawn(task(1, 0)), Ok(()));
    assert_eq!(executor.spawn(task(2, 0)), Ok(()));
    assert_eq!(executor.dispatch_next(), Ok(TaskId::new(1)));
    let before = executor.status();

    assert_eq!(
        executor.dispatch_next(),
        Err(ExecutorError::TaskAlreadyRunning)
    );
    assert_eq!(executor.status(), before);
}

#[test]
fn cooperative_executor_failed_sleep_keeps_current_task() {
    let mut executor = match LocalExecutor::<2, 1>::new(CoreId::new(0)) {
        Ok(executor) => executor,
        Err(error) => return assert_eq!(error, ExecutorError::Queue(TaskQueueError::QueueEmpty)),
    };

    assert_eq!(executor.spawn(task(1, 0)), Ok(()));
    assert_eq!(executor.spawn(task(2, 0)), Ok(()));
    assert_eq!(executor.dispatch_next(), Ok(TaskId::new(1)));
    assert_eq!(executor.sleep_current_on_timer(), Ok(()));
    assert_eq!(executor.dispatch_next(), Ok(TaskId::new(2)));
    let before = executor.status();

    assert_eq!(
        executor.sleep_current_on_timer(),
        Err(ExecutorError::Queue(TaskQueueError::QueueFull))
    );
    assert_eq!(executor.status(), before);
}

#[test]
fn cooperative_executor_dispatch_counter_overflow_is_atomic() {
    let mut executor = match LocalExecutor::<2, 1>::new(CoreId::new(0)) {
        Ok(executor) => executor,
        Err(error) => return assert_eq!(error, ExecutorError::Queue(TaskQueueError::QueueEmpty)),
    };

    assert_eq!(executor.spawn(task(1, 0)), Ok(()));
    executor.set_counters_for_test(u64::MAX, 0, 0, 0);
    let before = executor.status();

    assert_eq!(
        executor.dispatch_next(),
        Err(ExecutorError::CounterOverflow)
    );
    assert_eq!(executor.status(), before);
}

#[test]
fn cooperative_executor_yield_counter_overflow_is_atomic() {
    let mut executor = match LocalExecutor::<2, 1>::new(CoreId::new(0)) {
        Ok(executor) => executor,
        Err(error) => return assert_eq!(error, ExecutorError::Queue(TaskQueueError::QueueEmpty)),
    };

    assert_eq!(executor.spawn(task(1, 0)), Ok(()));
    assert_eq!(executor.dispatch_next(), Ok(TaskId::new(1)));
    executor.set_counters_for_test(1, u64::MAX, 0, 0);
    let before = executor.status();

    assert_eq!(
        executor.yield_current(),
        Err(ExecutorError::CounterOverflow)
    );
    assert_eq!(executor.status(), before);
}

#[test]
fn cooperative_executor_sleep_counter_overflow_is_atomic() {
    let mut executor = match LocalExecutor::<2, 1>::new(CoreId::new(0)) {
        Ok(executor) => executor,
        Err(error) => return assert_eq!(error, ExecutorError::Queue(TaskQueueError::QueueEmpty)),
    };

    assert_eq!(executor.spawn(task(1, 0)), Ok(()));
    assert_eq!(executor.dispatch_next(), Ok(TaskId::new(1)));
    executor.set_counters_for_test(1, 0, u64::MAX, 0);
    let before = executor.status();

    assert_eq!(
        executor.sleep_current_on_timer(),
        Err(ExecutorError::CounterOverflow)
    );
    assert_eq!(executor.status(), before);
}

#[test]
fn cooperative_executor_wake_counter_overflow_is_atomic() {
    let mut executor = match LocalExecutor::<2, 1>::new(CoreId::new(0)) {
        Ok(executor) => executor,
        Err(error) => return assert_eq!(error, ExecutorError::Queue(TaskQueueError::QueueEmpty)),
    };

    assert_eq!(executor.spawn(task(1, 0)), Ok(()));
    assert_eq!(executor.dispatch_next(), Ok(TaskId::new(1)));
    assert_eq!(executor.sleep_current_on_timer(), Ok(()));
    executor.set_counters_for_test(1, 0, 1, u64::MAX);
    let before = executor.status();

    assert_eq!(
        executor.wake_one_timer(),
        Err(ExecutorError::CounterOverflow)
    );
    assert_eq!(executor.status(), before);
}
