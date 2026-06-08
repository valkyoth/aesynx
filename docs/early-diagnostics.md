# Aesynx Early Diagnostics

Status: v0.6 implementation candidate

`v0.6.0` makes early kernel failures readable over serial before interrupts,
exceptions, page tables, or an allocator exist.

## Current Path

`crates/aesynx-kernel/src/diagnostics.rs` owns the safe diagnostics model:

- `BootPhase` records the current early boot phase.
- `set_boot_phase` stores the phase in a single atomic byte.
- `panic_snapshot` captures the early core and current phase for panic output.
- `DiagnosticRecord` formats no-alloc structured records with core, phase,
  component, log level, and bounded single-record messages.
- `DiagnosticComponent` validates component names before formatting, allowing
  only lowercase ASCII letters, digits, `-`, and `_` up to 32 bytes.

The target-specific serial emission remains in `crates/aesynx-kernel/src/main.rs`
so the diagnostics library stays `no_std`, safe Rust, and host-testable.

Structured records use this shape:

```text
[core=0][phase=cpu-setup][kernel][INFO] gdt and tss initialized
[core=0][phase=bootinfo-normalized][kernel][INFO] bootinfo normalized
```

Messages use `LogMessage`, which rejects record separators and bracket
metacharacters. Components are also validated so a component name cannot forge
bracketed fields or extra records.

## Panic Output

The panic handler prints:

```text
Aesynx: panic during early boot
[core=0][phase=panic-smoke][kernel][FATAL] panic handler entered
panic core=0 phase=<phase>
panic location=<file> line=<line> column=<column>
panic message=<message>
panic registers=rsp_present=<bool> rbp_present=<bool> rsp_align=<n> rbp_align=<n> rflags=0x<n> cr3_offset=0x<n>
```

The panic location emits only the escaped filename component, not the full
source path. The tracked workspace config uses a repo-local Rust compiler
wrapper that computes the workspace root dynamically and passes
`--remap-path-prefix <workspace>=.` for direct builds. Xtask kernel builds also
pass the same remap through encoded Rust flags, so embedded file paths do not
disclose the local workspace root in normal direct and release-image builds.

The panic message line is escaped and bounded before serial emission. Newlines,
carriage returns, tabs, backslashes, brackets, non-ASCII bytes, and other
control bytes cannot create forged diagnostic records or unbounded panic output.

On x86_64, `crates/aesynx-arch-x86_64/src/registers.rs` captures `rsp`, `rbp`,
`rflags`, and `cr3` for the panic path. Raw address-bearing values stay private
and are not printed; serial output exposes only presence, stack alignment,
arithmetic/status RFLAGS bits, and CR3 low flag/PCID bits. The v0.9 page-fault
path additionally captures CR2, CR3 low bits, public RFLAGS, and interrupt
state for early exception diagnostics.

The public RFLAGS summary is limited to arithmetic/status flags only. It
intentionally excludes trap/debug, interrupt-enable, I/O privilege, alignment,
virtual-interrupt, and CPU-identification state.

## Serial Contract

Normal boot smoke still requires:

```text
[TEST] gdt=ok
[TEST] idt=ok
[TEST] exception=ok
[TEST] bootinfo=ok
[TEST] boot=ok
```

The panic smoke path is opt-in:

```bash
cargo xtask qemu --panic-smoke
```

It builds a separate release-profile QEMU image with the `panic-smoke` feature
enabled and expects both the structured fatal record and the panic marker:

```text
[TEST] gdt=ok
[TEST] idt=ok
[TEST] exception=ok
[kernel][FATAL] panic handler entered
[TEST] panic=ok
```

## Boundaries

This milestone proves:

- Boot phase tracking works before allocator setup.
- Structured log-level records can be emitted before allocator setup.
- Panic output includes core, phase, file, line, column, message, fatal record,
  and redacted x86_64 register summary.
- QEMU can machine-check a deliberate panic path.

This milestone does not prove:

- Full interrupt or exception decoding.
- Page-fault address and error-code decoding.
- Raw register dumps.
- SMP-safe diagnostics.
- Persistent telemetry buffers.
