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
Location: crates/aesynx-arch-x86_64/src/lib.rs
Status: active in v0.5
Purpose: terminal x86_64 CPU halt path
Preconditions: called only for halt_forever states where the current core must stop executing normal work
Unsafe operation: core::arch::asm! hlt instruction
Safety argument: the instruction does not dereference Rust pointers, alias Rust memory, modify the stack, or access I/O ports; it is the architecture-defined idle instruction for a terminal halt loop
Tests/evidence: workspace tests and cargo xtask qemu compile and exercise the x86_64 architecture crate; code review confirms the unsafe block is confined to halt_forever
Limitations: interrupt-state control is still unsupported; early boot currently relies on the boot environment's interrupt state
```

```text
Location: crates/aesynx-arch-aarch64/src/lib.rs
Status: admitted for future AArch64 target builds in v0.5
Purpose: terminal AArch64 CPU halt path
Preconditions: called only for halt_forever states where the current core must stop executing normal work
Unsafe operation: core::arch::asm! wfi instruction when compiled for target_arch = "aarch64"
Safety argument: the instruction does not dereference Rust pointers, alias Rust memory, modify the stack, or access device registers; it is the architecture-defined wait-for-interrupt instruction for an idle halt loop
Tests/evidence: workspace tests compile the non-AArch64 fallback; target AArch64 execution remains a future milestone
Limitations: no AArch64 QEMU boot target exists yet; host builds use spin_loop as a compile-time fallback
```

```text
Location: crates/aesynx-kernel/src/main.rs
Status: active in v0.5
Purpose: export the architecture entry symbol consumed by the bootloader
Preconditions: Limine loads the x86_64 kernel ELF and transfers control to _start
Unsafe operation: Rust 2024 unsafe no_mangle attribute on _start
Safety argument: the symbol name is fixed by the linker script and boot contract; the function never returns and does not expose a callable safe API to Rust code
Tests/evidence: readelf shows _start as the ELF entry; cargo xtask qemu observes the BootInfo and Rust boot markers
Limitations: early boot only; full panic and fault diagnostics start in later milestones
```

```text
Location: crates/aesynx-kernel/src/limine.rs
Status: active in v0.5
Purpose: parse Limine bootloader handoff responses and normalize them into Aesynx BootInfo
Preconditions: Limine v12.3.2-compatible bootloader loads the kernel, honours request sections, fills response pointers before _start, and keeps response data valid in bootloader-reclaimable memory during early boot
Unsafe operation: raw reads of Limine response pointers, raw pointer traversal of memory-map and framebuffer arrays, linker-provided __kernel_end symbol address, and mutable Limine request statics written by the bootloader before Rust executes
Safety argument: the unsafe code is confined to the boot handoff boundary; it checks response and array pointers for null and alignment before forming references, bounds memory-map copies to MAX_EARLY_MEMORY_REGIONS, validates lossy integer conversions, compile-time asserts the transcribed framebuffer ABI layout, and passes only value-copied metadata into the safe aesynx-boot normalization API
Tests/evidence: cargo xtask qemu observes [TEST] bootinfo=ok and [TEST] boot=ok; aesynx-boot unit tests cover synthetic memory-map normalization and invalid empty-map rejection
Limitations: no bootloader memory reclamation, page-table ownership, SMP topology parsing, module parsing, or framebuffer output yet; Limine request statics are placed in the writable data segment because Limine writes them before _start, but Aesynx must not mutate them after handoff
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
