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
#![forbid(unsafe_code)]
```

Unsafe is forbidden by default. Any crate or module that needs unsafe must first
be admitted as an explicit exception in this document, with a narrowly scoped
purpose and tests/evidence.

Unsafe-bearing architecture or kernel-entry crates may keep
`#![deny(unsafe_code)]` only so their documented modules can use narrow local
`#[allow(unsafe_code)]` exceptions. Unsafe-free crates should use
`#![forbid(unsafe_code)]` so a later local `allow` cannot silently downgrade the
crate boundary.

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
Location: crates/aesynx-arch-x86_64/src/cpu_hardening.rs
Status: active candidate in v0.16.3
Purpose: enable early x86_64 CPU hardening bits after Aesynx owns CR3
Preconditions: called during the terminal single-core normal boot smoke after the Aesynx-owned page table root has been loaded; CPUID capability detection has produced the hardening plan; NX support is required for this release path
Unsafe operation: reads/writes the admitted EFER, IA32_SPEC_CTRL, and IA32_PRED_CMD MSRs through rdmsr/wrmsr and reads/writes CR0 and CR4
Safety argument: CPUID capability reads are non-privileged safe Rust calls that only copy CPU feature values and leaf-7 hardening features are read from one snapshot; the hardening plan is host-testable and fail-closed when NX is unavailable; MSR access is narrowed to an admitted enum before the raw u32 index reaches rdmsr/wrmsr; EFER.NXE is written only after CPUID reports NX; CR0 writes preserve existing bits and only force WP on; CR4 writes preserve existing bits and only enable SMEP, SMAP, and UMIP when CPUID reports support; IA32_SPEC_CTRL bits are requested only when CPUID reports their controls; IA32_PRED_CMD receives only the one-shot IBPB bit when the shared IBRS/IBPB CPUID bit is present; wrmsr is not marked nomem; serial output reports booleans only and never dumps raw control-register values
Tests/evidence: host unit tests cover the admitted MSR set including IA32_PRED_CMD, NX-required policy, optional feature gating, strict NX/SMEP/SMAP/UMIP/IBRS/IBPB/STIBP/SSBD/ARCH_CAPABILITIES policy rejection, SPEC_CTRL read-back verification, and redacted status projection; cargo xtask qemu observes cpu-hardening nx=<bool> wp=<bool> smep=<bool> smap=<bool> umip=<bool> ibrs=<bool> ibpb_supported=<bool> stibp=<bool> ssbd=<bool> arch_capabilities=<bool> and [TEST] cpu-hardening=ok
Limitations: early single-core terminal smoke only; no SMAP usercopy access-window helpers yet, no per-core policy yet, no PCID/INVPCID handling yet, no live userspace transitions yet, no retpoline/IBRS policy choice yet, and no broader side-channel mitigation strategy beyond the CPUID-gated controls requested here
```

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
Status: active in v0.6; expanded in v0.16.2
Purpose: capture a redacted early x86_64 register summary for panic diagnostics and expose the reviewed CR3 load primitive used by kernel-owned address-space activation
Preconditions: called during early kernel execution on x86_64 after Limine transfers control to the kernel
Unsafe operation: core::arch::asm! reads rsp, rbp, rflags, and cr3; the unsafe load_cr3 API writes cr3
Safety argument: the read instructions copy architectural register values into general-purpose outputs and do not create Rust references; pushfq/pop temporarily use the current stack to read RFLAGS and restore stack position before returning; raw address-bearing values remain private and serial output exposes only redacted alignment summaries, CR3 low flag/PCID bits, and arithmetic/status RFLAGS bits under mask 0x0cd5; the RFLAGS mask intentionally excludes trap/debug, interrupt-enable, I/O privilege, alignment, virtualization, and CPU-identification state; load_cr3 validates page alignment in release builds before the privileged write and is unsafe because callers must still prove the root points at a valid live level-4 table that maps the current instruction stream, active stack, and every static/data object touched after the switch
Tests/evidence: register snapshot unit tests verify Debug redaction and summary accessors; cargo xtask qemu --panic-smoke observes the panic register-summary line; cargo xtask qemu observes [TEST] kernel-cr3=ok after loading the Aesynx-owned root
Limitations: not a full interrupt-frame dump, does not capture fault address, does not expose raw KASLR-sensitive register values, and is x86_64-only; load_cr3 has no PCID/INVPCID handling or TLB shootdown policy yet
```

```text
Location: crates/aesynx-arch-x86_64/src/descriptors.rs
Status: active in v0.7
Purpose: install early x86_64 GDT, TSS, dedicated double-fault IST stack, and live segment-register state
Preconditions: called during early single-core kernel execution after Limine transfers control and before Aesynx enables interrupts
Unsafe operation: writes private static descriptor/TSS tables, executes lgdt, reloads CS through a far return, reloads SS/DS/ES, resets FS/GS selectors to null, clears admitted FS/GS base MSRs with wrmsr, executes ltr, and exposes an unsafe set_ring0_stack API for future privilege-transition setup
Safety argument: descriptor and TSS statics are private to the architecture module, protected by an atomic uninitialized/in-progress/ready state machine before CPU-visible installation, initialized once, and then treated as read-only CPU tables for the boot core; re-entrant initialization while a table is in progress fails closed instead of reporting ready; the double-fault stack is a private aligned static byte array and the TSS records its one-past-end stack pointer as required by x86_64; lgdt, segment reloads, FS/GS nulling, admitted FS/GS base MSR clearing, and ltr load CPU state from initialized static data and do not create Rust references or access untrusted pointers; FS/GS bases are explicitly zeroed here and must be set later by future per-CPU/TLS setup; set_ring0_stack is unsafe because callers must provide a valid current-core kernel stack before ring 3 execution is enabled, and release builds reject null, noncanonical, lower-half, non-16-byte-aligned, or interrupt-enabled RSP0 updates before writing TSS.rsp0; the RSP0 update is a single aligned 64-bit store and future refactors must not split it into multiple writes; SMP and ring 3 enablement must replace this global TSS/GDT storage with per-CPU tables before secondary cores or userspace transitions can use it
Tests/evidence: descriptor unit tests verify selector layout, TSS size, complete TSS descriptor base/limit encoding, double-fault IST slot properties, admitted FS/GS base MSRs, init-state constants, and release-mode RSP0 validation rules; cargo xtask qemu observes [TEST] gdt=ok
Limitations: early single-core setup only; no privilege transitions, syscall/sysret, SMP descriptor setup, or per-core TSS state yet; TSS.rsp0 intentionally remains zero until future ring 3 enablement installs a real per-core kernel stack; global GDT/TSS storage is an explicit SMP blocker
```

```text
Location: crates/aesynx-arch-x86_64/src/exceptions.rs
Status: active in v0.9
Purpose: install early x86_64 IDT entries and minimal breakpoint, page-fault, and double-fault handlers with page-fault register/error decoding
Preconditions: called during early single-core kernel execution after GDT/TSS setup and before Aesynx enables external interrupts
Unsafe operation: defines global assembly exception stubs, executes lidt, writes a private static IDT, reads a raw exception-frame pointer passed by the stubs, executes int3 for the returning breakpoint smoke, and deliberately executes an assembly load from address zero in the opt-in exception-smoke feature
Safety argument: IDT statics are private to the architecture module and protected by an atomic uninitialized/in-progress/ready state machine before CPU-visible installation; re-entrant initialization while a table is in progress fails closed instead of reporting ready; every vector is first assigned a fixed assembly catch-all stub that normalizes CPU error-code and non-error-code frames before dispatching to a deterministic halt path, and specialized breakpoint, page-fault, and double-fault handlers override their entries afterward; handler symbols are fixed assembly stubs in kernel text; the breakpoint stub saves general-purpose registers before using rdi for the dispatch argument, aligns the stack before calling Rust code, restores the exact saved-register stack before popping, removes the synthetic vector/error frame, and returns with iretq; page-fault, double-fault, and catch-all stubs align the stack, print bounded diagnostics, and halt instead of returning to faulting instructions; invalid raw exception-frame pointers are terminal and never return toward iretq; live non-exception gate installation is crate-private, boot/smoke-scoped, captures the previous IF state while masking IF with `pushfq; cli; pop`, rejects and restores callers that had already enabled maskable interrupts through a release/debug-identical Result path, and documents that NMIs require a future platform-specific exclusion strategy before this can become a general runtime IDT mutation API; bare-metal kernel rustflags disable SSE/AVX code generation until explicit FPU/SIMD context management exists; the raw exception-frame read copies only value fields used for bounded diagnostics and rejects null/misaligned pointers; page-fault diagnostics read CR2, CR3, and RFLAGS using non-memory architectural register moves, print only CR2 presence and page offset, and keep CR3 and RIP redacted to low bits; the address-zero assembly load is reachable only through the explicit exception-smoke path after the page-fault handler is installed and avoids constructing an invalid Rust pointer
Tests/evidence: IDT unit tests verify gate encoding, descriptor sizes, vector and init-state constants, invalid frame-pointer rejection, interrupt-frame copying, and page-fault error decoding; cargo clippy --target x86_64-unknown-none -p aesynx-kernel --features exception-smoke -- -D warnings passes; cargo xtask qemu observes [TEST] exception=ok; cargo xtask qemu --exception-smoke observes [TEST] pagefault=ok and the CR2/CR3/RFLAGS/decode markers
Limitations: early single-core setup only; no page-table walk, page-fault recovery, external interrupt stubs, syscall/sysret, userspace transitions, SMP-safe live IDT mutation, per-core IDT state, or recoverable unhandled-vector policy yet; the current `smp` compile-time tripwire is a release blocker, not a runtime synchronization primitive, and live IDT mutation must move to per-core tables, a shadow-IDT `lidt` swap, or a documented NMI-source quiescing strategy before secondary cores or NMI-capable runtime mutation are admitted
```

```text
Location: crates/aesynx-arch-x86_64/src/exceptions/frame.rs
Status: active candidate in v0.24.0
Purpose: decode raw x86_64 exception stack frames into bounded diagnostic values
Preconditions: called only from the private exception dispatch path with a frame pointer produced by an Aesynx assembly exception stub
Unsafe operation: copies a RawExceptionFrame from a raw pointer with read()
Safety argument: the caller-provided pointer is rejected when null or misaligned; only fixed-size integer fields are copied by value; no Rust reference to the raw frame is created or returned; vector conversion is checked before constructing the safe frame; compile-time size and field-offset assertions keep the Rust layout synchronized with the assembly stubs
Tests/evidence: exception unit tests cover invalid frame-pointer rejection, interrupt-frame field copying, public RFLAGS masking, instruction-pointer redaction helpers, and page-fault error-code decoding
Limitations: diagnostic snapshot only; no page-table walk, fault recovery, or user-copy fault containment yet
```

```text
Location: crates/aesynx-arch-x86_64/src/exceptions/idt.rs
Status: active candidate in v0.24.0
Purpose: encode x86_64 IDT descriptors and descriptor-table pointers for the private exception table
Preconditions: used only by the early single-core exception table setup and checked interrupt-gate installer
Unsafe operation: contains descriptor types consumed by lidt and interrupt-gate installation; no unsafe block is present in this module
Safety argument: descriptor fields are constructed from explicit handler addresses, the kernel code selector, the present interrupt-gate option bits, and a masked IST index; reserved bits are zeroed; the descriptor pointer is packed to match the architectural lidt operand shape
Tests/evidence: IDT unit tests verify descriptor size, handler offset encoding, selector, IST/options, and reserved fields
Limitations: descriptor writes are still non-atomic 16-byte stores in the caller; NMI-safe live replacement and per-core IDTs remain future SMP work
```

```text
Location: crates/aesynx-arch-x86_64/src/exceptions/tests.rs
Status: test-only in v0.15
Purpose: provide an unsafe extern "C" handler signature for IDT gate encoding tests
Preconditions: compiled only for architecture crate unit tests
Unsafe operation: declares a zero-body unsafe extern "C" handler fixture; the fixture is never called
Safety argument: the test needs a function item with the same ABI and type shape as real IDT handlers so descriptor encoding can be verified; no unsafe call is performed, no interrupt frame is fabricated through the fixture, and the handler body is empty
Tests/evidence: IDT gate encoding tests compare the encoded address, selector, IST slot, options, and reserved fields
Limitations: test-only; does not install a live gate or exercise hardware exception delivery
```

```text
Location: crates/aesynx-arch-x86_64/src/interrupts.rs and crates/aesynx-arch-x86_64/src/port.rs
Status: active in v0.10
Purpose: establish the early x86_64 interrupt-controller baseline
Preconditions: called during early single-core kernel execution after IDT setup and before Aesynx enables external interrupts
Unsafe operation: executes x86_64 in/out instructions for the admitted legacy 8259 PIC ports and existing COM1 ports
Safety argument: the port boundary admits only the fixed COM1 UART ports and the four 8259 PIC command/data ports; v0.10 remaps the legacy PIC out of the CPU exception-vector range before masking it, exposes checked EOI plumbing for valid legacy IRQ lines, and handles spurious IRQ7/IRQ15 through ISR checks; local APIC support is CPUID detection only and does not touch MMIO until future memory mapping owns the APIC window
Tests/evidence: unit tests verify admitted port addresses, IRQ vector allocation range, PIC remap constants, spurious IRQ constants, and explicit APIC deferred mode; cargo xtask qemu observes [TEST] irq=ok
Limitations: no APIC MMIO activation, APIC timer, production external IRQ dispatch, IRQ-to-driver routing, SMP, or per-core interrupt-controller state yet
```

```text
Location: crates/aesynx-arch-x86_64/src/timer.rs and crates/aesynx-arch-x86_64/src/lib.rs
Status: active candidate in v0.12
Purpose: prove controlled periodic timer delivery in QEMU before scheduler or production clock services exist
Preconditions: called only through the opt-in timer-smoke kernel feature after GDT, IDT, and interrupt-controller baseline setup
Unsafe operation: defines a global assembly IRQ0 stub, installs a vector 0x20 interrupt gate, writes PIT channel 0 and command ports, executes sti/cli for the smoke loop, and reads the timestamp counter through rdtsc
Safety argument: the timer smoke path admits only legacy IRQ0 and PIT ports 0x40/0x43; IRQ0 maps to the already remapped vector 0x20; the stub saves all general-purpose registers before calling Rust, preserves the exact saved-register stack while aligning the call stack for the ABI, restores registers before iretq, and sends EOI through the checked interrupt-controller path; bare-metal kernel rustflags disable SSE/AVX code generation until explicit FPU/SIMD context management exists; timer initialization uses an atomic one-time gate; the smoke wait loop has a bounded timeout diagnostic; the handler increments an atomic tick counter, disables IRQ0 once the fixed three-tick target is reached, and normal boot does not enable external interrupts
Tests/evidence: timer unit tests verify the IRQ/vector contract, PIT divisor, and configured rate; aesynx-time unit tests verify tick-to-monotonic conversion, earliest-due sleep queue ordering, timeout expiry, and overflow handling; cargo clippy --target x86_64-unknown-none -p aesynx-kernel --features timer-smoke -- -D warnings passes; cargo xtask qemu --timer-smoke observes timer tick 1, timer tick 2, timer delayed-log, [TEST] sleep=ok, timer tick 3, and [TEST] timer=ok
Limitations: QEMU PIT smoke only; no APIC timer, TSC-deadline timer, calibrated production clock source, scheduler-integrated sleep service, scheduler preemption, driver IRQ routing, SMP timer routing, or production interrupt policy yet
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
Location: crates/aesynx-kernel/src/kernel_heap/allocator.rs
Status: active candidate in v0.18
Purpose: provide a bounded reusable kernel global allocator so long-lived `alloc` containers can run after Aesynx-owned CR3 activation
Preconditions: used only on the normal single-core boot path after the kernel has loaded its own CR3 root and post-CR3 CPU hardening has passed; the static heap lives in kernel BSS and is mapped as part of the data range
Unsafe operation: none directly; owns allocator metadata, checked allocation, and checked deallocation policy
Safety argument: the heap buffer is private, page-aligned, fixed-size, and initialized exactly once before allocation; metadata mutation is serialized by a private IRQ-masked lock that rejects reentrant acquisition instead of spinning forever and restores the previous interrupt state on success and rejected acquisition paths; slab classes are fixed and pointer-sized; free-list links are written only into free heap blocks; free-list offset decoding is checked in release builds, reports corrupt sentinels through an explicit allocator error, and increments aggregate corruption telemetry when detected through checked deallocation; slab allocation, double-free membership checks, and slab-page reclaim bound their free-list walks by the cached number of blocks in slab pages for that class so corrupt valid cycles fail closed without a full heap page-state scan; slab-page reclaim validates the class free list before relinking it so corrupt metadata cannot leave a partially rewritten list behind; slab pages keep live-block counters so normal frees do not scan the full free list to detect page emptiness; page-run allocation changes page metadata before exposing the pointer; checked arithmetic guards range and alignment calculations; allocation accounting uses checked compare-exchange loops and records overflow telemetry separately from underflow/invalid-free and free-list-corruption telemetry instead of panicking, silently saturating, or conflating failure modes; helper-module field visibility is limited to `kernel_heap` internals to support implementation splitting and must remain subject to the allocator invariants described here; checked deallocation detects invalid frees and free-while-free double frees, increments aggregate tamper telemetry for invalid frees and corrupt free-list errors, and zeroes slab blocks and page runs before reuse; serial output reports aggregate byte counts, counters, and booleans only
Tests/evidence: cargo check -p aesynx-kernel --target x86_64-unknown-none compiles the alloc-enabled kernel; host tests cover pre-initialization rejection, reentrant-lock rejection without spinning, one-shot initialization, slab reuse, slab-page scan-limit cache increment/reclaim decrement, large page-run reuse, invalid-free telemetry, double-free detection, accounting-overflow telemetry, corrupt free-list head rejection, cyclic free-list rejection, corrupt free-list clear-state telemetry, zeroing before reuse, stats, and OOM without stat advancement; cargo xtask qemu observes heap bytes=<n> allocated=<n> peak=<n> slab_classes=<n> slab_allocations=<n> page_allocations=<n> frees=<n> double_free_detected=true invalid_free_detected=true accounting_overflow_detected=false corrupt_free_list_detected=false box_ok=true vec_ok=true btree_ok=true slab_reuse_ok=true page_run_ok=true stress_ok=true oom_rejected=true and [TEST] heap=ok; cargo xtask qemu-suite keeps diagnostic smokes isolated from the allocator path
Limitations: bounded static heap only; page-backed means page-sized runs inside the static kernel heap, not physical-frame-backed growth from the global frame allocator; one global allocator lock remains; double-free detection and free-page/run discovery still use bounded linear scans while interrupts are masked, so O(1) membership and free-page tracking are required before materially larger heaps or scheduler latency guarantees; the backing heap is still a `static mut` raw-address pattern that must move to explicit interior mutability or an equivalent ownership wrapper before SMP; enabling the kernel `smp` feature intentionally fails compilation while this single-core backing store remains; the standard `GlobalAlloc::dealloc(ptr, layout)` ABI cannot distinguish a delayed stale raw-pointer free from the current owner freeing the same address after reuse, so allocation-epoch ownership tokens or quarantine remain future work; no per-core heaps, allocation-while-locking policy, bounded IRQ-masked latency policy for materially larger heaps, backtrace leak reports, or full SMP allocator synchronization policy yet
```

```text
Location: crates/aesynx-kernel/src/kernel_heap/backing.rs
Status: active candidate in v0.32
Purpose: isolate the linker-retained static heap backing store and raw-address extraction from allocator policy
Preconditions: called only by `KernelHeapAllocator` during one-shot initialization before allocations are served
Unsafe operation: takes the raw address of a private static heap buffer and places that heap in a linker-retained section
Safety argument: the raw address extraction does not construct a Rust reference, read heap contents, or write heap contents; the returned numeric bound is used only to initialize allocator metadata; the static buffer is private to the kernel heap module and page-aligned by its type
Tests/evidence: cargo test -p aesynx-kernel kernel_heap covers one-shot initialization and allocation after initialization; cargo xtask qemu observes the heap smoke markers
Limitations: the backing heap remains a bounded static kernel buffer until frame-backed heap growth lands
```

```text
Location: crates/aesynx-kernel/src/kernel_heap/global_alloc.rs
Status: active candidate in v0.32
Purpose: adapt the checked kernel heap to Rust's global allocator ABI without making allocator corruption silent
Preconditions: called only through Rust allocation APIs after `KernelHeapAllocator` has been initialized on the single-core boot path
Unsafe operation: implements `GlobalAlloc` for `KernelHeapAllocator`, translating null-on-allocation-failure and ABI-provided deallocation requests into checked allocator operations
Safety argument: allocation delegates to `allocate_checked`; deallocation delegates to `deallocate_checked`; invalid free, double free, and corrupt free-list results are treated as fail-stop heap corruption in production instead of being silently discarded by the `GlobalAlloc::dealloc` ABI; the x86_64-none path emits only the redacted allocator error kind before halting
Tests/evidence: cargo test -p aesynx-kernel kernel_heap covers checked allocator rejection paths; cargo check -p aesynx-kernel --target x86_64-unknown-none compiles the target fail-stop path; scripts/checks.sh tracks this file in the unsafe inventory
Limitations: the standard `GlobalAlloc` ABI still cannot return deallocation errors to callers, so fatal heap-integrity errors halt rather than recover
```

```text
Location: crates/aesynx-kernel/src/kernel_heap/free_list.rs
Status: active candidate in v0.18
Purpose: isolate raw free-list link access and allocator-owned zeroing for the slab/page heap
Preconditions: called only by `KernelHeapAllocator` while its metadata lock is held and after pointer/range validation has selected a free slab block or page run owned by the heap
Unsafe operation: reads and writes `usize` free-list links through raw heap pointers, and uses `core::ptr::write_bytes` to zero validated allocator-owned ranges
Safety argument: all slab classes are pointer-sized and naturally aligned; links are read only from blocks already on allocator free lists and written only before publishing a block back to a free list; zeroing is requested only for validated heap blocks or page runs while the allocator owns the range; the helpers expose no public API and do not derive addresses themselves
Tests/evidence: kernel heap host tests cover slab reuse, page-run reuse, zeroing before reuse, invalid-free telemetry, double-free detection, and OOM behavior; cargo xtask qemu observes the aggregate heap smoke markers
Limitations: these helpers do not prove raw pointer ownership epoch; that remains a future allocator-token or quarantine design requirement
```

```text
Location: crates/aesynx-kernel/src/kernel_heap/test_support.rs
Status: test-only evidence in v0.18
Purpose: let host tests verify zero-before-reuse without placing raw pointer access in the test cases themselves
Preconditions: compiled only for tests; callers pass live heap allocations returned by `KernelHeapAllocator` and lengths bounded by the allocation's rounded size
Unsafe operation: writes a byte pattern to a live test allocation and reads bytes back to assert allocator-owned zeroing behavior
Safety argument: the helpers reject null pointers, are private to the kernel heap test module, and are used only immediately after successful test allocations; production code does not import this module
Tests/evidence: `cargo test -p aesynx-kernel kernel_heap` runs the slab and page-run zero-before-reuse regression tests that call these helpers
Limitations: host-test helpers are not production allocator validation and do not prove raw pointer ownership epoch
```

```text
Location: crates/aesynx-kernel/src/kernel_sections.rs
Status: active candidate in v0.16
Purpose: expose linker-provided page-granular kernel text, rodata, and data/BSS section boundaries for the kernel mapping policy smoke
Preconditions: the kernel is linked with linker/kernel-x86_64.ld, which defines the section boundary symbols consumed by this module
Unsafe operation: unsafe extern "C" declarations for linker-defined section boundary symbols
Safety argument: the module only takes symbol addresses with core::ptr::addr_of!, which does not read memory, construct references, or create mutable aliases; the resulting values are wrapped as raw virtual-address values and later validated for ordering, page alignment, and non-overlap before the policy smoke accepts them
Tests/evidence: cargo xtask qemu observes the paging-policy-model status line with text_pages, rodata_pages, data_pages, section_layout_ok=true, text_rx_ok=true, rodata_read_only_ok=true, data_rw_nx_ok=true, heap_reserved_ok=true, guard_page_ok=true, null_page_ok=true, and [TEST] paging-policy-model=ok
Limitations: linker-symbol model only; it does not replace Limine's active CR3, does not prove hardware faults on live text/rodata/data pages, and is x86_64 linker-script specific
```

```text
Location: crates/aesynx-kernel/src/page_table_install.rs
Status: active candidate in v0.16.3
Purpose: stream audited x86_64 hardware-shaped page-table entries into a page-aligned static kernel activation arena, validate the guarded activation-stack layout, switch to the private activation stack, and terminally activate the Aesynx-owned CR3 root
Preconditions: BootInfo normalization has accepted Limine's executable-address response; the static activation arena is part of kernel data/BSS; the activation stack is in the dedicated linker-managed `.aesynx_activation_stack` section preceded by a linker-reserved guard page; KernelImageInfo covers both arena and stack physical addresses; the mapper has passed kernel mapping policy and activation-stack guard verification; the destination arena is not exposed through Rust references during installation; the terminal activation path runs only after normal boot evidence has been emitted
Unsafe operation: raw address capture for the static arena and linker-provided activation-stack/guard symbols, volatile writes through raw pointers into the arena, custom `link_section` placement for the activation stack, inline assembly to switch RSP to the private activation stack, and a terminal jump into the CR3 activation continuation
Safety argument: the installer derives the activation root physical address from the arena's kernel virtual address through BootInfo's redacted KernelImageInfo translation helper, validates that the linker-provided stack guard and stack are page-aligned, adjacent, correctly sized, and ordered, rejects active-CR3 overlap before zeroing the arena and again before terminal activation, forces static pointers through runtime registers to avoid fragile high-half absolute stores, writes only the fixed ACTIVATION_TABLES * PAGE_TABLE_ENTRIES area with volatile stores, streams one table at a time from the audited mapper to avoid boot-stack pressure, aligns the private activation stack to the SysV function-entry convention before jumping to the terminal continuation, loads CR3 only after switching away from the Limine stack through a release-mode aligned-root check, and publishes compiler fences around installation/activation; serial output reports only counts, booleans, and the post-switch kernel-cr3 marker
Tests/evidence: aesynx-mm hardware-image tests verify streaming table export matches full image export and redacted image metadata; cargo xtask qemu observes hardware_tables_copied=<n>, hardware_copied=true, kernel_stack_pages=<n>, kernel_stack_guard_ok=true, [TEST] kernel-stack-guard=ok, kernel-cr3 active=true, and [TEST] kernel-cr3=ok; cargo xtask qemu-suite keeps boot, panic, exception, and timer smokes green
Limitations: early single-core terminal boot smoke only; no return to general post-switch services yet, no TLB/PCID handling yet, no per-core/per-task stack guards yet, and no reclamation of Limine page tables yet; before SMP, the static activation arena and stack must move away from `static mut` into explicit interior mutability such as `SyncUnsafeCell` where stable or an equivalent per-core ownership model; enabling the kernel `smp` feature intentionally fails compilation while the single-core activation arena and stack remain
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
Location: crates/aesynx-kernel/src/limine/abi.rs
Status: active in v0.5; split in v0.16.4
Purpose: parse Limine bootloader handoff responses and normalize them into Aesynx BootInfo
Preconditions: Limine v12.3.2-compatible bootloader loads the kernel, honours request sections, fills response pointers before _start, and keeps response data valid in bootloader-reclaimable memory during early boot
Unsafe operation: raw reads of Limine response pointers, raw pointer traversal of memory-map and framebuffer arrays, linker-provided __kernel_end symbol address, and mutable Limine request statics written by the bootloader before Rust executes
Safety argument: the unsafe code is confined to the boot handoff boundary; protocol structs, magic constants, request statics, link-section markers, and ABI layout assertions live in the private ABI module, while the normalization module checks response and array pointers for null and alignment before forming references, rejects Limine response pointers below the caller-supplied architecture kernel VMA floor, bounds memory-map copies to MAX_EARLY_MEMORY_REGIONS, validates framebuffer/HHDM/RSDP payload addresses separately before BootInfo exposure, validates lossy integer conversions, compile-time asserts the transcribed framebuffer ABI layout, and passes only value-copied metadata into the safe aesynx-boot normalization API
Tests/evidence: cargo xtask qemu observes [TEST] memory-map=ok, [TEST] bootinfo=ok, and [TEST] boot=ok; aesynx-boot unit tests cover synthetic memory-map normalization, checked memory accounting, frame counts, invalid empty-map rejection, overlapping memory rejection, and high-half kernel image validation; limine unit tests cover forward-compatible response revisions, one-shot normalization claims, and high-half canonical handoff-address validation
Limitations: no bootloader memory reclamation, page-table ownership, SMP topology parsing, module parsing, or framebuffer output yet; Limine request statics are placed in the writable data segment because Limine writes them before _start, but Aesynx must not mutate them after handoff; memory-map entry counts are bounded and each loaded entry pointer is validated, but the kernel accepts Limine's entries-array allocation extent as a bootloader trust boundary; aarch64 boot handoff must supply its own VMA floor rather than reusing x86_64 constants
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

The security policy gate now fails when the unsafe file inventory changes
without updating this document and `scripts/validate-security-policy.sh`
together. It also fails when an unsafe block is missing a local `SAFETY:`
comment. Before 1.0, extend that gate so it also fails when:

- Architecture-specific intrinsics appear outside architecture crates.
- MMIO volatile access appears outside driver/arch MMIO wrappers.
- Panic-like macros appear in security-critical runtime paths without a
  documented exception.
