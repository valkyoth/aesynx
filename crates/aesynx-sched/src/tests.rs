use aesynx_abi::{CoreId, TaskId};
use core::fmt::{self, Write};

use crate::{
    LocalRunQueue, MAX_PRIORITY, MAX_TASK_BUDGET_TICKS, Priority, SchedError, Task, TaskQueueError,
    TaskRejected, TaskState, TimeBudget, WaitQueue, WaitReason,
};

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
fn priority_ordering_uses_higher_values_for_higher_urgency() {
    let background = match Priority::new(0) {
        Ok(priority) => priority,
        Err(error) => return assert_eq!(error, SchedError::PriorityOutOfRange),
    };
    let urgent = match Priority::new(MAX_PRIORITY) {
        Ok(priority) => priority,
        Err(error) => return assert_eq!(error, SchedError::PriorityOutOfRange),
    };

    assert!(urgent > background);
}

#[test]
fn task_state_transitions_are_checked() {
    let mut task = task(1, 0);

    assert_eq!(
        task.transition(TaskState::WaitingOnMessage),
        Err(SchedError::InvalidStateTransition)
    );
    assert_eq!(task.transition(TaskState::Running), Ok(()));
    assert_eq!(task.transition(TaskState::WaitingOnMessage), Ok(()));
    assert_eq!(task.transition(TaskState::Runnable), Ok(()));
    assert_eq!(task.transition(TaskState::Dead), Ok(()));
    assert_eq!(
        task.transition(TaskState::Dead),
        Err(SchedError::InvalidStateTransition)
    );
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

#[test]
fn task_debug_redacts_task_identity() {
    let mut debug = TestBuffer::new();
    assert_eq!(write!(&mut debug, "{:?}", task(0xfeed_beef, 0)), Ok(()));

    assert!(debug.contains(b"<redacted>"));
    assert!(!debug.contains(b"4276993775"));
    assert!(!debug.contains(b"feed"));
}

#[test]
fn local_run_queue_preserves_fifo_for_runnable_local_tasks() {
    let mut queue = match LocalRunQueue::<3>::new(CoreId::new(0)) {
        Ok(queue) => queue,
        Err(error) => return assert_eq!(error, TaskQueueError::QueueCapacityZero),
    };

    assert_eq!(queue.push(task(1, 0)), Ok(()));
    assert_eq!(queue.push(task(2, 0)), Ok(()));
    assert_eq!(queue.push(task(3, 0)), Ok(()));

    assert_eq!(queue.pop().map(|task| task.id()), Ok(TaskId::new(1)));
    assert_eq!(queue.pop().map(|task| task.id()), Ok(TaskId::new(2)));
    assert_eq!(queue.pop().map(|task| task.id()), Ok(TaskId::new(3)));
    assert_eq!(queue.pop(), Err(TaskQueueError::QueueEmpty));
}

#[test]
fn local_run_queue_rejects_invalid_tasks_without_mutation() {
    let mut queue = match LocalRunQueue::<2>::new(CoreId::new(0)) {
        Ok(queue) => queue,
        Err(error) => return assert_eq!(error, TaskQueueError::QueueCapacityZero),
    };
    let before = queue.status();
    let mut waiting = task(1, 0);
    assert_eq!(waiting.transition(TaskState::Running), Ok(()));
    assert_eq!(waiting.transition(TaskState::WaitingOnMessage), Ok(()));

    assert_rejected(
        queue.push(task(0, 0)),
        TaskQueueError::TaskIdZero,
        TaskId::new(0),
    );
    assert_rejected(
        queue.push(task(1, 1)),
        TaskQueueError::WrongCore,
        TaskId::new(1),
    );
    assert_rejected(
        queue.push(waiting),
        TaskQueueError::TaskNotRunnable,
        TaskId::new(1),
    );
    assert_eq!(queue.status(), before);
}

#[test]
fn local_run_queue_full_and_duplicate_pushes_do_not_mutate() {
    let mut queue = match LocalRunQueue::<2>::new(CoreId::new(0)) {
        Ok(queue) => queue,
        Err(error) => return assert_eq!(error, TaskQueueError::QueueCapacityZero),
    };

    assert_eq!(queue.push(task(1, 0)), Ok(()));
    let before_duplicate = queue.status();
    assert_rejected(
        queue.push(task(1, 0)),
        TaskQueueError::DuplicateTask,
        TaskId::new(1),
    );
    assert_eq!(queue.status(), before_duplicate);

    assert_eq!(queue.push(task(2, 0)), Ok(()));
    let before_full = queue.status();
    assert_rejected(
        queue.push(task(3, 0)),
        TaskQueueError::QueueFull,
        TaskId::new(3),
    );
    assert_eq!(queue.status(), before_full);
}

#[test]
fn wait_queue_accepts_matching_wait_state_and_wakes_to_runnable() {
    let mut queue = match WaitQueue::<2>::new(WaitReason::Message) {
        Ok(queue) => queue,
        Err(error) => return assert_eq!(error, TaskQueueError::QueueCapacityZero),
    };
    let mut waiting = task(1, 0);

    assert_eq!(waiting.transition(TaskState::Running), Ok(()));
    assert_eq!(waiting.transition(TaskState::WaitingOnMessage), Ok(()));
    assert_eq!(queue.push(waiting), Ok(()));

    let woken = match queue.wake_one() {
        Ok(task) => task,
        Err(error) => return assert_eq!(error, TaskQueueError::QueueEmpty),
    };
    assert_eq!(woken.id(), TaskId::new(1));
    assert_eq!(woken.state(), TaskState::Runnable);
    assert_eq!(queue.status().len, 0);
}

#[test]
fn wait_queue_rejects_wrong_reason_without_mutation() {
    let mut queue = match WaitQueue::<2>::new(WaitReason::Timer) {
        Ok(queue) => queue,
        Err(error) => return assert_eq!(error, TaskQueueError::QueueCapacityZero),
    };
    let mut waiting = task(1, 0);
    assert_eq!(waiting.transition(TaskState::Running), Ok(()));
    assert_eq!(waiting.transition(TaskState::WaitingOnMessage), Ok(()));
    let before = queue.status();

    assert_rejected(
        queue.push(waiting),
        TaskQueueError::WaitReasonMismatch,
        TaskId::new(1),
    );
    assert_eq!(queue.status(), before);
}

#[test]
fn wait_queue_failed_wake_transition_restores_task() {
    let mut queue = match WaitQueue::<2>::new(WaitReason::Message) {
        Ok(queue) => queue,
        Err(error) => return assert_eq!(error, TaskQueueError::QueueCapacityZero),
    };
    let runnable = task(1, 0);
    assert_eq!(queue.inject_head_for_test(runnable), Ok(()));
    let before = queue.status();

    assert_eq!(queue.wake_one(), Err(TaskQueueError::InvalidWakeTransition));
    assert_eq!(queue.status(), before);
    assert!(queue.contains(TaskId::new(1)));
}

#[test]
fn zero_capacity_task_queues_are_rejected() {
    assert_eq!(
        LocalRunQueue::<0>::new(CoreId::new(0)),
        Err(TaskQueueError::QueueCapacityZero)
    );
    assert_eq!(
        WaitQueue::<0>::new(WaitReason::Object),
        Err(TaskQueueError::QueueCapacityZero)
    );
}

struct TestBuffer {
    bytes: [u8; 256],
    len: usize,
}

impl TestBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; 256],
            len: 0,
        }
    }

    fn contains(&self, needle: &[u8]) -> bool {
        if needle.is_empty() {
            return true;
        }
        if needle.len() > self.len {
            return false;
        }

        let mut start = 0usize;
        while start + needle.len() <= self.len {
            if &self.bytes[start..start + needle.len()] == needle {
                return true;
            }
            start += 1;
        }

        false
    }
}

impl Write for TestBuffer {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let bytes = value.as_bytes();
        let Some(end) = self.len.checked_add(bytes.len()) else {
            return Err(fmt::Error);
        };
        if end > self.bytes.len() {
            return Err(fmt::Error);
        }

        self.bytes[self.len..end].copy_from_slice(bytes);
        self.len = end;
        Ok(())
    }
}

fn assert_rejected(result: Result<(), TaskRejected>, error: TaskQueueError, task_id: TaskId) {
    let rejected = match result {
        Ok(()) => return assert_eq!(Ok::<(), TaskQueueError>(()), Err(error)),
        Err(rejected) => rejected,
    };

    assert_eq!(rejected.error(), error);
    assert_eq!(rejected.task().id(), task_id);
}
