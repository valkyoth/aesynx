use core::fmt;

use aesynx_abi::{CoreId, TaskId};

use crate::{MAX_SCHEDULE_QUEUE_FEATURE, TelemetryError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerDecisionReason {
    RoundRobinRunnable,
}

impl SchedulerDecisionReason {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::RoundRobinRunnable => "round-robin-runnable",
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SchedulerDecisionRecord {
    sequence: u64,
    core: CoreId,
    selected_task: TaskId,
    reason: SchedulerDecisionReason,
    runnable_before: u32,
    runnable_before_saturated: bool,
    timer_wait_before: u32,
    timer_wait_before_saturated: bool,
}

impl SchedulerDecisionRecord {
    #[must_use]
    pub const fn sequence(self) -> u64 {
        self.sequence
    }

    #[must_use]
    pub const fn core(self) -> CoreId {
        self.core
    }

    #[must_use]
    pub const fn selected_task(self) -> TaskId {
        self.selected_task
    }

    #[must_use]
    pub const fn reason(self) -> SchedulerDecisionReason {
        self.reason
    }

    #[must_use]
    pub const fn runnable_before(self) -> u32 {
        self.runnable_before
    }

    #[must_use]
    pub const fn runnable_before_saturated(self) -> bool {
        self.runnable_before_saturated
    }

    #[must_use]
    pub const fn timer_wait_before(self) -> u32 {
        self.timer_wait_before
    }

    #[must_use]
    pub const fn timer_wait_before_saturated(self) -> bool {
        self.timer_wait_before_saturated
    }
}

impl fmt::Debug for SchedulerDecisionRecord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SchedulerDecisionRecord")
            .field("sequence", &self.sequence)
            .field("core", &self.core)
            .field("selected_task", &"<redacted>")
            .field("reason", &self.reason)
            .field("runnable_before", &self.runnable_before)
            .field("runnable_before_saturated", &self.runnable_before_saturated)
            .field("timer_wait_before", &self.timer_wait_before)
            .field(
                "timer_wait_before_saturated",
                &self.timer_wait_before_saturated,
            )
            .finish()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct SchedulerTelemetry<const CAPACITY: usize> {
    records: [Option<SchedulerDecisionRecord>; CAPACITY],
    len: usize,
    next_sequence: u64,
}

impl<const CAPACITY: usize> SchedulerTelemetry<CAPACITY> {
    pub const fn new() -> Result<Self, TelemetryError> {
        if CAPACITY == 0 {
            return Err(TelemetryError::TelemetryCapacityZero);
        }

        Ok(Self {
            records: [const { None }; CAPACITY],
            len: 0,
            next_sequence: 0,
        })
    }

    #[must_use]
    pub const fn can_record(&self) -> bool {
        self.len < CAPACITY && self.next_sequence < u64::MAX
    }

    pub fn record_decision(
        &mut self,
        core: CoreId,
        selected_task: TaskId,
        reason: SchedulerDecisionReason,
        runnable_before: usize,
        timer_wait_before: usize,
    ) -> Result<SchedulerDecisionRecord, TelemetryError> {
        if self.len == CAPACITY {
            return Err(TelemetryError::TelemetryBufferFull);
        }

        let sequence = self.next_sequence;
        self.next_sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or(TelemetryError::CounterOverflow)?;

        let runnable_before = clamp_usize_to_u32(runnable_before, MAX_SCHEDULE_QUEUE_FEATURE);
        let timer_wait_before = clamp_usize_to_u32(timer_wait_before, MAX_SCHEDULE_QUEUE_FEATURE);
        let record = SchedulerDecisionRecord {
            sequence,
            core,
            selected_task,
            reason,
            runnable_before: runnable_before.value,
            runnable_before_saturated: runnable_before.saturated,
            timer_wait_before: timer_wait_before.value,
            timer_wait_before_saturated: timer_wait_before.saturated,
        };
        self.records[self.len] = Some(record);
        self.len += 1;
        Ok(record)
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub const fn capacity(&self) -> usize {
        CAPACITY
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.len == CAPACITY
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<SchedulerDecisionRecord> {
        if index >= self.len {
            return None;
        }
        self.records[index]
    }

    #[must_use]
    pub fn summary(&self) -> SchedulerTelemetrySummary {
        SchedulerTelemetrySummary {
            capacity: CAPACITY,
            decisions: self.len,
            next_sequence: self.next_sequence,
            last_reason: self
                .len
                .checked_sub(1)
                .and_then(|index| self.records[index])
                .map(SchedulerDecisionRecord::reason),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SchedulerTelemetrySummary {
    pub capacity: usize,
    pub decisions: usize,
    pub next_sequence: u64,
    pub last_reason: Option<SchedulerDecisionReason>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ClampedQueueDepth {
    value: u32,
    saturated: bool,
}

const fn clamp_usize_to_u32(value: usize, max: u32) -> ClampedQueueDepth {
    if value > max as usize {
        ClampedQueueDepth {
            value: max,
            saturated: true,
        }
    } else {
        ClampedQueueDepth {
            value: value as u32,
            saturated: false,
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::string::String;
    use core::fmt;

    use aesynx_abi::{CoreId, TaskId};

    use super::{SchedulerDecisionReason, SchedulerTelemetry};
    use crate::{MAX_SCHEDULE_QUEUE_FEATURE, TelemetryError};

    #[test]
    fn scheduler_telemetry_records_bounded_round_robin_decisions() {
        let mut telemetry = match SchedulerTelemetry::<2>::new() {
            Ok(telemetry) => telemetry,
            Err(error) => return assert_eq!(error, TelemetryError::TelemetryCapacityZero),
        };

        let record = telemetry.record_decision(
            CoreId::new(0),
            TaskId::new(7),
            SchedulerDecisionReason::RoundRobinRunnable,
            3,
            1,
        );

        assert_eq!(record.map(|record| record.sequence()), Ok(0));
        assert_eq!(telemetry.len(), 1);
        assert_eq!(
            telemetry.get(0).map(|record| record.selected_task()),
            Some(TaskId::new(7))
        );
        assert_eq!(
            telemetry.get(0).map(|record| record.reason()),
            Some(SchedulerDecisionReason::RoundRobinRunnable)
        );
        assert_eq!(
            telemetry.get(0).map(|record| (
                record.runnable_before_saturated(),
                record.timer_wait_before_saturated()
            )),
            Some((false, false))
        );
    }

    #[test]
    fn scheduler_telemetry_redacts_task_ids_in_debug_output() {
        let mut telemetry = match SchedulerTelemetry::<1>::new() {
            Ok(telemetry) => telemetry,
            Err(error) => return assert_eq!(error, TelemetryError::TelemetryCapacityZero),
        };
        let record = match telemetry.record_decision(
            CoreId::new(0),
            TaskId::new(0xfeed_beef),
            SchedulerDecisionReason::RoundRobinRunnable,
            1,
            0,
        ) {
            Ok(record) => record,
            Err(error) => return assert_eq!(error, TelemetryError::TelemetryBufferFull),
        };

        let mut debug = String::new();
        assert_eq!(fmt::write(&mut debug, format_args!("{record:?}")), Ok(()));
        assert!(debug.contains("selected_task: \"<redacted>\""));
        assert!(!debug.contains("feed"));
    }

    #[test]
    fn scheduler_telemetry_rejects_zero_capacity_and_full_buffers() {
        assert_eq!(
            SchedulerTelemetry::<0>::new().map(|telemetry| telemetry.capacity()),
            Err(TelemetryError::TelemetryCapacityZero)
        );

        let mut telemetry = match SchedulerTelemetry::<1>::new() {
            Ok(telemetry) => telemetry,
            Err(error) => return assert_eq!(error, TelemetryError::TelemetryCapacityZero),
        };
        assert_eq!(
            telemetry
                .record_decision(
                    CoreId::new(0),
                    TaskId::new(1),
                    SchedulerDecisionReason::RoundRobinRunnable,
                    usize::MAX,
                    usize::MAX,
                )
                .map(|record| record.runnable_before()),
            Ok(MAX_SCHEDULE_QUEUE_FEATURE)
        );
        assert_eq!(
            telemetry.get(0).map(|record| (
                record.runnable_before_saturated(),
                record.timer_wait_before_saturated()
            )),
            Some((true, true))
        );
        assert_eq!(
            telemetry.record_decision(
                CoreId::new(0),
                TaskId::new(2),
                SchedulerDecisionReason::RoundRobinRunnable,
                1,
                0,
            ),
            Err(TelemetryError::TelemetryBufferFull)
        );
        assert_eq!(telemetry.len(), 1);
    }
}
