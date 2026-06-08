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
Location: crates/aesynx-arch-x86_64/src/registers.rs
Status: active in v0.6
Purpose: capture a redacted early x86_64 register summary for panic diagnostics
Preconditions: called during early kernel execution on x86_64 after Limine transfers control to the kernel
Unsafe operation: core::arch::asm! reads rsp, rbp, rflags, and cr3
Safety argument: the instructions copy architectural register values into general-purpose outputs and do not create Rust references; pushfq/pop temporarily use the current stack to read RFLAGS and restore stack position before returning; raw address-bearing values remain private and serial output exposes only redacted alignment summaries, CR3 low flag/PCID bits, and arithmetic/status RFLAGS bits under mask 0x0cd5; the RFLAGS mask intentionally excludes trap/debug, interrupt-enable, I/O privilege, alignment, virtualization, and CPU-identification state
Tests/evidence: register snapshot unit tests verify Debug redaction and summary accessors; cargo xtask qemu --panic-smoke observes the panic register-summary line
Limitations: not a full interrupt-frame dump, does not capture fault address, does not expose raw KASLR-sensitive register values, and is x86_64-only
```

```text
Location: crates/aesynx-arch-x86_64/src/descriptors.rs
Status: active in v0.7
Purpose: install early x86_64 GDT, TSS, dedicated double-fault IST stack, and live segment-register state
Preconditions: called during early single-core kernel execution after Limine transfers control and before Aesynx enables interrupts
Unsafe operation: writes private static descriptor/TSS tables, executes lgdt, reloads CS through a far return, reloads SS/DS/ES, resets FS/GS selectors to null, executes ltr, and exposes an unsafe set_ring0_stack API for future privilege-transition setup
Safety argument: descriptor and TSS statics are private to the architecture module, initialized once, and then treated as read-only CPU tables for the boot core; the double-fault stack is a private aligned static byte array and the TSS records its one-past-end stack pointer as required by x86_64; lgdt, segment reloads, FS/GS nulling, and ltr load CPU state from initialized static data and do not create Rust references or access untrusted pointers; FS/GS bases are not configured here and must be set separately by future per-CPU/TLS setup; set_ring0_stack is unsafe because callers must provide a valid current-core kernel stack before ring 3 execution is enabled; SMP and ring 3 enablement must replace this global TSS/GDT storage with per-CPU tables before secondary cores or userspace transitions can use it
Tests/evidence: descriptor unit tests verify selector layout, TSS size, complete TSS descriptor base/limit encoding, and double-fault IST slot properties; cargo xtask qemu observes [TEST] gdt=ok
Limitations: early single-core setup only; no privilege transitions, syscall/sysret, SMP descriptor setup, or per-core TSS state yet; TSS.rsp0 intentionally remains zero until future ring 3 enablement installs a real per-core kernel stack
```

```text
Location: crates/aesynx-arch-x86_64/src/exceptions.rs
Status: active in v0.9
Purpose: install early x86_64 IDT entries and minimal breakpoint, page-fault, and double-fault handlers with page-fault register/error decoding
Preconditions: called during early single-core kernel execution after GDT/TSS setup and before Aesynx enables external interrupts
Unsafe operation: defines global assembly exception stubs, executes lidt, writes a private static IDT, reads a raw exception-frame pointer passed by the stubs, executes int3 for the returning breakpoint smoke, and deliberately executes an assembly load from address zero in the opt-in exception-smoke feature
Safety argument: IDT statics are private to the architecture module and initialized once before loading; handler symbols are fixed assembly stubs in kernel text; the breakpoint stub saves general-purpose registers before using rdi for the dispatch argument, aligns the stack before calling Rust code, restores the exact saved-register stack before popping, removes the synthetic vector/error frame, and returns with iretq; page-fault and double-fault stubs align the stack, print bounded diagnostics, and halt instead of returning to faulting instructions; the raw exception-frame read copies only value fields used for bounded diagnostics and rejects null/misaligned pointers; page-fault diagnostics read CR2, CR3, and RFLAGS using non-memory architectural register moves, print the CR2 fault address only on this early debug path, and keep CR3 and RIP redacted to low bits; the address-zero assembly load is reachable only through the explicit exception-smoke path after the page-fault handler is installed and avoids constructing an invalid Rust pointer
Tests/evidence: IDT unit tests verify gate encoding, descriptor sizes, vector constants, invalid frame-pointer rejection, interrupt-frame copying, and page-fault error decoding; cargo clippy --target x86_64-unknown-none -p aesynx-kernel --features exception-smoke -- -D warnings passes; cargo xtask qemu observes [TEST] exception=ok; cargo xtask qemu --exception-smoke observes [TEST] pagefault=ok and the CR2/CR3/RFLAGS/decode markers
Limitations: early single-core setup only; no page-table walk, page-fault recovery, external IRQ controller, syscall/sysret, userspace transitions, SMP/per-core IDT state, or comprehensive unhandled-vector diagnostics yet
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
