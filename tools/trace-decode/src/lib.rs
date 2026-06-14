#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreExportPolicy {
    VisibleLocalCoreId,
}

pub const CORE_EXPORT_POLICY: CoreExportPolicy = CoreExportPolicy::VisibleLocalCoreId;
pub const SUPPORTED_SCHEMA_VERSION: u16 = 1;

const TRACE_PREFIX: &str = "trace-event ";

#[derive(Debug, Eq, PartialEq)]
pub enum TraceDecodeError {
    DuplicateField,
    InvalidLabel,
    InvalidNumber,
    MissingField(&'static str),
    NoTraceEvents,
    RedactionViolation,
    UnsupportedEvent,
    UnsupportedSchema,
    UnknownField,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TraceExport {
    lines: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CommonFields {
    schema: u16,
    sequence: u64,
    core: u32,
}

impl TraceExport {
    #[must_use]
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    #[must_use]
    pub fn into_lines(self) -> Vec<String> {
        self.lines
    }
}

pub fn decode_serial_trace(input: &str) -> Result<TraceExport, TraceDecodeError> {
    let mut lines = Vec::new();
    for line in input.lines() {
        let Some(rest) = line.strip_prefix(TRACE_PREFIX) else {
            continue;
        };
        lines.push(decode_trace_event(rest)?);
    }

    if lines.is_empty() {
        return Err(TraceDecodeError::NoTraceEvents);
    }

    Ok(TraceExport { lines })
}

fn decode_trace_event(input: &str) -> Result<String, TraceDecodeError> {
    let fields = parse_fields(input)?;
    let common = decode_common_fields(&fields)?;

    let event = required_field(&fields, "event")?;

    match event {
        "boot-phase" => decode_boot_phase(&fields, common),
        "capability-fault" => decode_capability_fault(&fields, common),
        "scheduler-decision" => decode_scheduler_decision(&fields, common),
        _ => Err(TraceDecodeError::UnsupportedEvent),
    }
}

fn decode_boot_phase(
    fields: &[(&str, &str)],
    common: CommonFields,
) -> Result<String, TraceDecodeError> {
    validate_allowed_fields(fields, &["schema", "event", "sequence", "core", "phase"])?;
    let phase = required_field(fields, "phase")?;
    validate_label(
        phase,
        &[
            "entry",
            "cpu-setup",
            "exception-setup",
            "interrupt-setup",
            "bootloader-handoff",
            "bootinfo-normalized",
            "running",
            "panic-smoke",
            "exception-smoke",
            "timer-smoke",
            "panic",
            "unknown",
        ],
    )?;

    Ok(format!(
        "trace schema={} sequence={} core={} event=boot-phase phase={phase}",
        common.schema, common.sequence, common.core
    ))
}

fn decode_capability_fault(
    fields: &[(&str, &str)],
    common: CommonFields,
) -> Result<String, TraceDecodeError> {
    validate_allowed_fields(
        fields,
        &[
            "schema",
            "event",
            "sequence",
            "core",
            "kind",
            "total_cap_faults",
        ],
    )?;
    let kind = required_field(fields, "kind")?;
    let total = required_field(fields, "total_cap_faults")?;
    validate_label(
        kind,
        &[
            "invalid-id",
            "missing-permission",
            "revoked",
            "stale-id",
            "unknown",
        ],
    )?;
    let total = parse_u64(total)?;

    Ok(format!(
        "trace schema={} sequence={} core={} event=capability-fault kind={kind} total_cap_faults={total}",
        common.schema, common.sequence, common.core
    ))
}

fn decode_scheduler_decision(
    fields: &[(&str, &str)],
    common: CommonFields,
) -> Result<String, TraceDecodeError> {
    validate_allowed_fields(
        fields,
        &[
            "schema",
            "event",
            "sequence",
            "core",
            "selected_task",
            "reason",
            "runnable_before",
            "runnable_before_saturated",
            "timer_wait_before",
            "timer_wait_before_saturated",
        ],
    )?;

    if required_field(fields, "selected_task")? != "<redacted>" {
        return Err(TraceDecodeError::RedactionViolation);
    }

    let reason = required_field(fields, "reason")?;
    let runnable = required_field(fields, "runnable_before")?;
    let runnable_saturated = required_field(fields, "runnable_before_saturated")?;
    let timer_wait = required_field(fields, "timer_wait_before")?;
    let timer_wait_saturated = required_field(fields, "timer_wait_before_saturated")?;
    validate_label(reason, &["round-robin-runnable"])?;
    let runnable = parse_u32(runnable)?;
    let runnable_saturated = parse_bool(runnable_saturated)?;
    let timer_wait = parse_u32(timer_wait)?;
    let timer_wait_saturated = parse_bool(timer_wait_saturated)?;

    Ok(format!(
        "trace schema={} sequence={} core={} event=scheduler-decision selected_task=<redacted> reason={reason} runnable_before={runnable} runnable_before_saturated={runnable_saturated} timer_wait_before={timer_wait} timer_wait_before_saturated={timer_wait_saturated}",
        common.schema, common.sequence, common.core
    ))
}

fn parse_fields(input: &str) -> Result<Vec<(&str, &str)>, TraceDecodeError> {
    let mut fields = Vec::new();
    for token in input.split_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            return Err(TraceDecodeError::UnknownField);
        };
        if fields.iter().any(|(existing, _)| *existing == key) {
            return Err(TraceDecodeError::DuplicateField);
        }
        fields.push((key, value));
    }
    Ok(fields)
}

fn decode_common_fields(fields: &[(&str, &str)]) -> Result<CommonFields, TraceDecodeError> {
    let sequence = parse_u64(required_field(fields, "sequence")?)?;
    let core = parse_u32(required_field(fields, "core")?)?;

    let schema = parse_u64(required_field(fields, "schema")?)?;
    if schema != u64::from(SUPPORTED_SCHEMA_VERSION) {
        return Err(TraceDecodeError::UnsupportedSchema);
    }
    Ok(CommonFields {
        schema: SUPPORTED_SCHEMA_VERSION,
        sequence,
        core,
    })
}

fn validate_allowed_fields(
    fields: &[(&str, &str)],
    allowed: &[&str],
) -> Result<(), TraceDecodeError> {
    for (key, _) in fields {
        if !allowed.iter().any(|allowed_key| allowed_key == key) {
            return Err(TraceDecodeError::UnknownField);
        }
    }
    Ok(())
}

fn required_field<'a>(
    fields: &'a [(&str, &str)],
    key: &'static str,
) -> Result<&'a str, TraceDecodeError> {
    fields
        .iter()
        .find(|(field_key, _)| *field_key == key)
        .map(|(_, value)| *value)
        .ok_or(TraceDecodeError::MissingField(key))
}

fn validate_label(value: &str, allowed: &[&str]) -> Result<(), TraceDecodeError> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(TraceDecodeError::InvalidLabel)
    }
}

fn parse_u64(value: &str) -> Result<u64, TraceDecodeError> {
    value
        .parse::<u64>()
        .map_err(|_| TraceDecodeError::InvalidNumber)
}

fn parse_u32(value: &str) -> Result<u32, TraceDecodeError> {
    value
        .parse::<u32>()
        .map_err(|_| TraceDecodeError::InvalidNumber)
}

fn parse_bool(value: &str) -> Result<bool, TraceDecodeError> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(TraceDecodeError::InvalidNumber),
    }
}

#[cfg(test)]
mod tests {
    use super::{TraceDecodeError, decode_serial_trace};

    const TRACE_FIXTURE: &str = "\
boot noise
trace-event schema=1 event=boot-phase sequence=0 core=0 phase=running
trace-event schema=1 event=capability-fault sequence=1 core=0 kind=missing-permission total_cap_faults=1
trace-event schema=1 event=scheduler-decision sequence=2 core=0 selected_task=<redacted> reason=round-robin-runnable runnable_before=2 runnable_before_saturated=false timer_wait_before=0 timer_wait_before_saturated=false
";

    #[test]
    fn serial_trace_decodes_to_line_export() {
        let export = match decode_serial_trace(TRACE_FIXTURE) {
            Ok(export) => export,
            Err(error) => return assert_eq!(error, TraceDecodeError::NoTraceEvents),
        };

        assert_eq!(export.lines().len(), 3);
        assert_eq!(
            export.lines()[0],
            "trace schema=1 sequence=0 core=0 event=boot-phase phase=running"
        );
        assert!(export.lines()[2].contains("selected_task=<redacted>"));
    }

    #[test]
    fn scheduler_trace_rejects_raw_task_identity() {
        let raw_task_trace = "\
trace-event schema=1 event=scheduler-decision sequence=2 core=0 selected_task=7 reason=round-robin-runnable runnable_before=2 runnable_before_saturated=false timer_wait_before=0 timer_wait_before_saturated=false
";

        assert_eq!(
            decode_serial_trace(raw_task_trace),
            Err(TraceDecodeError::RedactionViolation)
        );
    }

    #[test]
    fn scheduler_trace_export_does_not_contain_raw_task_id() {
        let export = match decode_serial_trace(TRACE_FIXTURE) {
            Ok(export) => export,
            Err(error) => return assert_eq!(error, TraceDecodeError::NoTraceEvents),
        };
        let joined = export.into_lines().join("\n");

        assert!(joined.contains("selected_task=<redacted>"));
        assert!(!joined.contains("selected_task=7"));
    }

    #[test]
    fn trace_decoder_rejects_unknown_schema() {
        let unknown = "trace-event schema=2 event=boot-phase sequence=0 core=0 phase=running\n";

        assert_eq!(
            decode_serial_trace(unknown),
            Err(TraceDecodeError::UnsupportedSchema)
        );
    }

    #[test]
    fn trace_decoder_requires_trace_events() {
        assert_eq!(
            decode_serial_trace("telemetry-events schema=1 events=3\n"),
            Err(TraceDecodeError::NoTraceEvents)
        );
    }

    #[test]
    fn trace_decoder_rejects_unknown_labels() {
        let unknown_phase = "trace-event schema=1 event=boot-phase sequence=0 core=0 phase=admin\n";

        assert_eq!(
            decode_serial_trace(unknown_phase),
            Err(TraceDecodeError::InvalidLabel)
        );
    }

    #[test]
    fn trace_decoder_rejects_out_of_range_core_ids() {
        let out_of_range =
            "trace-event schema=1 event=boot-phase sequence=0 core=4294967296 phase=running\n";

        assert_eq!(
            decode_serial_trace(out_of_range),
            Err(TraceDecodeError::InvalidNumber)
        );
    }

    #[test]
    fn trace_decoder_rejects_out_of_range_queue_depths() {
        let out_of_range = "\
trace-event schema=1 event=scheduler-decision sequence=2 core=0 selected_task=<redacted> reason=round-robin-runnable runnable_before=4294967296 runnable_before_saturated=false timer_wait_before=0 timer_wait_before_saturated=false
";

        assert_eq!(
            decode_serial_trace(out_of_range),
            Err(TraceDecodeError::InvalidNumber)
        );
    }

    #[test]
    fn trace_decoder_canonicalizes_numeric_fields() {
        let trace = "\
trace-event schema=01 event=scheduler-decision sequence=0002 core=0000 selected_task=<redacted> reason=round-robin-runnable runnable_before=0002 runnable_before_saturated=false timer_wait_before=0000 timer_wait_before_saturated=true
";

        let export = match decode_serial_trace(trace) {
            Ok(export) => export,
            Err(error) => return assert_eq!(error, TraceDecodeError::NoTraceEvents),
        };

        assert_eq!(
            export.lines()[0],
            "trace schema=1 sequence=2 core=0 event=scheduler-decision selected_task=<redacted> reason=round-robin-runnable runnable_before=2 runnable_before_saturated=false timer_wait_before=0 timer_wait_before_saturated=true"
        );
    }

    #[test]
    fn trace_decoder_rejects_missing_required_fields() {
        let missing_phase = "trace-event schema=1 event=boot-phase sequence=0 core=0\n";

        assert_eq!(
            decode_serial_trace(missing_phase),
            Err(TraceDecodeError::MissingField("phase"))
        );
    }
}
