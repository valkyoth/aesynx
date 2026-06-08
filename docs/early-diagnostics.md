# Aesynx Early Diagnostics

Status: v0.6 implementation candidate

`v0.6.0` makes early kernel failures readable over serial before interrupts,
exceptions, page tables, or an allocator exist.

## Current Path

`crates/aesynx-kernel/src/diagnostics.rs` owns the safe diagnostics model:

- `BootPhase` records the current early boot phase.
- `set_boot_phase` stores the phase in a single atomic byte.
- `panic_snapshot` captures the early core and current phase for panic output.

The target-specific serial emission remains in `crates/aesynx-kernel/src/main.rs`
so the diagnostics library stays `no_std`, safe Rust, and host-testable.

## Panic Output

The panic handler prints:

```text
Aesynx: panic during early boot
panic core=0 phase=<phase>
panic location=<file> line=<line> column=<column>
panic message=<message>
panic registers=unavailable
```

Register capture is explicitly unavailable in v0.6. Real register and fault
decoding starts after descriptor tables and exception handlers land.

## Serial Contract

Normal boot smoke still requires:

```text
[TEST] bootinfo=ok
[TEST] boot=ok
```

The panic smoke path is opt-in:

```bash
cargo xtask qemu --panic-smoke
```

It builds a separate release-profile QEMU image with the `panic-smoke` feature
enabled and expects:

```text
[TEST] panic=ok
```

## Boundaries

This milestone proves:

- Boot phase tracking works before allocator setup.
- Panic output includes core, phase, file, line, column, and message.
- QEMU can machine-check a deliberate panic path.

This milestone does not prove:

- Interrupt or exception handling.
- Page-fault diagnostics.
- Real register dumps.
- SMP-safe diagnostics.
- Persistent telemetry buffers.
