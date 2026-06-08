# Aesynx Unsafe Policy

Status: policy

Aesynx treats unsafe Rust as part of the trusted computing base. Unsafe is not
for convenience. It is allowed only where the kernel must interact with hardware,
raw memory, interrupt frames, page tables, allocators, or lock-free queue
internals.

## Default Rule

Kernel and runtime crates under `crates/` must use:

```rust
#![no_std]
#![deny(unsafe_code)]
```

Unsafe is forbidden by default. Any crate or module that needs unsafe must first
be admitted as an explicit exception in this document, with a narrowly scoped
purpose and tests/evidence.

Host-only tools may use `std`, but they still inherit the workspace lint policy.

New Rust crates should also prefer:

```rust
#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(unused_must_use)]
```

## Allowed Unsafe Boundaries

Unsafe code may appear only in reviewed modules for:

- Architecture entry and CPU setup.
- Interrupt and trap entry.
- Context switching.
- Page table manipulation.
- Raw MMIO and port I/O.
- Atomic queue internals requiring `UnsafeCell`.
- Allocator internals.
- Bootloader handoff parsing.
- Explicit volatile cleanup or barriers, if a future userspace/runtime module
  needs them and the retention policy admits them.

## Required Documentation

Every unsafe block must have a nearby `SAFETY:` explanation covering:

- Pointer validity.
- Alignment.
- Aliasing.
- Lifetime.
- Interrupt/concurrency assumptions.
- Hardware or ABI invariants.

Every unsafe module must be documented here before release.

## Current Unsafe Inventory

```text
Location: crates/aesynx-arch-x86_64/src/port.rs
Status: active in v0.4
Purpose: early x86_64 COM1 port I/O for serial boot diagnostics
Preconditions: QEMU legacy COM1 UART is present; used during early single-core boot only
Unsafe operation: core::arch::asm! in/out instructions
Safety argument: the instructions access fixed I/O ports and do not dereference Rust pointers, alias Rust memory, or depend on Rust lifetimes; callers only expose safe byte-oriented serial operations
Tests/evidence: cargo xtask qemu observes the Rust _start serial marker; port construction is limited to a typed COM1 admitted-port set
Limitations: not synchronized for SMP, not a general serial driver, and not suitable for untrusted device probing; UART transmit polling is bounded and drops bytes on timeout
```

```text
Location: crates/aesynx-kernel/src/main.rs
Status: active in v0.4
Purpose: export the architecture entry symbol consumed by the bootloader
Preconditions: Limine loads the x86_64 kernel ELF and transfers control to _start
Unsafe operation: Rust 2024 unsafe no_mangle attribute on _start
Safety argument: the symbol name is fixed by the linker script and boot contract; the function never returns and does not expose a callable safe API to Rust code
Tests/evidence: readelf shows _start as the ELF entry; cargo xtask qemu observes the Rust boot marker
Limitations: no BootInfo argument is consumed yet; bootloader metadata normalization starts in v0.5
```

New entries should use this format:

```text
Location:
Status:
Purpose:
Preconditions:
Unsafe operation:
Safety argument:
Tests/evidence:
Limitations:
```

## Release Gate

Before 1.0, add a script that fails when:

- New unsafe appears outside admitted modules.
- An unsafe block lacks a nearby `SAFETY:` comment.
- Architecture-specific intrinsics appear outside architecture crates.
- MMIO volatile access appears outside driver/arch MMIO wrappers.
- Panic-like macros appear in security-critical runtime paths without a
  documented exception.
