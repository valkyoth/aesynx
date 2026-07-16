# Aesynx Serious Implementation Plan

Status: planning document

Repository name: `aesynx`

Kernel/system name: `Aesynx`

Naming rule: `Aesynx` is the project, kernel, and system name. Use it consistently in code, docs, boot strings, crate names, target names, and user-facing interfaces.

Repository evolution rule: the current monorepo may eventually become
`aesynx/kernel` or `aesynx/multikernel` under an `aesynx/` organization once
driver, SDK, package, and app repositories are split out. Until the driver ABI
is stable, QEMU and virtio drivers may live in-tree under `drivers/` as
ABI-shaping packages, but not as permanent kernel internals.

Primary 1.0 target: QEMU-hosted research OS, not a daily-driver desktop OS.

This document turns [initial-idea.md](initial-idea.md) and the follow-up design discussion into an implementation plan. It is intentionally concrete. The goal is to build a serious Rust operating-system project without pretending that a general-purpose Linux/Windows replacement can appear early.

The 1.0 target is a working QEMU version with:

- A Rust `no_std` kernel.
- x86_64 QEMU boot as the primary target.
- Architecture-neutral kernel policy from day one.
- A future-ready aarch64 backend skeleton from day one.
- Serial/framebuffer diagnostics.
- Interrupts, timers, paging, heap allocation, and memory ownership.
- Object-native memory that grows toward purpose-tagged allocation,
  capability-scoped mappings, revocation, secret memory, DMA isolation, and
  snapshot-aware state. See [Memory Model Roadmap](memory-model-roadmap.md).
- Software capabilities as the core authority model.
- Per-core ownership and message-passing design, even before full multicore
  bring-up.
- Native service queues instead of Unix syscalls.
- Native init, shell, runtime, and command utilities.
- A native OS world service that records kernel-stamped and service-stamped
  facts about boot, memory, capabilities, packages, drivers, snapshots, and
  policy decisions without moving high-level query logic into the kernel.
- Driver architecture prepared for isolated, restartable driver services.
- A top-level `drivers/` area for hardware-facing components so the core
  kernel does not become a driver warehouse.
- Object graph storage in RAM, with persistence planned but not required for 1.0 unless release capacity allows.
- Telemetry and AI-readiness from day one, with deterministic non-AI policies as the boot and safety baseline.
- A modular workspace structure from day one: focused crates, focused modules, no giant source files.
- A componentized system shape from day one: no one huge OS binary, even when
  a signed boot bundle packages many components together.
- Post-quantum readiness by design: cryptographic metadata, boot capsules,
  package manifests, update policy, and identity formats must stay
  algorithm-agile instead of baking in one permanent public-key scheme.

Unix/POSIX/Linux compatibility is not part of this plan. Native Aesynx userspace is part of this plan.

Container-like hosted execution is a long-term requirement, but it should be
implemented as Aesynx-native capsules: isolated object roots, explicit
capability sets, resource budgets, and virtualized service endpoints. A hosted
runtime can later run Aesynx userspace concepts on another host kernel for
development and CI, but Linux container compatibility must not define the
kernel ABI. See [Hosted Execution Roadmap](hosted-execution-roadmap.md).

Driver structure, external driver packages, and vendor/community driver policy
are tracked in [Aesynx Driver Roadmap](driver-roadmap.md).

Long-term memory policy is tracked in
[Aesynx Memory Model Roadmap](memory-model-roadmap.md). It should guide the
frame allocator, mapper, address-space, IPC, WASM, DMA, and snapshot work so
memory does not become an old process heap model with Aesynx names added later.

The native OS fact/world direction is tracked in
[Aesynx OS World Roadmap](os-world-roadmap.md). The kernel should remain a
small trust anchor, while userspace world services provide signed facts,
branchable system states, policy-aware queries, context packs, projections, and
AI-safe explanations.

## 1. Core Position

Aesynx is not "Linux in Rust" and not "Windows rewritten." It is a clean research kernel whose design center is:

- Explicit authority through capabilities.
- Per-core ownership instead of global mutable kernel state.
- Message passing instead of implicit shared-service calls.
- Revocable driver resources.
- Queue-based service APIs.
- Immutable object identities.
- Signed/versioned OS facts and branchable worlds for explainable system state.
- Deterministic policy first, AI-assisted policy later.
- Strong separation between portable policy and architecture-specific mechanism.

The project should resist early pressure to imitate Unix. A native command line is allowed. Native shell commands are allowed. Native toolchains are allowed. A Unix compatibility layer can be a separate long-term service, but it must not define the kernel model.

## 2. Non-Negotiable Engineering Rules

These rules apply from the first commit.

### 2.1 Rust and Unsafe Policy

Baseline toolchain:

- Rust stable `1.96.0`.
- Edition 2024.
- Workspace resolver `3`.
- `rust-src` is installed for `no_std` and target work.
- If a later bare-metal build step requires nightly-only functionality, that use must be isolated, documented, and treated as a toolchain exception rather than the default project baseline.

Kernel crates default to:

```rust
#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(unused_must_use)]
```

Use `#![deny(missing_docs)]` only when a crate becomes stable enough that documentation churn will not block rapid exploration. Early kernel crates can use a documented TODO policy instead.

Unsafe Rust is allowed only in these zones:

- CPU setup and architecture entry code.
- Interrupt/trap entry.
- Context switching.
- Page table manipulation.
- Raw MMIO and port I/O.
- Atomic queue internals where `UnsafeCell` is required.
- Allocator internals.
- Bootloader handoff parsing where raw physical/virtual addresses are involved.

Every unsafe block must have a `SAFETY:` comment explaining:

- Validity.
- Alignment.
- Aliasing.
- Lifetime.
- Concurrency or interrupt assumptions.

Every crate that contains non-trivial unsafe code gets an `unsafe.md` or crate-level safety section.

### 2.2 Modularity Policy

Aesynx must never be implemented as huge one-file crates.

Aesynx must also never be implemented as one huge binary OS. A boot image,
capsule, or release artifact may package multiple components for atomic
delivery and verification, but the kernel, drivers, userspace services,
commands, policies, models, and object roots must remain independently
identified, versioned, replaceable, and rollback-capable.

Core rules:

- Use focused workspace crates for subsystems.
- Keep `lib.rs` as module wiring, not implementation dumping ground.
- Keep `main.rs` or kernel entry files as orchestration only.
- Split parsing, validation, policy, state, I/O, and tests into separate modules.
- Preserve stable ABI/service boundaries between kernel, driver services,
  userspace runtime, commands, and policy/model objects.
- Prefer signed component manifests and object roots over relinking the whole
  OS for every update.
- Put pure logic into host model crates when it benefits from fuzzing, Miri, Kani, or property tests.
- Keep normal implementation files under 300 lines where practical.
- Split non-generated `.rs` files before they exceed 500 lines unless a temporary exception is documented in [modularity-policy.md](modularity-policy.md).

The release gate runs `scripts/validate-modularity-policy.sh`.

### 2.3 Architecture-Neutral Policy

Generic kernel code must not contain raw x86_64 assembly, raw aarch64 assembly, APIC-specific assumptions, or GIC-specific assumptions.

Generic policy owns:

- Scheduling decisions.
- Capability validation.
- Object ownership.
- Service queue routing.
- Driver lifecycle decisions.
- Telemetry collection contracts.
- AI policy interfaces.

Architecture backends own:

- CPU entry.
- Interrupt-vector mechanics.
- Page-table encoding.
- TLB flush mechanics.
- Context switch mechanics.
- User-mode entry.
- Per-architecture timers.
- Cache maintenance primitives.
- Atomic and memory-barrier helpers where needed.

Platform backends own:

- Boot metadata.
- Firmware discovery.
- ACPI parsing.
- Device Tree parsing.
- PCIe configuration discovery.
- QEMU machine quirks.

### 2.4 Deterministic Baseline

Every AI-related mechanism must have a deterministic fallback.

The kernel must boot and make correct decisions with:

- No AI model loaded.
- AI model rejected.
- AI model crashed.
- Telemetry unavailable.
- Performance counters unavailable.
- Scheduler model rolled back.

AI may advise. The kernel enforces.

### 2.5 Post-Quantum Readiness

Aesynx must be crypto-agile before it becomes crypto-dependent. See
[Post-Quantum Readiness Roadmap](post-quantum-readiness.md).

Core rules:

- Boot capsules, package manifests, update metadata, entitlement receipts,
  secure-channel identities, and model/policy signatures must carry algorithm
  identifiers and versioned signature envelopes.
- Stable ABIs must not assume RSA, ECDSA, Ed25519, ML-DSA, SLH-DSA, or any
  other single permanent algorithm.
- Stable ABIs must not use tiny fixed-size buffers for public keys,
  signatures, KEM ciphertexts, or certificate-like structures.
- Critical trust paths should be able to require hybrid classical plus
  post-quantum validation when the cryptographic provider layer exists.
- Unknown algorithms are rejected by default unless local policy explicitly
  admits them.
- Cryptographic migration is a generation transition with audit evidence, not
  in-place mutation of old objects.

Quantum processors are future accelerators, not the main post-quantum design
problem. If such hardware appears, Aesynx should support it through isolated
driver services, explicit device capabilities, queue-based APIs, and userspace
runtimes.

### 2.6 Capability Authority

Kernel subsystems should avoid accepting raw addresses or object IDs as authority. Where possible, APIs accept capabilities:

```rust
fn read_object(object: CapId, dst: CapId, offset: u64, len: u64) -> Result<usize>;
fn map_region(space: CapId, memory: CapId, at: VirtAddr) -> Result<()>;
fn submit_driver_request(endpoint: CapId, request: DriverRequest) -> Result<()>;
```

The capability system must defend against:

- Forged handles.
- Stale handles.
- Generation reuse.
- Cross-core stale authority.
- Confused deputy bugs.
- Capability leakage through logs.
- Use after revocation.
- Permission escalation during derivation.

### 2.7 No Global Mutable Kernel Registries

The long-term rule:

- Every object has exactly one owner core.
- Only the owner core mutates the object.
- Other cores interact with that object through messages.

Early bootstrap may use temporary global state, but it must be explicitly marked as early-only and removed before 1.0.

## 3. Workspace Layout

The workspace should be shaped for a kernel that will grow, not for a toy boot experiment.

```text
aesynx/
|-- Cargo.toml
|-- rust-toolchain.toml
|-- README.md
|-- docs/
|   |-- IMPLEMENTATION_PLAN.md
|   |-- RELEASE_PLAN.md
|   |-- ARCHITECTURE_DECISIONS.md
|   |-- initial-idea.md
|   |-- unsafe-policy.md
|   |-- supply-chain-security.md
|   |-- security-controls.md
|   |-- modularity-policy.md
|   |-- licensing.md
|   |-- capability-model.md
|   |-- ipc-protocol.md
|   |-- driver-roadmap.md
|   |-- memory-model-roadmap.md
|   |-- os-world-roadmap.md
|   |-- object-store.md
|   |-- ai-telemetry-plane.md
|   |-- native-userspace.md
|   `-- threat-model.md
|-- crates/
|   |-- aesynx-kernel/
|   |-- aesynx-boot/
|   |-- aesynx-platform/
|   |-- aesynx-arch/
|   |-- aesynx-arch-x86_64/
|   |-- aesynx-arch-aarch64/
|   |-- aesynx-log/
|   |-- aesynx-time/
|   |-- aesynx-mm/
|   |-- aesynx-cap/
|   |-- aesynx-object/
|   |-- aesynx-ipc/
|   |-- aesynx-sched/
|   |-- aesynx-telemetry/
|   |-- aesynx-ai-policy/
|   |-- aesynx-device/
|   |-- aesynx-abi/
|   |-- aesynx-rt/
|   |-- aesynx-init/
|   |-- aesynx-shell/
|   `-- aesynx-bytecode/
|-- drivers/
|   |-- README.md
|   |-- common/
|   |   |-- aesynx-driver-api/
|   |   `-- aesynx-driver-test/
|   |-- bus/
|   |   |-- pci/
|   |   |-- usb/
|   |   |-- xhci/
|   |   `-- virtio/
|   |-- console/
|   |   |-- virtio-serial/
|   |   `-- uart16550/
|   |-- network/
|   |   |-- virtio-net/
|   |   |-- e1000/
|   |   `-- rtl8139/
|   |-- storage/
|   |   |-- virtio-blk/
|   |   |-- usb-mass-storage/
|   |   |-- nvme/
|   |   `-- ahci/
|   |-- gpu/
|   |   |-- framebuffer/
|   |   |-- virtio-gpu/
|   |   |-- amd/
|   |   |-- intel/
|   |   `-- nvidia/
|   |-- input/
|   |   |-- ps2/
|   |   `-- usb-hid/
|   `-- firmware/
|       |-- acpi/
|       `-- uefi/
|-- models/
|   |-- aesynx-cap-model/
|   |-- aesynx-ipc-model/
|   |-- aesynx-object-model/
|   |-- aesynx-sched-model/
|   `-- aesynx-ai-policy-model/
|-- targets/
|   |-- x86_64-unknown-aesynx.json
|   `-- aarch64-unknown-aesynx.json
|-- linker/
|   |-- kernel-x86_64.ld
|   `-- kernel-aarch64.ld
|-- boot/
|   |-- limine.conf
|   `-- qemu/
|-- tools/
|   |-- xtask/
|   |-- image-builder/
|   |-- qemu-runner/
|   |-- serial-expect/
|   |-- trace-decode/
|   `-- model-tools/
`-- tests/
    |-- boot-smoke/
    |-- panic-smoke/
    |-- allocator-smoke/
    |-- cap-smoke/
    |-- ipc-smoke/
    |-- userspace-smoke/
    `-- qemu-fixtures/
```

## 4. Layered Architecture

### 4.1 Product Layers

```text
Aesynx
|-- Aesynx Core
|   |-- boot
|   |-- CPU setup
|   |-- interrupts
|   |-- memory
|   |-- heap
|   `-- panic/diagnostics
|-- Capability Matrix
|   |-- CapId encoding
|   |-- capability tables
|   |-- generation checks
|   |-- derivation
|   |-- revocation
|   `-- grants
|-- Aesynx Fabric
|   |-- per-core ownership
|   |-- SPSC rings
|   |-- message schemas
|   |-- service queues
|   `-- backpressure
|-- Object Graph Plane
|   |-- object IDs
|   |-- immutable nodes
|   |-- name indexes
|   |-- root sets
|   `-- RAM backend for 1.0
|-- Native Userspace Plane
|   |-- ABI
|   |-- runtime
|   |-- init
|   |-- shell
|   `-- native commands
|-- Device Plane
|   |-- driver manager
|   |-- bus discovery
|   |-- MMIO caps
|   |-- IRQ caps
|   |-- DMA caps
|   `-- virtio drivers
|-- Bytecode Plane
|   |-- tiny verifier first
|   |-- interpreter
|   |-- host calls
|   `-- driver/service extensions later
|-- Telemetry and AI Plane
|   |-- event schema
|   |-- per-core metrics
|   |-- scheduler traces
|   |-- offline training hooks
|   |-- fixed-point policy model
|   `-- rollback/fallback
`-- Compatibility Plane
    `-- explicitly out of 1.0 scope
```

### 4.2 Architecture Backends

Primary 1.0 backend:

- `x86_64` on QEMU.
- Limine or UEFI-based boot path.
- Serial UART 16550.
- GDT, IDT, TSS.
- Local APIC or simpler interrupt path depending on phase.
- x86_64 page tables.
- Ring 3 user mode.
- Virtio PCI or virtio MMIO depending on chosen QEMU configuration.

Future bootloader direction:

- Limine remains the pragmatic boot path while the OS matures.
- Aesynx should later grow a minimal Rust UEFI bootloader as a separate
  milestone.
- The future bootloader is a security gateway, not a mini-OS: verify and
  measure an Aesynx boot capsule, then hand off quickly.
- No bootloader shell, scripting language, network stack, broad filesystem
  driver set, or GRUB-style feature creep.
- See [Bootloader Roadmap](bootloader-roadmap.md).

Prepared backend:

- `aarch64` crate exists.
- Trait implementations are stubbed or partial.
- QEMU `virt` target is documented.
- No 1.0 requirement unless explicitly promoted.

Long-term backend:

- aarch64 QEMU `virt`.
- GICv3.
- Arm generic timer.
- Device Tree or ACPI.
- EL1/EL0.
- SMMU later.

Architecture priority:

1. Finish x86_64 QEMU as the reference implementation.
2. Port to aarch64 QEMU `virt` with EL1, GICv3, generic timer, PSCI, TTBR/ASID
   policy, and memory-attribute validation.
3. Port to RISC-V 64 QEMU `virt` with a reviewed SBI/M-mode boundary, S-mode
   kernel, Sv39, timer/IPI, PLIC/AIA roadmap, and PMP/Smepmp protection policy.
4. Consider RISC-V 32 only after the fabric ABI, address-width abstractions,
   and atomic requirements are proven portable.
5. Add i686, ARMv7, or MCU-style peers only for a concrete deployment need. Very
   constrained devices should usually be capability-bridged fabric peers rather
   than full Aesynx kernel targets.

## 5. Core Traits

The core trait set should be introduced early, even if only x86_64 implements it.

### 5.1 CPU

```rust
pub trait ArchCpu {
    fn arch_name() -> &'static str;
    fn wait_for_interrupt();
    fn halt_forever() -> !;
    fn enable_interrupts();
    fn disable_interrupts();
    fn interrupts_enabled() -> bool;
    fn current_core_id() -> CoreId;
    fn read_timestamp() -> u64;
}
```

### 5.2 Memory

```rust
pub trait ArchMemory {
    fn create_address_space() -> Result<AddressSpace, MemoryError>;
    fn map_page(
        space: &mut AddressSpace,
        virt: VirtAddr,
        phys: PhysAddr,
        flags: GenericPageFlags,
    ) -> Result<(), MemoryError>;
    fn unmap_page(
        space: &mut AddressSpace,
        virt: VirtAddr,
    ) -> Result<PhysAddr, MemoryError>;
    fn translate(space: &AddressSpace, virt: VirtAddr) -> Result<PhysAddr, MemoryError>;
    fn activate_address_space(space: &AddressSpace) -> Result<(), MemoryError>;
    fn flush_tlb(addr: Option<VirtAddr>) -> Result<(), MemoryError>;
}
```

Generic page flags:

```rust
pub struct GenericPageFlags {
    pub access: PageAccess,
    pub privilege: PagePrivilege,
    global: bool,
    device_memory: bool,
    cacheable: bool,
}
```

Global TLB mappings are exposed through a checked builder method and are only
valid for kernel mappings. Device/cacheability state is also set through
builders/read-only accessors so callers cannot accidentally create executable
device mappings by mutating public fields.

### 5.3 Interrupt Controller

```rust
pub trait InterruptController {
    fn init() -> Result<(), InterruptError>;
    fn enable_irq(irq: IrqLine) -> Result<(), InterruptError>;
    fn disable_irq(irq: IrqLine) -> Result<(), InterruptError>;
    fn acknowledge(irq: IrqLine) -> Result<(), InterruptError>;
    fn send_ipi(target: CoreId, vector: IpiVector) -> Result<(), InterruptError>;
}
```

### 5.4 Timer

```rust
pub trait Timer {
    fn init_periodic(rate_hz: u64) -> Result<(), TimerError>;
    fn init_oneshot(deadline_ns: u64) -> Result<(), TimerError>;
    fn now_ns() -> u64;
    fn acknowledge();
}
```

### 5.5 IOMMU and DMA

IOMMU enforcement is not required in the earliest QEMU releases, but the capability model must be designed as if IOMMU enforcement exists.

```rust
pub trait Iommu {
    fn create_domain() -> Result<DmaDomainId, DmaError>;
    fn attach_device(domain: DmaDomainId, device: DeviceId) -> Result<(), DmaError>;
    fn map_dma(
        domain: DmaDomainId,
        device_addr: DmaAddr,
        phys: PhysAddr,
        len: usize,
        perms: DmaPerms,
    ) -> Result<(), DmaError>;
    fn unmap_dma(domain: DmaDomainId, device_addr: DmaAddr) -> Result<(), DmaError>;
}
```

No-IOMMU policy:

- 1.0 QEMU may use trusted virtio/bootstrap drivers without full IOMMU.
- The driver model must label this explicitly as trusted/degraded.
- Untrusted driver service mode requires IOMMU or a safe emulation/bounce-buffer policy.

## 6. Boot and Diagnostics

### 6.1 First Boot Target

The first boot target is:

```text
cargo xtask qemu
```

Expected serial output:

```text
Aesynx: booting
arch=x86_64 platform=qemu
[TEST] boot=ok
```

The v0.4 serial path is an early single-core diagnostic path. It must use only
typed admitted UART ports, bounded transmit polling, and direct fixed-string
boot output until a real synchronized logger exists.

The v0.4 QEMU Limine config kept KASLR disabled because the kernel did not yet
consume Limine handoff metadata. v0.5 switches the QEMU Limine config to
`kaslr: yes` and uses the Limine executable-address response to populate
`KernelImageInfo`.

### 6.2 BootInfo

Bootloader-specific metadata is normalized into:

```rust
pub struct BootInfo {
    pub arch: ArchKind,
    pub platform: PlatformKind,
    pub memory_map: MemoryMap,
    pub framebuffer: Option<FramebufferInfo>,
    pub rsdp: Option<VirtAddr>,
    pub device_tree: Option<VirtAddr>,
    pub cpu_topology: CpuTopology,
    pub kernel_image: KernelImageInfo,
    pub modules: ModuleList,
}
```

The generic kernel receives only `BootInfo`.

`KernelImageInfo` contains KASLR-sensitive addresses. Its fields are private,
debug output is redacted, and address access is limited to the boot
initialization path.

BootInfo normalization is the point where the QEMU boot config switches to
KASLR enabled. A config or hardware boot path that leaves KASLR disabled after
`KernelImageInfo` is populated is a release-blocking security exception unless
the release notes justify it explicitly.

### 6.3 Logging

Early logging:

- UART 16550 on x86_64 QEMU.
- Optional framebuffer console after boot metadata exists.
- Panic logs always go to serial if possible.

Format:

```text
[core=0][phase=mm][INFO] frame allocator initialized
[core=0][phase=cap][ERROR] stale capability generation
```

Later format with epochs:

```text
[core=2][epoch=000001239][ipc][TRACE] sent GrantCap dst=3 kind=memory
```

### 6.4 Panic Handler

The panic handler should print:

- Panic message.
- File, line, column.
- Core ID.
- Current boot phase.
- Interrupts enabled.
- Stack pointer.
- Frame pointer if available.
- CR3 or active address-space ID.
- Faulting address for page faults.
- Last N telemetry events if buffer exists.

Definition of done:

- A deliberately triggered panic is readable without a debugger.
- A deliberately triggered page fault does not triple fault after IDT setup.

## 7. Memory Plan

### 7.1 Address Types

Do not pass naked `u64` except in architecture code.

```rust
#[repr(transparent)]
pub struct PhysAddr(u64);

#[repr(transparent)]
pub struct VirtAddr(u64);

#[repr(transparent)]
pub struct PhysFrame(u64);

#[repr(transparent)]
pub struct Page(u64);
```

### 7.2 Physical Allocator

Start with bitmap frame allocation.

Required invariants:

- 4 KiB frame alignment.
- No allocation from reserved memory.
- No allocation from kernel image.
- No allocation from bootloader data until reclaimed.
- No double-free in debug builds.
- No frame outside known memory map.

Debug state:

```rust
pub enum FrameState {
    Unknown,
    Free,
    Used,
    Reserved,
    Kernel,
    Bootloader,
    Device,
}
```

### 7.3 Virtual Memory

Initial x86_64 layout:

```text
0x0000_0000_0000_0000 - 0x0000_7fff_ffff_ffff    user space later
0xffff_8000_0000_0000 - 0xffff_8fff_ffff_ffff    direct physical map
0xffff_9000_0000_0000 - 0xffff_9fff_ffff_ffff    kernel heap
0xffff_a000_0000_0000 - 0xffff_afff_ffff_ffff    per-core regions
0xffff_b000_0000_0000 - 0xffff_bfff_ffff_ffff    IPC windows
0xffff_c000_0000_0000 - 0xffff_cfff_ffff_ffff    MMIO
0xffff_d000_0000_0000 - 0xffff_dfff_ffff_ffff    object cache
0xffff_e000_0000_0000 - 0xffff_efff_ffff_ffff    bytecode/cache
0xffff_f000_0000_0000 - 0xffff_ffff_ffff_ffff    kernel image/stacks
```

The layout must be documented and tested. Do not allow silent overlap.

Kernel mapping policy:

- Text: readable/executable, not writable.
- Rodata: readable, not writable, not executable unless required.
- Data/BSS: readable/writable, not executable.
- Heap: readable/writable, not executable.
- Stacks: guard pages.
- Direct map: non-executable.
- MMIO: non-executable, device/cache-disabled attributes.
- Null page: unmapped.

Shared memory policy:

- Shared buffers are object-backed capabilities, not raw physical-frame grants
  exposed to applications.
- A shared buffer may be mapped into multiple dispatchers only through explicit
  capability grants.
- Read-only sealed buffers are the preferred zero-copy path for large assets.
- Writable shared buffers require `SHARE_WRITE`, a declared synchronization
  protocol, audit events, and revocation/TLB-shootdown handling.
- Writable cross-domain memory is never represented as ordinary shared Rust
  references. It is exposed only through atomic fields, volatile byte regions,
  or audited protocol-specific wrappers. No safe `&mut T` or aliased
  non-atomic `&T` may be constructed over concurrently writable shared storage.
- Every writable-sharing protocol names permitted access widths, alignment,
  atomic orderings, ownership transitions, and recovery behavior. Non-atomic
  structured payloads require exclusive ownership transfer before access.
- Volatile access is not synchronization; non-atomic conflicting writers remain
  forbidden unless exclusive ownership has been transferred.
- The page-table mapper must distinguish intentional shared-buffer aliasing
  from accidental duplicate physical-frame ownership.

### 7.4 Heap

Stage 1:

- Bump allocator.
- No free.
- Early boot only.

Stage 2:

- Page-backed heap.
- Global allocator wrapper.
- `alloc` crate enabled.

Stage 3:

- Slab allocator classes.
- Large allocations page-backed.

Stage 4:

- Per-core allocator.
- Remote-free queues.
- No single global allocator lock.

## 8. Capability Matrix

### 8.1 Capability ID

```text
bits 0..31   index
bits 32..55  generation
bits 56..63  type tag
```

### 8.2 Capability Structure

```rust
#[repr(transparent)]
pub struct CapId(u64);

pub struct Capability {
    target: AuthorityHandle,
    base: Option<VirtAddr>,
    len: Option<u64>,
    perms: CapPerms,
    owner: PrincipalIncarnation,
    table: CapTableIncarnation,
    generation: u32,
    revocation_epoch: u64,
    kind: CapKind,
}
```

Capability fields are private. Bootstrap root construction stays inside the
capability crate, and normal authority transfer must use audited derivation or
grant paths.

Authority-bearing object identity must come from the object registry, not from
untrusted runtime callers. User-visible names, content hashes, or package/object
graph IDs may be caller supplied or content derived, but the handle used by
capabilities must carry a registry-minted incarnation that remains stable for
that logical object and cannot be recreated accidentally by moving the same
visible `ObjectId` into a different slot.

Caller identity must also be kernel-stamped. Enforcement paths must not accept
plain caller-supplied `CoreId`, `PrincipalId`, owner IDs, or table-owner values
as authorization evidence. Those IDs are useful for routing and diagnostics only
after the dispatcher has converted current CPU-local state and active
address-space state into a non-forgeable execution context or owner token.

Permissions:

```text
Common/meta:
DERIVE
GRANT
REVOKE
INTROSPECT
ADMIN (kind-scoped only)

Memory:
READ
WRITE
EXECUTE
MAP
SHARE_READ
SHARE_WRITE

Endpoint:
SEND
RECV
CALL
REPLY
NOTIFY

Other kinds use typed rights, not arbitrary reuse of unrelated bits:
AddressSpaceRights { map, unmap, protect, activate, inspect }
IrqRights { bind, ack, mask, unmask }
DmaRights { map, unmap, sync, invalidate }
SystemControlRights { typed operation IDs }
```

The wire format validates both capability kind and typed-right representation.
Invalid combinations such as endpoint execute, memory receive, or clock map are
rejected at mint, derive, decode, and live resolution. The typed-right wire
format is versioned and rejects unknown mandatory rights.

Kinds:

```text
Memory
Object
Endpoint
AddressSpace
Task
Process
Device
Mmio
Irq
Dma
Driver
Queue
Clock
Log
SystemControl
Model
Telemetry
```

### 8.3 Operations

Required operations:

- `create_memory_cap`.
- `create_shared_buffer`.
- `seal_shared_buffer_read_only`.
- `grant_shared_buffer`.
- `map_shared_buffer`.
- `derive_cap`.
- `resolve_live_authority`.
- `prepare_authorized_operation`.
- `commit_authorized_operation`.
- `grant_copy`.
- `grant_move`.
- `borrow`.
- `revoke_prospective`.
- `revoke_strong`.
- `seal`.
- `unseal`.
- `describe_for_debug`, redacted by default.

Production root minting should require a registry-issued mint ticket or a
clearly marked bootstrap-only audited path. Normal code must not be able to
construct authority by supplying arbitrary object ID, generation, and revocation
epoch values.

Move-only grants use escrow semantics:

```text
sender active
-> sender frozen, receiver pending
-> receiver active, sender invalid
```

On abort:

```text
sender frozen, receiver pending
-> sender active, receiver empty
```

The escrow coordinator owns the frozen state and commit record; sender or
receiver survival alone is not enough to recover the move. The commit
linearization point is the coordinator's durable or epoch-stamped commit
decision. The invariant is `committed active copies <= 1`. Coordinator or
receiver failure must recover without creating two active owners. Availability
is conditional on at least one trusted commit witness surviving; if a commit
might have been observed but all authoritative decision evidence is lost, the
safety-preserving result is quarantine or explicit resource loss, not blindly
aborting and restoring sender authority.

Derivation invariant:

```text
child range is within parent range
child permissions are subset of parent permissions
child kind is compatible with parent kind
parent is live
parent generation is current
child owner is explicit
```

Enforcement APIs should return short-lived checked proof types instead of
requiring every caller to remember a sequence of table, registry, endpoint, and
epoch checks. Examples:

```rust
AuthorizedOperation<'registry, MemoryRights>
CheckedEndpointSend<'dispatch>
MapPermit<'address_space>
```

Checked proofs are not ambient long-lived tickets. A proof either performs
commit-time generation/epoch revalidation, is registered as an in-flight lease
that revocation can freeze and drain, or is a read-side critical-section token
visible to the authority registry. Proofs that are only preflight evidence
cannot authorize the final mutation.

Low-level table permission checks may remain as internal preflight helpers, but
they must not be named or documented as complete authorization checks unless
they also validate the live object generation and revocation epoch.

Capability table ownership is part of the authority model. A production table
is bound to a domain/principal incarnation, owning address space or dispatcher,
quota, and revocation domain. Cross-table grants transfer authority between
those incarnations; they do not just copy slot metadata between arrays.

Grant over IPC is transactional. The sender reserves a pending receiver slot,
sends a grant proposal with a transaction ID, waits for explicit acceptance,
then commits the receiver slot. Abort, timeout, full queue, dead receiver, and
retry paths must leave no usable phantom authority and must be idempotent.
The final commit revalidates the sender's live authority and computes:

```text
delegated_rights <= requested_rights & live_sender_rights & delegable_rights
```

Powerful rights such as `ADMIN`, `REVOKE`, `GRANT`, executable/JIT,
writable-sharing, and DMA rights never propagate implicitly. `ADMIN` is not an
override bit: every administrative operation has an exact operation identifier,
`ADMIN` never satisfies a failed `READ`, `WRITE`, `MAP`, `GRANT`, or similar
typed-right check, delegation is prohibited unless the object kind explicitly
allows it, and every use is audited.

External `CapId` kind tags are routing hints only. The registry slot's live
object kind and incarnation control decoding and dispatch; a payload tag can
never authorize an unsafe downcast.

### 8.4 Revocation

Revocation must have a first simple implementation and a long-term correct design.

Early:

- Mark capability entry revoked.
- Generation mismatch prevents reuse.
- Direct checks fail after revoke.

Long-term:

- Revocation epoch.
- Derived-cap tree or revocation lists.
- Cross-core invalidation messages.
- Local capability cache flush.
- Audit event.

### 8.5 Tests

The capability model gets a `std` model crate before the kernel implementation becomes complex.

Test properties:

- No derived cap exceeds parent bounds.
- No derived cap exceeds parent permissions.
- Revoked cap fails.
- Stale generation fails.
- Stale object authority cannot resurrect when a visible object name or content
  ID is deleted, recreated, or placed in a different registry slot.
- Caller-supplied IDs cannot authorize owner-only operations without a
  kernel-stamped execution context.
- Transferred cap cannot be used by old owner.
- Copy grant preserves sender cap.
- Move grant invalidates sender cap.
- Sealed cap cannot be used until unsealed.

## 9. Object Graph Plane

### 9.1 Object Identity

```rust
#[repr(transparent)]
pub struct ObjectId(u128);

pub struct ObjectIdentity {
    pub id: ObjectId,
    pub content_hash: Hash256,
}
```

For 1.0, authority-bearing object handles are minted by the registry. They may
be backed by a monotonic boot-local allocator plus random/entropy bits if
available, but the handle must include or reference an incarnation that is tied
to the logical object identity, not only to the physical slot currently holding
the object record. The content hash is still calculated for immutable payloads.

Separate the concepts deliberately:

- Authority handle: private kernel capability target, registry minted, stable
  incarnation, not caller selected.
- Visible object ID or name: user-facing lookup key, package/object graph name,
  or content-addressed reference.
- Content hash: integrity and deduplication key for immutable payloads.

A stale capability must fail if its target was deleted, even if a later object
uses the same visible name or content ID. Recreating a visible object must create
a new authority incarnation, and generation/epoch counters must fail closed
instead of wrapping.

### 9.2 Kernel Objects

```rust
pub trait KernelObject {
    fn authority_handle(&self) -> AuthorityHandle;
    fn object_type(&self) -> ObjectType;
    fn owner_domain(&self) -> PrincipalIncarnation;
    fn routing_owner_core(&self) -> CoreId;
}
```

`routing_owner_core()` is a locality and mutation-routing fact. It is not a
security principal by itself. Authorization uses the owning domain/principal
incarnation plus capability proofs minted by the dispatcher and registry.

Object types:

- MemoryRegion.
- Endpoint.
- AddressSpace.
- Task.
- Process.
- Device.
- Driver.
- Queue.
- BytecodeModule.
- PersistentNode.
- ModelObject.
- TelemetryStream.

### 9.3 Immutable Object Graph

```rust
pub struct ObjectNode {
    pub id: ObjectId,
    pub kind: ObjectKind,
    pub hash: Hash256,
    pub parent: Option<ObjectId>,
    pub children: SmallVec<ObjectId, 8>,
    pub payload: ObjectPayload,
    pub created_epoch: u64,
    pub signature: Option<Signature>,
}
```

Mutation is append-only:

- Create new payload.
- Create new node.
- Publish new root.
- Old root remains valid.
- Garbage collection removes unreachable nodes later.

### 9.4 Human-Friendly Names

No traditional filesystem is required for 1.0. Native shell commands use name-index objects:

```text
/system/init
/system/shell
/bin/echo
/bin/caps
/config/boot
```

Internally:

```text
NameIndexObject("bin") -> ObjectId(...)
```

### 9.5 1.0 Object Store Target

For 1.0:

- RAM object graph.
- Boot object bundle loaded as module.
- Root object is visible through native shell.
- Objects can be listed and read.
- New immutable objects can be created in RAM.
- Persistence is optional for 1.0 unless release capacity allows.

Long-term:

- Content-addressed immutable object store.
- Versioned root references.
- Versioned name-index objects.
- Append log.
- Checkpoints.
- Crash recovery.
- Integrity verification on object reads.
- Deduplication by content hash.

Native disk persistence should follow the object model. Aesynx names such as
`/bin/aesh` are lookup entries in name-index objects, not proof that the kernel
has a path-first filesystem. A read-only FAT32 path may exist for EFI boot
compatibility, but it is a shim for loading the bootloader and initial object
bundle, not the native storage format.

See [Storage Roadmap](storage-roadmap.md).
- NVMe backend.
- Signed objects.
- Rollback and secure updates.

## 10. Aesynx Fabric

### 10.1 Message Model

```rust
#[repr(C)]
pub struct Message {
    pub header: MessageHeader,
    pub payload: MessagePayload,
}

#[repr(C)]
pub struct MessageHeader {
    src: CoreId,
    dst: CoreId,
    kind: MessageKind,
    seq: u64,
    reply_to: Option<MessageId>,
}

pub struct MessageRequest {
    pub dst: CoreId,
    pub kind: MessageKind,
    pub reply_to: Option<MessageId>,
}
```

Tasks submit message requests. The kernel stamps `src` and `seq` from verified
sender identity and kernel sequence state before dispatch.

Message kinds:

- Ping.
- Pong.
- SpawnTask.
- OpenObject.
- ReadObject.
- WriteObject.
- GrantCap.
- RevokeCap.
- MapMemory.
- UnmapMemory.
- DriverRequest.
- DriverReply.
- TelemetrySample.
- MigrateTask.
- SchedulerAdvice.
- ModelLoad.
- ModelReject.

### 10.2 Queue Design

Start with SPSC rings.

```rust
pub struct SpscRing<T, const N: usize> {
    head: CachePadded<AtomicUsize>,
    tail: CachePadded<AtomicUsize>,
    buffer: [UnsafeCell<MaybeUninit<T>>; N],
}
```

Memory ordering:

```text
producer writes payload
producer release-stores tail
consumer acquire-loads tail
consumer reads payload
consumer release-stores head
```

This must be correct on x86_64 and aarch64. Never rely on x86_64's stronger memory model.

### 10.3 Backpressure

Bounded queues are required. Full queue behavior:

- Return `WouldBlock`.
- Emit pressure telemetry.
- Optional retry queue.
- Scheduler can react later.

### 10.4 Capability Transfer

Sending a capability is not copying an integer. It is an authority operation.

Grant types:

- Copy grant.
- Move grant.
- Borrow grant.
- Sealed grant.
- Revocable grant.

Rules:

- Sender must hold `GRANT`.
- Receiver receives a new `CapId`.
- Grant is logged.
- Cross-core grant updates revocation metadata.

## 11. Scheduler and Execution

### 11.1 Kernel Tasks

Start cooperative.

```rust
pub struct Task {
    id: TaskId,
    owner_core: CoreId,
    state: TaskState,
    priority: Priority,
    budget: TimeBudget,
    pub context: KernelContext,
    pub telemetry: TaskTelemetry,
}
```

Task identity, ownership, priority, budget, and state are private. State changes
through checked transitions; scheduling configuration changes require explicit
future authority paths.

Task values are linear resources. Scheduler APIs that accept a task must either
commit the ownership transfer or return the rejected task together with the
error. Dropping a task on a failed queue admission is a resource leak and must
be treated like dropping an uncommitted capability.

States:

- Runnable.
- Running.
- WaitingOnMessage.
- WaitingOnTimer.
- WaitingOnObject.
- Suspended.
- Dead.

### 11.2 Scheduling Policy

v0:

- Per-core round-robin.
- No global runqueue.
- No migration.
- Deterministic.
- Small fixed queues may use linear membership scans, but any large or
  syscall-hot run/wait queue needs indexed membership tracking before it enters
  the fast path.
- Live queue mutation must be protected against local interrupt/preemption
  re-entry. Any multicore queue sharing requires explicit per-core ownership,
  IRQ-safe locking, and lock-ordering rules.
- Queue model types stay non-`Sync` until a dedicated IRQ-safe/per-core lock
  wrapper exists. Shared statics must not expose raw run/wait queues directly.

v1:

- Sleep queue.
- Timers.
- Wait queues.

v2:

- Work request messages.
- Idle core asks for work.
- Owner core may transfer.

v3:

- Affinity scoring.
- IPC locality.
- Object ownership locality.

v4:

- Fixed-point AI policy advisory model.

### 11.3 Preemption

Preemption comes after:

- Timer is stable.
- Interrupt exit path is stable.
- Context switch is tested.
- Scheduler invariants are modeled.

Do not start with preemption.

## 12. AMP/Multikernel Ownership On SMP Hardware

Aesynx should treat "SMP" as a hardware bring-up mechanism, not as the final
kernel architecture. On x86_64, additional cores are started through SMP/APIC
machinery because that is how the platform works. After a core is online,
Aesynx should move it into a software-defined AMP model: the core has an
explicit role, owns local state, and communicates with other cores through the
Aesynx fabric.

The goal is a multikernel shape:

- Per-core schedulers, allocators, registries, telemetry, and service queues.
- Explicit owner cores for mutable kernel state.
- Bounded messages and IPIs for cross-core work.
- IRQ routing to the core that owns the device or service domain.
- Capability-aware authority transfer instead of ambient shared access.
- Heterogeneous-core metadata for future aarch64 big.LITTLE and x86 P-core/E-core
  systems.
- Versioned fabric messages that can eventually cross architecture or
  accelerator-service boundaries.
- Replicated authority state with epochs and prepare/commit/abort for critical
  global changes.
- Topology-aware routing and backpressure once direct core-to-core queues are
  no longer enough.
- Fault-domain containment for restartable driver and service domains.

Traditional shared-everything SMP is a compatibility step only. It must not
become the default design for drivers, scheduling, heap growth, object
registries, or revocation.

### 12.1 Per-Core State

```rust
#[repr(C, align(64))]
pub struct CoreLocal {
    pub core_id: CoreId,
    pub apic_id_or_mpidr: CpuHardwareId,
    pub role: CoreRole,
    pub scheduler: LocalScheduler,
    pub allocator: PerCoreAllocator,
    pub object_registry: LocalObjectRegistry,
    pub cap_cache: LocalCapCache,
    pub ipc: LocalIpcState,
    pub telemetry: CoreTelemetry,
}
```

Core roles should start simple and become more explicit over time:

- Bootstrap/control-plane core.
- Scheduler/application core.
- Driver or device-service core.
- Idle/reserve core.
- Future heterogeneous performance/efficiency role.

### 12.2 1.0 Multicore Scope

The 1.0 QEMU release should support one of these two levels:

Minimum acceptable:

- Single-core kernel.
- Per-core/AMP architecture is present.
- IPC and scheduler abstractions do not block future multicore activation.
- x86_64 SMP hardware bring-up is planned and partially implemented.

Preferred:

- QEMU boots multiple x86_64 cores.
- Each core prints online.
- Each core has local state.
- Each core has an assigned role.
- Core-to-core ping/pong works.
- No global scheduler.
- No global allocator or object-registry lock is required for the ping/pong
  path.

The release plan treats multicore bring-up as a major pre-1.0 milestone. The
project can decide whether it is required for 1.0 based on complexity, but the
architecture should remain AMP/multikernel-shaped either way.

### 12.3 Mature Fabric Requirements

The mature fabric is closer to a small in-machine network than a lock-protected
shared data structure. See [Aesynx Multikernel Fabric Roadmap](multikernel-fabric-roadmap.md).

Required long-term capabilities:

- A stable fabric protocol with versioned headers, endianness rules, bounded
  payloads, sequence numbers, rejection records, and redacted diagnostics.
- Fixed-width authority fields for peer, service, object, endpoint, address
  space, and domain incarnation IDs. The protocol must not depend on native
  pointer width, Rust enum discriminants, host endianness, or compiler ABI.
- Peer metadata that can describe x86_64 cores, future aarch64 cores,
  performance/efficiency cores, driver-service domains, and trusted accelerator
  bridges.
- Replicated authority records for capability revocation, service ownership,
  routing tables, and policy state.
- Machine-local prepare/commit/abort for critical global authority updates.
- Fail-closed stale-epoch behavior.
- Topology facts for clusters, NUMA, device locality, recent latency, queue
  depth, and service load.
- Heartbeats, watchdogs, quarantine, revoke-on-fault, service rebinding, and
  restart budgets for contained service failures.

Early Aesynx should not promise full cloud-style distributed consensus. The
first target is deterministic owner-core coordination with explicit epochs and
two-phase critical updates. Quorum algorithms are later work if Aesynx ever
supports fault-tolerant peer groups or multi-machine clusters.

## 13. Native Service Queues

The kernel should expose services through queues rather than Unix-style syscalls.

### 13.1 Queue Pair

```rust
pub struct ServiceQueuePair {
    pub submit: Ring<Request>,
    pub complete: Ring<Completion>,
}
```

Queues are transport, not authority. A queue pair may carry a message only after
the caller has resolved a capability to an endpoint object with the required
rights. Endpoint metadata is kernel-stamped at send time and includes source
domain incarnation, protocol version, sequence number, transaction ID where
applicable, and redacted diagnostics context.

Endpoint rights are object-kind specific:

- `SEND` authorizes enqueueing a request or notification.
- `RECV` authorizes dequeueing or binding a receiver.
- Call/reply rights are distinct from one-way notification rights.
- Grant/revoke messages carry transaction records and are not just ordinary
  byte payloads.

Service handlers should consume checked endpoint or live-capability proof types
rather than raw `CapId` plus caller-selected owner fields.

### 13.2 Services

1.0 services:

- Log service.
- Console service.
- Clock/timer service.
- Object service.
- Process service.
- Capability inspection service.
- Driver status service.

Later:

- Network service.
- Storage service.
- Entropy service.
- GPU service.
- Model/AI service.

### 13.3 Doorbells

Early:

- Polling.

Later:

- IPI.
- Event counter.
- Sleep/wakeup.
- Hybrid spin-then-sleep.

## 14. Native Userspace

Unix is out of scope. Native Aesynx userspace is in scope.

The detailed userspace direction is captured in [Aesynx Userspace Vision](userspace-vision.md). The short version: Aesynx userspace is not Unix-compatible by default. It is capability-native, object-native, structured-data-native, WASM-extensible, and AI-assisted.

### 14.1 First User Process

The first user process is `aesynx-init`.

Boot flow:

```text
kernel initializes core services
kernel loads boot object bundle
kernel creates user address space
kernel maps aesynx-init
kernel grants initial capabilities
kernel enters user mode
aesynx-init starts shell
```

Initial capabilities:

- Console input.
- Console output.
- Process service.
- Object root.
- Log.
- Clock.
- System control.

### 14.2 User ABI

`aesynx-abi` contains all cross-boundary structs.

Requirements:

- `#[repr(C)]`.
- Versioned.
- Endianness documented.
- No Rust-specific layout crossing kernel/user boundary.
- Capability IDs are explicit.
- Queue descriptors are explicit.
- Raw wire values are separate from validated authority values. User ABI types
  such as raw object handles, raw endpoint IDs, and raw virtual addresses must
  be validated into kernel-owned safe types before enforcement.
- Message schemas should generate or centrally define codecs, redacted debug
  output, required capabilities, protocol versions, fuzz inputs, and rejection
  behavior so independent `service kind`, `message kind`, and `payload` fields
  cannot form nonsensical combinations.

### 14.3 Runtime

`aesynx-rt` provides:

- Entry macro.
- Boot-info parsing.
- Panic handling.
- Basic allocator.
- Console wrappers.
- Object wrappers.
- Process wrappers.
- Queue wrappers.

Example native app:

```rust
#![no_std]
#![no_main]

use aesynx_rt::{entry, println, Env};

entry!(main);

fn main(env: Env) -> i32 {
    println!("hello from native Aesynx userspace");
    0
}
```

### 14.4 Shell

First shell: `aesh`.

Built-ins:

- help.
- version.
- echo.
- clear.
- reboot.
- caps.
- objects.
- ps.
- cores.
- drivers.
- log.
- run.

External commands:

- `/bin/echo`.
- `/bin/caps`.
- `/bin/objects`.
- `/bin/ps`.
- `/bin/log`.
- `/bin/drivers`.

No Bash. No POSIX shell. No fork. No Unix file descriptors.

Longer-term `aesh` design:

- Native Rust built-ins for trusted core commands.
- WASM components for sandboxed plugins and automation.
- Typed structured pipelines instead of text-only pipes.
- Capability manifests for every command.
- `view` as a rich TUI renderer for tables, logs, traces, and object data.
- AI assistance for command explanation, query building, and schema-aware autocomplete without authority escalation.

### 14.5 Executable Format

Start with statically linked ELF wrapped in Aesynx executable objects.

Executable object:

```text
manifest
ELF image
required capability declarations
hash
signature optional before secure-update phase
```

Loader checks:

- Architecture.
- Hash.
- Required capabilities.
- Caller grant authority.
- Text is RX.
- Data is RW NX.
- Stack has guard page.

## 15. Device and Driver Model

### 15.1 Driver Rule

Drivers must not become unrestricted trusted kernel plugins.

Early bootstrap drivers can be in-kernel. Long-term drivers become:

- Isolated service drivers.
- Capability-limited.
- Restartable.
- Revocable.
- Signed.
- Optionally bytecode-verified.

The long-term source-tree rule is:

- `crates/` contains core kernel/system primitives and stable shared APIs.
- `drivers/` contains hardware-facing drivers grouped by bus and class.
- External community or vendor drivers are packages built against a stable
  driver ABI, not patches to the kernel tree.

The long-term user experience should be closer to an intentional driver
installer than to Linux kernel-module maintenance:

```text
aepkg search driver realtek
aepkg install driver:rtl8125
aesh drivers
aesh driver bind pci:10ec:8125 --driver driver:rtl8125
```

Those commands publish a new declarative system generation, verify the signed
driver package, match supported hardware IDs, ask policy for approval, then
start the driver as an isolated service with exact device capabilities.

Closed-source vendor drivers may be supported only as signed external driver
services. They must not link into the kernel and must not receive ambient
authority.

### 15.2 Driver Layers

```text
Driver Manager
Bus Drivers
Class Drivers
Device Drivers
```

Bus drivers discover. Device drivers operate. Class drivers expose stable service APIs.

### 15.3 Device Object

```rust
pub struct DeviceObject {
    id: ObjectId,
    name: DeviceName,
    bus: BusKind,
    address: DeviceAddress,
    resources: DeviceResources,
    owner_core: CoreId,
    state: DeviceState,
}
```

Device identity, resources, owner core, and state are private. State changes
through checked transitions so probing, binding, running, draining, and
terminal states remain ordered. Owner transfer requires an explicit future
authority path.

### 15.4 Driver Context

```rust
pub struct DriverContext {
    pub log: LogCap,
    pub device: DeviceCap,
    pub mmio: SmallVec<MmioCap, 8>,
    pub irqs: SmallVec<IrqCap, 8>,
    pub dma: Option<DmaDomainCap>,
    pub clock: ClockCap,
    pub object_store: Option<ObjectStoreCap>,
    pub service_bus: ServiceBusCap,
}
```

The driver does not get:

- Arbitrary kernel pointers.
- All physical memory.
- All devices.
- Global object-store authority.
- Raw interrupt registration.
- Unrestricted DMA.

### 15.5 Lifecycle

```text
Available
Loaded
Verified
Probing
Bound
Running
Quiescing
Draining
Stopped
Revoked
Unloaded
```

Failure states:

- Crashed.
- TimedOut.
- RevocationFailed.
- DeviceResetRequired.
- UnsafeToUnload.

### 15.6 QEMU Driver Order

1. UART 16550.
2. Framebuffer from bootloader.
3. Interrupt controller.
4. Timer.
5. PCI/virtio discovery.
6. Virtio block.
7. Virtio network.
8. Virtio RNG.

For 1.0, the minimum useful hardware set is:

- Serial console.
- Timer.
- Interrupts.
- Framebuffer or serial-only console.
- Virtio block or RAM boot bundle.
- Virtio network optional but desirable.
- Virtio RNG optional but desirable.

### 15.7 Driver Stop/Restart

Stopping a driver:

1. Mark quiescing.
2. Stop accepting new requests.
3. Notify clients.
4. Drain queues.
5. Stop DMA.
6. Disable IRQs.
7. Revoke IRQ caps.
8. Revoke MMIO caps.
9. Revoke DMA mappings.
10. Reset device if needed.
11. Kill or unload service.
12. Mark stopped or unbound.

This lifecycle is a major reason for the OS to exist.

## 16. Bytecode Plane

The bytecode plane is not necessary for first boot, but its host ABI and security model must be planned early.

### 16.1 Initial Bytecode

Start with tiny internal bytecode, not full Wasm.

Instructions:

- load_cap.
- check_perm.
- read_u64.
- write_u64.
- send_msg.
- branch_if.
- return.
- yield.

### 16.2 Verifier Rules

Verifier must prove:

- No out-of-bounds memory access.
- No uninitialized register use.
- No branch outside code.
- No invalid capability operation.
- No direct MMIO without device cap.
- No infinite loop without fuel.
- No blocking call while holding exclusive object access.

### 16.3 Fuel

Every module has fuel. Fuel exhaustion yields or kills the module according to policy.

### 16.4 Host Calls

Allowed host calls:

- host_send_message.
- host_read_object.
- host_write_object.
- host_map_buffer.
- host_complete_request.
- host_get_time.
- host_emit_telemetry.

Every host call requires a capability.

## 17. Telemetry and AI Plane

This is where the project must be careful. AI-readiness from day one does not mean putting an opaque neural model in the scheduler on day one.

It means:

- Events are structured.
- Decisions are explainable.
- Metrics are stable.
- Policies have versioned inputs and outputs.
- Models are treated as signed objects.
- Fallback is deterministic.
- Rollback exists.
- Model effects are measured.

### 17.1 AI Design Principles

1. The kernel is correct without AI.
2. AI can advise, not bypass capability checks.
3. AI decisions are bounded by policy constraints.
4. AI models are loaded as immutable model objects.
5. AI model input schema is versioned.
6. AI output schema is versioned.
7. Every AI-influenced decision emits a reason record.
8. A model can be disabled at boot.
9. A model can be rolled back.
10. A model can be rejected by signature, hash, schema, or safety policy.

### 17.2 Telemetry Schema

Core telemetry:

```rust
pub struct CoreTelemetry {
    pub run_queue_len: AtomicU64,
    pub ipc_rx_depth: AtomicU64,
    pub ipc_tx_pressure: AtomicU64,
    pub timer_ticks: AtomicU64,
    pub idle_ticks: AtomicU64,
    pub migrations_in: AtomicU64,
    pub migrations_out: AtomicU64,
    pub cap_faults: AtomicU64,
    pub page_faults: AtomicU64,
    pub driver_irqs: AtomicU64,
    pub service_queue_depth: AtomicU64,
}
```

`CoreTelemetry::snapshot()` is an advisory per-counter sample. It must not be
used as a coherent multi-counter transaction unless a future writer-side
generation or seqlock protocol is added.

Task telemetry:

```rust
pub struct TaskTelemetry {
    cpu_time_ns: u64,
    messages_sent: u64,
    messages_received: u64,
    object_reads: u64,
    object_writes: u64,
    cap_checks: u64,
    faults: u64,
    queue_wait_ns: u64,
}
```

Task telemetry is a moved, single-writer value rather than a copyable shared
counter set. Counters are updated through append-only increment/add methods and
read through snapshots.

Driver telemetry:

```rust
pub struct DriverTelemetry {
    pub requests: u64,
    pub completions: u64,
    pub errors: u64,
    pub irq_count: u64,
    pub dma_bytes_in: u64,
    pub dma_bytes_out: u64,
    pub queue_pressure: u64,
    pub restarts: u64,
}
```

Object telemetry:

```rust
pub struct ObjectTelemetry {
    pub reads: u64,
    pub writes: u64,
    pub published_versions: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub gc_pressure: u64,
}
```

### 17.3 Decision Records

Any scheduler or driver-management policy decision should be explainable:

```rust
pub struct DecisionRecord {
    pub decision_id: u64,
    pub epoch: u64,
    pub policy: PolicyId,
    pub model: Option<ModelId>,
    pub input_hash: Hash256,
    pub output: PolicyOutput,
    pub fallback_used: bool,
    pub reason: DecisionReason,
}
```

Examples:

- Task stayed on core 1 due to object locality.
- Task moved to core 3 due to runqueue pressure.
- Driver restart delayed because DMA drain incomplete.
- Model rejected because schema version mismatch.

### 17.4 AI Policy Interface

```rust
pub trait PolicyEngine {
    type Input;
    type Output;

    fn evaluate(&self, input: Self::Input) -> PolicyDecision<Self::Output>;
    fn fallback(&self, input: Self::Input) -> Self::Output;
    fn explain(&self, decision: &PolicyDecision<Self::Output>) -> DecisionReason;
}
```

Scheduler model input:

```rust
pub struct ScheduleFeatures {
    pub run_queue_len: i32,
    pub ipc_depth: i32,
    pub queue_pressure: i32,
    pub object_locality_score: i32,
    pub cache_miss_rate: i32,
    pub idle_ratio: i32,
    pub migration_cost: i32,
    pub priority: i32,
}
```

Scheduler output:

```rust
pub struct ScheduleAdvice {
    target_core: CoreId,
    confidence: Confidence,
    reason: DecisionReason,
}
```

`Confidence` is a bounded newtype, so model or heuristic confidence values
above the project maximum are rejected at construction.

All kernel models use fixed-point integer math until there is a proven reason to do otherwise.

### 17.5 Model Objects

AI models are immutable objects:

```rust
pub struct ModelObject {
    pub id: ObjectId,
    pub schema_version: u32,
    pub model_kind: ModelKind,
    pub input_schema_hash: Hash256,
    pub output_schema_hash: Hash256,
    pub weights_hash: Hash256,
    pub signature: Signature,
    pub safety_limits: ModelSafetyLimits,
}
```

Safety limits:

- Max evaluation time.
- Max memory.
- Allowed policy domain.
- Max confidence effect.
- Required fallback.
- Required telemetry fields.

### 17.6 Day-One AI Readiness Work

Required before any actual AI scheduler:

- Stable event IDs.
- Telemetry ring buffers.
- Trace export tool.
- Deterministic scheduler baseline.
- Decision record format.
- Model object manifest format.
- Policy engine trait.
- Safety gate for model loading.

### 17.7 AI Features Deferred Until After 1.0

- Online learning.
- Neural inference inside scheduler fast path.
- GPU/NPU-accelerated kernel models.
- Self-modifying policy.
- Automatic driver model selection.
- Autonomous security response without operator policy.

## 18. Security Model

Threats:

- Malicious user program.
- Buggy driver.
- Malicious driver bytecode.
- Malicious object payload.
- Compromised service.
- DMA-capable device.
- Cross-core confused deputy.
- Replay of old capability/object.
- Side-channel observer.
- Malicious or bad AI policy model.

Principles:

- Least authority.
- No ambient root.
- Capability required for every protected object.
- Immutable storage by default.
- Explicit revocation.
- Driver isolation.
- No native untrusted kernel modules by default.
- Cross-core grants are logged.
- AI never bypasses security checks.
- Unsafe code is audited.

## 19. Verification and Testing

### 19.1 Test Categories

Host tests:

- Capability model.
- IPC ring model.
- Object graph model.
- Scheduler model.
- AI policy model.

Kernel QEMU tests:

- Boot smoke.
- Panic smoke.
- Page fault smoke.
- Timer smoke.
- Allocation smoke.
- Capability smoke.
- Userspace smoke.
- Shell smoke.
- Driver smoke.

Model checking/fuzzing later:

- SPSC ring ordering.
- Capability derivation.
- Capability revocation.
- Object graph reachability.
- Bytecode parser/verifier.

### 19.2 CI Expectations

Before 1.0, CI should run:

- `cargo fmt`.
- `cargo check` host crates.
- model crate tests.
- kernel build.
- QEMU boot smoke.
- serial-output assertions.

From v0.4 onward, the CI boot smoke installs a checksum-pinned Limine release,
captures Rust/Limine/xorriso/QEMU versions in the image manifest, and verifies
the Rust-owned serial marker.

### 19.3 QEMU Serial Expectations

Every release tag should have machine-checkable serial markers:

```text
[TEST] boot=ok
[TEST] panic=ok
[TEST] mm=ok
[TEST] cap=ok
[TEST] userspace=ok
```

## 20. 1.0 Definition

Aesynx 1.0 is the QEMU research OS release.

Minimum 1.0:

- Builds reproducibly with documented toolchain.
- Boots in QEMU x86_64.
- Logs over serial.
- Has panic and fault diagnostics.
- Sets up core CPU structures.
- Owns physical memory.
- Controls page tables.
- Has heap allocation.
- Has a working software capability table.
- Has a local object registry.
- Has service queues.
- Runs at least one user-mode native process.
- Starts native init.
- Starts native shell.
- Supports basic shell commands.
- Provides RAM object graph and name index.
- Has at least one boot object bundle.
- Has structured telemetry.
- Has AI policy interfaces and deterministic fallback.
- Has driver model documentation and at least bootstrap drivers.
- Has QEMU smoke tests.

Preferred 1.0:

- Multi-core QEMU boot.
- Core-to-core ping/pong.
- Virtio block.
- Virtio network.
- Virtio RNG.
- Native shell external commands.
- Bytecode verifier prototype.
- Driver lifecycle prototype.
- Trace export tool.

Explicitly not required for 1.0:

- POSIX compatibility.
- Bash.
- Linux binary compatibility.
- Desktop GUI.
- GPU drivers.
- Wi-Fi/Bluetooth/audio.
- Real hardware support.
- Production security certification.
- Full formal verification.
- Online AI learning.
