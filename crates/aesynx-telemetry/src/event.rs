use core::fmt;

use aesynx_abi::{CoreId, TaskId};

use crate::{
    CapFaultEvent, CapFaultKind, SchedulerDecisionReason, SchedulerDecisionRecord, TelemetryError,
};

pub const TELEMETRY_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum TelemetryEventId {
    BootPhase = 1,
    CapabilityFault = 2,
    SchedulerDecision = 3,
}

impl TelemetryEventId {
    #[must_use]
    pub const fn raw(self) -> u16 {
        self as u16
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::BootPhase => "boot-phase",
            Self::CapabilityFault => "capability-fault",
            Self::SchedulerDecision => "scheduler-decision",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TelemetryEventHeader {
    schema_version: u16,
    event_id: TelemetryEventId,
    sequence: u64,
    core: CoreId,
}

impl TelemetryEventHeader {
    #[must_use]
    pub const fn schema_version(self) -> u16 {
        self.schema_version
    }

    #[must_use]
    pub const fn event_id(self) -> TelemetryEventId {
        self.event_id
    }

    #[must_use]
    pub const fn sequence(self) -> u64 {
        self.sequence
    }

    #[must_use]
    pub const fn core(self) -> CoreId {
        self.core
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelemetryBootPhase {
    Entry,
    CpuSetup,
    ExceptionSetup,
    InterruptSetup,
    BootloaderHandoff,
    BootInfoNormalized,
    Running,
    PanicSmoke,
    ExceptionSmoke,
    TimerSmoke,
    Panic,
    Unknown,
}

impl TelemetryBootPhase {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Entry => "entry",
            Self::CpuSetup => "cpu-setup",
            Self::ExceptionSetup => "exception-setup",
            Self::InterruptSetup => "interrupt-setup",
            Self::BootloaderHandoff => "bootloader-handoff",
            Self::BootInfoNormalized => "bootinfo-normalized",
            Self::Running => "running",
            Self::PanicSmoke => "panic-smoke",
            Self::ExceptionSmoke => "exception-smoke",
            Self::TimerSmoke => "timer-smoke",
            Self::Panic => "panic",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootPhaseTelemetryEvent {
    phase: TelemetryBootPhase,
}

impl BootPhaseTelemetryEvent {
    #[must_use]
    pub const fn new(phase: TelemetryBootPhase) -> Self {
        Self { phase }
    }

    #[must_use]
    pub const fn phase(self) -> TelemetryBootPhase {
        self.phase
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapabilityFaultTelemetryEvent {
    kind: CapFaultKind,
    total_cap_faults: u64,
}

impl CapabilityFaultTelemetryEvent {
    #[must_use]
    pub const fn from_cap_fault(event: CapFaultEvent) -> Self {
        Self {
            kind: event.kind,
            total_cap_faults: event.total_cap_faults,
        }
    }

    #[must_use]
    pub const fn kind(self) -> CapFaultKind {
        self.kind
    }

    #[must_use]
    pub const fn total_cap_faults(self) -> u64 {
        self.total_cap_faults
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SchedulerDecisionTelemetryEvent {
    reason: SchedulerDecisionReason,
    selected_task: TaskId,
    runnable_before: u32,
    runnable_before_saturated: bool,
    timer_wait_before: u32,
    timer_wait_before_saturated: bool,
}

impl SchedulerDecisionTelemetryEvent {
    #[must_use]
    pub const fn from_scheduler_decision(record: SchedulerDecisionRecord) -> Self {
        Self {
            reason: record.reason(),
            selected_task: record.selected_task(),
            runnable_before: record.runnable_before(),
            runnable_before_saturated: record.runnable_before_saturated(),
            timer_wait_before: record.timer_wait_before(),
            timer_wait_before_saturated: record.timer_wait_before_saturated(),
        }
    }

    #[must_use]
    pub const fn reason(self) -> SchedulerDecisionReason {
        self.reason
    }

    #[must_use]
    pub const fn selected_task(self) -> TaskId {
        self.selected_task
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

impl fmt::Debug for SchedulerDecisionTelemetryEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SchedulerDecisionTelemetryEvent")
            .field("reason", &self.reason)
            .field("selected_task", &"<redacted>")
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelemetryEventPayload {
    BootPhase(BootPhaseTelemetryEvent),
    CapabilityFault(CapabilityFaultTelemetryEvent),
    SchedulerDecision(SchedulerDecisionTelemetryEvent),
}

impl TelemetryEventPayload {
    #[must_use]
    pub const fn event_id(self) -> TelemetryEventId {
        match self {
            Self::BootPhase(_) => TelemetryEventId::BootPhase,
            Self::CapabilityFault(_) => TelemetryEventId::CapabilityFault,
            Self::SchedulerDecision(_) => TelemetryEventId::SchedulerDecision,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TelemetryEvent {
    header: TelemetryEventHeader,
    payload: TelemetryEventPayload,
}

impl TelemetryEvent {
    #[must_use]
    pub const fn header(self) -> TelemetryEventHeader {
        self.header
    }

    #[must_use]
    pub const fn payload(self) -> TelemetryEventPayload {
        self.payload
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct PerCoreEventRing<const CAPACITY: usize> {
    core: CoreId,
    events: [Option<TelemetryEvent>; CAPACITY],
    len: usize,
    next_sequence: u64,
}

impl<const CAPACITY: usize> PerCoreEventRing<CAPACITY> {
    pub const fn new(core: CoreId) -> Result<Self, TelemetryError> {
        if CAPACITY == 0 {
            return Err(TelemetryError::TelemetryCapacityZero);
        }

        Ok(Self {
            core,
            events: [const { None }; CAPACITY],
            len: 0,
            next_sequence: 0,
        })
    }

    pub fn record(
        &mut self,
        payload: TelemetryEventPayload,
    ) -> Result<TelemetryEvent, TelemetryError> {
        if self.len == CAPACITY {
            return Err(TelemetryError::TelemetryBufferFull);
        }

        let sequence = self.next_sequence;
        self.next_sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or(TelemetryError::CounterOverflow)?;
        let event = TelemetryEvent {
            header: TelemetryEventHeader {
                schema_version: TELEMETRY_SCHEMA_VERSION,
                event_id: payload.event_id(),
                sequence,
                core: self.core,
            },
            payload,
        };
        self.events[self.len] = Some(event);
        self.len += 1;
        Ok(event)
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<TelemetryEvent> {
        if index >= self.len {
            return None;
        }
        self.events[index]
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
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
    pub const fn capacity(&self) -> usize {
        CAPACITY
    }

    #[must_use]
    pub fn summary(&self) -> TelemetryEventRingSummary {
        let mut summary = TelemetryEventRingSummary {
            capacity: CAPACITY,
            events: self.len,
            next_sequence: self.next_sequence,
            boot_phase_events: 0,
            capability_events: 0,
            scheduler_events: 0,
        };
        let mut index = 0;
        while index < self.len {
            if let Some(event) = self.get(index) {
                match event.payload {
                    TelemetryEventPayload::BootPhase(_) => summary.boot_phase_events += 1,
                    TelemetryEventPayload::CapabilityFault(_) => summary.capability_events += 1,
                    TelemetryEventPayload::SchedulerDecision(_) => summary.scheduler_events += 1,
                }
            }
            index += 1;
        }
        summary
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TelemetryEventRingSummary {
    pub capacity: usize,
    pub events: usize,
    pub next_sequence: u64,
    pub boot_phase_events: usize,
    pub capability_events: usize,
    pub scheduler_events: usize,
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::string::String;
    use core::fmt;

    use aesynx_abi::{CoreId, TaskId};

    use super::{
        BootPhaseTelemetryEvent, CapabilityFaultTelemetryEvent, PerCoreEventRing,
        TELEMETRY_SCHEMA_VERSION, TelemetryBootPhase, TelemetryEventId, TelemetryEventPayload,
    };
    use crate::{
        CapFaultEvent, CapFaultKind, SchedulerDecisionReason, SchedulerTelemetry, TelemetryError,
    };

    #[test]
    fn per_core_event_ring_records_versioned_events_in_order() {
        let mut ring = match PerCoreEventRing::<3>::new(CoreId::new(0)) {
            Ok(ring) => ring,
            Err(error) => return assert_eq!(error, TelemetryError::TelemetryCapacityZero),
        };
        assert_eq!(
            ring.record(TelemetryEventPayload::BootPhase(
                BootPhaseTelemetryEvent::new(TelemetryBootPhase::Running)
            ))
            .map(|event| event.header().event_id()),
            Ok(TelemetryEventId::BootPhase)
        );
        assert_eq!(
            ring.record(TelemetryEventPayload::CapabilityFault(
                CapabilityFaultTelemetryEvent::from_cap_fault(CapFaultEvent {
                    kind: CapFaultKind::MissingPermission,
                    total_cap_faults: 1,
                })
            ))
            .map(|event| event.header().sequence()),
            Ok(1)
        );

        let summary = ring.summary();
        assert_eq!(summary.events, 2);
        assert_eq!(summary.boot_phase_events, 1);
        assert_eq!(summary.capability_events, 1);
        assert_eq!(
            ring.get(0).map(|event| event.header().schema_version()),
            Some(TELEMETRY_SCHEMA_VERSION)
        );
        assert_eq!(ring.get(2), None);
    }

    #[test]
    fn per_core_event_ring_rejects_zero_capacity_and_full_buffers() {
        assert_eq!(
            PerCoreEventRing::<0>::new(CoreId::new(0)).map(|ring| ring.capacity()),
            Err(TelemetryError::TelemetryCapacityZero)
        );
        let mut ring = match PerCoreEventRing::<1>::new(CoreId::new(0)) {
            Ok(ring) => ring,
            Err(error) => return assert_eq!(error, TelemetryError::TelemetryCapacityZero),
        };
        let payload = TelemetryEventPayload::BootPhase(BootPhaseTelemetryEvent::new(
            TelemetryBootPhase::Entry,
        ));
        assert!(ring.is_empty());
        assert!(ring.record(payload).is_ok());
        assert!(ring.is_full());
        assert_eq!(
            ring.record(payload),
            Err(TelemetryError::TelemetryBufferFull)
        );
        assert_eq!(ring.len(), 1);
    }

    #[test]
    fn scheduler_event_debug_redacts_selected_task() {
        let mut decisions = match SchedulerTelemetry::<1>::new() {
            Ok(decisions) => decisions,
            Err(error) => return assert_eq!(error, TelemetryError::TelemetryCapacityZero),
        };
        let decision = match decisions.record_decision(
            CoreId::new(0),
            TaskId::new(4_276_993_775),
            SchedulerDecisionReason::RoundRobinRunnable,
            1,
            0,
        ) {
            Ok(decision) => decision,
            Err(error) => return assert_eq!(error, TelemetryError::TelemetryBufferFull),
        };
        let payload = TelemetryEventPayload::SchedulerDecision(
            super::SchedulerDecisionTelemetryEvent::from_scheduler_decision(decision),
        );

        let mut debug = String::new();
        assert_eq!(fmt::write(&mut debug, format_args!("{payload:?}")), Ok(()));
        assert!(debug.contains("selected_task: \"<redacted>\""));
        assert!(!debug.contains("4276993775"));
    }
}
