use aesynx_abi::{CoreId, TaskId};
use aesynx_telemetry::{
    BootPhaseTelemetryEvent, CapFaultKind, CapabilityFaultTelemetryEvent, CoreTelemetry,
    PerCoreEventRing, SchedulerDecisionReason, SchedulerTelemetry, TELEMETRY_SCHEMA_VERSION,
    TelemetryBootPhase, TelemetryError, TelemetryEventId, TelemetryEventPayload,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TelemetryEventsSmokeStatus {
    pub schema_version: u16,
    pub events: usize,
    pub boot_events: usize,
    pub capability_events: usize,
    pub scheduler_events: usize,
    pub boot_event_id: u16,
    pub capability_event_id: u16,
    pub scheduler_event_id: u16,
    pub schema_ok: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelemetryEventsSmokeError {
    Telemetry(TelemetryError),
    UnexpectedSchema,
}

pub fn run() -> Result<TelemetryEventsSmokeStatus, TelemetryEventsSmokeError> {
    let mut ring =
        PerCoreEventRing::<4>::new(CoreId::new(0)).map_err(TelemetryEventsSmokeError::Telemetry)?;
    let boot = ring
        .record(TelemetryEventPayload::BootPhase(
            BootPhaseTelemetryEvent::new(TelemetryBootPhase::Running),
        ))
        .map_err(TelemetryEventsSmokeError::Telemetry)?;

    let core = CoreTelemetry::default();
    let cap_fault = core
        .record_cap_fault(CapFaultKind::MissingPermission)
        .map_err(TelemetryEventsSmokeError::Telemetry)?;
    let capability = ring
        .record(TelemetryEventPayload::CapabilityFault(
            CapabilityFaultTelemetryEvent::from_cap_fault(cap_fault),
        ))
        .map_err(TelemetryEventsSmokeError::Telemetry)?;

    let mut decisions =
        SchedulerTelemetry::<1>::new().map_err(TelemetryEventsSmokeError::Telemetry)?;
    let decision = decisions
        .record_decision(
            CoreId::new(0),
            TaskId::new(7),
            SchedulerDecisionReason::RoundRobinRunnable,
            2,
            0,
        )
        .map_err(TelemetryEventsSmokeError::Telemetry)?;
    let scheduler = ring
        .record(TelemetryEventPayload::SchedulerDecision(
            aesynx_telemetry::SchedulerDecisionTelemetryEvent::from_scheduler_decision(decision),
        ))
        .map_err(TelemetryEventsSmokeError::Telemetry)?;

    let summary = ring.summary();
    let schema_ok = boot.header().schema_version() == TELEMETRY_SCHEMA_VERSION
        && capability.header().schema_version() == TELEMETRY_SCHEMA_VERSION
        && scheduler.header().schema_version() == TELEMETRY_SCHEMA_VERSION
        && boot.header().sequence() == 0
        && capability.header().sequence() == 1
        && scheduler.header().sequence() == 2
        && summary.events == 3
        && summary.boot_phase_events == 1
        && summary.capability_events == 1
        && summary.scheduler_events == 1
        && matches!(
            ring.get(2).map(|event| event.header().event_id()),
            Some(TelemetryEventId::SchedulerDecision)
        );

    if !schema_ok {
        return Err(TelemetryEventsSmokeError::UnexpectedSchema);
    }

    Ok(TelemetryEventsSmokeStatus {
        schema_version: TELEMETRY_SCHEMA_VERSION,
        events: summary.events,
        boot_events: summary.boot_phase_events,
        capability_events: summary.capability_events,
        scheduler_events: summary.scheduler_events,
        boot_event_id: TelemetryEventId::BootPhase.raw(),
        capability_event_id: TelemetryEventId::CapabilityFault.raw(),
        scheduler_event_id: TelemetryEventId::SchedulerDecision.raw(),
        schema_ok,
    })
}

#[cfg(test)]
mod tests {
    use super::run;

    #[test]
    fn telemetry_events_smoke_records_versioned_event_schema() {
        let status = match run() {
            Ok(status) => status,
            Err(error) => return assert_eq!(format!("{error:?}"), ""),
        };

        assert_eq!(status.schema_version, 1);
        assert_eq!(status.events, 3);
        assert_eq!(status.boot_events, 1);
        assert_eq!(status.capability_events, 1);
        assert_eq!(status.scheduler_events, 1);
        assert!(status.schema_ok);
    }
}
