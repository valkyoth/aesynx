# Aesynx Telemetry Event Schema

Status: v0.31.0 tagged trace export schema

Aesynx telemetry events are kernel-stamped facts for boot diagnostics and
future world-model ingestion. The current schema is intentionally small and
line-oriented so it can be decoded from QEMU serial logs without a stable
userspace ABI or persistent log format.

## Serial Trace Format

Kernel smoke output may include lines prefixed with `trace-event `. Schema v1
uses space-separated `key=value` fields:

```text
trace-event schema=1 event=boot-phase sequence=0 core=0 phase=running
trace-event schema=1 event=capability-fault sequence=1 core=0 kind=missing-permission total_cap_faults=1
trace-event schema=1 event=scheduler-decision sequence=2 core=0 selected_task=<redacted> reason=round-robin-runnable runnable_before=2 runnable_before_saturated=false timer_wait_before=0 timer_wait_before_saturated=false
```

Supported event kinds:

- `boot-phase`: `phase`.
- `capability-fault`: `kind`, `total_cap_faults`.
- `scheduler-decision`: `selected_task`, `reason`, `runnable_before`,
  `runnable_before_saturated`, `timer_wait_before`,
  `timer_wait_before_saturated`.

## Export Policy

- `schema=1` is the only accepted schema.
- Unknown event kinds and unknown fields fail closed.
- Unknown enum labels fail closed. Schema v1 accepts only documented phase,
  capability-fault kind, and scheduler reason labels.
- Serial lines are bounded to 16 fields before duplicate detection, and decoded
  output is bounded to 4096 trace events per file.
- The CLI refuses input files larger than 16 MiB before reading them into
  memory.
- `core`, `runnable_before`, and `timer_wait_before` must fit the kernel's
  current `u32` telemetry fields; `sequence` and `total_cap_faults` are `u64`.
- Decoded output is canonicalized from typed values, so accepted numeric fields
  are re-emitted in their normal decimal form rather than copied verbatim from
  the serial line.
- `core` remains visible in exported trace lines. It is treated as local
  non-secret scheduling context until SMP tenancy makes core identity a
  boundary.
- Scheduler `selected_task` must be `<redacted>` before export. The decoder
  rejects a scheduler trace line that contains a raw task ID.
- The line format is an offline analysis format for boot traces, not a stable
  userspace ABI, persistent storage format, or cross-core ordering contract.

## Decoder

Decode a QEMU serial log:

```bash
cargo xtask trace-decode build/qemu/aesynx-v0.31.0.serial.log
```

The underlying tool also runs directly:

```bash
cargo run -p trace-decode -- build/qemu/aesynx-v0.31.0.serial.log
```

Current output is line-based:

```text
trace schema=1 sequence=0 core=0 event=boot-phase phase=running
trace schema=1 sequence=1 core=0 event=capability-fault kind=missing-permission total_cap_faults=1
trace schema=1 sequence=2 core=0 event=scheduler-decision selected_task=<redacted> reason=round-robin-runnable runnable_before=2 runnable_before_saturated=false timer_wait_before=0 timer_wait_before_saturated=false
```
