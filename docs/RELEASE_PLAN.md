# Aesynx Release Plan To 1.0

Status: planning document

This release plan is intentionally granular. The project is an operating system, so each tag should represent a small, testable step. 1.0 is the QEMU version, not a general-purpose production OS.

Naming rule: `Aesynx` is the project, kernel, and system name.

The plan uses semantic version-like tags:

```text
v0.N.0      milestone release
v0.N.P      patch/fix release for milestone N
v1.0.0      QEMU research OS release
```

The project can add more patch tags whenever needed. The list below is not a maximum. It is a serious baseline.

## Release Principles

Every release must have:

- A clear definition of done.
- A QEMU or host-test verification command.
- A completed pentest report for the exact commit being tagged.
- Serial-output markers where applicable.
- No hidden dependency on the developer's machine.
- Documentation of known limitations.
- No accidental Unix compatibility promise.

Every release should prefer:

- Small bootable increments.
- Host model tests before kernel implementation for tricky logic.
- Deterministic behavior before AI-assisted behavior.
- Capability-aware APIs even when enforcement is still simple.

## Pentest Before Tags

Every version must pass a security review and pentest before it is tagged.
This applies to tiny `v0.N.P` patch tags as well as milestone tags. A version is
not tag-ready until:

- `scripts/checks.sh` passes.
- `cargo deny check` passes.
- `cargo audit` passes.
- `scripts/generate-sbom.sh` succeeds when release artifacts exist.
- A pentest report exists at `security/pentest/<tag>.md`.
- The pentest report names the exact `Commit:` being tagged.
- The pentest report has `Status: PASS`.
- `scripts/validate-release-readiness.sh <tag>` passes.

When a version's implementation criteria are done, say so explicitly and do not
create the tag until the pentest has been completed and recorded.

### Pentest Handoff Flow

Use this loop for every version:

1. The implementation owner finishes the criteria and says it is time for a
   pentest, including the exact commit under review.
2. The maintainer runs the pentest and writes temporary findings to root
   `PENTEST.md`.
3. The findings are reviewed, release-scope issues are addressed where
   appropriate, documentation or release notes are updated, `PENTEST.md` is
   deleted, and the fixes are reported.
4. The maintainer either runs another follow-up pentest or requests a commit
   and waits for GitHub.
5. When GitHub CI and CodeQL default setup are green, the maintainer updates
   `security/pentest/<tag>.md` with the exact commit and `Status: PASS`.
6. `cargo xtask release-ready <tag>` must pass before tagging.

Never commit root `PENTEST.md`; it is a local scratch handoff file and is
ignored by git.

## Phase 0: Project Foundation

### v0.1.0 - Repository Foundation

Goal:

Create the initial Rust workspace and documentation structure.

Deliverables:

- Root `Cargo.toml`.
- `rust-toolchain.toml`.
- Rust stable `1.96.0` baseline.
- Workspace resolver `3`.
- `README.md` with project identity.
- `docs/IMPLEMENTATION_PLAN.md`.
- `docs/RELEASE_PLAN.md`.
- `docs/unsafe-policy.md`.
- `docs/modularity-policy.md`.
- `docs/threat-model.md`.
- Workspace crates declared but not all implemented.
- `xtask` crate scaffold.
- Local security/modularity check scripts.

Verification:

- `cargo check --workspace` succeeds for host-only placeholder crates.
- Documentation links are valid enough for local use.

Exit criteria:

- A new contributor can read the README and understand the 1.0 target.
- The project policy clearly forbids huge one-file implementations.

### v0.2.0 - Toolchain And Build Skeleton

Goal:

Make the x86_64 custom-target kernel build shape reviewable and locally
verifiable on stable Rust.

Deliverables:

- `targets/x86_64-unknown-aesynx.json`.
- `.cargo/config.toml`.
- `linker/kernel-x86_64.ld`.
- `crates/aesynx-kernel` scaffold.
- `crates/aesynx-log` scaffold.
- `tools/xtask` commands:
  - `cargo xtask build-kernel`
  - `cargo xtask build-kernel --custom-target-probe`
  - `cargo xtask image`
  - `cargo xtask qemu`

Verification:

- `cargo xtask build-kernel` validates the target metadata, linker script,
  Cargo config, and host kernel crate.
- Optional `cargo xtask build-kernel --custom-target-probe` documents the
  nightly-only `build-std` custom target path without making it the stable
  release gate.
- Host crates check.

Exit criteria:

- Build system shape is stable and no command silently implies a complete
  bootable kernel image.

### v0.3.0 - QEMU Image Skeleton

Goal:

Create a bootable image pipeline.

Deliverables:

- Temporary Aesynx stage-0 boot probe.
- Boot config in `boot/qemu/stage0.toml`.
- Raw image builder in `cargo xtask image`.
- QEMU runner in `cargo xtask qemu`.
- Serial marker capture.
- Documentation that real kernel boot and final bootloader handoff begin in
  `v0.4.0`.

Verification:

- `cargo xtask image` creates an image.
- `cargo xtask qemu` starts QEMU and observes
  `[TEST] bootloader=skeleton` over serial.

Exit criteria:

- Boot attempt reaches the stage-0 probe and produces deterministic serial
  output.

## Phase 1: First Boot

### v0.4.0 - First Serial Boot

Goal:

Boot x86_64 QEMU and print over serial.

Deliverables:

- Stable `x86_64-unknown-none` release kernel ELF build.
- Limine ISO image path.
- Boot config in `boot/qemu/limine.conf`.
- KASLR explicitly disabled only for the v0.4 QEMU smoke config.
- `_start` entry.
- Minimal panic handler.
- UART 16550 write path.
- Early `serial_println!`.
- Documented unsafe boundary for x86_64 port I/O.
- Image manifest records Rust, Limine, xorriso, and QEMU version banners.
- CI QEMU boot smoke validates the serial marker.

Expected serial:

```text
Aesynx: booting
arch=x86_64 platform=qemu
[TEST] boot=ok
```

Verification:

- `cargo xtask build-kernel` builds the release freestanding kernel ELF.
- `cargo xtask image` creates the release-profile Limine ISO and manifest.
- `cargo xtask qemu` observes `[TEST] boot=ok` over serial.

Exit criteria:

- First real boot.

### v0.5.0 - BootInfo Normalization

Goal:

Normalize bootloader metadata into generic `BootInfo`.

Deliverables:

- `aesynx-boot`.
- `BootInfo`.
- Memory map structures.
- Kernel image metadata.
- KASLR-enabled QEMU boot after Limine handoff parsing can populate
  `KernelImageInfo`.
- Optional framebuffer metadata.
- Optional RSDP metadata.
- Limine request boundary documented in `docs/unsafe-policy.md`.
- QEMU serial marker for successful BootInfo normalization.

Expected serial:

```text
Aesynx: booting
arch=x86_64 platform=qemu
memmap regions=<n> usable=<n> usable_bytes=<n>
rsdp=present
[TEST] bootinfo=ok
[TEST] boot=ok
```

Verification:

- Serial prints memory-map summary.
- BootInfo unit tests for synthetic maps.
- `cargo xtask qemu` observes `[TEST] bootinfo=ok` and `[TEST] boot=ok`.

Exit criteria:

- Generic kernel no longer depends directly on bootloader structs.

### v0.6.0 - Early Diagnostics

Goal:

Make panic/fault investigation viable.

Deliverables:

- Log levels.
- Boot phase tracking.
- Panic output includes file/line/core/phase.
- Early register dump where possible.
- Serial-expect panic test.

Expected serial:

```text
[TEST] panic=ok
```

Verification:

- `cargo xtask qemu --panic-smoke` produces `[TEST] panic=ok`.

Exit criteria:

- Kernel failures are readable.

## Phase 2: x86_64 CPU Foundation

### v0.7.0 - GDT And TSS

Goal:

Install basic x86_64 descriptor tables.

Deliverables:

- `aesynx-arch` trait crate.
- `aesynx-arch-x86_64`.
- GDT.
- TSS.
- Dedicated double-fault stack.

Verification:

- Boot still succeeds.
- Serial prints GDT/TSS initialized and `[TEST] gdt=ok`.

Exit criteria:

- CPU setup is no longer a placeholder.

### v0.8.0 - IDT And Exceptions

Goal:

Handle exceptions without triple faulting.

Deliverables:

- IDT.
- Exception handlers.
- Page-fault handler skeleton.
- Breakpoint handler.
- Double-fault handler.

Expected serial:

```text
[TEST] exception=ok
```

Verification:

- `cargo xtask qemu` triggers a returning breakpoint and prints
  `[TEST] exception=ok`.
- `cargo xtask qemu --exception-smoke` triggers a page fault and prints
  `[TEST] pagefault=ok`.
- No triple fault.

Exit criteria:

- Faults are diagnosable.

### v0.9.0 - Register And Fault Decoding

Goal:

Improve exception output.

Deliverables:

- Interrupt frame structure.
- Page fault error decode.
- CR2 read.
- CR3 read.
- RFLAGS/interrupt-state output.
- QEMU exception smoke must require CR2, CR3, RFLAGS, interrupt-state, and
  decoded page-fault error markers.
- CR3 output remains redacted to low flag/PCID bits; CR2 output is limited to
  presence and page offset.

Verification:

- Intentional invalid access prints redacted fault-address summary and flags.

Exit criteria:

- Page faults are useful for debugging memory work.

## Phase 3: Timer And Time

### v0.10.0 - Interrupt Controller Baseline

Goal:

Prepare hardware interrupt handling.

Deliverables:

- PIC disable path if applicable.
- Local APIC detection/init or documented interim timer path.
- IRQ vector allocation.
- EOI path.

Verification:

- Boot succeeds with interrupts configured.

Exit criteria:

- Timer work can begin.

### v0.11.0 - Timer Ticks

Goal:

Get reliable periodic ticks.

Deliverables:

- PIT-backed x86_64 QEMU timer smoke as the first chosen timer path.
- Tick counter.
- Timer interrupt handler.
- `aesynx-time` crate.

Expected serial:

```text
timer tick 1
timer tick 2
timer tick 3
[TEST] timer=ok
```

Verification:

- Serial-expect test sees controlled ticks.

Exit criteria:

- Time exists.

### v0.12.0 - Monotonic Time And Sleep Queue

Goal:

Convert ticks into useful kernel time.

Deliverables:

- Monotonic nanosecond-ish abstraction.
- Sleep queue.
- Timer callbacks.
- Basic timeout support.

Verification:

- Kernel schedules delayed log event.

Exit criteria:

- Cooperative tasks can wait on time later.

## Phase 4: Physical And Virtual Memory

### v0.13.0 - Physical Memory Map

Goal:

Classify memory correctly.

Deliverables:

- Memory region kinds.
- Memory report.
- Kernel/bootloader reserved ranges.
- Frame accounting.

Expected serial:

```text
total memory: ...
usable memory: ...
reserved: ...
[TEST] memory-map=ok
```

Verification:

- Synthetic memory-map tests.
- QEMU memory report stable.

Exit criteria:

- Allocator can trust map input.

### v0.14.0 - Bitmap Frame Allocator

Goal:

Own physical frames.

Deliverables:

- Bitmap allocator.
- Alloc/free one frame.
- Alloc contiguous.
- Debug frame states.
- Double-free detection in debug mode.

Expected serial:

```text
[TEST] frame-allocator=ok
```

Verification:

- Alloc/free smoke test in kernel.
- Host model tests.

Exit criteria:

- Physical memory is kernel-owned.

### v0.15.0 - Page Table Mapper

Goal:

Control virtual memory.

Deliverables:

- x86_64 page table structures.
- Map/unmap/translate.
- Generic page flags.
- TLB flush.

Verification:

- Map test page.
- Translate test page.
- Unmap test page.

Exit criteria:

- Kernel can create mappings intentionally.

### v0.16.0 - Kernel Mapping Policy

Goal:

Apply real memory permissions.

Deliverables:

- Kernel text RX.
- Rodata R.
- Data/BSS RW NX.
- Heap area reserved.
- Guard page test.
- Null page unmapped.

Expected serial:

```text
[TEST] paging-policy=ok
```

Verification:

- Fault when writing text.
- Fault when executing NX page.
- Fault on guard page.

Exit criteria:

- Page tables enforce basic safety.

### v0.17.0 - Early Heap

Goal:

Enable `alloc`.

Deliverables:

- Bump allocator.
- Global allocator wrapper.
- `Box`, `Vec`, `BTreeMap` smoke.
- Out-of-memory behavior.

Expected serial:

```text
[TEST] heap=ok
```

Verification:

- Kernel alloc smoke.

Exit criteria:

- Kernel can use owned data structures.

### v0.18.0 - Slab/Page Heap

Goal:

Replace bump-only heap for long-lived kernel data.

Deliverables:

- Slab classes.
- Page-backed large allocations.
- Heap stats.
- Leak/double-free debug checks where feasible.

Verification:

- Allocate/free stress smoke.

Exit criteria:

- Heap is suitable for capability/object structures.

## Phase 5: Capabilities

### v0.19.0 - Capability Model Crate

Goal:

Model capability logic under `std`.

Deliverables:

- `models/aesynx-cap-model`.
- CapId layout.
- Permission bitset.
- Derivation tests.
- Revocation tests.
- Generation tests.

Verification:

- Host tests pass.

Exit criteria:

- Model is trusted enough to implement in kernel.

### v0.20.0 - Kernel Capability Table

Goal:

Add software capabilities to the kernel.

Deliverables:

- `aesynx-cap`.
- Capability table.
- Create/check/derive/revoke.
- Generation counters.
- Redacted debug output.

Expected serial:

```text
[TEST] cap=ok
```

Verification:

- Kernel cap smoke.
- Host model tests.

Exit criteria:

- Kernel has explicit authority objects.

### v0.21.0 - Memory Capabilities

Goal:

Require caps for memory-related services.

Deliverables:

- Memory-region capabilities.
- Map permissions based on caps.
- Derive subrange cap.
- Reject extra permissions.

Verification:

- READ without permission fails.
- WRITE without permission fails.
- Derived cap cannot escape range.

Exit criteria:

- Capability model affects real kernel APIs.

### v0.22.0 - Capability Audit Events

Goal:

Make authority movement observable.

Deliverables:

- Grant/revoke audit event.
- Redaction rules.
- Telemetry event for cap faults.
- Serial debug view.

Verification:

- Grant emits event.
- Revoke emits event.
- Secret payloads are not logged.

Exit criteria:

- Capability changes are traceable.

## Phase 6: Local Objects And Services

### v0.23.0 - Object Model Crate

Goal:

Model object identity and graph logic under `std`.

Deliverables:

- `models/aesynx-object-model`.
- ObjectId.
- Object kinds.
- Immutable node model.
- Reachability tests.

Verification:

- Host tests pass.

Exit criteria:

- Object rules are clear before kernel implementation.

### v0.24.0 - Kernel Object Registry

Goal:

Add local object ownership.

Deliverables:

- `aesynx-object`.
- Local object registry.
- Memory object.
- Endpoint object.
- Queue object.
- Task placeholder object.

Verification:

- Create/list/delete local objects.
- Object caps reference objects.

Exit criteria:

- Kernel objects have identity and owner.

### v0.25.0 - Service Queue Model

Goal:

Model queue-based services.

Deliverables:

- `aesynx-ipc-model`.
- Ring queue model.
- Request/completion structures.
- Acquire/release ordering tests.

Verification:

- Host tests pass.

Exit criteria:

- Queue design is ready for kernel.

### v0.26.0 - Kernel Service Queues

Goal:

Use queues for internal services.

Deliverables:

- `aesynx-ipc`.
- Ring implementation.
- Log service queue.
- Timer service queue.
- Object service queue skeleton.

Expected serial:

```text
[TEST] service-queue=ok
```

Verification:

- Kernel client submits log request through queue.

Exit criteria:

- Service calls are not just direct function calls.

## Phase 7: Cooperative Execution

### v0.27.0 - Kernel Task Model

Goal:

Create task objects and states.

Deliverables:

- `aesynx-sched`.
- TaskId.
- TaskState.
- Local run queue.
- Wait queues.

Verification:

- Create multiple task objects.

Exit criteria:

- Scheduler data model exists.

### v0.28.0 - Cooperative Executor

Goal:

Run multiple kernel tasks cooperatively.

Deliverables:

- Local executor.
- Yield.
- Sleep.
- Wake.
- Round-robin policy.

Expected serial:

```text
task A ...
task B ...
[TEST] cooperative-sched=ok
```

Verification:

- Interleaved task smoke.

Exit criteria:

- Kernel can multiplex work.

### v0.29.0 - Scheduler Telemetry Baseline

Goal:

Start AI-readiness through scheduler traces.

Deliverables:

- Core telemetry counters.
- Task telemetry counters.
- Scheduler decision records.
- Deterministic round-robin decision reasons.

Verification:

- Trace shows why tasks ran.

Exit criteria:

- Scheduler decisions are observable.

## Phase 8: AI-Ready Telemetry Plane

### v0.30.0 - Telemetry Event Schema

Goal:

Define structured telemetry from day one.

Deliverables:

- `aesynx-telemetry`.
- Event IDs.
- Event header.
- Per-core event ring.
- Boot-phase events.
- Capability events.
- Scheduler events.

Verification:

- Events can be dumped over serial.

Exit criteria:

- Telemetry format is versioned.

### v0.31.0 - Trace Export Tool

Goal:

Make telemetry useful outside the kernel.

Deliverables:

- `tools/trace-decode`.
- Serial trace parser.
- JSON or line-based output.
- Event schema docs.

Verification:

- QEMU run produces decodable trace.

Exit criteria:

- Offline analysis is possible.

### v0.32.0 - AI Policy Interface

Goal:

Prepare for AI without using AI yet.

Deliverables:

- `aesynx-ai-policy`.
- Policy trait.
- Model object manifest structure.
- Fixed-point feature structures.
- Deterministic fallback interface.
- Safety gates.

Verification:

- Dummy model rejected/accepted according to manifest.
- Fallback always works.

Exit criteria:

- Kernel can host advisory policy modules later.

### v0.33.0 - Scheduler Policy Model Prototype

Goal:

Use a non-AI fixed-point policy in the AI policy interface.

Deliverables:

- Heuristic scheduler scorer.
- Features:
  - run queue length
  - idle ratio
  - IPC pressure
  - object locality
- Decision record.
- Fallback round-robin.

Verification:

- Heuristic can be disabled.
- Fallback produces same decisions as v0 scheduler.

Exit criteria:

- AI pathway is structurally present but safe.

## Phase 9: SMP And Aesynx Fabric

### v0.34.0 - SMP Data Structures

Goal:

Prepare per-core ownership.

Deliverables:

- CoreId.
- CoreLocal.
- Per-core registries.
- Per-core telemetry.
- Boot barriers.

Verification:

- Single-core boot uses CoreLocal.

Exit criteria:

- No subsystem assumes only global state.

### v0.35.0 - x86_64 QEMU SMP Boot

Goal:

Bring up multiple cores in QEMU.

Deliverables:

- CPU topology parsing.
- AP stacks.
- AP startup path.
- Per-core GDT/IDT/TSS where needed.
- Per-core local state.

Expected serial:

```text
core 0 online
core 1 online
core 2 online
core 3 online
[TEST] smp-boot=ok
```

Verification:

- QEMU `-smp 4` boot smoke.

Exit criteria:

- Multiple cores are online.

### v0.36.0 - Core-to-Core Ping/Pong

Goal:

Prove message fabric across cores.

Deliverables:

- Pairwise SPSC queues.
- Ping/Pong messages.
- Sequence numbers.
- Backpressure event.

Expected serial:

```text
[TEST] ipc-pingpong=ok
```

Verification:

- Core 0 pings core 1.
- Core 1 replies.

Exit criteria:

- Cores communicate by message.

### v0.37.0 - Capability Grant Over IPC

Goal:

Transfer authority across cores.

Deliverables:

- Grant message.
- Receiver CapId allocation.
- Sender permission check.
- Cross-core revoke notification.
- Audit event.

Verification:

- Grant READ cap.
- Reject WRITE without permission.
- Revoke invalidates receiver.

Exit criteria:

- IPC and capabilities are integrated.

## Phase 10: Driver Foundation

### v0.38.0 - Device Model

Goal:

Introduce devices as objects.

Deliverables:

- `aesynx-device`.
- DeviceObject.
- DeviceResources.
- DeviceState.
- Driver manifest format.
- Driver manager skeleton.
- Documented `drivers/` source-tree/package boundary.
- Driver package identity and trust-track rules.

Verification:

- Register fake device.
- Match fake driver.

Exit criteria:

- Driver lifecycle has a home.

### v0.39.0 - Bootstrap Driver Set

Goal:

Classify early drivers explicitly.

Deliverables:

- Initial top-level `drivers/` tree or documented placeholder layout.
- UART driver crate or bootstrap serial classification.
- Framebuffer driver crate or boot framebuffer wrapper classification.
- Timer driver classification.
- Interrupt controller classification.
- Bootstrap-trusted driver policy.
- Explicit statement that bootstrap drivers are exceptions, not the long-term
  driver model.

Verification:

- Boot logs show bootstrap driver states.

Exit criteria:

- Early drivers are not informal hacks.

### v0.40.0 - PCI Or Virtio Discovery

Goal:

Discover virtual hardware in QEMU.

Deliverables:

- PCI scan or virtio-mmio scan, depending on chosen QEMU path.
- Device objects created.
- Resources listed.
- Bus mastering disabled until driver bind where applicable.

Verification:

- QEMU virtio device appears in driver list.

Exit criteria:

- Hardware discovery exists.

### v0.41.0 - MMIO And IRQ Capabilities

Goal:

Give drivers narrow hardware authority.

Deliverables:

- MmioCap.
- IrqCap.
- Safe MMIO wrapper.
- IRQ endpoint object.
- Driver context grants.

Verification:

- Fake driver can read granted MMIO.
- Access outside range fails.

Exit criteria:

- Drivers are capability-limited.

### v0.42.0 - Virtio RNG

Goal:

Add simple entropy-capable virtio device.

Deliverables:

- Virtio common support.
- Virtio RNG driver.
- Entropy service queue.
- Driver telemetry.

Verification:

- Entropy request completes.

Exit criteria:

- First real virtio service works.

### v0.43.0 - Virtio Block

Goal:

Read blocks in QEMU.

Deliverables:

- Virtio block driver.
- DMA/buffer policy for QEMU.
- Block request queue.
- Read one block.

Expected serial:

```text
[TEST] virtio-blk=ok
```

Verification:

- Known block content is read.

Exit criteria:

- Storage service path exists.

### v0.44.0 - Virtio Network

Goal:

Bring up basic QEMU networking.

Deliverables:

- Virtio net driver.
- RX/TX queues.
- Basic packet send/receive smoke.
- Driver telemetry.

Verification:

- Loopback or test packet path succeeds.

Exit criteria:

- Network device path exists.

## Phase 11: Native Userspace

### v0.45.0 - User Address Space

Goal:

Create isolated user memory.

Deliverables:

- User page tables.
- User text/data/stack mappings.
- Guard page.
- Shared service queue mapping.

Verification:

- Kernel validates mapping layout.

Exit criteria:

- User-mode entry can begin.

### v0.46.0 - Enter Ring 3

Goal:

Run first user instruction.

Deliverables:

- User entry with `iretq` or chosen path.
- User stack.
- User fault handling.
- Return/exit path.

Expected serial:

```text
[TEST] user-entry=ok
```

Verification:

- Tiny user program exits or loops safely.

Exit criteria:

- User mode works.

### v0.47.0 - aesynx-abi And aesynx-rt

Goal:

Give user programs a native ABI and runtime.

Deliverables:

- `aesynx-abi`.
- `aesynx-rt`.
- Entry macro.
- Console write wrapper.
- Panic wrapper.
- Basic allocator if needed.

Verification:

- User program writes through console/log queue.

Exit criteria:

- Native userspace is ergonomic enough to grow.

### v0.48.0 - aesynx-init

Goal:

Start first native user process.

Deliverables:

- `aesynx-init`.
- Initial capability bundle.
- Boot object lookup.
- Init writes banner.

Expected serial:

```text
Aesynx userspace online
[TEST] init=ok
```

Verification:

- Kernel launches init.

Exit criteria:

- Userspace boot exists.

### v0.49.0 - Native Shell Built Into Init

Goal:

Get an interactive command loop.

Deliverables:

- Prompt.
- Line input over serial.
- Built-ins:
  - help
  - version
  - echo
  - reboot

Verification:

- Serial script sends `help`.
- Output matches.

Exit criteria:

- First command-line experience.

### v0.50.0 - Separate aesh Process

Goal:

Run shell as its own program.

Deliverables:

- `aesynx-shell`.
- Process spawn service.
- Init starts shell.
- Shell owns console caps.
- Initial shell telemetry events.

Verification:

- Init restarts shell if it exits.

Exit criteria:

- User process management works.

### v0.51.0 - Native Commands

Goal:

Run external native commands.

Deliverables:

- `/bin/echo`.
- `/bin/caps`.
- `/bin/objects`.
- `/bin/ps`.
- `/bin/log`.
- `run` shell command.
- Initial Aesynx Value Model for simple records and tables.
- Command manifest format with declared capabilities and output type.

Verification:

- Shell runs `/bin/echo hello`.
- Shell can inspect a command manifest.

Exit criteria:

- Native command ecosystem begins.

### v0.51.1 - Structured Pipeline Preview

Goal:

Start the modern Aesynx userspace model from `docs/userspace-vision.md`.

Deliverables:

- Typed pipeline channel prototype.
- `where` filter for simple record fields.
- `view` text fallback renderer for tables.
- Pipeline type mismatch error.
- Pipeline provenance event.

Verification:

- `objects /bin | view` renders a table.
- `objects /bin | where kind == "Executable" | view` filters records.
- Invalid pipeline type fails before execution.

Exit criteria:

- Aesynx pipelines are no longer text-only by design.

## Phase 12: Object Graph And Boot Bundle

### v0.52.0 - Boot Object Bundle

Goal:

Load userspace objects from a bundle.

Deliverables:

- Bundle format.
- Kernel module loading.
- Root object.
- Name index.
- `/system/init`.
- `/system/shell`.
- `/bin/*`.

Verification:

- Kernel loads init from bundle.

Exit criteria:

- Userspace is not hardcoded into kernel.

### v0.53.0 - RAM Object Graph

Goal:

Implement immutable object graph in RAM.

Deliverables:

- Object nodes.
- Content hash.
- Root set.
- Version append.
- Read old/new object.

Verification:

- Create object A.
- Create object B as new version.
- Read both.

Exit criteria:

- Object model exists beyond boot bundle.

### v0.53.1 - Object Store API Shape

Goal:

Keep the object graph independent from its future backing store.

Deliverables:

- Storage-neutral object read/write traits.
- Immutable object record shape.
- Versioned root reference model.
- Name-index object model.
- Content hash field for immutable payloads.
- Capability checks documented for object reads and root/name-index updates.

Verification:

- Host tests create objects through the storage-neutral API.
- Host tests update a name index by publishing a new immutable object.
- Host tests roll back a root reference to a previous object.

Exit criteria:

- Moving from RAM storage to persistent storage does not require changing shell
  or kernel object APIs.

### v0.54.0 - Object Shell Commands

Goal:

Expose object graph to native shell.

Deliverables:

- `objects`.
- `cat` equivalent for immutable payloads.
- `store roots`.
- `store publish`.

Verification:

- `objects /bin` lists commands.
- Read config object.

Exit criteria:

- Object store is usable from CLI.

### v0.55.0 - Object GC Prototype

Goal:

Collect unreachable RAM objects.

Deliverables:

- Mark roots.
- Trace children.
- Sweep unreachable.
- GC telemetry.

Verification:

- Test object collected only when unreachable.

Exit criteria:

- RAM object graph can run long enough for 1.0 demos.

## Phase 13: Bytecode Prototype

### v0.56.0 - Bytecode Model

Goal:

Define tiny bytecode and verifier model.

Deliverables:

- `aesynx-bytecode`.
- Instruction enum.
- Parser model.
- Verifier model.
- Fuel model.

Verification:

- Valid program accepted.
- Out-of-bounds program rejected.
- Infinite/no-fuel program rejected.

Exit criteria:

- Bytecode safety rules are concrete.

### v0.57.0 - Bytecode Interpreter

Goal:

Run tiny verified bytecode.

Deliverables:

- Interpreter.
- Fuel decrement.
- Host calls.
- Capability checks.

Verification:

- Fake module handles request.
- Unauthorized memory read fails.

Exit criteria:

- Verified extension path exists.

### v0.58.0 - Bytecode Service Demo

Goal:

Show bytecode as service logic.

Deliverables:

- Fake device or object filter module.
- Host call to send completion.
- Telemetry events.

Verification:

- Bytecode service completes request.

Exit criteria:

- Bytecode plane is real enough for 1.0 optional demo.

### v0.58.1 - WASM Component Preview

Goal:

Prepare the userspace extension model around sandboxed components.

Deliverables:

- WASM component manifest shape.
- No-authority WASM command demo.
- Capability request/deny path for WASM command.
- WASM command emits a typed value.
- AOT/cache plan documented, even if not fully implemented.

Verification:

- WASM command runs without ambient authority.
- WASM command attempting missing authority receives structured denial.

Exit criteria:

- WASM is established as the preferred untrusted extension and automation format.

## Phase 14: AI Policy Hardening

### v0.59.0 - Model Object Loader

Goal:

Load model objects safely.

Deliverables:

- Model object manifest.
- Schema version check.
- Hash check.
- Signature placeholder or real signature if crypto exists.
- Safety limits.

Verification:

- Bad schema rejected.
- Bad hash rejected.
- Safe dummy model loaded.

Exit criteria:

- AI models are treated as objects, not code blobs.

### v0.60.0 - Offline Trace Dataset Export

Goal:

Prepare future training.

Deliverables:

- Trace export includes scheduler, driver, cap, object events.
- Stable JSON schema.
- Dataset metadata.
- Boot/session IDs.

Verification:

- QEMU run produces trace dataset.

Exit criteria:

- Future AI work has data.

### v0.61.0 - Advisory Scheduler Policy

Goal:

Run an advisory policy safely.

Deliverables:

- Fixed-point heuristic model.
- Confidence score.
- Deterministic fallback.
- Model disable boot flag.
- Decision records.

Verification:

- Disable flag restores baseline.
- Model cannot choose invalid core.

Exit criteria:

- AI-ready policy path is safe and testable.

### v0.61.1 - AI-Assisted Shell Preview

Goal:

Add bounded AI hooks to native userspace without authority escalation.

Deliverables:

- Command explanation interface.
- Pipeline explanation interface.
- Capability request explanation.
- AI plan preview format.
- Explicit user approval gate before running suggested commands.
- Proof that AI context is limited to granted readable objects.

Verification:

- AI can explain `objects /bin | view`.
- AI cannot read or summarize an object without a capability.
- Suggested command does not run without explicit approval.

Exit criteria:

- AI assistance exists as a constrained shell helper, not an authority source.

## Phase 15: Integration And 1.0 Hardening

### v0.62.0 - QEMU Smoke Suite

Goal:

Make regressions visible.

Deliverables:

- Boot smoke.
- Panic smoke.
- Paging smoke.
- Capability smoke.
- Service queue smoke.
- Userspace smoke.
- Shell smoke.
- Optional virtio smoke.

Verification:

- `cargo xtask smoke-qemu` passes.

Exit criteria:

- Development can safely continue.

### v0.63.0 - Documentation Freeze 1

Goal:

Document current architecture accurately.

Deliverables:

- Architecture overview.
- Capability model.
- IPC protocol.
- Driver model.
- Native userspace.
- AI telemetry plane.
- Threat model.
- Unsafe policy.

Verification:

- Docs match code reality.

Exit criteria:

- Project is reviewable.

### v0.64.0 - Security Review Pass

Goal:

Review authority and unsafe surfaces.

Deliverables:

- Capability audit.
- Unsafe audit.
- Driver authority audit.
- Boot authority audit.
- Model-loading safety audit.

Verification:

- Findings filed or fixed.

Exit criteria:

- No known critical authority bypass remains.

### v0.65.0 - Performance And Stability Pass

Goal:

Ensure QEMU 1.0 feels stable.

Deliverables:

- Boot time measured.
- Heap stats.
- Queue stats.
- Scheduler stats.
- Long-running shell session test.
- Reboot test.

Verification:

- QEMU can run smoke loop repeatedly.

Exit criteria:

- No common crash path in 1.0 demo workflow.

### v0.66.0 - 1.0 Feature Freeze

Goal:

Stop adding features.

Deliverables:

- Final 1.0 feature list.
- Deferred feature list.
- Known issues.
- Release test checklist.

Verification:

- All required 1.0 tests pass or are explicitly blocked.

Exit criteria:

- Only fixes remain.

### v0.67.0 - 1.0 Release Candidate 1

Goal:

First complete 1.0 candidate.

Deliverables:

- QEMU image.
- Source tag.
- Release notes draft.
- Smoke test log.

Verification:

- Full smoke suite.
- Manual shell demo.

Exit criteria:

- Candidate is usable by someone other than the author.

### v0.68.0 - 1.0 Release Candidate 2

Goal:

Fix RC1 issues.

Deliverables:

- Bug fixes only.
- Updated release notes.
- Updated known issues.

Verification:

- Full smoke suite.

Exit criteria:

- No known blocker remains.

### v0.69.0 - 1.0 Release Candidate 3

Goal:

Final stabilization candidate if needed.

Deliverables:

- Bug fixes only.
- Reproducibility check.
- Documentation check.

Verification:

- Full smoke suite.
- Fresh clone build.

Exit criteria:

- Ready to tag 1.0.

## v1.0.0 - QEMU Research OS Release

Goal:

Release the first complete QEMU version of Aesynx.

Required deliverables:

- Reproducible documented build.
- QEMU x86_64 boot.
- Serial logging.
- Panic diagnostics.
- GDT/IDT/TSS.
- Timer.
- Physical frame allocator.
- Page-table mapper.
- Heap allocator.
- Capability table.
- Local object registry.
- Service queues.
- Structured telemetry.
- AI policy interface with deterministic fallback.
- Native user-mode init.
- Native shell.
- Native commands.
- Initial structured value model.
- Command capability manifests.
- Basic typed pipeline support.
- Text fallback `view` renderer.
- RAM object graph.
- Boot object bundle.
- Component manifests and object roots remain distinct inside boot bundles.
- Driver model and bootstrap drivers.
- QEMU smoke suite.
- Documentation set.

Preferred deliverables:

- QEMU SMP boot.
- Core-to-core ping/pong.
- Capability grant over IPC.
- Virtio RNG.
- Virtio block.
- Virtio network.
- Bytecode interpreter prototype.
- WASM no-authority component demo.
- WASM capability request/deny demo.
- Advisory scheduler policy demo.
- AI-assisted command explanation demo.
- Trace export dataset.

Explicit non-goals:

- POSIX compatibility.
- Bash.
- Linux binary compatibility.
- One huge monolithic OS binary.
- Unix shell semantics.
- Text-only pipeline model as the native design.
- Desktop UI.
- GPU driver.
- Real hardware support.
- Production-grade storage persistence.
- Online AI learning.
- Formal proof of the kernel.

1.0 demo script:

```text
cargo xtask image
cargo xtask qemu

Aesynx boots
panic/fault diagnostics are available
userspace starts
aesh prompt appears
commands:
  help
  version
  caps
  objects /bin
  ps
  cores
  drivers
  log
  objects /bin | view
  run /bin/echo hello
  store roots
  reboot
```

1.0 acceptance:

- A clean checkout can build and boot the QEMU image.
- Smoke tests pass.
- The shell demo works.
- Documentation clearly says what works and what does not.

## Post-1.0 Direction

### v1.1 - Storage Persistence

- Content-addressed object backend.
- Append-log persistence.
- Versioned root references.
- Versioned name-index objects.
- Native snapshots as retained object roots.
- Atomic rollback of named roots.
- Snapshot/diff/rollback userspace commands.
- Integrity verification on reads.
- Deduplication by content hash.
- Checkpoints.
- Reboot recovery.
- Garbage collection for unreachable immutable objects.
- Virtio block integration.
- Read-only FAT32 EFI boot shim where needed for boot compatibility.

### v1.2 - Stronger Driver Isolation

- Driver services outside core kernel.
- Revocation lifecycle.
- Restart demo.
- IOMMU plan implementation where QEMU/hardware allows.

### v1.3 - aarch64 QEMU Preview

- QEMU `virt` boot.
- PL011 serial.
- GICv3.
- Generic timer.
- Basic memory map.

### v1.4 - Bytecode Driver Prototype

- Fake driver in bytecode.
- Verified host calls.
- Fuel enforcement.

### v1.5 - Advisory AI Scheduler Preview

- Offline trace collection.
- Fixed-point model object.
- Safe scheduler advice.
- Regression/rollback testing.

### v1.6 - Hosted Execution And Capsules Preview

- Native capsule model.
- Capsule manifest format.
- Object namespace root per capsule.
- Capability root per capsule.
- CPU, memory, IPC, and object-store budgets.
- Virtualized clock, entropy, console, network, and storage service endpoints.
- Capsule lifecycle: create, start, suspend, revoke, kill.
- Hosted Aesynx runtime design for running Aesynx userspace on another host
  kernel during development and CI.

### v1.7 - Micro-VM Compatibility Research

- Micro-VM service design.
- Virtio-style virtual devices backed by Aesynx services.
- Capability-scoped guest storage and networking.
- Explicit non-goal: unchanged OCI/Linux containers before the native capsule
  model is mature.

### v1.8 - Minimal Rust Bootloader Research

- UEFI-first Rust `no_std` bootloader prototype.
- Aesynx boot capsule manifest.
- Crypto-agile signature envelope model for the boot capsule.
- Signature verification before handoff.
- Post-quantum readiness review for boot trust metadata; no hardcoded
  permanent public-key algorithm in the capsule ABI.
- TPM measurement plan and QEMU swtpm experiment where practical.
- Declarative config only.
- No bootloader shell or scripting.
- No broad filesystem driver set.
- Limine remains the fallback boot path until the Aesynx bootloader is proven.

### v1.9 - Native Package Manager Preview

- `aesynx-pkg` host model crate.
- Package manifest model.
- Crypto-agile signature-envelope model and track policy for accepted
  algorithms.
- Track model for core, official, community, market, sovereign, and vendor.
- Local fixture registry.
- Content-addressed in-memory package store.
- Generation planning for install, remove, update, rollback, and garbage
  collection.
- Capability-manifest policy validation.
- SBOM/provenance references.
- `aepkg` host CLI prototype for search, show, list, and transaction planning.
- Explicit non-goal: network registry fetching, paid marketplace, and GUI store
  before the host model is proven.

### v2.0 - Multi-Architecture Research Release

- x86_64 QEMU stable.
- aarch64 QEMU stable.
- Capability and object model mature.
- Driver services mature.
- Persistent object store.
- AI advisory policy stable enough for controlled workloads.
