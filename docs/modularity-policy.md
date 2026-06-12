# Aesynx Modularity Policy

Status: policy

Aesynx must be built as a long-lived operating-system project, not as a pile of
large files. The structure should make security review, testing, replacement,
and future fixes local.

This policy is inspired by the workspace/module discipline used in projects
such as Fluxheim and Mjolni: split by ownership and behavior, keep crates
focused, keep source files small, and make boundaries obvious.

## Core Rule

Aesynx must never grow around a huge one-file implementation.

Every major subsystem gets an explicit crate or module boundary. Every large
module must be split by responsibility before it becomes hard to review.

Aesynx must also never become a "one huge binary OS." A boot image or signed
boot bundle may package many components for delivery, but the system design
must preserve independently versioned and replaceable components:

- The kernel stays small and focused on core authority, memory, scheduling,
  IPC, and boot orchestration.
- Drivers move toward isolated services rather than permanently linked kernel
  blobs.
- Userspace commands, services, runtime components, models, and policies have
  their own manifests and version identities.
- Updates can replace a component or object root without relinking the whole
  OS.
- Rollback can target a component root or system root through the object graph.
- Stable ABI/service contracts matter more than sharing one implementation
  binary.

The release gate should treat monolithic growth as a security regression, not
as a simplification.

## Workspace Shape

Use focused crates:

- `aesynx-kernel`: boot orchestration and top-level kernel flow.
- `aesynx-arch`: architecture-neutral traits.
- `aesynx-arch-x86_64`: x86_64 mechanism.
- `aesynx-arch-aarch64`: aarch64 mechanism.
- `aesynx-boot`: boot metadata normalization.
- `aesynx-log`: logging and diagnostics.
- `aesynx-mm`: physical/virtual memory model.
- `aesynx-cap`: capabilities.
- `aesynx-object`: object graph and local registries.
- `aesynx-ipc`: rings, messages, and service queues.
- `aesynx-sched`: tasks and scheduler.
- `aesynx-telemetry`: event schemas and trace buffers.
- `aesynx-ai-policy`: bounded policy/model interfaces.
- `aesynx-device`: device and driver manager model.
- `aesynx-bytecode`: verifier/interpreter.
- `aesynx-abi`: kernel/userspace ABI.
- `aesynx-rt`: native userspace runtime.
- `aesynx-shell`: `aesh`.

Host model crates live under `models/` and should be preferred for pure logic
that needs fuzzing, Miri, Kani, or property testing.

## File Size Rule

Target:

- Normal implementation files: 300 lines or less.
- Complex implementation files: 500 lines maximum before splitting is required.
- Tests may be larger, but should still be split by behavior.
- Generated files must live under generated/output directories and must not be
  hand-edited.

Hard gate:

- Any non-generated `.rs` file over 500 lines fails `scripts/checks.sh` unless
  it has a documented temporary exception in this file.

Current exceptions:

```text
Path: crates/aesynx-kernel/src/limine.rs
Reason: Concentrated early-boot Limine ABI boundary; v0.16.3 pentest fixes add payload-address validation without changing the bootloader request/link-section layout.
Owner: kernel boot handoff
Split plan: Move Limine protocol structs, constants, request statics, and ABI layout assertions into a private limine/abi.rs module; keep normalization flow in limine.rs.
Removal deadline: v0.16.4
```

Exception format:

```text
Path:
Reason:
Owner:
Split plan:
Removal deadline:
```

## Module Split Rules

Split a file when it contains more than one clear reason to change.

Prefer modules such as:

```text
src/
|-- lib.rs
|-- error.rs
|-- types.rs
|-- config.rs
|-- model.rs
|-- validate.rs
|-- tests.rs
`-- subsystem/
    |-- mod.rs
    |-- state.rs
    |-- protocol.rs
    |-- validate.rs
    `-- tests.rs
```

Avoid:

- Thousands of lines in `lib.rs`.
- Large `main.rs` doing real logic.
- Mixed parsing, validation, state mutation, I/O, and policy in one file.
- Architecture-specific code in generic crates.
- Tests hidden inside massive production files.

## Security Rationale

Small modules make it easier to:

- Review unsafe code.
- Prove invariants.
- Fuzz parsers.
- Test policy in host model crates.
- Replace drivers/services.
- Audit capability checks.
- Find missing telemetry.
- Keep AI policy advisory rather than authoritative.

## Review Checklist

Before merging code:

- Does this belong in an existing crate or a new crate?
- Can the pure logic be moved to a host model crate?
- Is `lib.rs` only wiring public modules?
- Is `main.rs` only orchestration?
- Can this file be split before it reaches 500 lines?
- Are tests close enough to behavior but not bloating production files?
- Are security-sensitive invariants documented near the type or module?
