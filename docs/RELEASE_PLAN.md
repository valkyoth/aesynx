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
- Release notes exist at `docs/releases/<tag>-rc.md` and identify the exact
  tag.
- A pentest report exists at `security/pentest/<tag>.md`.
- The pentest report names the exact `Commit:` being tagged.
- The pentest report has `Status: PASS`.
- The pentest report has non-blank `Tester:` and `Scope:` fields and a
  `Date: YYYY-MM-DD` field.
- `sbom/aesynx.spdx.json` exists and is non-empty when the Rust workspace is
  active.
- The tag does not already exist locally.
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
   `security/pentest/<tag>.md` with the exact commit, `Status: PASS`, tester,
   date, and scope.
6. `cargo xtask release-ready <tag>` must pass before tagging; it fails if the
   root scratch `PENTEST.md` still exists, release notes are missing or do not
   match the tag, required report metadata is missing, SBOM evidence is missing
   for an active Rust workspace, or the tag already exists locally.

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
- Timer callback model represented as delayed wake events, not arbitrary
  executable callbacks in IRQ context.
- Basic timeout support.

Expected serial:

```text
timer tick 1
timer tick 2
timer delayed-log task=0 wake_id=1 at_ns=<n> ticks=<n>
[TEST] sleep=ok
timer tick 3
[TEST] timer=ok
```

Verification:

- Kernel schedules delayed log event and the QEMU timer smoke checks it.

Exit criteria:

- Cooperative tasks can wait on time later.

## Phase 4: Physical And Virtual Memory

The long-term direction for this phase is object-native, purpose-tagged,
capability-scoped memory with revocable mappings, secret-memory handling,
IOMMU-scoped DMA, copy-free IPC transfers, and snapshot-aware state. See
[Memory Model Roadmap](memory-model-roadmap.md).

### v0.13.0 - Physical Memory Map

Goal:

Classify memory correctly.

Deliverables:

- Memory region kinds.
- Memory report.
- Kernel/bootloader reserved ranges.
- Frame accounting.
- Fail-closed rejection of overlapping memory regions.

Expected serial:

```text
memory total_bytes=... total_frames=... regions=...
memory usable_bytes=... usable_frames=... usable_regions=...
memory reserved_bytes=... reserved_frames=... reserved_regions=... kernel_bytes=... bootloader_bytes=...
[TEST] memory-map=ok
```

Verification:

- Synthetic memory-map tests.
- QEMU memory report stable.

Exit criteria:

- Allocator can trust non-overlapping map input.

### v0.14.0 - Bitmap Frame Allocator

Goal:

Introduce bounded physical-frame ownership.

Deliverables:

- Safe bitmap allocator model.
- Alloc/free one frame.
- Alloc contiguous.
- Atomic failure behavior for region seeding and contiguous frees.
- Debug frame states.
- Double-free detection in debug mode.
- QEMU smoke over a bounded usable memory-map window.

Expected serial:

```text
[TEST] frame-allocator=ok
```

Verification:

- Alloc/free smoke test in kernel.
- Host model tests.
- Regression tests prove failed mark/free calls leave allocator state unchanged.

Exit criteria:

- A bounded allocator window is kernel-owned and ready to feed page-table work.

### v0.15.0 - Page Table Mapper

Goal:

Control virtual memory.

Deliverables:

- Safe bounded x86_64-shaped page-table mapper model.
- Mapper-issued opaque typed root-table identity for future address-space
  activation work without exposing raw physical addresses, model table indices,
  or allowing external root handle construction.
- Checked root-table identity that runs mapper audit before reporting the model
  root and does not mutate mapper state on success or failure.
- Unchecked page-table root reporting is test-only; external callers must use
  the audit-backed checked root path.
- Checked status path that reports mapper counters only after audit validation
  and does not mutate mapper state on success or failure.
- Unchecked page-table status reporting is test-only; external callers must use
  the audit-backed checked status path.
- Kernel address-space candidate preflight that combines audit, checked root,
  checked status, non-empty mapping state, no-user-space-mapping,
  no-user-mapping, no-device gates, and the map-time no-physical-alias
  invariant before future activation code can consume a mapper.
- User address-space candidate preflight that combines audit, checked root,
  checked status, non-empty mapping state, no-kernel-space-user-mapping,
  no-user-space-kernel-mapping, at least one low-half user mapping, no-device,
  no-global gates, and the map-time no-physical-alias invariant before future
  per-task address-space code can consume a mapper.
- Kernel and user address-space candidate preflight success and failure paths
  stay read-only against the mapper being verified.
- Kernel and user address-space candidate preflights report structural mapper
  corruption before mapping-policy violations.
- Kernel and user address-space candidate preflights reject structurally valid
  but empty address spaces before policy validation.
- User address-space candidate preflight rejects structurally valid but
  kernel-only address spaces before future per-task code can consume them.
- Map/unmap plus fail-closed single-address and contiguous byte-range
  translation, audit-backed read-only mapping lookup, and checked permission
  changes for 4 KiB pages.
- Generic page flag access and privilege fields are read-only through public
  accessors, keeping callers on constructor/builder paths for flag changes.
- Checked public page-mapping descriptor construction for callers that need to
  validate physical-address shape and flags before handing mappings to future
  activation code.
- Unchecked page-mapping descriptor construction is crate-private; external
  callers must use the checked constructor before comparing or passing mapping
  descriptors around.
- Raw x86_64 page-table entry encoding stays crate-internal until real
  activation code needs a reviewed export.
- Single-page map/protect paths build mappings through the checked descriptor
  constructor instead of open-coding a separate validation shape.
- Single-page map/protect/unmap operations validate caller input, audit existing
  mapper structure, and only then mutate slots or accounting.
- Contiguous map/protect/unmap operations are covered by accounting-drift and
  malformed-link regression tests and must reject corrupt mapper state without
  committing candidate changes back into the original mapper.
- Internal map-capacity validation rejects empty table arenas before table
  indexing.
- Internal table-path validation rejects empty table arenas and invalid slot
  indices before root-table indexing.
- Frame allocator checked status reporting rejects impossible private bitmap
  combinations, keeps status fields read-only through public accessors, is used
  by the QEMU frame-allocator smoke, and leaves unchecked status reporting
  underflow-resistant.
- Frame allocator region marking, allocation, and free paths reject corrupt
  private bitmap combinations before committing bitmap mutations.
- Frame allocator contiguous-allocation tokens are allocator-produced; external
  code can inspect start/count but cannot directly construct tokens for free
  paths, and debug output redacts the start frame.
- Frame allocator debug output redacts the allocator base frame and raw bitmap
  words while preserving aggregate counters and corruption status.
- Read-only mapping visitor for future policy checks.
- Mapping visitor rejects hidden table ownership drift before policy checks can
  trust its output.
- Read-only virtual range permission verification without physical-contiguity
  assumptions.
- Upfront flag validation for range map/protect/verification paths.
- Empty intermediate table reclamation after unmap.
- Read-only consistency audit for reachable tables and mapped-page accounting
  that does not mutate mapper state on success or failure.
- Redacted mapping summaries for page classes without reporting physical
  addresses, with fail-closed accounting-drift and corrupt-table coverage.
- Mapper-produced audit and mapping-summary reports are inspectable but not
  externally constructible through public constructors.
- Page-table debug output for mapper, mapping, translation, flush, outcome,
  root-token, raw-entry, internal raw-slot, and validated-range types does not
  dump raw table slots or address-bearing fields.
- Range operation outcome debug output is aggregate-only and reports page
  counts plus flush class without exposing page mappings or addresses.
- MM debug redaction coverage includes mapper summaries and frame allocator
  status reports without exposing raw bitmaps or address-bearing frame values.
- Mapper-produced status, audit, and mapping-summary debug output reports only
  aggregate counters.
- Page-table root debug output reports only model-root wording without exposing
  the internal model table index, physical root, or CR3 claims.
- Checked root and checked status gates reject multiple corrupt mapper shapes
  without mutating mapper state, including unreachable used tables and
  duplicate table-parent links.
- Mapping visitors run structural audit before invoking callbacks, so corrupt
  mapper state fails closed before emitting mapping records.
- Single-address read-only lookup, presence, and checked-translation failures
  preserve mapper state for invalid addresses, unmapped pages, and corrupt
  mapper accounting.
- Single-address policy rejection paths preserve mapper state when mappings
  violate address-space, privilege, executable, writable, device-memory, or
  global-bit policy.
- Read-only range and range-policy checks preserve mapper state after
  malformed table links, intermediate leaves, or accounting drift fail
  structural audit.
- Contiguous range policy validators preserve mapper state when rejecting
  gaps, zero-length ranges, oversized walks, and address-overflow ranges.
- Contiguous range lookup and checked byte-range translation preserve mapper
  state when rejecting malformed ranges, gaps, non-contiguous pages, flag
  mismatches, and walk-bound failures.
- Address-space wrapper debug output redacts the root physical frame.
- Mapper-produced status reports are inspectable but not externally
  constructible through public fields or constructors.
- Mapper-produced operation outcomes and range reports are inspectable but not
  externally constructible through public constructors.
- Raw x86_64 page-table entries expose only the checked mapping encoder and raw
  value accessor; empty/internal slot construction stays private to the mapper,
  and internal raw-slot decoding reports malformed non-empty slots, nonzero
  non-present slots, malformed next-table links, and decoded root-table child
  links as `CorruptTable`.
- Internal next-table slot construction rejects root-table child links.
- Internal next-table traversal helpers reject dangling and out-of-range child
  links before returning a table index.
- Internal next-table traversal helpers reject invalid parent/slot indices as
  corruption instead of panicking.
- Internal empty-table reclamation rejects invalid, root-child, unused, or
  out-of-range paths before mutating mapper state.
- Internal empty-table reclamation validates the full reclaim span before
  committing slot/table changes.
- Internal empty-table reclamation validates parent slots link to the exact
  child table being reclaimed.
- Fail-closed leaf decoding for lookup, protect, unmap, and page-presence
  checks.
- Map, protect, and unmap reject malformed next-table links without mutating
  mapper state.
- Kernel-only mapping policy check for candidate kernel address spaces.
- High-half kernel user-access guard policy check for future mixed address
  spaces.
- Low-half user kernel-privilege guard policy check for future mixed address
  spaces.
- No-user-space mapping policy check for candidate kernel address spaces.
- No-executable mapping policy check for candidate data-only address spaces.
- No-writable mapping policy check for candidate read-only address spaces.
- No-device mapping policy check for candidate normal-RAM address spaces.
- No-global mapping policy check for candidate local address spaces.
- Map-time no-physical-alias policy check for candidate exclusive frame
  ownership.
- Const-capacity bounded frame side index for duplicate-frame rejection and
  audit-time table/index agreement checks.
- x86_64 leaf decoding that accepts hardware-managed Accessed and Dirty bits
  without emitting them from clean mapping construction.
- Kernel-privileged virtual range policy check for mixed address spaces.
- User-privileged virtual range policy check for future user address spaces.
- Write-protected virtual range policy check for text and read-only data
  regions.
- Non-executable virtual range policy check for data, stack, and device
  regions.
- Executable virtual range policy check for text regions.
- Normal-memory virtual range policy check for RAM-only regions.
- Local virtual range policy check for non-global per-address-space mappings.
- High-half kernel-space virtual range policy check.
- Low-half user-space virtual range policy check.
- Internal virtual-space range validation rejects zero-page input before
  endpoint arithmetic.
- Internal range-walk validation rejects zero-page input before bounded walk
  accounting.
- Generic page flags carried through mappings.
- Explicit TLB flush target shape with conservative merge semantics.
- QEMU smoke for root-table identity, checked root-table identity, checked
  status, map, fail-closed translation, checked byte-range translation, mapping
  lookup, page-presence checks,
  mapped-range checks, kernel-range policy
  checks, user-range policy checks, write-protected range checks,
  non-executable range checks, executable range checks, normal-memory range
  checks, local range checks, kernel-space range checks, user-space range
  checks, no-user-space policy checks, no-executable policy checks, no-writable
  policy checks, no-device policy checks, no-global policy checks, no-alias
  policy checks, high-half kernel user-access guard checks, low-half user
  kernel-privilege guard checks, kernel-only policy checks, kernel/user
  address-space candidate preflight audit counts, permission change,
  translated offset, mapping visit, range permission verification, unmap,
  audit, reclamation, and flush target checks.

Verification:

- Host tests for root-table identity, checked root-table identity, checked
  status, map, fail-closed translation, checked byte-range translation,
  mapping lookup, page-presence checks, mapped-range checks, permission changes,
  unmap, empty-table reclamation, sibling preservation, double-map rejection,
  invalid address rejection, atomic capacity failure, mapping visitor behavior,
  mapping visitor corruption rejection, hidden table ownership drift rejection,
  mutation and lookup rejection on pre-existing accounting drift, malformed
  next-link mutation rejection,
  malformed leaf rejection, redacted page-table debug output, kernel-range policy
  checks, user-range policy checks, write-protected range checks,
  non-executable range checks, executable range checks, normal-memory range
  checks, local range checks, kernel-space range checks, user-space range
  checks, no-user-space policy checks, no-executable policy checks, no-writable
  policy checks, no-device policy checks, no-global policy checks, no-alias
  policy checks, high-half kernel user-access guard checks, low-half user
  kernel-privilege guard checks, kernel-only policy checks, kernel/user
  address-space candidate preflights, redacted mapping
  summaries, range permission verification, invalid range flag validation,
  consistency audit failures, conservative TLB flush merging, and x86_64 entry
  flag encode/decode validation.
- Normal boot emits page-table status and `[TEST] page-table=ok`.
- `cargo xtask qemu-suite` runs the boot, panic, exception, and timer smoke
  paths before v0.15 pentest handoff.

Exit criteria:

- Kernel can model intentional mappings without activating production page
  tables yet.

Post-v0.15 page-table backlog:

- Upgrade the v0.15 bounded sorted frame side index into a scalable `no_std`
  frame index before exclusive-frame checks move onto hot syscall or
  address-space activation paths. The current side index has an explicit
  mapper const-generic capacity, with a conservative QEMU-smoke default, and
  gives bounded binary duplicate lookup plus audit-time table/index agreement
  checks, but insertion and removal still shift fixed-array entries.
- Add typed 2 MiB and 1 GiB huge-page leaves only after the 4 KiB mapper is
  stable. Huge pages must include strict alignment validation, mixed-size alias
  detection, reviewed split/unmap behavior, and audit coverage.
- Add property and model-checking coverage for mapper invariants, including
  map/unmap round trips, failed-operation atomicity, duplicate physical-frame
  exclusion, and audit detection of table/index drift. Prefer host property
  tests first, then Kani/CBMC-style bounded proofs for pure mapper logic.
- Keep hardware features behind explicit arch capability checks: PCID/INVPCID
  for future address-space switching, SMEP/SMAP/UMIP for kernel/user hardening,
  CET/shadow-stack for control-flow hardening, PKU/PKS for optional
  page-granularity compartments, confidential-computing memory attributes for
  SEV-SNP/TDX-style backends, and LAM-style pointer tags only as optional
  fast-path metadata that never replaces authoritative capability generation
  checks.

### v0.16.0 - Kernel Mapping Policy

Goal:

Define and smoke-test the kernel mapping policy before live CR3 activation.

Deliverables:

- Safe `aesynx-mm` policy descriptor for kernel text, rodata, data, reserved
  heap, guard page, and null-page ranges.
- Text range must be kernel-space read/execute, write-protected, normal memory,
  and local.
- Rodata range must be kernel-space read-only, non-executable, normal memory,
  and local.
- Data/BSS range must be kernel-space read/write, non-executable, normal
  memory, and local.
- Reserved heap and guard ranges must be canonical high-half ranges that remain
  unmapped.
- Null page must be exactly page zero and remain unmapped.
- Policy ranges must be non-empty, checked for overflow, and non-overlapping.
- The normal-boot smoke must derive text, rodata, and data/BSS ranges from
  linker-exported section boundary symbols rather than hardcoded section sizes.
- Safe planning must reject linker-derived section and reserved heap/guard
  windows outside canonical high-half kernel virtual space before building a
  policy descriptor.
- Normal QEMU boot must validate the policy after the page-table mapper smoke.

Expected serial:

```text
paging-policy-model mapped_pages=<n> reserved_pages=<n> text_pages=<n> rodata_pages=<n> data_pages=<n> section_layout_ok=true text_rx_ok=true rodata_read_only_ok=true data_rw_nx_ok=true heap_reserved_ok=true guard_page_ok=true null_page_ok=true
[TEST] paging-policy-model=ok
```

Verification:

- Unit tests reject writable text, writable/executable rodata, executable data,
  user-accessible text, device/global data, mapped reserved heap, mapped guard
  pages, mapped null pages, bad null-page descriptors, low-half reserved
  ranges, zero-page ranges, overflowing ranges, and overlapping policy ranges.
- Linker script exports page-granular text, rodata, and data/BSS boundaries
  consumed by the policy smoke.
- Host unit tests cover section-plan derivation, malformed ordering, unaligned
  boundaries, low-half and noncanonical section ranges, empty reserved
  heap/guard ranges, and arithmetic overflow.
- QEMU boot requires both the policy status line and `[TEST] paging-policy-model=ok`.
- QEMU status booleans must come from successful section-plan derivation and
  `KernelMappingPolicyReport` accessors, not from freestanding smoke-local
  constants.
- Xtask marker tests and image manifests must track the status line, every
  paging-policy-model `*_ok=true` boolean, and the final paging-policy-model ok marker.
- Release notes must state that this is a policy model and smoke gate, not live
  replacement of Limine's active CR3.

Exit criteria:

- Kernel mapping policy invariants are represented in safe `no_std` code and
  release-gated by host tests plus QEMU smoke.

### v0.16.1 - BootInfo Fuzzing And Mapper Properties

Goal:

Close the first parser and mapper proof gaps before model state starts driving
hardware state.

Deliverables:

- Host fuzz target for `aesynx-boot` normalization with synthetic Limine-shaped
  memory maps, kernel image metadata, HHDM metadata, RSDP metadata, and
  framebuffer metadata.
- Seed corpus for valid maps, empty maps, overlapping maps, adjacent maps,
  overflowing ranges, malformed kernel image windows, bad HHDM values, and
  redaction-sensitive metadata.
- Property tests for mapper map/unmap round trips, failed-operation atomicity,
  duplicate physical-frame exclusion, range-walk bounds, table/index agreement,
  and audit detection of raw table corruption.
- Document which properties remain host-only and which are expected to become
  Kani/CBMC proof targets later.

Verification:

- `cargo xtask fuzz-smoke` runs the bounded host fuzz/property gate.
- Fuzz target builds and runs for a bounded CI-safe smoke duration.
- Host property tests pass under `cargo test`.
- Existing QEMU suite remains green.

Exit criteria:

- Bootloader-shaped input has fuzz coverage, and mapper invariants have
  repeatable host property evidence before live CR3 activation.

### v0.16.2 - Kernel-Owned Address Space Activation

Goal:

Turn the v0.16 kernel mapping policy from a verified model into the active
kernel address space.

Deliverables:

- Construct real x86_64 page tables from the verified kernel mapping policy.
- Allocate and zero page-table frames through the checked frame allocator path.
- Map kernel text as RX/write-protected, rodata as read-only/NX, data/BSS as
  RW/NX, and required boot/diagnostic mappings with explicit flags.
- Keep null page, guard page, and reserved heap windows unmapped.
- Minimum hardening before CR3 activation:
  - detect NX support;
  - enable EFER.NXE before installing tables that use NX bits;
  - enable CR0.WP before relying on supervisor read-only page protection;
  - fail closed if the selected boot profile requires either bit and read-back
    verification fails.
- Re-run the kernel mapping policy verifier against the hardware-shaped table
  image before loading CR3.
- Hardware-table export is admitted only from a sealed validated address-space
  proof. Raw mapper arenas, partially checked roots, and advisory status
  reports must not be accepted by the CR3 installation path.
- Switch CR3 to the Aesynx-owned root table.
- Read back CR3 in redacted form and verify that execution continues under the
  Aesynx-owned address space.
- Read back EFER.NXE, CR0.WP, and CR3 after activation; AP Rust execution stays
  blocked until the minimum bootstrap hardening state is verified.
- Keep Limine's active mappings as an input to the transition, not as the final
  security claim.

Expected serial:

```text
[TEST] kernel-cr3=ok
```

Verification:

- QEMU normal boot survives the CR3 switch and still emits all prior boot
  markers.
- Fault smoke proves that the page-fault path still works after the switch.
- Host tests prove raw mapper values cannot bypass the sealed validated
  address-space proof required for hardware export.
- QEMU or host tests prove CR3 activation does not proceed when required
  NXE/WP read-back fails.
- Release notes clearly distinguish "Aesynx-owned CR3 active" from full
  process isolation or userspace enforcement.

Exit criteria:

- Normal boot runs on kernel-owned page tables, and v0.16 mapping policy checks
  describe live kernel page-table state rather than only a synthetic mapper
  model.

### v0.16.3 - CPU Hardening And Kernel Stack Guards

Goal:

Enable cheap hardware hardening once Aesynx owns the active page tables.

Deliverables:

- CPUID-gated EFER.NXE enablement with a fail-closed path if NX is unavailable
  for a release that requires it.
- CR0.WP enablement so supervisor writes respect read-only page permissions.
- CPUID-gated SMEP, SMAP, and UMIP detection and enablement when supported.
- Read-back verification that requested EFER/CR0/CR4 hardening bits actually
  stuck, while serial output remains boolean-only and redacted.
- Strict high-assurance hardening policy exists as a tested constructor; a
  future deployment configuration mechanism must select it explicitly before
  Aesynx claims strict real-hardware enforcement of NX, SMEP, SMAP, and UMIP.
- Explicit SMAP access-window policy placeholder; no direct user-memory access
  is allowed outside audited helpers once userspace exists.
- Guard-page-backed boot stack and kernel stack layout for the active core.
- Redacted status reporting for enabled hardening bits without dumping full
  control-register state.

Expected serial:

```text
[TEST] cpu-hardening=ok
[TEST] kernel-stack-guard=ok
```

Verification:

- QEMU smoke reports the expected hardening-bit status.
- Host tests cover the CPUID policy matrix, read-back status derivation, and
  fail-closed unsupported or not-stuck cases.
- Exception smoke remains operational after stack guards are present.

Exit criteria:

- NX/write-protect and available supervisor/user separation bits are enforced
  by hardware, and kernel stack overflow is intended to fault instead of
  silently corrupting adjacent memory.

### v0.16.4 - Limine Handoff Module Split

Goal:

Remove the temporary modularity exception introduced during v0.16.3 pentest
follow-up without changing boot behavior.

Deliverables:

- Split `crates/aesynx-kernel/src/limine.rs` into a focused normalization
  module plus a private Limine ABI module.
- Move Limine protocol structs, constants, request statics, link-section
  markers, and ABI layout assertions into the private ABI module.
- Keep the safe public handoff API and `EarlyBootScratch` flow unchanged.
- Preserve all existing pointer validation, payload-address validation,
  one-shot normalization, response revision policy, and high-half VMA checks.
- Remove the `limine.rs` temporary exception from
  `docs/modularity-policy.md` once the file-size gate passes without it.

Expected serial:

```text
[TEST] bootinfo=ok
[TEST] boot=ok
```

Verification:

- `scripts/validate-modularity-policy.sh` passes without a `limine.rs`
  exception.
- Limine unit tests continue to cover response revisions, one-shot
  normalization, and high-half payload address validation.
- Normal QEMU boot, panic smoke, exception smoke, and timer smoke remain green.
- Release notes explicitly state that this is a structure-preserving split, not
  a boot protocol change.

Exit criteria:

- The bootloader handoff boundary remains auditable without a >500-line
  exception, and v0.17 heap work can start without carrying known modularity
  debt.

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

- Fixed slab classes for small allocations.
- Page-sized large allocation runs inside the static kernel heap.
- Checked deallocation and slab/page reuse.
- Freed slab blocks and page-sized runs are zeroed before reuse.
- Per-page slab live counters avoid a full free-list walk on normal frees.
- Aggregate heap stats: allocated bytes, peak bytes, allocation counts, frees,
  invalid-free telemetry, and double-free detection.
- Host tests for pre-initialization rejection, one-shot initialization, slab
  reuse, page-run reuse, invalid-free telemetry, double-free detection, zeroing
  before reuse, stats, and OOM without stat advancement.
- QEMU smoke for `Box`, `Vec`, `BTreeMap`, slab reuse, page-run allocation,
  mixed allocation/free stress, invalid-free telemetry, double-free detection,
  and explicit OOM rejection.
- Non-claim: the standard global allocator ABI cannot distinguish a delayed
  stale raw-pointer free from the current owner freeing the same address after
  reuse; future ownership-token or quarantine work must close that class.

Verification:

- Host kernel heap tests.
- Allocate/free stress smoke in QEMU.
- Existing panic, exception, and timer smokes remain isolated from the normal
  allocator path.

Exit criteria:

- Heap is suitable for capability/object structures.
- Remaining physical-frame-backed heap growth, per-core heaps, quarantine,
  non-`static mut` backing storage, bounded IRQ-masked latency before material
  heap growth, and allocation-while-locking policy are documented as
  non-claims.

### v0.18.1 - Early Entropy And Generation Semantics

Goal:

Make early identity-generation assumptions explicit before capability and
object identifiers become security-relevant.

Deliverables:

- Early entropy service interface with explicit sources and quality labels.
- x86_64 RDRAND/RDSEED probing behind CPUID checks, treated as one input and
  not as a sole trust anchor.
- Runtime self-test evidence must be represented separately from CPUID feature
  presence; CPUID alone must not enable random-token policy.
- Runtime self-tests must detect stuck or repeated sample patterns before
  classifying a hardware path as suitable seed material.
- Random-token policy requires a DRBG path with separate self-test evidence;
  raw `RDRAND`/`RDSEED` reads are not exposed as tokens directly.
- Until a DRBG implementation and kernel smoke path set
  `drbg_self_test_passed=true`, `random_tokens_available=false` is the expected
  production state and no security-sensitive capability token, ASLR seed, IPC
  nonce, or package/update secret may consume this interface as randomness.
- Deterministic boot-local monotonic fallback for identifiers that are
  anti-confusion only.
- Clear distinction between generation counters used to reject stale authority
  and random tokens used to resist guessing.
- Redacted entropy status telemetry that never logs raw random material.

Expected serial:

```text
entropy-policy rdrand=<bool> rdseed=<bool> hardware_self_test=<bool> drbg_self_test=<bool> hardware_present=<bool> fallback_used=<bool> generation_counter_ok=true random_tokens_available=<bool> source=<source>
[TEST] entropy-policy=ok
```

Verification:

- Host tests cover source classification, fallback behavior, counter overflow,
  and non-claims.
- QEMU smoke reports whether CPUID hardware entropy features were seen, whether
  runtime hardware self-test and DRBG self-test evidence were present, whether
  fallback mode was used, whether random tokens are available, and whether
  generation-counter overflow is rejected.

Exit criteria:

- Later capability and object milestones can state whether an identifier is
  anti-confusion, anti-replay, or attacker-unpredictable.

### v0.18.2 - DRBG Implementation And Token Readiness

Goal:

Provide the approved attacker-unpredictable randomness path before any
security-sensitive nonce, secret handle, KASLR seed, update/model token, or
cross-boot authoritative identity consumes randomness.

Deliverables:

- Chosen no_std CSPRNG/DRBG construction and security rationale.
- Entropy-source combination policy for hardware seed inputs and deterministic
  fallback classification.
- Known-answer tests and startup self-tests for the DRBG implementation.
- Runtime health tests on hardware seed inputs before they are accepted as seed
  material.
- Initial seeding and reseeding thresholds.
- Per-core streams derived through domain-separated labels.
- Backtracking-resistance policy after state compromise where feasible.
- Core-restart and suspend/resume behavior.
- DRBG state zeroization on teardown or failed initialization.
- Checked generation and reseed counters that fail closed instead of wrapping.
- Fail-closed behavior when reseeding is required but seed material is
  unavailable.
- Explicit prohibition on copying DRBG state during task migration, core
  migration, AP restart, suspend/resume, or snapshot creation.
- Identity classification:
  - boot-local anti-confusion identity may use monotonic generation;
  - cross-boot authoritative session identity requires a nonrepeating
    persistent generation, verified random nonce, or both;
  - unpredictable secret/token generation requires the approved DRBG.
- World Service and package/update identity rules must not use a deterministic
  boot-local counter as the sole cross-boot identity.

Expected serial:

```text
drbg self_test=true reseed_counter_ok=true random_tokens_available=true
[TEST] drbg=ok
```

Verification:

- Known-answer and startup self-tests pass.
- Host tests reject reseed-counter overflow and unavailable-required-reseed
  paths without producing a token.
- Tests prove per-core stream labels produce distinct streams from the same
  seed.
- Tests prove DRBG state is not `Copy`/`Clone` and is not accepted by migration
  or snapshot paths.
- QEMU smoke reports `random_tokens_available=true` only after DRBG self-test
  and seed policy succeed.

Exit criteria:

- Aesynx has one approved random-token path and keeps anti-confusion
  generation counters distinct from attacker-unpredictable secrets.

## Phase 5: Capabilities

### v0.19.0 - Capability Model Crate

Goal:

Model capability logic in safe Rust before the kernel capability table becomes
live.

Deliverables:

- `aesynx-cap` as the no-unsafe capability model crate.
- Checked `CapId` layout over `CapId(u64)`.
- Permission bitset.
- Derivation tests.
- Revocation tests.
- Generation tests.
- Redacted capability debug output.

Verification:

- `cargo test -p aesynx-cap`.
- `scripts/checks.sh`.

Exit criteria:

- Model is trusted enough to implement the kernel capability table in v0.20.0.

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

Expected serial:

```text
[TEST] memory-cap=ok
```

Verification:

- READ without permission fails.
- WRITE without permission fails.
- Derived cap cannot escape range.
- Mapper-facing checked mapping descriptor construction is attempted only after
  memory-cap authorization.

Exit criteria:

- Capability model affects real kernel APIs.

### v0.22.0 - Capability Audit Events

Goal:

Make authority movement observable.

Deliverables:

- Grant and revoke audit events.
- Revoke requires an audited table path; unaudited table revoke is not a public
  API.
- Redaction rules for audit debug output.
- Telemetry event for cap faults.
- Serial debug view.

Expected serial:

```text
[TEST] cap-audit=ok
```

Verification:

- Grant emits event.
- Revoke emits event.
- Secret payloads are not logged.
- Cap-fault telemetry reports aggregate redacted events.

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
- Pentest follow-up: cross-owner capability derivation and grant must strip
  `GRANT`, `REVOKE`, and `ADMIN`; every x86_64 IDT vector must have a
  deterministic halt-and-log catch-all before specialized handlers override
  selected vectors; descriptor `rsp0` updates must validate the interrupt-mask
  contract in release builds; unbounded memory root capabilities must not
  directly authorize map requests.

Verification:

- Create/list/delete local objects.
- Object caps reference objects.
- Host tests cover cross-owner `REVOKE`/`ADMIN` stripping and the arch exception
  table changes.
- Host tests cover rejection of direct map authorization from unbounded memory
  capabilities while preserving bounded derived memory caps.

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
- Any router or dispatcher that uses `MessageRequest::dst` as a queue/core
  selector must first convert it into a validated core identifier through a
  typed live-core-set check; raw `CoreId` values are not valid indexing
  authority.
- Service queues record an owner core; push, pop, completion, and pending-count
  inspection reject non-owner callers before queue mutation or observation.
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
- Linear ownership rule for task movement: failed queue admission returns the
  rejected task to the caller.
- Documented queue scaling gate: small fixed queues may use linear membership
  scans, but large or syscall-hot queues need indexed membership tracking before
  they become a live hot path.
- Documented live-scheduler gate: queue mutation must be protected against
  local interrupt/preemption re-entry before a real executor depends on these
  queues.

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

Telemetry should grow into the lower layer of the Aesynx OS world model: small,
bounded, deterministic facts emitted by the kernel and services, later consumed
by native world/query/context services. See
[OS World Roadmap](os-world-roadmap.md).

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
- Central CoreId redaction/non-redaction export policy.
- Serialization tests proving scheduler task IDs cannot leak through the trace
  exporter.

Verification:

- QEMU run produces decodable trace.
- Exported trace fixture redacts raw task IDs when scheduler events are present.

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

### v0.33.1 - Concurrency Discipline

Goal:

Define synchronization rules before multicore hardware bring-up makes global
single-core assumptions dangerous.

Architecture note:

Aesynx uses x86_64 SMP mechanisms only as the hardware path for discovering and
starting additional cores. The intended kernel architecture is not a classic
shared-everything SMP kernel. The long-term target is software-defined AMP and a
multikernel fabric: cores have explicit roles, own local state, communicate by
bounded messages, and avoid shared mutable kernel state except for narrow,
documented bootstrap or hardware-control paths.

Deliverables:

- IRQ-safe spinlock primitive or deliberately narrower early-lock primitive.
- Interrupt-disable guard with non-forgeable lifetime semantics.
- Lock-ordering policy for core kernel subsystems.
- Documented rule for which code may block, allocate, or log while holding a
  lock.
- Per-core versus shared-state ownership checklist.
- Service queue concurrency policy must state that any future shared-memory or
  multi-core queue slot vacate path zeroes or otherwise scrubs payload storage
  before the slot can be observed outside the current trust domain.
- Explicit migration plan for current single-core `static mut` GDT/TSS/IDT
  storage, including non-atomic IDT gate writes, before any secondary core can
  observe or mutate descriptor state.
- Static activation arenas/stacks must move to explicit interior mutability
  such as `SyncUnsafeCell`, or to per-core owned storage, before multi-core
  activation paths can use them.
- The kernel heap backing store must move away from the current `static mut`
  raw-address pattern before multicore activation or material heap growth.
- Heap operations that run with interrupts masked must have bounded latency, or
  use a two-phase design that performs bulk work outside the lock, before the
  heap grows beyond the current fixed static bound.
- Heap accounting patterns such as load, saturating arithmetic, then store are
  safe only while protected by the current single-core heap lock and `smp`
  compile-time tripwire; per-core heaps or atomic fetch/update accounting are
  required before SMP heap access is enabled.
- Tests for double-unlock prevention, nested interrupt guard behavior, and
  lock-order validation where feasible.

Verification:

- Host tests cover lock/guard state transitions.
- QEMU single-core boot remains green with the new primitives compiled in.
- Multicore milestones cannot graduate until the concurrency policy is
  referenced by their release notes.

Exit criteria:

- Multicore work has an explicit synchronization contract instead of inheriting
  accidental single-core behavior.

## Phase 9: AMP/Multikernel Fabric On SMP Hardware

Phase intent:

This phase deliberately separates mechanism from architecture:

- **SMP hardware mechanism:** x86_64 QEMU uses SMP/APIC/IPI mechanisms to bring
  additional cores online.
- **AMP kernel policy:** once online, cores are not treated as fully
  interchangeable peers sharing one large kernel state. Aesynx assigns explicit
  ownership and roles.
- **Multikernel fabric:** cross-core work moves by bounded messages,
  capability-aware handoff, and IRQ routing to the owning service core, not by
  growing global locks.
- **CPU-driver/monitor split:** long-term ring 0 is a local per-core mechanism
  layer; global policy, topology knowledge, capability agreement, AI, package,
  telemetry aggregation, and world queries move to isolated monitor/service
  domains as userspace arrives.
- **Heterogeneous readiness:** future aarch64 big.LITTLE and x86 P-core/E-core
  systems should fit the same model through core capability/role metadata.

Classic SMP behavior is allowed only as a bring-up compatibility step or a
documented fallback. It must not become the default design for schedulers,
drivers, heap ownership, object registries, or capability revocation.
Cross-core shared locks are not an accepted production mechanism for mutable OS
state; use owner-core messages, replicated-state protocols, or explicit
capability-scoped shared buffers.

### v0.34.0 - AMP Core Data Structures

Goal:

Prepare per-core ownership and role metadata.

Deliverables:

- CoreId.
- Core role classification for bootstrap, scheduler, driver/service, and idle
  roles.
- Core capability metadata for future heterogeneous systems.
- CoreLocal.
- Per-core registries.
- Per-core telemetry.
- Boot barriers.
- Policy that mutable state has a named owning core or a documented shared
  synchronization boundary.

Verification:

- Single-core boot uses CoreLocal and records the bootstrap core role.

Exit criteria:

- No subsystem assumes only global shared state as the future multicore model.

### v0.35.0 - QEMU Multicore Topology Baseline

Goal:

Run the normal QEMU smoke paths under `-smp 4` and model the four visible
cores under Aesynx AMP ownership policy before secondary-core execution is
enabled.

Deliverables:

- QEMU smoke runner uses `-smp 4` and records the virtual CPU count in the
  generated image manifest from the same `aesynx-core` topology-capacity
  constant used by the kernel smoke.
- Safe no_std topology model for discovered cores.
- Hardware state machine that distinguishes discovered, startup-staged, online,
  and quarantined cores.
- Assignment state machine that distinguishes hardware online from assigned
  Aesynx role.
- Owner-scoped topology, registry, and boot-barrier setup mutation. Non-owner
  callers fail before mutation.
- Service queue owner checks remain active under the four-vCPU QEMU smoke, so
  queue mutation and inspection still fail closed on non-owner callers even
  before shared-memory rings exist.
- Role assignment allowed only before hardware-online state; startup-staged is
  required before a core can become online.
- Reachable quarantine transition for modeled failed cores.
- Public topology status that redacts the internal mutation epoch.
- Four-core QEMU topology smoke with bootstrap, scheduler, driver/service, and
  idle roles.
- Boot barrier evidence covering all four modeled cores.
- Documentation that this is topology/ownership evidence under a multicore VM,
  not AP execution and not a commitment to a shared-everything SMP kernel.

Expected serial:

```text
multicore-topology qemu_smp_cores_ok=true hardware_online_ok=true role_assignment_ok=true bootstrap_ok=true scheduler_ok=true driver_service_ok=true idle_ok=true multicore_barrier_ok=true
[TEST] multicore-topology=ok
```

Verification:

- QEMU `-smp 4` boot smoke.
- Serial evidence shows the modeled four-core topology has hardware online
  state, local state, and assigned roles.
- Host tests cover duplicate hardware IDs, owner mismatches, role reassignment
  rejection after online, direct discovered-to-online rejection, quarantine, and
  failed state transitions.

Exit criteria:

- The boot smoke proves Aesynx can run under a four-vCPU QEMU machine while
  keeping core ownership explicit and fail-closed.

Non-goals:

- No AP startup trampoline.
- No secondary core executes Rust code yet.
- No per-core GDT/IDT/TSS/IST installation yet.
- No cross-core message fabric yet.

### v0.35.1 - AP Startup Evidence Contract

Goal:

Require topology-online transitions to flow through non-forgeable startup
evidence before the real x86_64 AP trampoline lands.

Deliverables:

- `CoreStartupTicket` issued only by owner-scoped startup staging.
- `CoreStartupArrival` evidence derived only from a matching ticket.
- Hardware-online transition requires validated arrival evidence for the target
  core, hardware ID, coordinator, and startup epoch.
- Mismatched arrival core or hardware ID fails before topology mutation.
- Direct online-without-startup is unrepresentable through the public topology
  API.
- QEMU topology smoke records `startup_evidence_ok=true` before
  `[TEST] multicore-topology=ok`.
- Current candidate metadata and image names move to `v0.35.1`.
- Documentation keeps this as AP startup evidence, not AP execution.
- Confirm the entropy DRBG implementation remains a scheduled blocker before
  any AP startup work consumes attacker-unpredictable tokens; v0.35.1 must not
  introduce random-token consumers while QEMU reports `drbg_self_test=false`.
- Keep the current general CPU-hardening policy for QEMU unless a deployment
  selector is added; the strict `NX+SMEP+SMAP+UMIP` policy remains tested but
  not selected by default.

Expected serial:

```text
multicore-topology qemu_smp_cores_ok=true ... startup_evidence_ok=true ...
[TEST] multicore-topology=ok
```

Verification:

- Host tests cover ticket issuance, arrival mismatch rejection, role assignment
  between stage and arrival, and evidence-backed online transition.
- QEMU `-smp 4` boot smoke includes `startup_evidence_ok=true`.

Exit criteria:

- Aesynx has a fail-closed AP arrival contract for the later hardware startup
  path.

Non-goals:

- No AP startup trampoline.
- No secondary core executes Rust code yet.
- No per-core GDT/IDT/TSS/IST installation yet.
- No cross-core message fabric yet.

### v0.35.2 - AP Startup Preflight

Goal:

Define and smoke-test the fail-closed resource contract that a future x86_64 AP
startup path must satisfy before any secondary core is allowed to execute
Aesynx Rust code.

Deliverables:

- `aesynx-core` AP startup preflight model with owner-scoped mutation.
- Startup resources are accepted only for topology entries already in
  `StartupStaged`/`Booting`.
- Dedicated AP stack ranges must be page-aligned, at least 32 KiB for early AP
  entry, inside caller-supplied platform stack bounds, non-overlapping, and
  unique per core.
- Duplicate logical core IDs, duplicate hardware IDs, overlapping startup
  stacks, missing watchdog ticks, and non-owner callers fail before mutation.
- Descriptor-table readiness is explicit. Shared bootstrap-only descriptors are
  allowed as a documented blocker but make execution disallowed.
- QEMU topology smoke records `ap_preflight_ok=true` and
  `ap_execution_blocked_ok=true` before `[TEST] multicore-topology=ok`.
- Confirm the entropy DRBG implementation remains a scheduled blocker before
  any AP startup work consumes attacker-unpredictable tokens; v0.35.2 must not
  introduce random-token consumers while QEMU reports `drbg_self_test=false`.
- Keep the current general CPU-hardening policy as the default for QEMU while
  adding an opt-in `strict-cpu-hardening` build selector for deployments that
  must fail closed unless `NX+SMEP+SMAP+UMIP` are all available.
- Documentation keeps this as AP startup preflight, not AP execution.

Expected serial:

```text
multicore-topology qemu_smp_cores_ok=true ... startup_evidence_ok=true ap_preflight_ok=true ap_execution_blocked_ok=true ...
[TEST] multicore-topology=ok
```

Verification:

- Host tests cover staged-only preflight resources, non-owner rejection,
  duplicate stack rejection without mutation, missing watchdog rejection, and
  descriptor-readiness blocking.
- QEMU `-smp 4` boot smoke requires the AP preflight markers.

Exit criteria:

- A later AP startup trampoline has a typed, fail-closed launch-resource gate
  and cannot honestly claim execution readiness while descriptor tables remain
  shared-bootstrap-only.

Non-goals:

- No AP startup trampoline.
- No secondary core executes Rust code yet.
- No per-core GDT/IDT/TSS/IST installation yet.
- No cross-core message fabric yet.

### v0.35.3 - AP Startup State Table

Goal:

Make the AP startup topology state machine explicit and enforce it as the single
source of truth before any secondary-core execution path lands.

Deliverables:

- `aesynx-core` startup state table covering the cross-product of
  `CoreHardwareState`, `CoreAssignmentState`, and `CoreState`.
- Valid combinations are explicit: discovered/offline,
  startup-staged/booting, online/online, and quarantined/quarantined, each
  with assigned or unassigned role state.
- Startup staging, role assignment, hardware-online marking, and quarantine all
  consult the same table before mutation and revalidate the resulting state
  before commit.
- Host tests cover valid and invalid joint states, transition intent helpers,
  table cardinality, and topology mutation through the table.
- QEMU topology smoke records `state_table_ok=true` alongside the existing
  startup evidence and AP preflight markers.
- Documentation keeps this as state-machine hardening only. It does not start
  APs and does not weaken the v0.35.2 descriptor-table execution blocker.

Expected serial:

```text
multicore-topology qemu_smp_cores_ok=true ... role_assignment_ok=true state_table_ok=true startup_evidence_ok=true ap_preflight_ok=true ap_execution_blocked_ok=true ...
[TEST] multicore-topology=ok
```

Verification:

- QEMU `-smp 4` boot smoke.
- Serial evidence shows the state table is audited and every modeled QEMU core
  remains in a valid joint state.
- Host tests prove impossible joint states are rejected by the table and cannot
  be used by topology mutation helpers.

Exit criteria:

- The AP startup state machine has one auditable source of truth before
  hardware startup code can consume it.

Non-goals:

- No AP startup trampoline.
- No secondary core executes Rust code yet.
- No per-core GDT/IDT/TSS/IST installation yet.
- No cross-core message fabric yet.

### v0.35.4 - Multi-Domain Hardening Blockers

Goal:

Close the hardening gaps that must not be carried into multi-domain execution,
ring-3 userspace, or real-hardware deployment claims.

Deliverables:

- Spectre-class control policy for x86_64 with CPUID/MSR gates for
  `IBRS/IBPB`, `STIBP`, `SSBD`, and `ARCH_CAPABILITIES`, plus a documented
  retpoline/IBRS choice.
- `IA32_SPEC_CTRL` admitted MSR handling and redacted read-back evidence when
  supported.
- KASLR/PIE boot plan: kernel build flags, Limine config, executable-address
  response use, relocation assumptions, and QEMU evidence. Full KASLR
  implementation is tracked as the v0.44.6 blocker before hostile userspace.
- x86_64 `RDRAND`/`RDSEED` instruction path with bounded retries and runtime
  stuck-sample self-test. Raw hardware output must seed only a DRBG, never be
  exposed directly as a random token.
- DRBG implementation plan and smoke path that can make
  `drbg_self_test=true`; until this lands, `random_tokens_available=false`
  remains the only acceptable production state.
- Documentation updates that keep static-address/no-DRBG/no-Spectre-control
  limitations visible as non-deployment claims.
- NMI-safe live-IDT mutation plan: either shadow-IDT plus `lidt` swap,
  platform-specific NMI-source quiescing, or a hard rule that runtime interrupt
  gate mutation remains unavailable until per-core descriptor ownership lands.
- Arch-backed IRQ-disable proof token design so future `try_lock_irq`-style
  APIs cannot be mistaken for hardware interrupt masking when they only carry
  the software model.
- Root-capability bootstrap token plan: the unaudited
  `CapabilityTable::insert_root` path must either become crate/private
  bootstrap scaffolding or require a non-forgeable bootstrap token before
  authenticated object/capability call paths exist. Audited root insertion
  remains the normal runtime direction.

Expected serial:

```text
[TEST] cpu-hardening=ok
[TEST] entropy-policy=ok
```

Verification:

- Host tests for CPUID feature matrix and selected MSR policy.
- QEMU boot smoke stays honest about unsupported controls.
- Entropy tests reject stuck or repeated hardware samples.

Exit criteria:

- Aesynx has a concrete, release-gated path for speculative-execution controls,
  address randomization, and attacker-unpredictable token generation before any
  multi-domain deployment claim.

### v0.35.4.1 - Firmware Topology Normalization

Goal:

Treat ACPI/MADT and related firmware topology data as hostile-shaped input
before it can influence AP startup, routing, IRQ policy, or NUMA placement.

Deliverables:

- RSDP, XSDT, and RSDT pointer provenance checks and mapping bounds.
- ACPI table length, checksum, revision, alignment, and integer-overflow
  validation before parsing.
- Bounded MADT subtable walking with malformed-length rejection.
- Unknown MADT entry skip rules that cannot desynchronize the walker.
- Duplicate or conflicting local APIC/x2APIC ID rejection.
- Disabled versus enabled CPU entry policy.
- BSP duplication and nonexistent BSP handling.
- Local NMI and interrupt-source-override record normalization.
- APIC base and x2APIC mode consistency checks.
- SRAT/SLIT NUMA consistency rules before NUMA data is used for allocation or
  routing decisions.
- Normalized topology copied into kernel-owned storage before AP launch.
- No later reread of mutable firmware memory for security decisions.

Verification:

- Coverage-guided and deterministic host fuzzing over ACPI/MADT-shaped byte
  inputs.
- Truncation tests at every byte boundary in the table header and subtable
  walker.
- Mutation tests for duplicate entries, conflicting APIC IDs, disabled CPUs,
  malformed subtable lengths, checksum failures, and integer-overflow lengths.
- Differential normalization tests against an independent host parser fixture.
- QEMU smoke proves AP startup consumes the normalized kernel-owned topology,
  not raw firmware table pointers.

Exit criteria:

- AP startup receives a bounded, normalized, kernel-owned topology description
  with explicit non-claims for any firmware topology feature not yet parsed.

### v0.35.5 - x86_64 QEMU AP Startup

Goal:

Bring up secondary cores in QEMU using x86_64 SMP/APIC mechanisms, then place
each executing core under Aesynx AMP ownership policy.

Deliverables:

- CPU topology parser backed by firmware or ACPI/MADT data when available.
- AP stacks backed by the v0.35.2 preflight contract and v0.35.3 state table.
- AP startup path.
- AP startup dispatch token. The APIC INIT/SIPI writer must accept only a
  consuming token produced from an execution-allowed `ApStartupPreflight`; raw
  advisory status checks are not enough for the hardware launch path.
- Startup-attempt generation in every AP launch record. An AP arrival from an
  old timed-out attempt must not satisfy a later startup attempt.
- Publication barriers for per-core boot parameters before AP launch and
  consumption barriers on AP entry.
- Bootstrap-owned writable AP parameter pages are zeroed, revoked, or sealed
  read-only after the AP consumes them.
- Per-core GDT/IDT/TSS where needed.
- The current single-core `static mut` descriptor, TSS, IDT, double-fault IST,
  activation-arena, and activation-stack storage is either migrated to
  per-core ownership/explicit interior mutability or remains blocked by the
  existing `smp` tripwire; AP execution must not run on shared bootstrap
  descriptor statics.
- Per-core double-fault IST stacks with unmapped guard pages before stack-trace
  or deep diagnostic work is allowed on the double-fault path.
- Per-core local state written by each executing core.
- Core online state machine tied to actual AP arrival evidence.
- Watchdog timeout policy that quarantines a non-arriving AP instead of leaving
  startup in an ambiguous state.
- Recovery/reset story for permanently quarantined core trackers.
- High-assurance builds can select strict CPU hardening and fail closed when
  NX, SMEP, SMAP, or UMIP are unavailable.
- Owner-scoped topology mutation remains enforced after AP execution begins;
  APs report arrival through bounded messages or proof tokens, not arbitrary
  topology writes.
- AP-side ring-0 work remains within the local CPU-driver subset: local
  protection, local descriptor/stack setup, local interrupt handling, local
  message delivery, and local evidence reporting.
- Documentation that this is multicore bring-up, not a commitment to a
  shared-everything SMP kernel.

Expected serial:

```text
core 0 online
core 1 online
core 2 online
core 3 online
[TEST] multicore-boot=ok
```

Verification:

- QEMU `-smp 4` boot smoke.
- Serial evidence shows each executing core has a local state block and
  assigned role.

Exit criteria:

- Multiple cores execute Aesynx code and are owned by the AMP/multikernel
  policy.

### v0.36.0 - Core-to-Core Ping/Pong

Goal:

Prove the first pairwise multikernel message-fabric contract. In the tagged
v0.36.0 release this remains model-backed; if a later branch has already
completed AP execution, this milestone still does not claim live hardware
delivery until v0.37.8 replaces the queue with a cache-aware atomic AP-backed
path.

Deliverables:

- Pairwise SPSC queues.
- Ping/Pong messages.
- Sequence numbers.
- Backpressure event.
- Producer/consumer core identity checks.
- Route validation against kernel-stamped message headers.
- Release/acquire publish-observe evidence.
- Non-`Copy` ping/pong state so the sequence allocator cannot be forked.
- Sequence commit only after successful enqueue.
- Pong correlation through `reply_to`.
- Bidirectional backpressure evidence.
- QEMU marker gating for `ipc-pingpong ping_seq=`,
  `ipc_backpressure_ok=true`, `ipc_release_acquire_ok=true`, and
  `ipc_pairwise_route_ok=true`.

Expected serial:

```text
ipc-pingpong ping_seq=1 pong_seq=2 backpressure_events=2 ipc_backpressure_ok=true ipc_release_acquire_ok=true ipc_pairwise_route_ok=true
[TEST] ipc-pingpong=ok
```

Verification:

- Core 0 pings core 1.
- Core 1 replies.
- Both link directions report backpressure without overwriting unread messages.
- Wrong producer, wrong consumer, loopback, empty, and mismatched-route cases
  fail before mutation in host tests.

Exit criteria:

- Cores communicate by message.
- No global run queue, allocator lock, or object-registry lock is required for
  the ping/pong path.

Non-goals:

- No APIC IPI delivery path.
- No live cross-core atomics yet.
- No claim that ping/pong delivery is performed concurrently by two executing
  APs; that belongs to v0.37.8.

### v0.37.0 - Capability Grant Over IPC

Goal:

Transfer authority across cores.

Deliverables:

- Grant message.
- Receiver CapId allocation.
- Sender permission check.
- Cross-core revoke notification.
- Revoke across IPC must drive the object registry's revocation epoch bump, not
  only the sender table's local `revoke_with_audit`, so every table holding a
  capability for the same object observes epoch invalidation.
- Audit event.

Verification:

- Grant READ cap.
- Reject WRITE without permission.
- Revoke invalidates receiver.

Exit criteria:

- IPC and capabilities are integrated.

### v0.37.0.1 - Protocol Specification Gate

Goal:

Require executable protocol specifications before implementing the live
mechanisms that depend on them.

Deliverables:

- TLA+ or Quint model for transactional grants before v0.37.1 grant
  implementation is considered complete.
- TLA+ or Quint model for live queue publication, reuse, cancellation, and
  wraparound before v0.37.8 atomic fabric queues can claim AP-backed safety.
- TLA+ or Quint model for prospective revoke, strong revoke, coordinator
  failure, participant timeout, and recovery before v0.37.9 can guard live
  shared mappings or DMA.
- TLA+ or Quint model for derived-object edge creation, promotion/detachment,
  v1 single-parent publication, provenance recording, and cascading revocation
  before v0.37.1 derived-edge implementation is considered complete. Future
  parent-set publication requires a separate `ParentSetManifest` model before
  any implementation accepts multi-parent children.
- TLA+ or Quint model for AP startup, late arrival, permanent quarantine, and
  restart/hotplug fencing before v0.37.11 can relax the no-restart rule.
- Model interfaces name the Rust state machines and event fields they refine
  later in v0.37.12.
- Explicit split between this early specification gate and the later v0.37.12
  conformance gate. Implementation must not rely on "the model comes later"
  for authority-bearing behavior.

Verification:

- Each model has at least one positive path and one intentionally broken
  negative variant proving the property catches a relevant bug.
- Documentation gate rejects v0.37.1, v0.37.8, v0.37.9, or v0.37.11 release
  notes that claim completion while the matching model is absent.

Exit criteria:

- Live authority, queue, revocation, and AP-fencing work has a formal
  executable contract before implementation detail dominates the design.

### v0.37.1 - Authority Identity And Endpoint Hardening

Goal:

Close the remaining model-level authority gaps before Aesynx builds richer
shared-memory, endpoint, and replicated-fabric behavior on top of capability
IPC.

Rationale:

The v0.37.0 grant/revoke-over-IPC path proves useful integration, but future
hostile-boundary work needs stronger identities than caller-selected object
IDs and caller-supplied core IDs. This milestone makes authority identity,
principal identity, endpoint rights, and live checks explicit before they
become user or multicore enforcement APIs.

Implementation slicing:

This milestone is security-critical and must not land as one giant change.
Implementation must be split into reviewable units with their own tests:

1. Registry-minted authority handles, table/domain incarnations, table
   ownership, and root-minting restrictions.
2. Typed rights, kind-to-right matrix, canonical wire representation, and
   non-authoritative `CapId` tags.
3. Live authorization proof APIs that replace boundary use of table-only
   `check()` plus optional registry resolution.
4. Transaction journal plus copy grants.
5. Move-only grant escrow.
6. Endpoint call/reply authority, one-shot replies, cancellation, and
   server-death cleanup.
7. Distributed quota accounting foundations.
8. Derived-object edge identity, adjacency indexes, quotas, and immutable
   relation policies.
9. Transactional child creation, edge publication, and recovery.
10. Promotion and detachment semantics.
11. Prospective lineage traversal and model-level derived-object revocation.
12. Strong live-resource cascade integration remains deferred to v0.37.9,
   where edges connect to live mappings, TLBs, DMA, leases, and endpoint
   operations.

Deliverables:

- Registry-minted authority handles for kernel objects. Untrusted callers must
  not be able to choose an authority-bearing object identity directly.
- Stable logical object incarnation tracking that cannot resurrect stale
  capabilities if an object ID is deleted and later recreated in a different
  registry slot.
- Separate user-visible/content object names from authority-bearing kernel
  handles.
- Multi-slot stale-capability resurrection regression tests covering delete,
  slot reuse, object-ID recreation, generation wrap/retirement, and lookup
  through live object resolution.
- Non-forgeable execution context or owner-token model for enforcement paths.
  Requests may carry claimed core/domain IDs for diagnostics, but
  authorization must use kernel-stamped current execution identity.
- Capability tables bound to an owning domain/principal incarnation, with quota
  and revocation-domain metadata.
- Central capability-kind permission matrix using common meta-rights plus
  kind-specific typed rights instead of one ever-growing universal bitset:
  - `CommonRights`: derive, grant, revoke, introspect, and narrowly scoped
    admin where the kind permits it.
  - `MemoryRights`: read, write, execute, map, share-read, share-write.
  - `EndpointRights`: send, receive, call, reply, notify.
  - `AddressSpaceRights`: map, unmap, protect, activate, and inspect.
  - `IrqRights`: bind, acknowledge, mask, and unmask.
  - `DmaRights`: map, unmap, synchronize, and invalidate.
  - `SystemControlRights`: typed administrative operations only.
  - `DomainFactoryRights`: spawn.
  - `DomainControlRights`: stop, kill, restart, inspect-status, and
    set-exception-endpoint.
  - `TaskFactoryRights`: create-task.
  - `TaskControlRights`: stop, kill, inspect-status, and
    set-exception-endpoint.
  - `TaskJoinRights`: wait, read-result, and consume-result.
  - `DebugRights`: read-registers, write-registers, read-memory,
    write-memory, suspend, and resume.
  - `ExceptionRights`: receive-fault, inspect-frame, modify-frame, and
    resume-frame.
  - `PagerRights`: receive-page-fault, supply-page, and reject-fault.
  - `SchedulingContextRights`: donate, set-ceiling, cancel-donation, and
    inspect-budget.
  - `ClockRights`: read-coarse, read-precise, create-deadline, and
    synchronized-compare.
  - parent-object derivation operations: create-derived, promote, and
    detach-derivation. These are not a second cross-cutting capability-rights
    bitset in the wire format; each applicable parent object kind embeds the
    admitted relation operations in its single kind-specific rights
    representation, and the central matrix records them as a documentation and
    validation family.
  Mint, derive, decode, and live resolution validate both capability kind and
  typed-right representation. Invalid examples such as `Endpoint|EXECUTE`,
  `Memory|RECV`, and `Clock|MAP` fail closed before an operation can ignore
  nonsense permissions.
- The central matrix records, for every object kind, the wire encoding, mint
  authority, derivation/attenuation rule, whether delegation is allowed,
  one-shot/transaction/incarnation binding, revocation behavior, audit event
  class, cross-domain behavior, and exact coexistence rules for `ADMIN`,
  `GRANT`, and `REVOKE`.
- High-risk rights are non-delegable by default unless a later object-specific
  policy explicitly proves why delegation is safe. Initial non-delegable
  examples include `DomainControl::KILL`, `Debug::WRITE_MEMORY`,
  `Exception::MODIFY_FRAME`, `Exception::RESUME_FRAME`, and
  `SchedulingContext::SET_CEILING`, `TaskControl::KILL`, and
  `TaskControl::SET_EXCEPTION_ENDPOINT`.
- Task lifecycle rights are distinct from domain lifecycle rights. Domain
  possession, address-space possession, and executable possession do not imply
  `TaskFactory::CREATE_TASK`; creating a task requires task quota, stack/TLS
  memory, scheduling-context authority, and target-domain authority. A task
  capability cannot be reinterpreted as a domain capability or vice versa.
- Task join/result capabilities are task-incarnation-bound but have distinct
  consumption semantics:
  - `WAIT` may be called repeatedly until completion and does not consume the
    capability;
  - `READ_RESULT` returns a bounded immutable result and is either repeatable
    for observer capabilities or explicitly marked one-shot by object policy;
  - `CONSUME_RESULT` is always one-shot and transitions the result object to
    consumed for that holder at a named consume linearization point;
  - move-only join capabilities have exactly one consumer;
  - independently granted observer capabilities may wait/read but cannot
    consume or control the task; each observer has its own capability-table
    slot and generation, authority lineage, revocation state,
    result-read policy, and expiry/retention accounting;
  - consuming one observer capability does not destroy the result for other
    authorized observers unless object policy explicitly selects
    single-consumer semantics.
  Reading a task result does not imply task control, task-local exit cannot
  revoke the whole domain unless it is the last task under configured policy,
  and domain teardown dominates every task-local capability by consuming or
  retiring outstanding join/result objects.
- `ADMIN` is removed from generic enforcement paths or constrained so it never
  implies another right. Every administrative operation has an exact typed
  operation identifier, `ADMIN` is not an override for failed `READ`, `WRITE`,
  `MAP`, `GRANT`, `REVOKE`, or similar checks, delegation is prohibited by
  default, and every use is audited.
- Versioned typed-right wire representation. Decoding rejects rights that are
  unknown, mandatory-but-unsupported, or not valid for the encoded object kind.
- External `CapId` kind tags are non-authoritative routing hints. The registry
  slot's live kind, incarnation, and typed-right representation control decode
  and operation dispatch; a payload tag can never authorize an unsafe downcast.
- Capability-table, domain, endpoint, peer, and boot/session incarnations are
  part of every authority interpretation context. A table-local `CapId` is not
  meaningful after its table or domain is destroyed and recreated.
- Authority-bearing messages carry transaction IDs scoped by source domain,
  target domain/table incarnation, endpoint incarnation, and boot/session
  incarnation.
- Sequence-wrap retirement and replay windows for grant, revoke, map, endpoint,
  and routing messages.
- Fail-closed generation/epoch exhaustion for objects, tables, peers,
  endpoints, and transactions.
- Root minting restricted to registry-issued mint tickets or bootstrap-only
  audited paths; normal code must not supply arbitrary object ID, generation,
  and epoch tuples as authority.
- Enforcement APIs that return short-lived checked proof types, such as a live
  capability lease, endpoint send permit, or address-space map permit, instead
  of letting callers combine `check()` plus optional registry validation by
  convention.
- Checked proof lifetime semantics. Every proof must be one of:
  - an `AuthorizedOperation` consumed exactly once at the final mutation
    boundary with commit-time generation/epoch revalidation;
  - a registry-counted in-flight lease that revocation can freeze and drain;
  - a read-side critical section whose lifetime is visible to the object
    registry;
  - an internal preflight token that cannot authorize mutation by itself.
- Grant commit revalidates the sender's live capability, current revocation
  epoch, and attenuated permissions. Revoking the sender between proposal and
  commit aborts the grant.
- Delegation attenuation rule:

```text
delegated_common_rights <= requested_common_rights
                         & live_sender_common_rights
                         & delegable_common_rights

delegated_kind_rights   <= requested_kind_rights
                         & live_sender_kind_rights
                         & delegable_kind_rights
```

  `ADMIN`, `REVOKE`, `GRANT`, executable/JIT, writable-sharing, and DMA rights
  never propagate implicitly, and receiver-supplied grant records cannot select
  or widen permissions.
- Rename or restrict table-only permission checks so they cannot be mistaken
  for complete live authority validation.
- Endpoint objects with typed `SEND`, `RECV`, call/reply, and notification
  rights; queue objects remain transport mechanisms, not the authority
  boundary.
- Kernel-stamped endpoint metadata: source principal/domain incarnation,
  protocol ID/version, sequence number, transaction ID, and bounded payload
  schema.
- Confused-deputy prevention contract for every RPC/service operation:
  - the request schema declares which capability arguments authorize which
    operation;
  - caller identity is context and audit evidence, not a substitute for an
    object, endpoint, memory, package, or policy capability;
  - a service does not automatically spend its administrative, indexing,
    storage, world, or package capabilities on behalf of an untrusted caller;
  - delegated capability arguments are attenuated, transaction-bound, and
    consumed or revalidated at the final mutation boundary;
  - nested service calls propagate only explicitly delegated authority, never
    ambient server authority;
  - scheduling-context or CPU-budget donation never implies authority donation;
  - reply capabilities authorize returning a result for the original call, not
    performing unrelated operations;
  - service-local caches are keyed by caller authority, classification context,
    domain incarnation, and relevant capability generation/epoch;
  - a request cannot substitute another caller's capability-table handle,
    principal ID, object name, or namespace path as authority.
- Transactional capability grant protocol shape:
  - reserve pending receiver slot;
  - send grant proposal with transaction ID;
  - receiver accepts or rejects;
  - commit makes authority usable;
  - abort/timeout expires pending authority;
  - retries are idempotent.
- Common bounded transaction journal record format shared by grants,
  move-grants, ownership transfer, strong revocation, and other
  authority-moving operations. "Shared" means common protocol and schema, not a
  single system-wide journal. Each record stores transaction ID, participant
  incarnations, source and destination capability identities, frozen source
  generation, prepared/committed/aborted state, witness acknowledgements,
  commit certificate or decision epoch, recovery owner, timeout owner, journal
  generation, torn-record integrity evidence, bounded replay window, reserved
  capacity class, and terminal-record reclamation state.
- Owner-local or sharded one-writer journal ownership. A transaction ID names
  the owning shard and shard incarnation; cross-shard transactions name
  explicit participants and witnesses. No journal may become a global
  cache-coherency or availability bottleneck, and each transaction class has
  reserved journal capacity.
- Coordinator restart recovers from the transaction journal. If no
  authoritative coordinator record or trusted commit witness survives, the
  availability guarantee no longer applies. If commit might have been observed
  but evidence is lost, the safe result is quarantine or explicit resource
  loss; recovery never reconstructs commit from sender-controlled or
  receiver-controlled claims and never blindly aborts by restoring sender
  authority.
- TLA+ or Quint transactional-grant model before implementation is considered
  complete. The model proves no usable receiver capability before commit, abort
  restores the original state, duplicate proposal/accept/commit/abort is
  idempotent, sender revocation before commit prevents receiver activation,
  timeout never leaves phantom authority, rights cannot be amplified, and
  exactly one final outcome exists.
- Move-only grant escrow protocol:
  - sender active;
  - sender frozen and receiver pending;
  - receiver active and sender invalid on commit;
  - sender active and receiver empty on abort.
  The escrow coordinator, not the sender or receiver alone, owns the frozen
  state and commit record. The commit linearization point is the coordinator's
  durable or epoch-stamped commit decision. The formal invariant is
  `committed active copies <= 1`, with crash recovery that prevents both
  duplicate owners and permanent loss of an irreplaceable resource.
- Kernel-minted one-shot reply capabilities for `CALL`/`REPLY` endpoints.
  Reply caps are bound to caller, callee endpoint, transaction ID, boot/domain
  incarnation, and timeout/cancellation state.
- Reply authority is exactly-once by default, cannot be redirected to an
  unrelated caller, cannot be delegated unless an endpoint type explicitly
  permits it, and is rejected after cancellation, timeout, or server restart.
- Server death resolves outstanding reply capabilities through a typed
  cancellation result or retryable failure chosen by endpoint policy; authority
  cleanup is part of the server-restart transaction.
- Reply cancellation is idempotent and keyed by endpoint incarnation plus
  server incarnation so a restarted server cannot consume stale reply authority.
- Bounded outstanding calls per principal and endpoint.
- Per-principal quotas for kernel objects, page-table pages, pinned frames,
  lineage nodes, pending calls, and ordinary audit-rate usage. Emergency audit
  capacity remains system-reserved and non-delegable.
- Distributed quota accounting uses escrowed credits rather than one global
  shared counter:
  - a quota coordinator grants bounded credits to owner cores;
  - a core allocates only from local credit;
  - credit transfer is transactional;
  - dead-incarnation recovery retires or reclaims outstanding credits only
    after the old incarnation is fenced from committing further allocations;
  - every allocation consumes credit under the current coordinator and owner
    incarnation;
  - credit spends prepared before an epoch change either commit through an
    accepted certificate or abort;
  - an expired lease alone does not make credits reusable while the old core
    could still publish a valid-looking spend;
  - coordinator restart changes the credit epoch and fences old spend/refund
    messages;
  - duplicate spend and refund messages are idempotent;
  - local offline operation cannot exceed previously escrowed credit;
  - if a credit holder cannot be fenced, its remaining credits stay unavailable
    as quarantined credit;
  - the invariant is
    `allocated + prepared + local_free + quarantined == configured_ceiling`.
- Mapping-authority split between memory-object capability, destination
  address-space capability, and optional executable/JIT policy authority.
- Generic cross-object dependency edges for derived objects whose child object
  has a different object incarnation than the parent. Ordinary same-object
  capability derivation remains attenuation over one object; cross-object
  derivation records an explicit `DerivedObjectEdge`:

```text
DerivedObjectEdge {
    parent_object_incarnation,
    parent_capability_lineage,
    child_object_incarnation,
    relation_kind,
    edge_generation,
    edge_state,
    transaction_id,
    transaction_decision,
    parent_participant_state,
    child_participant_state,
    relation_policy_identity,
    parent_owner_incarnation,
    child_owner_incarnation,
    revocation_policy,
}
```

  Initial authority-dependency relation kinds include `ExecutableImage`,
  `Snapshot`, `CopyOnWriteChild`, `SealedTransform`, and `DerivedIndex`.
  `PromotedSharedCode` is a provenance relation kind recorded after successful
  promotion, not a live authority-dependency edge that can cascade-revoke the
  newly independent root.
- `DerivedObjectEdge` records are internal authority metadata, not
  caller-addressable capability objects. Boundary code must never accept a
  caller-supplied parent/child/edge tuple as authority. The original caller
  authority for create, promote, and detach operations comes from a resolved
  parent-object capability whose kind-specific rights include the exact
  relation operation, plus whatever child-side authority the operation
  requires. `CommonRights::DERIVE` only attenuates authority over the same live
  object; `CREATE_DERIVED` creates a new object incarnation and therefore
  always uses the transactional edge protocol. Relation policies define the
  exact child-side right required for each relation and operation and whether
  that child-side right is consumed, retained, or merely revalidated.
  `PROMOTE` and `DETACH_DERIVATION` are nondelegable by default unless a
  relation policy explicitly admits constrained delegation.
  The parent owner mints an internal `DerivationControlPermit` after resolving
  the caller's authority. For v1 single-parent derivation, the parent owner is
  also the derived-edge transaction coordinator and journal-shard owner, and the
  transaction ID embeds that journal shard plus parent-owner incarnation. The
  relation policy, execution identity, parent lineage, child incarnation,
  quotas, audit capacity, destination principal/table incarnation, and
  transaction identity are validated before the permit exists. That permit is
  an internal proof type only: it is not table-storable, serializable,
  delegable, wire-decodable, persisted in the journal, or reproduced during
  recovery. It authorizes exactly the local parent-owner coordinator/journal
  transition from `Undecided` to `Committed`; parent-side authority, child-side
  authority, pinned policy, generations, prepared acknowledgements, audit
  placeholder evidence, and transaction-bound reservations are revalidated
  immediately before that transition. Commit consumes the permit. Abort,
  timeout, or failed prepare destroys it without permitting reuse. Child and
  destination owners send generation-stamped prepared acknowledgements and
  never receive the permit. Local child publication is authorized by the
  replay-protected commit certificate plus publication-time parent-state
  validation, not by the permit. Cross-core messages carry only journal
  decisions, prepared acknowledgements, or owner-issued commit certificates, not
  the permit. If the parent owner fails before commit, recovery may only abort
  or quarantine based on durable journal evidence. Recovery may reproduce an
  existing commit certificate but can never create a new commit decision by
  reconstructing the permit. If a future design needs a permit to cross IPC, it
  must be redesigned as a full authority-bearing capability with incarnation,
  replay, delegation, and revocation semantics.
- Cross-object dependency rules:
  - the parent owner records the edge transactionally before the child object
    becomes usable;
  - child publication and edge publication share one commit decision;
  - failure cannot produce a usable orphan child that parent/source revocation
    cannot discover;
  - parent and child IDs are registry-minted incarnations;
  - edges are bounded by per-object and per-principal quotas;
  - v1 child publication is single-parent-only: it uses a newly minted,
    unpublished child with exactly one immutable incoming dependency edge;
  - child-owner selection is kernel-controlled. Caller-supplied core IDs,
    owner IDs, and locality preferences are hints only and never authority;
  - placement considers current owner incarnation, topology epoch, NUMA
    locality where known, available escrowed quota, registry capacity, and
    quarantine/restart/drain state;
  - the selected child owner and placement-policy identity are frozen into the
    canonical reservation plan before prepare;
  - owner selection cannot change mid-transaction. Any placement change requires
    abort plus a new transaction ID;
  - quarantined, draining, restarting, or stale-incarnation cores cannot be
    selected as child owner;
  - destination-table ownership does not automatically imply child-object
    ownership;
  - only a `Live`, published, unfrozen parent may authorize child creation;
    `Pending`, `Revoking`, `Recovering`, `QuarantinedAwaitingEvidence`,
    `Retired`, or unpublished objects cannot act as parents;
  - no incoming edge may be added to an already `Live` child, and a v1 child
    with more than one incoming parent is rejected before mutation;
  - every dependency carries a checked creation depth. For v1,
    `child_depth = checked(parent_depth + 1)` with a fixed maximum makes cycles
    structurally impossible; general DAG search and distributed topology epochs
    are reserved for the future `ParentSetManifest` feature gate;
  - `DerivedIndex` and similar v1 derived views derive from one collection or
    one snapshot, not a parent set;
  - multi-parent children are a future feature gate, not v1 behavior. They must
    remain rejected until a later `ParentSetManifest` milestone defines
    canonical parent-set identity, all-parent approvals, complete-set commit,
    rights intersection, concurrent parent revocation coalescing, promotion
    semantics, and failure recovery;
  - cycles are prohibited or rejected through a bounded DAG rule;
  - edge traversal has strict maximum depth and work budgets;
  - deleting or recycling an edge requires the child to be dead or
    independently promoted under an explicit policy;
  - edge generation cannot wrap or silently migrate between registry slots;
  - revoking a parent capability follows its configured prospective policy;
  - revoking a parent lineage subtree traverses only edges derived through that
    lineage;
  - strong object-wide parent revocation traverses every dependent edge whose
    policy requires cascading revoke;
  - revoking a child does not revoke the parent unless a separately defined
    reverse dependency requires it;
  - promoting shared text into an independent code object consumes the original
    dependency and establishes a new explicit authority root.
- Authority dependencies and provenance are separate records:
  - authority dependency edges participate in cascading revocation, pinning,
    promotion, and lifecycle/resource retention;
  - immutable provenance records record origin and inherited provenance but
    grant no authority, cause no cascade, and do not retain resources by
    themselves;
  - provenance records are kernel-stamped, append-only, integrity-protected,
    and bound to child incarnation, child content hash when content exists,
    source incarnation/hash, promotion transaction, policy identity, and
    boot/domain incarnation where those fields affect replay or trust;
  - every provenance field is classified by origin:
    caller-asserted, publisher-signed, loader-validated, or kernel-observed;
  - a provenance record can never resolve a retired source incarnation to a
    recycled object, and bounded provenance reclamation cannot silently change
    an authorization decision;
  - provenance alone never proves publisher trust. Any operation that requires
    publisher trust must revalidate signatures or attestation through an
    explicit trust-policy authority;
  - successful promotion retires the authority dependency and appends a
    provenance link, so independent objects do not remain accidentally
    revocation-linked and provenance does not disappear when the dependency
    edge is reclaimed.
- Relation policies are immutable and downgrade-resistant:
  - an edge pins a policy identity or semantic hash, not a mutable version
    number alone;
  - policy records are never reinterpreted after edges are created under them;
  - old policy records remain available until every associated edge and replay
    window retires;
  - weaker policy cannot be installed through replay, peer negotiation, or
    coordinator restart;
  - migrating a live edge to another policy is an explicit
    freeze/revalidate/commit transaction;
  - policy strengthening and weakening have separately defined migration rules;
  - unknown relation kinds or policy identities received over IPC fail closed.
- Derived-object promotion and detachment are security-sensitive operations,
  not mere child control:
  - promotion requires explicit `PROMOTE` or `DETACH_DERIVATION` authority from
    the parent authority path;
  - `PROMOTE` and `DETACH_DERIVATION` are typed
    relation operations inside the parent object's kind-specific rights, not
    universal permission bits, caller-held edge capabilities, or a second
    composed rights family;
  - appropriate child-side authority is also required; parent-side detachment
    approval alone cannot spend child authority;
  - each relation kind has a kernel-owned relation-policy entry that states
    whether detachment is permitted;
  - the caller cannot directly select `revocation_policy`; it comes from the
    kernel-owned relation-policy table and pinned policy identity;
  - promotion revalidates the parent capability, parent lineage, and edge
    generation at the promotion linearization point;
  - promotion fails if parent revocation is pending or already linearized;
  - promotion authority is bounded by:

```text
new_root_rights <= requested_rights
                & live_child_rights
                & relation_policy.promotable_rights
```

  - `GRANT`, `REVOKE`, `ADMIN`, executable/JIT, and writable-sharing authority
    never appear in the promoted root unless explicitly allowed by both
    live-child authority and the relation policy;
  - promotion is bound to exact parent lineage, child incarnation, edge
    generation, destination principal, and destination table incarnation;
  - destination capability-table slot and required audit capacity are reserved
    during prepare, not merely preflighted;
  - the destination principal cannot be redirected after prepare;
  - promotion is atomic: either the old dependency remains intact, or the new
    root exists and the old edge is retired;
  - promotion creates a new object incarnation and records inherited
    provenance;
  - promoted shared executable text follows these rules before it can become an
    independent immutable code object.
- Distributed edge ownership and recovery for shared-nothing cores:
  - edge state is `Pending | Live | Revoking | Recovering |
    QuarantinedAwaitingEvidence | Retired`;
  - transaction decision is `Undecided | Committed | Aborted`;
  - terminal resolution is separate from the transaction decision:
    `None | ResourceLost`. `ResourceLost` never reinterprets the journal as a
    normal commit or abort;
  - participant state is `Absent | Prepared | Applied | Acknowledged`;
  - a received `edge_state` value is never authoritative by itself;
  - state transitions require the journal decision or commit certificate plus
    expected prior generation;
  - aborted pending edges remain replay-detectable until journal retirement;
  - the parent owner holds the authoritative outgoing edge;
  - the child owner holds an inbound dependency record;
  - child and destination owners send prepared acknowledgements bound to owner
    incarnation, reservation generation, policy identity, edge generation, and
    transaction ID;
  - commit is impossible until the complete required reservation manifest and
    audit placeholder evidence have been acknowledged;
  - if parent, child, and destination table are owned by the same live owner
    incarnation, the implementation may elide IPC and remote acknowledgements,
    but it must still use the same transaction states, reservation manifest,
    parent-local audit placeholder, permit-consumption point, commit
    certificate shape, and publication checks;
  - local participant acknowledgements are represented in the same logical
    manifest as remote acknowledgements, so recovery semantics do not diverge;
  - there is no separate trusted-local authorization path;
  - transaction commit decision and child publication are separate points: the
    journal commit determines the irreversible transaction outcome; child
    publication occurs later when the child owner locally changes
    `Pending -> Live`;
  - before local publication, the child owner verifies the commit certificate,
    parent-edge discoverability, owner incarnations, pinned policy identity,
    edge generation, the single v1 parent edge, and current parent
    revocation/freeze state;
  - if parent revocation linearizes between transaction commit and local child
    publication, the child enters `Revoking`,
    `QuarantinedAwaitingEvidence`, or a terminal resource-loss outcome and
    never becomes briefly usable;
  - parent revocation traverses both `Live` edges and committed but not yet
    published edges;
  - no capability or user-visible handle is returned until local child
    publication completes;
  - edge creation API terminal outcomes are named:
    `Aborted`, `Published`, `CommittedButRevoked`, and `ResourceLost`;
  - recoverable observations such as `Recovering` and
    `QuarantinedAwaitingEvidence` are not terminal outcomes. Repeated queries
    with the same transaction ID may observe monotonic progress, but retries
    never initiate a second transaction or child;
  - recovery can transition to a terminal outcome only from trusted
    journal/witness evidence, and capacity remains charged until the terminal
    decision plus replay-window retirement;
  - `ResourceLost` can become terminal only after every participant capable of
    publishing commit or abort evidence is incarnation-fenced, reset, or
    permanently denied execution; candidate capabilities, child handles, and
    commit certificates are retired; a replay-resistant terminal tombstone is
    installed; no delayed commit or abort can reactivate authority; and resource
    pins, mappings, DMA records, and TLB obligations are drained before physical
    reclamation;
  - if those conditions cannot be proved, the state remains
    `QuarantinedAwaitingEvidence` or the system halts. It cannot report
    terminal resource loss and recycle identifiers;
  - idempotent means no duplicate side effect: a client cannot treat "no handle
    returned" after a committed-but-revoked, recovering, quarantined, or
    resource-lost transaction as an ordinary abort and accidentally create a
    duplicate committed child under that transaction;
  - the journal owner is authoritative for transaction decision, the parent
    owner for outgoing edge state, and the child owner for inbound dependency
    record plus local publication state; remote participant state is
    generation-stamped observation only;
  - the child is published only after required parent/child records reach the
    committed generation and local publication validation succeeds;
  - a committed edge is discoverable by parent revocation before the child can
    become usable;
  - every edge message is bound to transaction ID, relation-policy identity,
    parent owner incarnation, child owner incarnation, parent object
    incarnation, child object incarnation, and edge generation;
  - recovery handles crash or restart after reservation, after one-sided
    installation, and after commit before acknowledgement;
  - uncertain commit never turns into an inferred abort;
  - the existing common transaction journal provides the edge commit decision;
    the edge protocol must not grow a second independent commit mechanism;
  - edge retirement is not garbage-collected until both owners acknowledge it
    and its replay window has closed;
  - quarantine records are incarnation-bound and cannot be recycled as ordinary
    edge records;
  - messages involving obsolete owner incarnations fail closed.
- Edge publication reserves revocation progress resources:
  - implementation must define explicit fixed-memory sizing constants:
    `MAX_PENDING_DERIVED_TX_PER_CORE`,
    `MAX_PENDING_DERIVED_TX_PER_PRINCIPAL`,
    `MAX_RESERVATIONS_PER_TX`, `MAX_CHILDREN_PER_PARENT`,
    `MAX_DERIVATION_DEPTH`, `MAX_REPLAY_TOMBSTONES`,
    `MAX_RECOVERING_EDGES`, `MAX_AUDIT_PLACEHOLDERS`,
    `RESERVED_ABORT_RELEASE_RECORDS`, and
    `RESERVED_REVOCATION_PROGRESS`;
  - compile-time or boot-time checked multiplication/addition computes total
    storage for those constants, with a documented per-core and system-wide byte
    budget;
  - capacity is split into ordinary, recovery, abort/release, and revocation
    classes. Ordinary requests cannot consume emergency capacity;
  - capacity amounts use strongly typed per-class units. Journal records,
    message credits, audit slots, revocation work items, participant recovery
    records, and TLB/DMA cleanup records cannot be added, compared, or
    substituted as interchangeable scalar capacity;
  - capacity changes alter the relevant capacity-manifest/configuration
    identity, so old acknowledgements cannot silently carry into a differently
    sized reserve configuration;
  - exhaustion returns typed errors without partial mutation;
  - quarantined and `ResourceLost` records remain charged until their documented
    retirement points;
  - required terminal-progress reserves are proven per resource class, not as
    one homogeneous scalar. The configuration defines:

```text
terminal_need[class] =
    max_preparing_transactions
        * terminal_units_per_preparing_tx[class]
  + max_recovering_edges
        * recovery_units_per_edge[class]
  + max_concurrent_strong_revokes
        * revoke_units_per_operation[class]
  + fixed_system_emergency_margin[class]

reserved_capacity[class] >= terminal_need[class]
```

  - terminal-progress classes include journal transitions and tombstones,
    abort/release messages and acknowledgements, audit finalization, revocation
    cursors/work items, participant recovery records, TLB/DMA cleanup records
    where applicable, and any other protocol sharing the same emergency pool,
    including grants and strong revocation;
  - worst-case child fan-out is bounded by total edge quotas, not by an
    optimistic average transaction shape;
  - admission into `Preparing` consumes the corresponding terminal-progress
    credits first, before remote side effects;
  - checked arithmetic plus a compile-time, link-time, or boot-time assertion
    rejects configurations whose per-class proof does not fit the allocated
    memory budget;
  - before prepare, the parent-owned coordinator derives a canonical
    kernel-generated reservation plan from the immutable relation policy and
    operation:

```text
RequiredReservationPlan {
    operation_kind,
    relation_policy_identity,
    placement_policy_identity,
    topology_epoch,
    participant_set_epoch,
    selected_child_owner_incarnation,
    capacity_config_identity,
    canonical_entries_hash,
    mandatory_resource_class_bitmap,
}
```

  - `capacity_config_identity` is the canonical identity of the immutable
    participant-capacity manifest for this transaction, not a system-wide hash
    of every core:

```text
ParticipantCapacityBinding {
    resource_owner_incarnation,
    resource_owner_capacity_generation,
    relevant_class_limits_digest,
}
```

  - `relevant_class_limits_digest` is a nested domain-separated digest over the
    exact typed limits used by that participant binding:

```text
relevant_class_limits_digest =
    H(
        "aesynx-capacity-class-limits-v1"
        || owner_capacity_generation
        || canonical_typed_class_limits
    )
```

  - `canonical_typed_class_limits` uses stable resource-class IDs, strongly
    typed units, ordinary/emergency capacity class, configured limit, reserved
    amount where relevant, fixed-width little-endian fields, canonical ordering,
    explicit schema version, and no Rust enum or memory layout bytes;
  - unknown mandatory capacity classes reject the binding before prepare or
    commit;
  - class-limit hash algorithm or schema migration creates a new digest and
    cannot reuse old prepared acknowledgements;
  - owners validate the actual canonical typed limits and recompute
    `relevant_class_limits_digest`; they never accept an opaque caller- or
    coordinator-supplied digest as authority;

```text
capacity_config_identity =
    H(canonical_sorted(required_participant_capacity_bindings))
```

  - plan identity is computed over a versioned canonical byte string:

```text
plan_identity =
    H("aesynx-derived-reservation-plan-v1" || canonical_plan_bytes)
```

  - the hash algorithm and algorithm version are explicit fields in the plan
    identity. The v1 encoding uses fixed-width fields, little-endian integers,
    canonical ordering, explicit length prefixes for variable lists, and no Rust
    enum or structure layout in the hashed bytes;
  - `canonical_plan_bytes` includes operation kind, relation-policy identity,
    placement-policy identity, topology epoch, participant-set epoch, selected
    child-owner incarnation, capacity-configuration identity, resource
    owners, resource classes, bounded quantities/slots, terminal release
    policies, and the mandatory-class bitmap;
  - `capacity_config_identity` is a recovery/configuration identity for
    fixed-memory sizing and reserve accounting. It is scoped to owners and
    resource classes required by the canonical reservation plan, not unrelated
    cores. It is not automatically the fabric wire protocol version, and cores
    may have different fixed capacities without becoming wire-incompatible when
    the canonical plan names the immutable participant manifest identity used
    for that transaction;
  - global quota-policy identity or escrow-coordinator identity is included as
    a separate required participant binding only when that policy or coordinator
    supplies a required reservation or limit. It is not smuggled into the plan
    by hashing every core's local capacity;
  - the coordinator constructs the expected participant-capacity binding set
    from kernel policy. Callers cannot omit owners, classes, quota authorities,
    or escrow coordinators;
  - owners validate the canonical fields they receive against their local
    resource state and relation-policy view, not merely equality of a
    caller-provided hash;
  - placement is recomputed or validated immediately before prepare. A topology
    epoch change, placement-policy change, selected-owner-incarnation change,
    or capacity-configuration identity change before prepare requires
    replanning under a new transaction ID;
  - an owner capacity change either changes the owner incarnation or invalidates
    every outstanding reservation under the old owner capacity generation;
  - after prepare, the frozen participant set and selected child owner remain
    authoritative unless an owner incarnation becomes invalid. Invalid
    incarnations drive abort, recovery, or quarantine rather than transparent
    replacement under the same transaction ID;
  - retries with the same transaction ID return the original placement decision
    and do not rerun placement as a hidden side effect;
  - hash-algorithm migration creates a new plan identity and cannot reuse old
    prepared acknowledgements or reservations;
  - callers cannot select `resource_class`, `terminal_release_policy`,
    participant set, or required quantities;
  - required resource classes are determined only by operation kind,
    relation-policy identity, object state, destination identity, and the
    current participant-set epoch;
  - reservation entries are canonically ordered and unique. Missing, duplicated,
    conflicting, or unknown mandatory entries reject commit;
  - the commit certificate binds the required-plan hash and every prepared
    reservation generation;
  - an owner acknowledgement cannot substitute a different slot or amount from
    the one requested by the canonical plan;
  - policy, placement, topology, participant-set, selected-owner, or
    capacity-configuration migration changes the plan identity and cannot reuse
    old prepared reservations;
  - before the first remote prepare request, the parent owner reserves a
    parent-local journal slot plus abort/recovery capacity, persists a
    torn-record-protected `Preparing` record, and installs the parent-local audit
    placeholder;
  - the `Preparing` record contains transaction ID, coordinator incarnation,
    required reservation-plan identity, canonical plan entries, complete
    participant set, policy/object/edge/destination incarnations,
    timeout/recovery owner, and the initial audit placeholder generation;
  - if parent-local bootstrap reservation or `Preparing` persistence fails, the
    operation aborts before producing any remote side effect;
  - remote prepare requests are sent only after the `Preparing` record and audit
    placeholder are recoverable;
  - acknowledgement progress is persisted as generation-stamped participant
    records or a participant bitmap before it can count toward commit;
  - commit is allowed only after all required acknowledgements are recoverably
    represented in the parent-owned journal;
  - after coordinator restart, recovery queries or releases every participant
    named in the persisted plan even if the crash happened after a remote owner
    reserved resources but before its acknowledgement was recorded;
  - every distributed reservation is represented by a fixed manifest entry:

```text
PreparedReservation {
    transaction_id,
    resource_class,
    resource_owner_incarnation,
    resource_owner_capacity_generation,
    reservation_generation,
    bounded_amount_or_slot,
    terminal_release_policy,
}
```

  - the parent owner reserves outgoing edge slots, journal records, audit
    placeholders it owns, object/lineage/principal quota credits it owns, and
    revocation-progress resources;
  - the child owner reserves the child registry slot, inbound edge record, and
    required backing resources it owns;
  - the destination table owner reserves the destination capability-table slot;
  - each owner is the only writer for its own reservation and duplicate
    commit/abort/release messages are idempotent;
  - prepared acknowledgements include reservation generations, owner-local
    capacity generations, and the relevant class-limit digest, and are bound
    into the commit certificate;
  - an owner cannot acknowledge using capacity from one generation and commit
    under another;
  - unknown or mismatched capacity generations fail before commit;
  - timeout alone cannot release a reservation while commit may have been
    observed;
  - reservations from an obsolete owner incarnation cannot satisfy a new
    transaction;
  - reservations are acquired in canonical order by protocol rank, owner
    incarnation, and resource class;
  - no owner synchronously waits for another owner while holding an owner-state
    guard;
  - if the next reservation is unavailable, the coordinator records abort and
    releases already prepared reservations through reserved control capacity;
  - transaction priority is a kernel-generated total order over coordinator
    epoch and transaction sequence; callers cannot influence priority;
  - each resource owner grants a conflicting reservation to the highest-priority
    eligible transaction according to the same comparison rule;
  - a losing transaction receives a deterministic conflict response and must
    abort rather than retain partial reservations and retry in place;
  - prepared or committed transactions cannot be displaced by a later
    transaction;
  - bounded owner-local admission queues or aging rules prevent indefinite
    starvation of lower-priority principals;
  - priority comparisons involving obsolete coordinator incarnations fail
    closed;
  - before a commit decision, coordinator timeout may record abort and initiate
    release; after commit may have been observed, timeout cannot independently
    release anything;
  - pending reservations and retries are bounded per principal, object, and
    owner to prevent reservation-based denial of service;
  - prepare holds transaction-bound reservations for child registry slots, edge
    slots, destination capability-table slots, object/lineage/principal quota
    credits, journal records, replay records, revocation-progress credit,
    required backing pins or pending resources, and required security-audit
    record capacity;
  - commit consumes those reservations, abort releases them idempotently, and
    recovery or quarantine keeps them charged until terminal resolution and
    replay retirement;
  - the invariant is: `Committed => reaching a safe terminal state requires no
    ordinary allocation`;
  - every live edge either consumes reserved revocation-progress credit at
    creation or stores progress directly in edge records using revocation
    generations and restartable cursors that need only constant reserved
    executor state;
  - edge publication fails if future revocation cannot be represented;
  - revocation does not depend on the ordinary allocator, ordinary IPC
    capacity, or best-effort journal space;
  - credits are returned only after edge retirement and replay-window closure.
- Edge publication couples audit evidence to the commit decision:
  - v1 audit placeholders are parent-owner-local and do not introduce a separate
    audit owner participant;
  - prepare installs a security-audit placeholder bound to transaction ID,
    operation, pinned policy identity, principal, and reserved audit generation;
  - the journal commit references the audit reservation generation;
  - commit either finalizes the record atomically on the parent owner or leaves
    enough prepared evidence for deterministic recovery to finalize it without
    ordinary allocation;
  - abort finalizes or retires the placeholder as aborted;
  - torn or missing prepared audit evidence prevents commit;
  - audit finalization cannot synchronously reenter the parent transaction or
    capability table;
  - audit payloads remain redacted and cannot contain reusable authority
    identifiers.
- Strong revocation with bounded traversal cannot report partial success:
  - root revocation first freezes new delegation, derivation, mapping, and
    promotion at the root;
  - a bounded revocation worklist is persisted or preallocated before work
    begins;
  - descendants are processed incrementally with generation-stamped
    continuation cursors;
  - affected objects remain `Revoking` and unusable while traversal continues;
  - success is reported only after every required owner has fenced or
    invalidated its descendants;
  - queue exhaustion, owner timeout, or traversal-budget exhaustion leaves the
    operation in quarantined/revoking state, never partial success.
- Concurrent edge insertion must make cycle prevention race-resistant:
  - incoming edge commits are serialized at the child owner and combined with
    ordered owner-to-owner transactions, or the validation snapshot carries a
    topology/edge epoch that must remain unchanged through commit;
  - parent and child object generations are revalidated at commit;
  - the transaction is rejected if the validation snapshot changed;
  - intrinsically hierarchical relation kinds carry checked derivation depth or
    rank;
  - unknown relation kinds or relation-policy identities received over IPC are
    rejected rather than interpreted through defaults.
- Wire-format v1 notes for all authority-bearing IDs: fixed widths,
  endianness, versioning, domain incarnation fields, and no Rust enum layout
  crossing fabric or userspace boundaries.
- Security-control update that marks the current caller-ID/object-ID model as
  scaffolding until the new authority identity rules are implemented.

Verification:

- Host tests prove stale capabilities cannot become live again after object
  deletion, slot reuse, and same visible object-name recreation.
- Host tests prove enforcement paths cannot be authorized by passing a forged
  `CoreId`, `PrincipalId`, `ObjectId`, or table owner value.
- Host tests prove table-only checks are unavailable or clearly non-enforcing
  outside internal/preflight contexts.
- Host tests prove failed grant proposals leave receiver tables unchanged and
  pending slots expired or reclaimable.
- Host tests prove a sender revoke between grant proposal and grant commit
  aborts the grant without receiver authority becoming usable.
- Model tests prove transactional grant has exactly one final outcome and no
  phantom authority after abort, timeout, duplicate message, or sender revoke.
- Host/model tests prove move-grant escrow never produces two active owners and
  never loses the authority on abort.
- Host tests prove invalid typed-right/kind combinations fail at mint, derive,
  decode, and live resolution.
- Host tests prove invalid kind/right combinations also fail during grant
  proposal, grant commit, and checked live proof creation.
- Host tests prove high-risk rights are stripped or rejected when delegation is
  not explicitly allowed by the central kind-to-right matrix.
- Host tests prove task factory/control/join rights participate in mint,
  derive, grant, decode, live-resolution, audit, and revocation checks; generic
  `ADMIN` does not satisfy task lifecycle rights.
- Host tests prove task capabilities and domain capabilities cannot be decoded,
  granted, or resolved as each other.
- Host tests prove unknown mandatory typed rights and rights invalid for the
  encoded object kind fail during wire decode.
- Host/model tests prove common-right and kind-specific attenuation are both
  subsets of the sender's live authority.
- Host tests prove receiver-supplied grant records cannot widen delegated
  rights.
- Host tests prove `ADMIN` never satisfies an unrelated typed-right check and
  external `CapId` kind tags cannot override the registry slot's live kind.
- Host/model tests prove coordinator restart uses only transaction-journal
  evidence and cannot commit from participant-controlled claims.
- Host/model tests cover transaction-journal torn records, capacity
  exhaustion, record reuse before replay-window retirement, coordinator
  evidence loss after prepare, and coordinator evidence loss after possible
  commit.
- Host tests prove old table/domain incarnations cannot interpret a recycled
  `CapId`.
- Host tests prove replayed grant/revoke/map messages outside the accepted
  transaction window fail closed.
- Host tests prove endpoint send/receive checks require endpoint rights and
  kernel-stamped source metadata.
- Confused-deputy adversarial tests prove a privileged storage, index, world,
  or package service cannot be induced by an unprivileged client to read or
  mutate an arbitrary object by supplying only a name, path, principal ID, or
  another caller's capability-table handle.
- Host tests prove scheduling-context donation changes CPU budget only and
  never authorizes a service operation without the declared capability
  arguments.
- Host tests prove reply capabilities are one-shot, caller/transaction-bound,
  rejected after timeout/cancellation/server death, cleaned up during restart,
  and not redirectable to another caller.
- Host tests prove reply cancellation is idempotent and stale reply authority
  from a previous server incarnation cannot be consumed after restart.
- Host tests prove map requests require both memory-object and address-space
  authority.
- Host/model tests prove no cross-object child becomes usable before its
  dependency edge commits and local child publication validation completes.
- Host tests prove boundary code cannot authorize edge operations from
  caller-supplied parent/child/edge tuples. `DerivationControlPermit` values
  are minted only after validating caller-held parent-object kind-specific
  relation-operation rights, required child-side authority, relation policy,
  execution identity, quotas, audit capacity, destination identity, and
  transaction identity.
- Host tests prove `CommonRights::DERIVE` cannot create a new object
  incarnation, `CREATE_DERIVED` always uses the edge protocol, relation policies
  name the exact child-side right and consume/retain/revalidate behavior, and
  `PROMOTE`/`DETACH_DERIVATION` are nondelegable by default.
- Host tests prove `DerivationControlPermit` values are internal proof objects:
  nondelegable, one-shot, exact-operation-bound, unavailable for
  capability-table storage, and rejected by stable wire decoders or cross-core
  IPC schemas.
- Host/model tests prove the parent owner mints a permit only after resolving
  caller authority, the permit authorizes only the `Undecided -> Committed`
  journal transition on the parent-owned coordinator/journal shard, commit
  consumes it, abort/timeout destroys it without reuse, local publication uses
  the commit certificate rather than the permit, and coordinator restart or
  recovery never persists or recreates a permit.
- Host/model tests prove v1 derived-edge transaction IDs bind the parent-owner
  journal shard and owner incarnation, child/destination owners never receive a
  permit, parent-owner failure before commit cannot be recovered as a new commit
  decision, and recovery may only reproduce an existing commit certificate from
  durable journal evidence.
- Host/model tests prove duplicate commit attempts, coordinator restart,
  delayed requests, and replay cannot consume the same permit twice or mint a
  replacement from stale authority.
- Host/model tests prove source-wide or object-wide revocation finds all
  cascade-bound children, lineage-local revocation does not affect children
  created through unrelated lineages, and child revocation does not
  accidentally revoke the parent.
- Host/model tests prove v1 child publication is single-parent-only, rejects
  adding an incoming edge to an already `Live` child, rejects any second parent
  edge before mutation, accepts only `Live` published unfrozen parents, rejects
  pending/revoking/recovering/quarantined/unpublished parents, computes
  `child_depth = checked(parent_depth + 1)` under a fixed maximum, and cascades
  revocation from the committed parent when policy requires it.
- Host/model tests prove child-owner selection is kernel-controlled:
  caller-supplied core/owner IDs and locality preferences are hints only,
  placement uses owner incarnation, topology epoch, NUMA locality where known,
  escrowed quota, registry capacity, and quarantine/drain/restart state, the
  selected child owner and placement-policy identity are frozen into the
  reservation plan, mid-transaction placement changes require abort plus a new
  transaction ID, stale/quarantined/draining/restarting owners are rejected, and
  destination-table ownership does not imply child-object ownership.
- Differential host/model tests prove same-owner local transactions and
  distributed transactions produce equivalent decisions, rights, audit records,
  revocation behavior, and failure outcomes. The local path may elide IPC but
  must retain the same logical manifest, transaction states, reservations,
  permit consumption, audit placeholder, commit certificate, publication checks,
  and recovery semantics.
- Host/model tests prove any attempted multi-parent child is rejected until a
  future `ParentSetManifest` milestone defines canonical parent-set identity,
  all-parent approvals, complete-set commit, rights intersection, concurrent
  revocation coalescing, promotion semantics, and recovery.
- Host/model tests prove edge capacity exhaustion leaves no usable orphan,
  crash between child creation and edge publication aborts, recovers, or
  reports resource loss without making the child usable, v1 cycles are
  structurally impossible through parent-state/depth checks, and edge retirement
  plus source/object ID reuse cannot resurrect a stale child relationship.
- Host/model tests prove successful promotion retires authority dependencies
  while appending immutable provenance records, and provenance records do not
  grant authority, retain resources, or trigger cascading revocation.
- Host/model tests prove provenance records are kernel-stamped,
  integrity-protected, bound to object/source incarnations, content hashes,
  promotion transaction, pinned policy identity, and boot/domain incarnation
  where relevant; field origins are classified; retired source incarnations
  cannot resolve to recycled objects; bounded provenance reclamation cannot
  silently change an authorization decision; and provenance alone never proves
  publisher trust without explicit trust-policy authority.
- Host/model tests prove promotion requires parent-side `PROMOTE` or
  `DETACH_DERIVATION` authority, revalidates the parent and edge generation at
  the linearization point, fails during parent revocation, and cannot
  caller-select a weaker revocation policy.
- Host/model tests prove promoted-root rights are bounded by requested rights,
  live child rights, and relation-policy promotable rights; `GRANT`, `REVOKE`,
  `ADMIN`, executable/JIT, and writable-sharing authority do not appear
  implicitly.
- Host/model tests prove promotion is bound to exact parent lineage, child
  incarnation, edge generation, destination principal, and destination table
  incarnation, reserves destination table/audit capacity during prepare, and
  rejects destination principal redirection after prepare.
- Host/model tests prove promotion is atomic: the old dependency remains live
  on failure, or the new root exists with inherited provenance and the old edge
  is retired.
- Host/model tests prove immutable relation-policy identity cannot be
  reinterpreted after edge creation, weak policies cannot be installed through
  replay/peer negotiation/coordinator restart, and live policy migration
  requires freeze/revalidate/commit.
- Host/model tests prove distributed edge recovery handles reservation,
  one-sided installation, commit-before-acknowledgement, obsolete owner
  incarnations, and replay-window retirement without producing usable orphan
  children.
- Host/model tests prove prepared reservation manifests record transaction ID,
  resource class, resource-owner incarnation, resource-owner capacity
  generation, reservation generation, bounded amount/slot, and terminal release
  policy for each resource; parent, child, and destination owners write only
  their own reservations; prepared acknowledgements bind reservation
  generations, owner-local capacity generations, and relevant class-limit
  digests into the commit certificate; an owner cannot acknowledge under one
  capacity generation and commit under another; unknown or mismatched capacity
  generations or class-limit digests fail before commit; and obsolete
  owner-incarnation reservations cannot satisfy a new transaction.
- Host/model tests prove the canonical `RequiredReservationPlan` is derived
  from operation kind, immutable relation-policy identity, placement-policy
  identity, topology epoch, participant-set epoch, selected child-owner
  incarnation, capacity-configuration identity, object state, and destination
  identity; callers cannot choose resource classes, release policy,
  participants, placement outcome, or quantities; entries are canonical,
  ordered, and unique; missing, duplicated, conflicting, or unknown mandatory
  entries reject commit; commit certificates bind the plan hash plus every
  prepared reservation generation; acknowledgements cannot substitute another
  slot/amount; same-transaction retries return the original placement decision;
  pre-prepare topology, placement-policy, owner-incarnation, or
  capacity-manifest changes force replanning with a new transaction ID; owner
  capacity changes either change owner incarnation or invalidate outstanding
  reservations under the old owner capacity generation; asymmetric per-core
  capacities remain wire-compatible; unrelated owner-capacity changes do not
  invalidate the transaction; required parent, child, destination, quota,
  escrow, or other participant-owner changes do invalidate it; and
  policy/placement/topology/capacity migration changes the plan identity.
- Host/model tests prove plan identity uses the
  `aesynx-derived-reservation-plan-v1` domain-separation label, explicit hash
  algorithm/version, fixed-width little-endian canonical encoding, explicit list
  lengths, no Rust enum/layout bytes, and includes operation, policy identity,
  placement policy, topology epoch, participant epoch, selected child owner,
  capacity-configuration identity, resource owners/classes/quantities, release
  policies, and mandatory-class bitmap. The capacity identity is the hash of a
  canonical sorted manifest of required participant-capacity bindings, each
  carrying owner incarnation, owner-local capacity generation, and a digest of
  the relevant typed per-class limits. The class-limit digest uses the
  `aesynx-capacity-class-limits-v1` domain-separation label, owner capacity
  generation, stable resource-class IDs, strongly typed units,
  ordinary/emergency capacity class, configured limit, reserved amount where
  relevant, fixed-width little-endian fields, canonical ordering, explicit
  schema version, no Rust enum/layout bytes, unknown mandatory-class rejection,
  and hash/schema migration rules. Owners recompute the digest from actual
  canonical typed limits rather than accepting an opaque caller- or
  coordinator-supplied digest. The manifest is
  O(number of transaction participants), not O(system cores). Owners validate
  canonical fields rather than trusting a supplied hash, and hash migration
  rejects old acknowledgements.
- Checked-in cross-endian golden byte vectors cover reservation-plan canonical
  encoding, participant-capacity bindings, class-limit digest input, final
  plan identity, and final capacity identity. The vector corpus includes
  empty, minimum, and maximum lists; unknown mandatory capacity classes;
  asymmetric owners; and hash/schema-version changes. Encoders and decoders
  must reproduce these vectors before the format is treated as stable.
- Fixed negative golden vectors prove decoders reject noncanonical ordering,
  duplicate or conflicting entries, truncated input, trailing bytes, incorrect
  list counts, integer overflow, maximum-length-plus-one encodings, wrong
  endianness, unknown mandatory classes while tolerating documented unknown
  optional classes, correct fields paired with an incorrect digest, and old
  schema/hash versions paired with new acknowledgements.
- Host/model crash tests prove parent-local bootstrap order: no remote prepare
  is sent before parent journal slot plus abort/recovery capacity are reserved,
  the torn-record-protected `Preparing` record is persisted, and the
  parent-local audit placeholder is installed. Crash points cover before the
  preparing record, after the record before first message, after each prepare
  request, after remote reservation before acknowledgement, and after
  acknowledgement arrival before local journal update.
- Host/model recovery tests prove coordinator restart queries or releases every
  participant named in the persisted plan even when acknowledgement progress was
  not locally recorded, and commit is impossible until every required
  acknowledgement is recoverably represented in the parent-owned journal.
- Host/model tests prove commit is impossible until the complete reservation
  manifest is acknowledged, timeout alone cannot release a reservation while
  commit may have been observed, and duplicate commit/abort/release messages are
  idempotent.
- Host/model tests exercise every derived-edge sizing constant at zero, one,
  maximum, and exhaustion, including simultaneous per-principal and global
  limits. Tests prove checked total-storage arithmetic, documented per-core and
  system-wide byte budgets, separate ordinary/recovery/abort-release/revocation
  capacity classes, no ordinary request consuming emergency capacity, typed
  exhaustion errors without partial mutation, capacity changes updating
  configuration identity, charged quarantine/`ResourceLost` records until
  retirement, and per-class terminal-progress proofs for combinations of
  preparing transactions, recovering edges, and concurrent strong revokes.
  Tests saturate combinations, not only each limit independently, and prove
  admission into `Preparing` consumes the required terminal-progress credits
  before any remote side effect. Tests also cover asymmetric per-core capacity
  manifests, an unrelated owner changing capacity without affecting the
  transaction, and required participant capacity-generation changes during
  planning, prepare, recovery, and commit.
- Host/model tests cover two-or-more-coordinator reservation contention with
  opposite acquisition orders, full capacity, delayed release messages,
  repeated retries, coordinator failure, and eventual progress under stated
  fairness assumptions. The model proves canonical acquisition order,
  no-wait-while-guarded discipline, deterministic transaction priority,
  reserved abort/release capacity, and per-principal/object/owner pending
  bounds prevent reservation deadlock, livelock, and denial-of-service.
- Host/model tests prove every resource owner applies the same
  kernel-generated priority rule over coordinator epoch and transaction
  sequence, callers cannot influence priority, conflicts go to the
  highest-priority eligible transaction, losing transactions abort instead of
  retaining partial reservations, prepared/committed transactions are not
  displaced by later requests, owner-local admission/aging prevents starvation,
  and obsolete coordinator incarnations fail closed.
- Host/model tests prove edge state, transaction decision, and participant
  progress are interpreted separately; received `edge_state` is not
  authoritative by itself, aborted pending edges remain replay-detectable until
  journal retirement, uncertain commit never becomes inferred abort, and
  recoverable quarantine records are incarnation-bound.
- Host/model tests prove journal commit decision and local child publication
  are separate: no user-visible handle returns before local publication,
  publication verifies commit certificate, policy identity, owner
  incarnations, single v1 parent edge, parent-edge discoverability, and parent
  freeze/revocation state, and parent revocation between commit and publication
  sends the child to `Revoking`, `QuarantinedAwaitingEvidence`, or
  `ResourceLost` without becoming usable.
- Host/model tests prove committed-but-not-published terminal outcomes are
  stable: `Aborted`, `Published`, `CommittedButRevoked`, and `ResourceLost` are
  idempotent by transaction ID. Recoverable `Recovering` and
  `QuarantinedAwaitingEvidence` observations may make monotonic progress from
  trusted journal/witness evidence, retries never create a second child, and no
  handle absence is treated as permission to duplicate a committed child under
  the same transaction.
- Host/model tests prove `ResourceLost` is represented as terminal resolution,
  not a normal journal decision, and can be installed only after all
  potentially publishing participants are incarnation-fenced, reset, or denied
  execution; candidate capabilities, handles, and commit certificates are
  retired; a terminal tombstone blocks replay; delayed commit/abort evidence is
  rejected; and pins, mappings, DMA, and TLB obligations are drained before
  reclamation.
- Host/model fault tests inject a delayed valid-looking commit after
  `ResourceLost` and prove the terminal tombstone plus incarnation fence rejects
  it without resurrecting authority.
- Host/model tests inject every interleaving between commit decision, parent
  revoke, participant apply, acknowledgement, and child publication.
- Host/model tests exhaust ordinary allocation, IPC, and best-effort journal
  capacity while proving root freeze and derived-edge revocation can still
  begin and make bounded progress through reserved revocation credit or
  restartable in-edge progress cursors.
- Host/model tests exhaust child registry slots, edge slots, destination table
  slots, quota credits, journal/replay records, audit records,
  revocation-progress credit, backing pins, and pending resources after initial
  validation but before prepare. The transaction must either hold a reservation
  through commit/recovery/quarantine or abort before commit; no committed
  transaction may require ordinary allocation to reach a safe terminal state.
- Host/model crash tests cover audit placeholder reservation, journal commit,
  and audit finalization. Authority-creating commits must reference the audit
  reservation generation, torn or missing prepared audit evidence prevents
  commit, committed-but-unfinalized audit records are finalized deterministically
  during recovery without ordinary allocation, abort retires or finalizes the
  placeholder as aborted, v1 audit placeholders are parent-owner-local rather
  than a separate participant, audit finalization cannot synchronously reenter
  the parent transaction or capability table, and audit payloads remain redacted
  without reusable authority identifiers.
- Host/model tests prove strong parent revocation freezes new child creation,
  processes descendants through generation-stamped continuation cursors, leaves
  affected descendants unusable while `Revoking`, and reports success only when
  all required owners have fenced or invalidated descendants.
- Host/model tests prove traversal-budget exhaustion, queue exhaustion, and
  owner timeout leave the operation quarantined/revoking instead of partially
  successful.
- Host/model tests prove concurrent opposite-edge insertion cannot create a
  cycle, stale validation snapshots are rejected at commit, hierarchical
  relation depth/rank is enforced, and unknown relation kinds or policy
  identities from IPC fail closed.
- Host tests prove stale table entries are tombstoned or reclaimed within
  bounded quotas after revocation.
- Model tests prove quota-credit accounting preserves the configured ceiling
  through spend, prepare, commit, abort, duplicate refund, coordinator restart,
  dead owner, quarantined-but-not-fenced owner, and offline local-spend cases.

Exit criteria:

- Capability IPC has a hardened identity and endpoint foundation suitable for
  the later shared-memory and multikernel fabric milestones.

### v0.37.2 - Shared Memory Object Model

Goal:

Model explicit zero-copy sharing between dispatchers without claiming live
cross-address-space mappings before strong revocation, TLB shootdown, and
atomic fabric queues exist.

Design rule:

Shared memory is never raw physical authority at the user API. A caller asks for
a typed shared-buffer object or derives authority from an existing memory
object. The kernel decides the physical backing internally and returns
capabilities with bounded range, permission, lifetime, and revocation metadata.

This milestone is model-only. It may build descriptors, proofs, host tests, and
QEMU markers for the object/capability shape, but it must not claim that live
shared mappings are safe until the later strong-revocation and live mapping
integration milestones land.

Deliverables:

- Shared-buffer object kind or typed memory-object mode.
- Shared memory capability derivation:
  - `SHARE_READ` for read-only shared mappings.
  - `SHARE_WRITE` only with an explicit synchronization protocol.
  - `MAP` still required before any address-space mapping is created.
- Multi-address-space mapping descriptor that describes the same backing object
  being mapped into multiple dispatchers through separate capability grants.
- Read-only seal/freeze operation for large asset buffers, such as geometry,
  texture, model, or package-block data.
- Prospective revocation model showing that a later live validation rejects
  stale derived mapping descriptors.
- TLB shootdown requirement list for every core/address space that observed the
  mapping; execution of the shootdown is explicitly deferred to v0.37.9 and the
  live shared-mapping milestone.
- Audit events for create, grant, map, seal, downgrade, revoke, and unmap.
- Redacted diagnostics that expose sizes, permissions, and participant counts
  without exposing physical frames or raw object IDs.
- Explicit policy that mutable shared memory is exceptional; ordinary
  cross-core coordination still uses messages and owner-core mutation.

Example shape:

```text
create shared-buffer size=2GiB purpose=geometry
seal shared-buffer read-only
grant shared-buffer to dispatcher render-core-1 perms=MAP|READ
grant shared-buffer to dispatcher render-core-2 perms=MAP|READ
map shared-buffer into render-core-1
map shared-buffer into render-core-2
```

Verification:

- Host tests prove read-only shared-buffer descriptors can be produced for two
  dispatchers without copying.
- Host tests reject writable sharing without `SHARE_WRITE` and a declared
  synchronization protocol.
- Prospective revocation invalidates every later descriptor validation.
- Mapper tests distinguish allowed shared-frame aliasing from accidental
  physical-frame double ownership.

Exit criteria:

- The object/capability descriptor model for zero-copy shared assets exists,
  while live shared mappings remain blocked on strong revocation and TLB
  invalidation enforcement.

### v0.37.3 - Fabric Protocol And Heterogeneous Peer Metadata

Goal:

Define the machine-local message protocol that lets Aesynx treat cores and
future service domains as fabric peers instead of assuming one shared kernel
memory model.

Deliverables:

- `docs/multikernel-fabric-roadmap.md`.
- Versioned fabric message header.
- Explicit sender, receiver, sequence, message kind, and epoch fields.
- Core/domain role metadata that can describe x86_64 cores, future aarch64
  cores, P-core/E-core style heterogeneity, driver service domains, and trusted
  accelerator bridges.
- Peer and service identity records with generation/epoch fields so a restarted
  peer cannot inherit stale authority accidentally.
- Endianness, alignment, and ABI rules so the protocol does not rely on
  Rust-specific layout or x86_64-only assumptions.
- Bounded payload and extension-field policy.
- Protocol downgrade and extension policy:
  - each endpoint/service declares a minimum accepted protocol version;
  - version negotiation is bound to peer/domain incarnations;
  - no silent fallback after authenticated negotiation;
  - extension fields are marked required or optional;
  - unknown required extensions fail closed;
  - duplicate, noncanonical, or out-of-order fields are rejected according to
    the canonical wire encoding;
  - negotiated version and feature set are included in authority transaction
    and audit records.
- Kernel-managed channel/session object stores negotiation results:
  protocol ID, version, feature-set hash, peer/domain incarnations,
  session generation, and negotiation transcript hash.
- Every later message inherits protocol version and extension semantics from
  the channel/session object instead of selecting its own version or extension
  set.
- Peer restart, service-owner transfer, or route replacement invalidates the
  channel/session and requires renegotiation.
- Per-peer queue, retry, and outstanding-request bounds.
- Rejection/dead-letter message shape.
- Redacted debug output for peer identities and authority-bearing fields.
- Cross-core time semantics:
  - protocols do not compare raw timestamps from different cores unless a
    synchronized clock with a documented skew bound exists;
  - messages prefer relative TTLs over sender-provided absolute deadlines;
  - receivers stamp local deadlines on authenticated receipt;
  - timeout decisions are made by the coordinator's local monotonic clock;
  - epoch/incarnation changes invalidate old deadlines;
  - suspend, migration, TSC instability, and counter rollover fail closed.
- AI advice expiry uses receiver-local deadlines or scheduling epochs, not
  untrusted sender-provided absolute timestamps.

Verification:

- Host tests encode/decode fabric headers without raw pointer layout.
- Host tests reject unknown versions, oversized payloads, invalid peer roles,
  and non-monotonic sequence use where tracked.
- Host tests reject downgrade attempts where an intermediary removes supported
  versions or required extensions.
- Host tests reject duplicate and noncanonical extension encodings.
- Host tests prove post-negotiation messages cannot downgrade the channel by
  selecting older versions or omitting negotiated required extensions.
- Host tests prove peer restart, service-owner transfer, and route replacement
  invalidate the channel/session.
- Host tests prove sender-provided absolute timestamps are not accepted as
  cross-core authority deadlines without a synchronized-clock capability.

Exit criteria:

- Aesynx has one documented internal fabric ABI before adding more cross-core
  protocols.

### v0.37.3.1 - Security-Grade Monotonic Timebase

Goal:

Provide the monotonic-clock contract required by leases, deadlines, retries,
watchdogs, coordinator timeouts, AI advice expiry, and fabric progress without
assuming one global clock or letting timeouts manufacture authority.

Deliverables:

- Per-core monotonic clock-source selection and capability classification.
- x86_64 invariant/nonstop TSC detection with fallback policy.
- Frequency calibration with checked conversion into kernel time units.
- Core-to-core offset and skew measurement where cross-core comparison is
  required.
- Counter rollover handling and tested conversion boundaries.
- VM migration, frequency-change, suspend/resume, and AP-restart behavior.
- Clock generation/incarnation attached to deadlines, leases, watchdog
  decisions, and retry timers.
- Fail-closed behavior after detected backward jumps, unstable calibration,
  inconsistent per-core clocks, or unsupported timebase changes.
- Independent watchdog-source requirement where coordinator failure must be
  detected despite scheduler or fabric stalls.
- Rule that timeouts may drive abort, retry, cancellation, quarantine, or
  escalation, but never manufacture a commit, grant, mapping success, revoke
  success, or ownership transfer.
- Authenticated synchronized-clock capability required before comparing
  absolute times from different cores or peers.
- Redacted diagnostics for clock source, stability, generation, and skew class
  without exposing raw high-resolution timestamps to untrusted consumers.

Verification:

- Host/model tests inject backward jumps, forward jumps, skew, counter
  rollover, calibration failure, AP restart, suspend/resume discontinuity, and
  VM-migration-like discontinuity.
- Tests prove stale clock generations invalidate deadlines instead of reusing
  them after restart or resume.
- Tests prove timeout-triggered recovery can abort/quarantine but cannot commit
  a transaction without the required authority evidence.
- QEMU smoke records stable clock-source evidence before any live fabric
  timeout, lease, or watchdog claim depends on it.

Exit criteria:

- Fabric and authority protocols have a concrete monotonic-time foundation
  before live transactional timeouts or watchdogs become enforcement inputs.

### v0.37.4 - Replicated Authority State Protocol

Goal:

Handle global authority changes without a hidden global lock.

Deliverables:

- TLA+ or Quint model for grant, revoke, coordinator failure, participant
  timeout, duplicate commit/abort, and recovery before the protocol is treated
  as implementable.
- Owner/coordinator rule for replicated authority records.
- Monotonic epoch records for capability revocation, service ownership, routing
  table, and policy updates.
- Prepare/commit/abort message types for critical authority changes.
- Coordinator incarnation and fencing token for every critical transaction.
- Precise participant-set rule: required participants are selected from an
  epoch-stamped topology/service snapshot and cannot change silently
  mid-transaction.
- Coordinator-failure recovery rules for pending transactions, including
  restart/resynchronization, idempotent duplicate commit/abort handling, and
  bounded timeout escalation.
- Fail-closed stale-epoch handling.
- Timeout and participant-dead handling.
- Audit events linking proposal, acknowledgement, commit, abort, and revoke.
- Emergency audit-loss rule: authority creation, grant, executable mapping,
  DMA mapping, and policy expansion fail closed if required audit evidence
  cannot be recorded; revoke, quarantine, and permission reduction proceed
  fail-safe, reserve emergency audit capacity, and set a sticky audit-loss
  digest or halt after authority is removed rather than preserving authority
  because the normal audit queue is full.
- Emergency audit capacity is system-reserved and non-delegable, not a
  principal-owned quota. It is preallocated per core or authority class, uses
  allocation-free logging, can set a sticky `audit_lost` record without heap,
  service locks, or ordinary queues, and is not directly writable by ordinary
  principals.
- Emergency audit records have explicit recovery and clearing semantics so an
  operator or trusted recovery service can distinguish a handled audit-loss
  condition from a silently reset one.
- Explicit non-goal that full quorum/distributed consensus is later work unless
  Aesynx grows fault-tolerant peer groups.

Verification:

- Host model tests prove a revoke proposal cannot commit if a required
  participant rejects or times out.
- Host model tests prove coordinator death leaves participants in a recoverable
  pending, abortable, or quarantined state rather than unbounded limbo.
- Host model tests prove duplicate commit and abort messages are idempotent.
- Host model tests prove stale epochs cannot regain authority after commit.
- Audit logs preserve proposal-to-commit linkage without exposing raw object
  IDs.
- Negative model variants prove audit exhaustion cannot preserve revocable
  authority.
- Host tests prove compromised principals cannot exhaust or write emergency
  audit capacity directly, and revocation still proceeds when normal audit
  delivery, heap allocation, and ordinary service queues fail.

Exit criteria:

- Cross-core revocation and system policy updates have a machine-local
  agreement protocol.

### v0.37.5 - Topology-Aware Fabric Routing

Goal:

Move beyond direct ping/pong by recording topology and load facts for routing
decisions.

Deliverables:

- Topology facts for core, cluster, NUMA node where available, device locality,
  peer role, queue depth, and recent latency.
- Deterministic route selection policy.
- Backpressure signals.
- Retry and dead-letter policy.
- Routing telemetry with redacted peer identities.
- Explicit fallback to direct routing when topology facts are unavailable.

Verification:

- Host tests choose stable routes from synthetic topology facts.
- Host tests prove overloaded or dead peers are avoided when a valid fallback
  exists.
- Routing diagnostics expose reason codes, not raw addresses.

Exit criteria:

- Aesynx can route fabric messages through policy rather than hardcoded
  core-to-core assumptions.

### v0.37.6 - Component Fault Containment

Goal:

Make driver/service-domain failure a contained event where possible instead of
an automatic whole-kernel halt.

Deliverables:

- Fabric heartbeat and watchdog records.
- Fault-domain model for driver/service cores and future accelerator peers.
- Quarantine state.
- Capability revoke-on-fault flow.
- In-flight message cancel/replay policy.
- DMA/IOMMU cleanup requirement before a driver service restarts.
- Service rebinding plan.
- Restart budget and escalation policy.
- Telemetry for fault, quarantine, revoke, restart, and escalation.

Verification:

- Host model tests simulate a service timeout and prove new grants are rejected
  while the domain is quarantined.
- Host model tests prove restart cannot occur until authority and DMA cleanup
  policy has completed.

Exit criteria:

- The roadmap has an explicit path from isolated drivers to restartable service
  domains.

### v0.37.7 - Monitor Boundary And Minimal Ring-0 TCB

Goal:

Define the boundary between the per-core privileged CPU-driver layer and the
user-space monitor/service domains before the fabric becomes rich enough to
tempt ring-0 policy growth.

Deliverables:

- CPU-driver contract for local traps, interrupts, address-space switching,
  capability enforcement, and message endpoint delivery.
- Monitor/service-domain contract for global capability agreement, routing
  policy, topology/world queries, telemetry aggregation, package decisions, AI
  advice, driver policy, and restart orchestration.
- Migration inventory for current in-kernel scaffolds that must move or split
  once native userspace exists.
- Explicit rule that AI/model execution and rich world queries never run in
  ring 0.
- Explicit rule that raw physical frame allocation stays owner-local/per-core
  where possible; capabilities govern memory objects, mappings, sharing, DMA,
  transfer, executable authority, and revocation.
- Privileged per-core kernel crate dependency allowlist.
- Maximum privileged protocol-decoder surface for local traps, endpoint
  delivery, and authority checks.
- Static-memory budgets for topology, capability, journal, and lineage state.
- Explicit inventory of heap-using ring-0 paths.
- Unsafe-island count, ownership, and extraction/non-growth policy.
- Every in-kernel policy scaffold names a userspace extraction target.
- Rich routing, graph traversal, world queries, model loading, signature
  verification, and package policy remain outside the per-core fast path.
- Formal-verification target list for local capability checks, fabric message
  decoding, shared-buffer alias rules, and replicated authority protocols.
- Updated security controls that distinguish current QEMU scaffolding from
  future production TCB claims.

Verification:

- Documentation gate proves every planned fabric authority path names its
  privileged local mechanism and its monitor/service policy owner.
- Host model tests or static checks reject new fabric protocol definitions that
  lack an owner, timeout, stale-epoch behavior, and redaction rule.
- Static checks enforce the privileged dependency allowlist and require a
  reviewed exception for new ring-0 heap use, unsafe islands, or protocol
  decoder growth.
- Monitor failure tests prove the local kernel continues enforcing existing
  authority without accepting new global policy.

Exit criteria:

- Aesynx has a documented path to a small per-core kernel plus isolated
  monitor/services before distributed policy becomes live.

### v0.37.7.1 - Minimum Core Incarnation Contract

Goal:

Provide the minimum authoritative identity contract that live AP-backed queues
consume before the richer restart/hotplug model arrives.

Deliverables:

- Machine boot nonce or machine-session nonce available to fabric identity.
- Immutable core incarnation minted only after successful AP startup.
- Startup-attempt generation distinct from reusable logical core ID.
- Late-arrival rejection before the AP can publish, consume, or acknowledge
  live fabric messages.
- Endpoint and link identities bound to boot nonce, topology epoch, logical
  core, core incarnation, endpoint incarnation, and link generation.
- Permanent quarantine until reboot for failed/timed-out APs; no identity reuse
  and no restart/hotplug in this milestone.
- Explicit statement that v0.37.11 owns restart/hotplug recovery, but v0.37.8
  may not consume core-incarnation fields until these minimum semantics exist.

Verification:

- Host tests prove a stale startup attempt cannot mint a current core
  incarnation.
- Host tests prove endpoint/link identity validation rejects mismatched boot
  nonce, topology epoch, core incarnation, endpoint incarnation, and link
  generation before payload parsing.
- QEMU model smoke proves a quarantined AP identity is not reused.

Exit criteria:

- Live atomic IPC has authoritative core-incarnation fields to consume, while
  restart and hotplug remain explicitly disabled until v0.37.11.

### v0.37.7.2 - Per-Core Memory Ownership Contract

Goal:

Define allocator and frame ownership before AP-backed services can mutate
kernel memory from multiple cores.

Deliverables:

- Per-core heap arena or slab-cache ownership model.
- Owner-stamped physical-frame allocation records.
- Remote-free queues for memory freed on a non-owner core.
- Explicit frame-ownership transfer protocol with prepare/accept/commit/abort.
- Remote free remains pending ownership until acknowledged by the owning
  allocator. The freeing core cannot reuse a frame merely because it enqueued
  or attempted to enqueue a free.
- Full remote-free queues use bounded quarantine storage, retry, or explicit
  backpressure; they never silently drop a free.
- Every remote free carries frame incarnation, allocation generation, allocator
  owner incarnation, and transaction ID.
- Duplicate remote frees are rejected idempotently.
- Ordinary callers cannot consume emergency remote-free capacity.
- NUMA-aware refill requests and policy hooks where topology information is
  available.
- Bounded emergency allocation reserves for faults, quarantine, revocation, and
  audit-loss handling.
- Rule that a core never directly mutates another core allocator's metadata.
- Recovery policy for frames, heap slabs, and remote-free queues owned by a
  quarantined core.
- Allocator metadata owned by a quarantined-but-still-running core cannot be
  recovered until that core is hardware fenced/reset or the system halts.
- A dead owner's frames remain unavailable rather than guessed reclaimable from
  another core's observations.
- No frame reuse while remote references, pending frees, DMA mappings, TLB
  obligations, or owner-transfer transactions are live.

Verification:

- Host/model tests prove a frame cannot simultaneously belong to two
  allocators.
- Host/model tests prove remote-free delivery, duplicate remote-free, owner
  quarantine, and refill failure do not leak or double-own frames.
- Host/model tests prove full remote-free queues retain pending ownership and
  do not silently drop frees or allow the freeing core to reuse the frame.
- Host tests prove emergency reserves cannot be consumed by ordinary allocation
  paths.

Exit criteria:

- Shared-nothing memory ownership has a concrete allocator boundary before
  live AP queues and services rely on per-core state.

### v0.37.8 - Cache-Aware Atomic Fabric Queues

Goal:

Replace model-only core-to-core queue evidence with the hardware-ordering shape
required for live multicore endpoints.

Rationale:

The v0.36/v0.37 IPC smokes prove route validation, sequencing, and fail-closed
backpressure, but the queue implementation remains sequential model code with
plain indices. Live AP execution needs queues whose memory layout, ownership,
cache behavior, and atomic publication protocol are correct on weakly ordered
architectures as well as x86_64.

Deliverables:

- Loom or equivalent SPSC publication model before live AP-backed queue use.
- Hardware SPSC queue design that removes shared mutable `len` from producer
  and consumer hot paths.
- Monotonic producer and consumer cursors, each written by exactly one endpoint.
- Every queue endpoint names its exact writer execution context: task context,
  IRQ level, softirq/deferred worker, or NMI. "One core" is not enough to prove
  single-writer safety when task, IRQ, NMI, and panic paths can reenter one
  another.
- Ordinary task and IRQ producers cannot share a producer cursor without local
  serialization. Preferred design is per-context local staging feeding one
  canonical producer.
- NMI and machine-check paths never use ordinary fabric queues; they use fixed
  emergency records or a separately proven wait-free channel.
- Queue operations state whether interrupts and preemption are disabled, and
  for the maximum bounded duration.
- Reentrant enqueue/dequeue is detected before slot mutation.
- Panic paths cannot recursively enter a queue already owned by the interrupted
  context.
- Cached remote-cursor observations that are explicitly advisory and refreshed
  through acquire loads.
- Producer and consumer metadata separated onto distinct cache lines, with an
  option to place endpoint metadata on separate pages when permissions differ.
- Slot publication protocol:
  - producer writes payload;
  - producer scrubs or initializes authority-bearing padding;
  - producer performs a release store of slot sequence or tail;
  - consumer performs an acquire load before reading payload.
- Publication-to-doorbell ordering:
  - producer initializes payload;
  - producer release-publishes the slot;
  - producer executes the architecture-required barrier for APIC/MMIO/doorbell
    ordering;
  - producer rings the doorbell or sends the IPI.
- Consumer no-lost-wakeup handshake:
  - consumer marks the endpoint armed/sleeping;
  - consumer rechecks the queue with acquire semantics;
  - consumer sleeps only if the queue is still empty;
  - producer publishes first, then observes or clears the armed state and sends
    a wakeup when required.
- Doorbells are hints; queue state is authoritative. Lost, duplicated,
  coalesced, or early doorbells must not lose messages.
- Reliable notification progress contract:
  - either a persistent pending bit remains set until the receiver acknowledges
    the observed work;
  - or the producer retransmits after a bounded local timeout;
  - or a receiver watchdog/periodic timer polls pending inbound summaries;
  - or a platform level-triggered notification source is used;
  - or deep idle is unavailable for that endpoint/core while no reliable wake
    source exists.
- Notification acknowledgement is bound to doorbell generation, link
  incarnation, receiver core incarnation, and the observed producer cursor, so
  an acknowledgement for old work cannot clear newer pending work.
- Notification properties are documented separately:
  - safety: duplicated, stale, coalesced, or early notifications never duplicate
    message consumption or authorize stale payload reuse;
  - liveness: under stated scheduler and hardware fairness assumptions, a
    published message is eventually observed even if the first doorbell/IPI is
    lost.
- An IPI is not acknowledged before the consumer has made the corresponding
  queue work observable to its dispatcher.
- MMIO/APIC doorbell ordering uses architecture-specific barriers; Rust
  atomic ordering alone does not order device writes.
- Reverse slot-reuse edge:
  - consumer finishes reading payload;
  - consumer release-stores acknowledgement or cursor advancement;
  - producer acquire-loads that acknowledgement;
  - producer may then scrub and reuse the slot.
- Named linearization points for enqueue, dequeue completion, cancellation,
  acknowledgement, and slot reuse.
- Slot reuse protocol with generation or sequence numbers so wraparound cannot
  expose stale payloads.
- Mandatory zero/scrub-on-vacate policy before a slot can be observed by a
  different trust domain.
- Doorbell bitmap or equivalent pending-link summary so each core does not have
  to poll every inbound link at high core counts.
- IPI coalescing and batch receive/send policy.
- Queue placement policy for NUMA-local allocation and traffic-class
  separation.
- Separate traffic classes for authority-critical revoke/topology messages,
  best-effort telemetry, and ordinary service requests. Revocation and topology
  control must not sit behind best-effort telemetry in the same FIFO.
- Traffic class is selected from endpoint/protocol capability and kernel-stamped
  metadata, not from an untrusted message field. Reserved control capacity and
  rate limits are required even for authority-critical endpoints.
- Bounded scheduling rule that prevents telemetry floods from starving control
  traffic and control floods from permanently starving ordinary service
  traffic.
- Per-principal/service credits, deadlines, retry budgets, cancellation, and
  dead-letter records for noisy or stalled peers.
- Sparse link creation. Aesynx must not preallocate every pairwise queue at high
  core counts when a route is never used.
- Direct-link threshold policy that names when pairwise links are preferred and
  when cluster-local or NUMA-local routers are required.
- Dedicated direct links or reserved traffic class for revocation and topology
  control messages.
- Sharded or hierarchical doorbell bitmaps. A single many-writer bitmap must be
  treated as a cache-coherency hotspot unless measurement proves otherwise.
- Stamped endpoint/link identity on every live AP-backed message:
  - boot or machine-session nonce;
  - topology epoch;
  - sender and receiver logical core IDs;
  - sender and receiver core incarnations;
  - endpoint incarnation;
  - link generation;
  - protocol version and required extension set.
  Mismatches are rejected before payload parsing or capability interpretation.
- Quantitative targets for bytes per core pair, messages per second, p99
  latency, IPIs per message, and cache-line invalidations per message.
- Shared queue page ownership policy:
  - producer owns and writes payload and slot-publication pages;
  - consumer maps producer pages read-only;
  - consumer owns and writes acknowledgement/cursor pages;
  - producer scrubs payload storage only after observing consumption and before
    reuse.
- Implementation boundary decision: either fixed-width wire frames encoded
  entirely through atomics, or a tiny audited queue-storage unsafe island with a
  local safety proof, Miri/model wrappers, and no general unsafe exposure from
  `aesynx-ipc`.
- Explicit memory-ordering tests and model checks for x86_64, aarch64, and
  RISC-V assumptions. Release/acquire evidence must correspond to actual atomic
  stores/loads, not metadata fields.
- AP restart and hotplug are explicitly prohibited in this milestone. Any
  timed-out or failed AP stays permanently quarantined until reboot; reusable
  core identities and authority-bearing live messages remain blocked until
  v0.37.11 incarnation fencing exists.

Verification:

- Host model tests prove full, empty, wraparound, stale-slot, and retry cases
  fail closed without payload reuse.
- Host tests prove stale topology epochs, core incarnations, endpoint
  incarnations, link generations, boot/session nonces, and protocol versions
  are rejected before payload parsing.
- Host/model tests cover task-to-IRQ, IRQ-to-NMI, and panic-during-enqueue
  interleavings and prove the single-writer invariant still holds.
- Host/model tests cover publication while a consumer arms sleep and consumer
  drain while a producer decides whether to send an IPI.
- Loom/Kani-style or equivalent bounded model tests prove the SPSC publication
  protocol does not permit payload reads before release publication or payload
  scrubbing before consumer acknowledgement is observed.
- Cache-line layout tests prove producer and consumer hot metadata do not share
  a cache line.
- QEMU live-AP smoke proves producer and consumer run concurrently on different
  APs.
- QEMU live-AP smoke exercises actual IPI or doorbell delivery, duplicate
  doorbells, delayed doorbells, coalesced doorbells, queue-full behavior while
  the receiver is descheduled, and AP quarantine/termination while messages are
  pending.
- QEMU/model tests cover lost first doorbell, stale acknowledgement, pending-bit
  persistence, retransmission/watchdog wakeup, deep-idle denial when no reliable
  wake source exists, and fairness assumptions for eventual observation.
- QEMU correctness smokes cover 2, 4, and 8 virtual CPUs. 16-core and 32-core
  runs are scaling benchmarks when the host can provide them; lower-capacity
  hosts must run the largest safe configured count and report the cap
  explicitly.

Exit criteria:

- Aesynx has a queue implementation proven by real concurrent AP execution and
  actual doorbell/IPI delivery, not only by model `Ordering` evidence.

### v0.37.8.1 - Fabric Link Lifecycle And Deadlock Rules

Goal:

Make sparse link creation safe to tear down and reuse, and prevent distributed
protocol waits from becoming hidden lock cycles.

Deliverables:

- Fabric link lifecycle state machine:
  - `Absent`;
  - `Creating`;
  - `Active`;
  - `Draining`;
  - `Retired`;
  - `Reclaimable`.
- Link teardown protocol:
  - stop new publication;
  - drain or cancel outstanding messages;
  - retire doorbell/vector identity;
  - invalidate endpoint and link generations;
  - wait for producer and consumer acknowledgement;
  - unmap shared queue pages and complete required TLB invalidation;
  - scrub payload pages;
  - return pages only after stale endpoints are fenced.
- Delayed IPI, acknowledgement, or completion messages cannot bind to a reused
  link generation.
- Logical quarantine is not hardware fencing. A still-running AP is not treated
  as harmless merely because routing tables label it quarantined; protocols
  must either prove it cannot execute stale authority, fence/reset it, or halt.
- Distributed no-deadlock rules:
  - never wait synchronously on another core while holding a kernel lock or
    mutable owner-state guard;
  - no nested authority transaction unless protocol ranks explicitly allow it;
  - every blocking protocol names its rank or acyclic dependency edge;
  - callbacks cannot synchronously reenter the waiting coordinator;
  - cancellation and revocation traffic cannot require resources held by the
    operation being cancelled.

Verification:

- Host/model tests cover each link lifecycle transition and reject invalid
  skip transitions.
- Fault-injection tests cover delayed IPI/acknowledgement after link retirement
  and prove the old message cannot affect a reused link.
- Model tests cover cyclic waits among grant, mapping, scheduler, allocator,
  and revocation protocols and prove they fail closed or follow an acyclic
  rank.

Exit criteria:

- Fabric links can be retired, reclaimed, and recreated without stale messages
  or distributed wait cycles resurrecting authority.

### v0.37.9 - Strong Revocation And Mapping Invalidation Semantics

Goal:

Define and model the difference between prospective revocation and strong
revocation before mappings, DMA, or in-flight endpoint operations can carry
real authority across domains.

Rationale:

Incrementing a revocation epoch is enough for later live checks to fail, but it
does not automatically remove receiver table entries, tear down mappings, flush
remote TLBs, cancel DMA, cancel in-flight IPC, or prove every core has observed
the new epoch. Aesynx needs an explicit revoke contract before shared memory,
driver DMA, or cross-core delegation becomes live.

Implementation slicing:

This milestone is too large to implement as one security change. It is split
into reviewable units:

1. Prospective revoke and operation fences.
2. Lineage and selective revocation.
3. Memory-object pin/freeze/reclaim lifecycle.
4. TLB invalidation, residency barriers, and address-space activation permits.
5. DMA/device revoke integration.
6. Failure recovery and strong-revoke completion semantics.

Deliverables:

- TLA+ or Quint model for prospective revoke, strong revoke linearization,
  coordinator death, participant timeout, and recovery before strong revocation
  can guard live shared mappings or DMA.
- Two documented revoke classes:
  - Prospective revoke: no operation beginning after the revoke linearization
    point may succeed.
  - Strong revoke: when revoke returns, no stale operation, mapping, DMA
    request, delegated entry, or in-flight endpoint operation can still commit.
- Revocation messages carry object incarnation, previous epoch, new epoch,
  transaction ID, reason code, coordinator proof, and affected authority class.
- Selective revocation scope carried by every strong-revoke transaction:
  - revoke one table entry;
  - revoke a delegation subtree;
  - revoke a revocation domain;
  - revoke the whole object.
- Capability derivation/lineage identity for every delegated authority.
- Lineage node identity contains object incarnation, lineage generation, parent
  lineage reference, revocation-domain reference, and bounded child count.
- Maximum derivation depth and maximum children per lineage node are explicit
  release constants or policy values; exhaustion fails closed.
- Ancestry checks are either O(1) through an index, or a bounded walk with a
  documented maximum. Unbounded graph walks are not allowed in enforcement
  paths.
- Object-wide epochs and lineage-specific epochs are both part of live
  validation. Object-wide revocation dominates lineage-specific liveness.
- Mapping records, DMA records, leases, queued operations, and pending grants
  are indexed by lineage so selective revocation can find every affected use.
- Revoke-one either preserves descendants through an explicit promoted lineage
  rule or rejects while descendants exist; the chosen policy is documented and
  tested.
- Retired lineage nodes are not reused until generation retirement proves stale
  descendants cannot bind to the recycled node.
- Rules for whether descendants may create independent revocation domains and
  which principals may choose each revocation scope.
- Bounded lineage reclamation so selective revocation does not require
  unbounded in-kernel graphs.
- Distributed prepare/freeze/ack/commit/abort model for strong revocation.
- Bounded pending transaction counts per object, capability table, endpoint,
  principal, and coordinator.
- Sequence-number widths, retirement thresholds, and wraparound retirement
  rules for revocation messages and acknowledgements.
- Lease-freezing contract that states whether a freeze path is wait-free,
  bounded-spin, or scheduler-assisted; strong revoke cannot wait forever for a
  malicious holder.
- Mapping teardown protocol that unmaps every affected address space before
  strong revoke commit.
- Memory-object lifecycle uses independent axes rather than overloading
  "frozen" for both immutable sharing and revocation:
  - Mutability: `Mutable | SealedReadOnly`.
  - Authority: `Live | Revoking | Revoked`.
  - Residency: `Mapped | Unmapping | Reclaimable | Dead`.
  - Resource state: checked pin/reference counters plus pending invalidation
    records.
- Sealing a shared buffer read-only does not imply revocation, and entering
  revocation does not imply the object is a reusable immutable artifact.
- Owner-core pin acquisition protocol:
  - the object's owner core owns lifecycle transitions and pin accounting;
  - new pins are acquired only while authority is `Live`;
  - pin acquisition is atomic relative to entering `Revoking`;
  - usable reference implies owner-recorded live pin;
  - the owner atomically validates `Live` plus epoch and installs a `PinLease`
    before any reference becomes usable, or publishes only a
    `PendingReference` that cannot be consumed until a final owner-authorized
    commit converts it into an active reference;
  - publishing a usable reference before revalidation is prohibited because
    rollback cannot undo another core consuming that reference;
  - entering `Revoking` prevents every new mapping, DMA binding, lease, and
    cross-core reference;
  - counters use checked non-wrapping arithmetic;
  - remote pins are explicit owner-recorded references, not globally modified
    shared refcounts.
- Legal memory-state table for the independent axes:
  - `Authority::Live` cannot coexist with `Residency::Dead`;
  - new mappings require `Authority::Live`;
  - `Residency::Reclaimable` requires zero pins and no pending invalidation
    records;
  - `Residency::Dead` is terminal except through a newly minted object
    incarnation;
  - sealing is monotonic unless a separately authorized copy-on-write
    operation creates a new object;
  - physical reclamation must not implicitly change authority or mutability
    state.
- Backing frames remain pinned while referenced by any installed mapping,
  pending TLB invalidation, DMA/IOMMU mapping, checked operation or in-flight
  lease, shared queue, IPC transaction, page-table edit operation, executable
  transition, snapshot, or persistent object reference.
- Logical persistence is distinct from physical pinning. A snapshot or
  persistent reference copies/seals content or explicitly owns a bounded
  residency pin; it does not silently keep frames resident forever.
- Reclamation occurs only after every reference class is drained, all required
  remote acknowledgements complete, and the next owner cannot observe previous
  contents. Frame zeroing occurs before reuse after stale observers are fenced,
  not as a substitute for fencing stale observers.
- Mandatory local and remote TLB invalidation acknowledgements before a
  permission reduction or unmap is reported complete.
- TLB shootdown acknowledgements bind address-space incarnation, ASID/PCID and
  reuse generation, mapping generation, virtual range, operation class,
  target-core incarnation, and revocation transaction ID.
- A TLB acknowledgement is emitted only after the invalidation instruction and
  required architectural serialization have completed.
- DMA quiesce/cancel/drain requirement before strong revocation of device-visible
  memory.
- In-flight IPC cancellation or replay policy for operations authorized before
  the revoke linearization point.
- Operation-class side-effect table. For every revocable operation class,
  define:
  - authorization linearization point;
  - side-effect commit point;
  - whether cancellation is possible;
  - drain or completion evidence;
  - what strong revoke promises about effects already committed;
  - whether rollback is meaningful;
  - whether the only safe response is device reset, quarantine, or halt.
- Explicit non-claim: strong revoke prevents future effects and drains or
  fences previously accepted work, but it cannot undo an already observed
  external side effect such as a transmitted packet or completed device write.
- Failure handling for dead cores or domains: timeout leads to quarantine,
  degraded fail-closed state, or system halt depending on authority class.
- Audit records linking proposal, freeze, acknowledgement, TLB/DMA cleanup,
  commit, abort, and timeout.
- Revocation, quarantine, and permission reduction must not be blocked solely
  by normal audit-buffer exhaustion. These paths reserve emergency audit
  capacity and, if even emergency evidence cannot be retained, set a sticky
  audit-loss digest or halt after authority is removed.
- NMI and machine-check behavior for page-table edit windows, TLB shootdown,
  lease freezing, and lock-rank enforcement.
- Redacted diagnostics that expose counts, classes, and reason codes without
  raw object IDs, physical frames, or table slots.

Verification:

- Host model tests prove prospective revoke rejects every operation starting
  after the linearization point.
- Host model tests prove stale local epoch-cache authorization fails during
  prospective revoke unless backed by a current owner-issued lease or an
  installed revocation fence.
- Host model tests prove strong revoke cannot complete until modeled mappings,
  TLB acknowledgements, DMA ownership, and pending grants are resolved.
- Host/model tests prove each operation class follows its side-effect table and
  never reports rollback for an already externally observed effect.
- Host model tests prove revoke-one, revoke-subtree, revoke-domain, and
  revoke-object scopes invalidate exactly the intended descendants without
  leaving stale authority live.
- Host model tests include lineage-node reuse, generation exhaustion, object
  versus lineage epoch interaction, maximum-depth exhaustion, maximum-children
  exhaustion, and revoke-one behavior when descendants exist.
- Host model tests prove frames cannot enter `Reclaimable` while any mapping,
  TLB obligation, DMA mapping, lease, queue, transaction, page-table edit,
  executable transition, snapshot, or persistent reference is still live.
- Host model tests prove usable reference publication is impossible without an
  owner-recorded live pin, pending references cannot be consumed before commit,
  pin acquisition races with `Revoking` fail closed, rollback releases partial
  pins, checked counters never wrap, and remote pins are visible to the
  owner-core lifecycle record.
- Host tests prove illegal lifecycle cross-products are rejected, including
  `Live + Dead`, new mappings while not `Live`, `Reclaimable` with live pins or
  invalidation records, reuse of `Dead` without a new incarnation, and
  reclamation that changes authority or mutability state by side effect.
- Timeout tests prove dead participants cannot let stale authority remain
  silently usable.
- Coordinator-death tests prove pending strong-revoke transactions recover,
  abort, or quarantine according to a bounded rule.
- Tests prove quarantine of a dead participant is sufficient to complete strong
  revocation only for authority classes whose mappings, DMA, and endpoint
  operations have been fenced or made unreachable.
- Tests prove an unresponsive-but-still-running core holding a stale TLB entry
  prevents strong-revoke success unless the core is demonstrably hardware-reset,
  fenced from execution, or the system halts.
- Mapper tests prove permission reduction/unmap cannot be acknowledged until
  the required flush obligation is consumed.

Exit criteria:

- Revocation semantics are precise enough for live shared memory, driver DMA,
  and cross-core capability transfer to build on them.

### v0.37.10 - Kernel-Owned Shared Mapping Infrastructure

Goal:

Enable live shared-buffer mapping infrastructure inside kernel-owned address
spaces only after authority identity, atomic fabric queues, and strong
revocation semantics exist. Hostile user-domain shared mappings are deliberately
deferred until real isolated user address spaces and ring-3 execution exist.

Deliverables:

- Shared-buffer object descriptors from v0.37.2 are connected to the live
  mapper through checked memory-object plus kernel-address-space authority
  proofs.
- At least two real hardware page-table roots for kernel/test domains.
- Real CR3 switching between those roots in QEMU, with the selected root
  validated before install.
- Real TLB shootdown path for permission reduction and unmap, consuming the
  v0.37.9 acknowledgement contract.
- Address-space residency barrier for permission reduction and unmap:
  - track which cores have run each address space for the relevant mapping
    generation;
  - prevent new activation or migration into the address space;
  - snapshot the resident-core set;
  - apply page-table changes and shootdowns;
  - wait for every required acknowledgement;
  - only then permit activation again or release affected frames.
- Non-forgeable address-space activation permit:
  - contains address-space incarnation, mapping generation, and residency
    epoch;
  - context switching consumes or validates the permit;
  - freezing the address space invalidates outstanding permits before the
    shootdown target set is captured;
  - scheduler and mapper code cannot bypass each other by consulting advisory
    state directly.
- Initial x86_64 TLB mode before a PCID allocator exists:
  - PCID disabled;
  - PCID/ASID field fixed to zero in acknowledgement metadata;
  - CR3 switch performs the required full non-global flush;
  - permission reductions use `invlpg` or a full-context flush according to
    the affected range;
  - CR4.PGE stays disabled because reloading CR3 does not invalidate global
    translations;
  - global mappings are enabled only after a tested global-invalidation
    protocol exists;
  - enabling PCID/INVPCID is blocked on a later tagged-TLB milestone with
    allocator, reuse generation, rollover, and invalidation tests.
- Read-only shared mappings require `Mutability::SealedReadOnly` and
  `Authority::Live`. A revocation-frozen object is not accepted as a normal
  shared immutable object.
- Writable shared mappings require `SHARE_WRITE`, a declared synchronization
  protocol, traffic-class policy, audit evidence, and strong-revocation support.
- All aliases of the same memory object obey global W^X. A frame or memory
  object cannot be writable in one address space while executable in another.
- Executable transition protocol:
  - freeze writable mappings;
  - complete local and remote TLB invalidation;
  - perform architecture-required instruction-cache synchronization;
  - seal the memory object;
  - create executable mappings.
- Cache-attribute consistency across every alias of the same memory object.
- Page-table pages are protected from user mappings, DMA, and ordinary kernel
  writes after installation.
- Protected page-table update mechanism after installation: either a narrow
  owner-core edit window with hardening and audit evidence, or a dedicated
  temporary mapping protocol that restores page-table pages to protected state
  before returning.
- Live unmap/revoke of a shared buffer consumes the v0.37.9 strong-revocation
  path and cannot return success until mapping teardown and required
  invalidation acknowledgements complete.

Verification:

- QEMU smoke maps a sealed read-only shared buffer into two real kernel/test
  page-table roots and proves reads observe the same backing object after real
  CR3 switches.
- QEMU smoke proves permission reduction or unmap requires real TLB invalidation
  completion before success is reported.
- QEMU or host tests prove the initial no-PCID path records PCID zero, performs
  a full non-global flush on CR3 switch, and keeps CR4.PGE/global mappings
  disabled until the later global-invalidation protocol exists.
- QEMU or host tests prove a core cannot newly activate or migrate into an
  address space between shootdown target-set calculation and permission
  reduction/unmap completion.
- Context-switch tests prove address-space activation requires a valid
  activation permit and freezing an address space invalidates outstanding
  permits before the shootdown target set is captured.
- Host and QEMU tests reject writable/executable aliases across kernel/test
  address spaces.
- Host tests reject conflicting cache attributes for aliases.
- Strong revoke tests prove live shared mappings are torn down or the affected
  domains are quarantined before success is reported.

Exit criteria:

- Zero-copy shared assets are live through explicit capabilities between
  kernel-owned address spaces without weakening global W^X, cache-attribute
  consistency, or revocation semantics. No hostile user-domain shared mapping
  claim is made until the later post-ring-3 milestone.

### v0.37.11 - AP Incarnation, Restart, And Parameter Fencing

Goal:

Prevent stale AP arrivals, reused hardware IDs, or writable bootstrap
parameters from confusing the live multicore topology.

Deliverables:

- TLA+ or Quint model for AP restart, late arrival, duplicate hardware IDs,
  topology snapshot mismatch, and hotplug fencing before restart or hotplug can
  interact with authority-bearing queues.
- Startup-attempt generation or boot incarnation for every AP launch attempt.
- Every AP arrival carries the exact startup attempt generation; late arrivals
  after timeout or restart are quarantined and cannot satisfy a later attempt.
- APIC-ID reuse and duplicate detection across discovery, startup, restart, and
  hotplug paths.
- Startup preflight binds the startup ticket, AP incarnation, expected CR3 or
  page-table root, trampoline page, boot-parameter page, guarded stack
  allocation, INIT/SIPI generation, APIC identity, arrival doorbell, and
  timer/calibration contract into one checked startup descriptor.
- Offline, failed, quarantined, restarted, and hotplug transition table entries.
- Topology snapshot epochs for routing, scheduler, and authority protocols.
- Per-core boot-parameter publication barriers before SIPI/entry and
  consumption barriers on AP entry.
- Bootstrap-owned writable AP parameter pages are revoked, zeroed, or sealed
  read-only after consumption.
- Restarted cores receive a new core/domain incarnation so old endpoint,
  capability-table, and routing messages cannot replay into the new instance.

Verification:

- Host tests cover stale late-arrival rejection, duplicate APIC ID rejection,
  restart incarnation changes, topology snapshot mismatch rejection, and
  bootstrap-parameter seal/zero policy.
- Host tests prove startup evidence cannot be reused with a different CR3,
  trampoline, parameter page, stack, INIT/SIPI generation, APIC identity,
  arrival doorbell, or timer/calibration contract.
- QEMU AP smoke proves timeout plus late-arrival model paths quarantine rather
  than accidentally marking a core online.
- If this milestone claims actual AP restart or hotplug, QEMU must exercise a
  real restart path and prove old queued messages, endpoint incarnations,
  capability-table IDs, and topology snapshots are rejected after restart. If
  QEMU or hardware cannot support that evidence yet, actual restart/hotplug
  remains prohibited and the milestone is model/fencing-only.

Exit criteria:

- AP startup and restart have incarnation/fencing semantics strong enough for
  live fabric queues and authority-bearing messages. Actual AP restart/hotplug
  is not enabled until real restart evidence exists.

### v0.37.12 - Formal Models And Fault-Injection Conformance

Goal:

Turn the multikernel proof targets into concrete executable artifacts before
drivers and userspace services depend on the fabric, capability, and revocation
protocols.

Deliverables:

- TLA+ or Quint models for:
  - transactional grant;
  - prospective revoke;
  - strong revoke linearization;
  - derived-object edge creation, promotion/detachment, traversal, and
    cascading revocation;
  - coordinator failure and recovery;
  - AP restart and late-arrival quarantine.
- Required safety properties:
  - no authority amplification;
  - no authority resurrection;
  - no split-brain commit;
  - no W+X alias;
  - no stale-core acceptance;
  - no usable orphan derived child;
  - no promotion-based authority amplification or revocation escape;
  - no cycle under concurrent edge transactions;
  - exactly one promotion/create outcome after failure recovery;
  - no child publication before local owner validation after transaction
    commit;
  - no use of multi-parent derived children before the future
    `ParentSetManifest` feature gate exists;
  - no authority retained by immutable provenance records;
  - no strong-revoke success while a cascade-bound child remains usable.
- Required bounded-liveness properties:
  - healthy grant and revoke transactions eventually commit or abort;
  - revocation traffic is not starved under telemetry floods;
  - coordinator restart converges to one final transaction result;
  - queues progress under explicit producer/consumer scheduling assumptions;
  - derived-edge revocation makes bounded progress when ordinary allocation,
    IPC, and best-effort journal pools are exhausted.
- Explicit fairness assumptions in every TLA+/Quint model.
- Kani, Verus, or equivalent bounded proof targets for:
  - permission attenuation;
  - range containment;
  - generation/epoch retirement and exhaustion behavior;
  - scheduler action validation and rejection.
- Pin/freeze/reclaim race model covering pin acquisition versus `Revoking`,
  remote owner-recorded pins, checked counter overflow, rollback, and
  transition to `Reclaimable`.
- Address-space activation during TLB shootdown model covering resident-core
  snapshots, activation barriers, migration denial, acknowledgement loss, and
  frame release after shootdown completion.
- Loom model for the SPSC publication protocol, including producer cursor,
  consumer cursor, cached remote cursor observations, slot sequence reuse, and
  scrub-before-reuse ordering.
- Architecture litmus tests for x86_64, aarch64, and RISC-V ordering
  assumptions. Loom evidence is not enough for MMIO, IPI, DMA, TLB, or cache
  maintenance ordering.
- Executing lock-rank and IRQ/NMI-context checking in debug/QEMU builds, not
  only documentation of the ranking policy.
- Differential and metamorphic BootInfo normalization tests in addition to the
  existing deterministic fuzz corpus.
- Cross-endian fabric golden vectors and decoder fuzzing for the fixed-width
  fabric ABI.
- Test-only fault-injection harnesses for:
  - dropped, duplicated, delayed, and reordered messages;
  - lost, duplicated, delayed, and coalesced IPIs or doorbells;
  - coordinator death;
  - AP late arrival after timeout;
  - TLB-ack loss;
  - full queues and backpressure storms;
  - epoch/generation exhaustion.
- Global W^X alias property tests proving a memory object or physical frame is
  never writable through one alias while executable through another.
- Refinement tests showing the executable Rust state machines agree with the
  formal transition models for grant, revoke, AP restart, queue publication,
  scheduler action validation, derived-object edge creation, edge promotion,
  single-parent publication, future parent-set publication if enabled,
  provenance recording, and edge revocation traversal.
- Negative refinement tests where a deliberately broken Rust transition and its
  model disagree.
- Release-gate checklist for the authority-bearing sequence:
  - real concurrency gate with Rust executing on APs at 2, 4, 8, and configured
    higher core counts, randomized delay/saturation, cache-miss/IPI/fairness
    measurements, and reserved-lane latency evidence;
  - memory-model gate with Loom queue wrap/reuse/cancel models, architecture
    litmus tests, and Miri over any audited unsafe queue-storage island;
  - capability gate with grant/revoke/restart/replay models proving no
    authority amplification, revocation closure, and no strong-revoke success
    before TLB/IOMMU/in-flight-operation acknowledgement;
  - lock gate with real path rank checks and injected audit-full, lock-poison,
    allocation-failure, and peer-timeout faults;
  - boot gate with coverage-guided differential fuzzing plus permutation,
    split/merge, overlap, truncation, alignment, and maximum-count metamorphic
    BootInfo properties;
  - paging gate with differential PTE walks, random map/unmap/protect, global
    W^X alias tests, and TLB-shootdown tests;
  - AI gate with finite-action validation, arbitrary advice fuzzing, evaluator
    fuel exhaustion, service hangs, stale responses, and epoch changes during
    commit;
  - telemetry gate proving permanently full advisory rings cannot change
    scheduling, IPC, revocation, or fault-handling progress.

Verification:

- `cargo xtask` or script targets run the selected model/proof/fault-injection
  suites locally with bounded defaults suitable for CI.
- Each formal model has at least one negative test or intentionally broken
  variant proving the property would catch a relevant bug.
- Golden-vector tests prove fabric messages decode identically on little-endian
  and big-endian host fixtures.
- Fault-injection tests prove injected loss, replay, timeout, and exhaustion
  paths fail closed or enter documented quarantine.
- Model checks prove pin/freeze/reclaim races and address-space activation
  during TLB shootdown cannot produce stale mappings, premature frame reuse, or
  missing acknowledgements.

Exit criteria:

- Aesynx has concrete model/proof/fault-injection evidence for the
  authority-bearing fabric before driver services and userspace domains depend
  on it.

### v0.37.13 - Tagged TLB And Restart Live-Evidence Gate

Goal:

Keep advanced TLB tagging and actual AP restart/hotplug behind explicit live
evidence instead of letting earlier model metadata become an implementation
claim.

Deliverables:

- PCID/INVPCID enablement remains disabled by default until this milestone or a
  later replacement provides:
  - PCID allocator;
  - per-address-space PCID reuse generation;
  - rollover and retirement policy;
  - INVPCID single-context and all-context invalidation plan;
  - interaction with global mappings and CR4.PGE;
  - host and QEMU tests for stale-PCID rejection.
- Live AP restart/hotplug remains disabled until this milestone or a later
  replacement provides:
  - QEMU or hardware restart path;
  - new core/domain incarnation after restart;
  - rejection of old queued messages, endpoints, capability-table IDs, and
    topology snapshots;
  - cleanup of outstanding reply capabilities, grants, leases, and control
    messages from the old incarnation.

Verification:

- PCID tests prove stale translations cannot survive PCID reuse, rollover, or
  missed invalidation.
- AP restart tests prove old authority-bearing messages from the previous
  incarnation are rejected after a real restart path.

Exit criteria:

- Aesynx may enable tagged TLBs or actual AP restart/hotplug only after these
  live-evidence gates pass.

### v0.37.14 - Bare-Metal Multicore Evidence Gate

Goal:

Separate deterministic QEMU/KVM evidence from production multicore coherence
claims.

Deliverables:

- Bare-metal x86_64 SMP queue stress with actual AP execution.
- Bare-metal TLB-shootdown stress covering permission reduction, unmap, and
  address-space activation denial.
- KVM and TCG QEMU runs with distinct documented expectations.
- Hardware performance-counter collection for cache-line bouncing, IPI cost,
  queue saturation, and reserved-lane latency where the platform exposes it.
- Multi-socket or NUMA testing when available; otherwise the test report states
  the topology limit.
- Physical AArch64 weak-memory testing before the aarch64 backend claims live
  AP-backed fabric correctness.
- Long-duration wraparound, saturation, and core-offline/quarantine stress.
- Clear non-claim that QEMU 16/32-core scaling is useful evidence, but not the
  final cache-coherence, NUMA, SMT-interference, or weak-memory proof.

Verification:

- Bare-metal report records CPU model, core/thread count, memory topology,
  firmware mode, AP count, test duration, wrap counters, IPI counts, and
  failure/quarantine events.
- Stress tests prove control/revocation lanes keep bounded latency while
  best-effort traffic saturates queues.
- Any platform that cannot expose performance counters or NUMA data reports
  that limitation rather than silently claiming coverage.

Exit criteria:

- Aesynx has real-hardware multicore evidence before making production
  multikernel concurrency claims.

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
- Documented split-out triggers for moving QEMU/virtio drivers to future
  `aesynx/drivers-qemu` or `aesynx/drivers-virtio` repositories once the driver
  ABI, manifests, capability grants, package flow, and CI contract are stable.
- Documented repository evolution option where this repository may become
  `aesynx/kernel` or `aesynx/multikernel` under a future `aesynx/`
  organization.

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

### v0.41.1 - Directed IRQ Ownership And Transfer

Goal:

Make interrupt ownership explicit before real driver services depend on
directed IRQ routing.

Deliverables:

- Vector and interrupt-source incarnations.
- Exactly one owner core/domain for each routable interrupt source.
- IRQ transfer protocol:
  - mask source;
  - drain in-service state;
  - clear or acknowledge device-level interrupt state in documented order;
  - reroute or remap vector/MSI/MSI-X/interrupt-remapping entry;
  - publish new owner incarnation;
  - unmask only after ownership and routing evidence is visible.
- Correct EOI ordering for edge-triggered and level-triggered interrupts.
- Handling for level-triggered lines that remain asserted.
- Stale interrupt rejection after driver, core, or source incarnation restart.
- MSI/MSI-X and interrupt-remapping teardown rules before device/domain
  revocation.
- Storm rate limiting, quarantine, and escalation policy.
- Explicit special-case policy for NMI, machine-check, timer, and non-routable
  interrupts.

Verification:

- Host model tests prove an interrupt cannot be owned by two domains.
- Host tests prove stale vector/source incarnations are rejected after transfer
  or restart.
- QEMU smoke or model test proves mask/drain/remap/unmask ordering is enforced
  before a fake driver receives IRQ authority.

Exit criteria:

- IRQ delivery is an owned, transferable capability path rather than ambient
  hardware state.

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

### v0.43.1 - Virtio Serial

Goal:

Add a structured QEMU communication channel that is separate from bootstrap
COM1 serial logging.

Deliverables:

- Virtio serial device recognition through the chosen virtio transport.
- One control/data port model.
- Capability-scoped console or diagnostic endpoint.
- Bounded RX/TX queues.
- Clear policy distinction between bootstrap COM1 logs and virtio-serial
  service traffic.
- Redacted diagnostic output for port identity and queue state.

Expected serial:

```text
[TEST] virtio-serial=ok
```

Verification:

- QEMU virtio-serial smoke sends and receives one bounded message.
- Full/empty queue behavior fails closed without corrupting queue state.

Exit criteria:

- Aesynx has a non-legacy virtual serial service path for QEMU diagnostics,
  shell transport experiments, and later host tooling.

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

### v0.44.1 - Virtio GPU Display Baseline

Goal:

Add a QEMU-first graphics device path without pretending to have a full desktop
graphics stack.

Deliverables:

- Virtio GPU device recognition through the chosen virtio transport.
- Basic resource creation and framebuffer scanout path.
- Capability-scoped display surface object.
- Fallback policy to bootloader framebuffer when virtio-gpu is absent.
- Explicit non-goal for 3D acceleration, shader execution, audio/video sync,
  compositor protocols, or vendor GPU stacks.

Expected serial:

```text
[TEST] virtio-gpu=ok
```

Verification:

- QEMU smoke creates a basic display resource or reports the expected fallback.
- Driver diagnostics expose dimensions and format while redacting raw backing
  addresses.

Exit criteria:

- Aesynx has a planned QEMU display-driver path beyond the bootloader
  framebuffer wrapper.

### v0.44.2 - QEMU Local Input Baseline

Goal:

Provide a simple local keyboard/mouse path for QEMU without requiring the full
USB stack first.

Deliverables:

- PS/2 i8042 discovery/classification for QEMU local input, or an explicitly
  documented decision to keep local input serial-only for this milestone.
- Basic keyboard scancode input path if PS/2 is enabled.
- Optional PS/2 mouse packet classification for later graphical input.
- Capability-scoped input endpoint.
- Clear priority rule: serial and virtio-serial remain the first CI-friendly
  input paths; PS/2 is for local QEMU interaction; USB HID is later through
  xHCI.
- Redacted input diagnostics that report device state without logging raw user
  keystreams by default.

Expected serial:

```text
[TEST] qemu-input=ok
```

Verification:

- QEMU smoke or host model test proves keyboard input can be classified without
  granting broad port/IRQ authority.
- Input queues reject overflow without leaking stale input bytes.

Exit criteria:

- Aesynx has a planned local QEMU keyboard path that does not depend on USB.

### v0.44.3 - USB Roadmap And xHCI Discovery Stub

Goal:

Prepare USB support without pulling a large USB stack into the first virtio
driver path.

Deliverables:

- xHCI controller discovery model.
- USB device/class roadmap covering HID, mass storage, and serial adapters.
- Capability model for controller MMIO, IRQ, DMA rings, ports, and attached
  devices.
- Explicit statement that early QEMU storage uses virtio-blk; USB mass storage
  is later.
- Explicit statement that early QEMU input/diagnostics use serial,
  virtio-serial, and optionally PS/2; USB HID is later.

Verification:

- Host tests classify synthetic xHCI controller and USB class descriptors.
- No USB driver receives broad DMA or all-port authority by default.

Exit criteria:

- Reading from USB is planned through xHCI plus USB mass-storage class support,
  but not confused with the first QEMU persistence milestone.

### v0.44.4 - Usercopy And User Memory Access Discipline

Goal:

Define how the kernel may touch user memory before ring 3 and the native ABI
make user pointers real inputs.

Deliverables:

- Checked user-memory accessor API for copy-in, copy-out, and bounded string or
  byte-slice reads.
- Copy-then-validate rule for syscall arguments that must not be re-read from
  user memory after validation.
- Page-table permission checks before every user-memory access.
- SMAP `stac`/`clac` access-window design for x86_64 when SMAP is enabled.
- TOCTOU guidance for shared service queues and memory objects.
- Fault containment path for failed user copies.
- All-or-nothing hostile user-copy semantics:
  - destination snapshots are initialized before copying;
  - a partial fault returns `CopyResult::Incomplete`;
  - partial bytes are discarded and never parsed;
  - kernel shadow cursors do not advance;
  - no authority check, endpoint action, or completion is produced from an
    incomplete snapshot.
- Completion copy-out failure keeps a kernel-owned pending completion and
  either retries or cancels according to endpoint policy. The user slot is not
  acknowledged or reused until the completion is delivered or explicitly
  cancelled.
- Bounded ingress work budget per principal, endpoint, and doorbell so
  repeated maximum-size copies or deliberate copy faults cannot become an
  unbounded kernel CPU-denial path.
- Redacted audit events for rejected user memory access.

Expected serial:

```text
[TEST] usercopy=ok
```

Verification:

- Host tests cover valid copy, invalid pointer, cross-page copy, noncanonical
  pointer, overflow, unmapped page, and permission mismatch cases.
- Host tests cover partial copy-in faults, incomplete-snapshot discard, failed
  completion copy-out, no shadow-cursor advance on failure, and bounded ingress
  budget exhaustion from invalid submissions.
- QEMU smoke exercises at least one rejected user-memory access without
  corrupting kernel state.

Exit criteria:

- The kernel has one reviewed path for user memory access before userspace can
  pass pointers into kernel services.

### v0.44.5 - Domain Transition And Speculation Hardening

Goal:

Implement or fail-closed gate the CPU and address-space hardening required
before ring 3, mutually distrusting domains, or context switches can be treated
as a real security boundary.

Deliverables:

- Correct x86_64 CPUID feature detection for Intel and AMD speculative controls,
  including AMD extended-leaf IBRS bit 14, IBPB bit 12, STIBP bit 15, and SSBD
  bit 24.
- `IA32_ARCH_CAPABILITIES` field decoding for eIBRS/IBRS_ALL, RDCL_NO,
  MDS_NO, TAA_NO, RSBA, and newer vendor-documented controls where available.
- Redacted CPU family, model, stepping, vendor, and microcode-revision evidence
  for the BSP and every AP that will execute hostile-domain code.
- Minimum-revision or vulnerability-policy table for supported production
  profiles, with a QEMU/general profile that remains honest about missing
  evidence.
- Consistent acceptable mitigation state across every active core before
  hostile userspace is entered. A late AP with weaker or unverifiable mitigation
  state is quarantined before it can run hostile-domain work.
- Mixed-vendor, mixed-stepping, or mixed-mitigation topology policy.
- Re-evaluation of CPUID and relevant MSRs after any accepted microcode update.
- Explicit ownership of microcode updates: firmware/bootloader-owned,
  Aesynx-owned through an audited boot path, or unsupported. Raw microcode blobs
  from untrusted userspace services are never accepted.
- Fail-closed hostile-userspace policy when a selected mitigation depends on
  unavailable, inconsistent, or unverified microcode behavior.
- Implemented context-transition mitigation policy for switching between mutually
  distrusting domains:
  - when IBPB is required;
  - when STIBP is redundant or required;
  - when RSB stuffing or BHB/BHI mitigation is required;
  - when VERW-based MDS/RFDS buffer clearing is required;
  - when L1D flush policy is required.
- Explicit SMT-domain policy for high-assurance workloads, with a build or boot
  selector that fails closed if the requested SMT isolation cannot be enforced.
- KPTI or dual-root page-table implementation for processors affected by rogue
  data-cache load, or fail-closed refusal to enter hostile userspace on affected
  CPUs.
- L1TF hygiene policy: non-present PTEs are all-zero unless a reviewed
  exception exists, and physical page zero must never contain secrets.
- Hardened syscall/sysret or interrupt-return entry/exit assembly with per-core
  TSS/RSP0, swapgs fencing if used, contained user faults, and no reliance on
  compiler-generated prologues for critical transition assembly.
- Instruction-level verification for critical entry/exit assembly:
  - disassembly checks for required instructions and forbidden compiler-generated
    prologues on hand-written transition paths;
  - stack-alignment and frame-layout assertions shared between Rust `repr(C)`
    frame definitions and assembly offsets;
  - checked correspondence between assembly constants and Rust frame fields;
  - no accidental `SYSRET` path bypassing validation;
  - all return paths pass through the same final RFLAGS/address validation;
  - emulator/QEMU fault injection after each critical entry/exit phase;
  - NMI, double-fault, and machine-check injection at sensitive transition
    points where the architecture allows meaningful testing.
- Syscall/user-return invariants:
  - validate user RIP and RSP canonicality before return;
  - never execute unsafe `SYSRET`; use a validated fast path or fall back to
    `IRETQ`;
  - mask user-controlled RFLAGS including IOPL, NT, TF, AC, DF, RF, and
    reserved bits according to policy;
  - execute `CLD` on entry;
  - guarantee `CLAC` on every exit from usercopy;
  - apply the selected `SWAPGS` fencing strategy;
  - normalize exception frames with and without hardware error codes;
  - scrub kernel-sensitive scratch registers before user return;
  - prevent user-selected segment or compatibility-mode state unless
    explicitly supported;
  - handle faults during entry/exit through a dedicated tested failure path;
  - ensure NMI around `SWAPGS` cannot select the wrong per-core state.
- Full architectural state-switch implementation including SIMD/FPU ownership
  and XSAVE/XRSTOR state sanitization before
  SSE/AVX is enabled in kernel or user contexts.
- Trampoline or boot-order policy that enables compatible NX/WP/SMEP/SMAP/UMIP
  protections before untrusted code or APs can execute with the final CR3.
- Explicit boot hardening state machine:
  - validate CPUID and selected deployment baseline;
  - enable EFER.NXE and CR0.WP before switching to tables that rely on NX or
    supervisor write-protection;
  - activate the verified CR3 through an identity/current-stack-safe
    transition;
  - enable SMEP, SMAP, and UMIP after all required supervisor accesses are
    routed through controlled helpers;
  - assert final EFER/CR0/CR3/CR4/MSR state before accepting interrupts,
    starting AP Rust execution, or entering userspace.
- Redacted serial markers for supported, requested, applied, and deferred
  hardening controls.
- Per-core CR/MSR read-back for the bootstrap core and every executing AP.
- SMAP usercopy windows with guaranteed `clac` restoration on every normal,
  fault, and panic path that can exit the access window.
- CET shadow-stack/IBT policy. If CET is deferred, this milestone must document
  why the target CPU/QEMU profile can still enter userspace under the selected
  threat model.

Verification:

- Host tests cover Intel/AMD CPUID matrices, including the AMD IBRS bit-14
  regression and unrelated-bit rejection.
- Host tests cover `ARCH_CAPABILITIES` decode cases and strict/general policy
  selection.
- Host tests cover microcode-revision policy, mixed-core mitigation policy,
  post-update CPUID/MSR reevaluation, and late-AP quarantine for weaker
  mitigation evidence.
- QEMU smoke reports boolean hardening evidence without raw MSR values.
- QEMU or host tests prove hostile userspace entry is blocked when a required
  mitigation is selected but unavailable.
- Boot-state-machine tests prove NXE/WP precede NX-bearing table activation
  when required and SMEP/SMAP/UMIP are not enabled until their access-window
  and supervisor-access contracts exist.
- Hostile-register tests cover noncanonical user RIP/RSP, unusual RFLAGS,
  nested NMI around `SWAPGS`, usercopy fault, and return-path fault injection.
- Disassembly and frame-layout tests prove critical transition assembly uses
  the expected instructions, offsets, stack alignment, validation funnel, and no
  compiler-generated prologue/epilogue where prohibited.
- Fault-path tests prove SMAP access windows restore the access flag before
  returning or halting.
- Documentation states which mitigations are active, which are planned, and
  which are not relevant on the current QEMU CPU model.

Exit criteria:

- The ring-3 path either enforces the selected domain-transition hardening on
  every executing core or refuses to enter hostile userspace.

### v0.44.6 - Kernel PIE And KASLR Activation

Goal:

Turn the earlier KASLR/PIE planning work into a concrete implementation gate
before hostile userspace can rely on address-space randomization as
defense-in-depth.

Deliverables:

- PIE-capable kernel code-generation path for the selected x86_64 boot profile.
- Audited supported relocation-type list. Unknown, malformed, unsupported, or
  overflowing relocations fail closed before execution continues.
- Random virtual load bias selected from the approved DRBG path once v0.18.2
  has made `random_tokens_available=true`.
- Slide alignment and canonical-address constraints.
- Collision checks against direct map, framebuffer, boot modules, heap,
  activation/per-core stacks, page-table arenas, MMIO windows, ACPI/firmware
  windows, and reserved guard/null regions.
- BSP/AP agreement on the selected slide before APs can execute relocated
  kernel code.
- Exception, unwind, panic, symbol, and diagnostic handling under relocation.
- No raw slide leakage through serial telemetry, panic output, capability
  debug, page-table diagnostics, trace export, or World Service public facts.
- High-assurance failure policy when adequate entropy, relocation support, or
  collision-free address space is unavailable.
- Measured effective entropy after alignment, canonical-address, direct-map,
  framebuffer, module, MMIO, and reserved-window constraints.
- Clear statement that KASLR remains defense-in-depth and never substitutes for
  W^X, KPTI, SMEP/SMAP, usercopy discipline, or speculative mitigations.

Verification:

- Host tests reject unsupported relocation records, malformed relocation
  sections, relocation overflow, noncanonical slides, and collisions with every
  reserved region class.
- Tests prove slide selection fails closed when approved random tokens are
  unavailable under the selected deployment profile.
- QEMU smoke boots at multiple deterministic test slides and at least one
  entropy-derived slide when the DRBG gate is available.
- Diagnostic tests prove public output exposes only redacted slide-class
  evidence and effective entropy, not the raw slide.
- AP startup tests prove AP-visible bootstrap parameters and per-core metadata
  agree with the BSP-selected slide.

Exit criteria:

- Aesynx either boots from a PIE/KASLR kernel image with redacted evidence or
  refuses the deployment profile that requires KASLR.

### v0.44.7 - High-Assurance Side-Channel Isolation Profile

Goal:

Define an optional deployment profile for high-assurance workloads where
side-channel isolation has concrete hardware, scheduler, telemetry, and memory
placement rules instead of broad non-claims.

Deliverables:

- Same-security-domain SMT sibling policy, or SMT disabled for the profile.
- Security-domain-aware core placement and migration restrictions.
- Capability-gated high-resolution clocks and performance counters.
- Quantized and rate-limited telemetry for untrusted consumers.
- Cache/LLC partitioning through Intel CAT, ARM MPAM, or equivalent where
  available.
- Memory-bandwidth allocation or rate limiting where supported.
- Page coloring only if measured on the target hardware and included in the
  allocator policy, proof obligations, and documentation.
- Branch-predictor, TLB, and buffer-clearing mitigation policy tied to domain
  transitions.
- Shared-buffer, shared-cache, shared-core, SMT-sibling, and shared-device
  relationships recorded as deliberate covert-channel edges in diagnostics and
  world facts.
- Explicit noninterference claims limited to named hardware profiles, measured
  cache/topology evidence, and selected mitigations.
- Fail-closed behavior when the requested profile cannot enforce required
  isolation controls on the current CPU, firmware, or topology.

Verification:

- Host tests prove profile selection rejects incompatible SMT, clock,
  telemetry, placement, and partitioning configurations.
- QEMU/general profile documents non-claims instead of pretending to enforce
  unavailable hardware partitioning.
- Real-hardware tests, when available, record redacted CPU/topology evidence,
  selected partition controls, and denied profile reasons.
- Documentation names which covert-channel edges remain accepted risks for each
  profile.

Exit criteria:

- Aesynx has a named high-assurance side-channel profile with explicit
  enforcement requirements, fallback behavior, and non-claims.

## Phase 11: Native Userspace

### v0.45.0 - User Address Space

Goal:

Create isolated user memory.

Deliverables:

- User page tables.
- User text/data/stack mappings.
- Guard page.
- Shared service queue mapping with zero-on-vacate or equivalent payload
  scrubbing so stale inline payload bytes cannot be observed after pop.

Verification:

- Kernel validates mapping layout.

Exit criteria:

- User-mode entry can begin.

### v0.45.1 - Hostile Userspace Queue Ingress Contract

Goal:

Define the safe ingress boundary for userspace submission queues before the
kernel or service domains consume requests from hostile producers.

Rationale:

The internal kernel-to-kernel fabric can use a trusted-producer SPSC contract,
but userspace producers remain writable owners of their queue slots after
publication unless ownership is explicitly transferred. The kernel must not
validate a shared slot and then execute data reread from the same mutable
slot.

Deliverables:

- Published userspace slots are treated as untrusted bytes, never as
  `&Request` or any other safe borrowed structured request.
- User-controlled queue metadata is also untrusted: head, tail, slot index,
  length, flags, generation, acknowledgements, and completion cursors are
  observations only.
- Kernel-owned shadow cursors define trusted queue progress. User cursors can
  request work but cannot force reuse, rollback, skip ahead, or allocate.
- Cursor rollback, replay, jumps larger than queue capacity, integer overflow,
  and wraparound are rejected before mutation.
- Length fields are bounded against the fixed slot size before any parse or
  copy decision. The preferred path copies a fixed maximum-sized slot into
  initialized kernel storage and parses only the validated prefix.
- Exactly one fault-contained raw byte copy into kernel-owned or service-owned
  snapshot storage before parsing or authority validation.
- The owned bytes are treated as arbitrary attacker-controlled input, even if
  they were assembled while userspace was concurrently mutating the slot.
- Slot generation and publication state checks are cooperative race diagnostics
  only. They are useful for retry and telemetry, but they are not a security
  proof against a malicious producer that can modify payload bytes without
  changing the generation, restore the old generation, or create a torn but
  structurally valid request.
- If coherent publication is required for semantics rather than convenience,
  the operation uses kernel-controlled ownership transfer, write protection, or
  an authenticated immutable submission object.
- Authority-bearing fields are validated and executed only from the owned
  snapshot.
- Kernel stamps source identity, endpoint identity, traffic class, sequence,
  transaction ID, and receive deadline after copying.
- No authority-bearing field is reread from the shared slot after validation.
- Completions are built entirely in kernel-owned storage and copied out. A
  malicious completion acknowledgement cannot make the kernel reuse an
  out-of-range, stale, or unsafe slot.
- Optional expensive path for later: page or slot write ownership transfer via
  page protection, used only when the cost is justified.
- Runtime and ABI documentation distinguishes trusted internal fabric queues
  from hostile userspace ingress queues.

Verification:

- Host tests mutate a userspace slot during ingress copying and prove no mixed
  snapshot can bypass parsing, validation, authorization, or bounds checks.
- Host tests fuzz queue control metadata with arbitrary cursor values, maximum
  integers, wraparound, rollback, replay, out-of-capacity jumps, overlong
  lengths, and faults midway through copying a slot.
- Host tests prove malicious completion acknowledgements cannot cause unsafe
  slot reuse or out-of-range completion processing.
- Host tests prove generation changes before-copy, during-copy, and after-copy
  are reported as race diagnostics or retries, without being required for the
  security proof.
- Host tests prove a structurally valid mixed snapshot is still treated as
  attacker input and cannot gain authority through fields not present in the
  owned snapshot.
- Host tests prove traffic class and source identity come from kernel-stamped
  context, not from the user-provided payload.
- Static tests or lint-like checks reject APIs that expose published userspace
  queue payloads as safe references.

Exit criteria:

- Userspace queue ingress has a clear copy-validate-execute contract that avoids
  double-fetch, shared-reference hazards, and security dependence on
  user-controlled generation counters.

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

### v0.46.1 - Cooperative User-Domain Shared Mapping Proof

Goal:

Prove shared-buffer mapping mechanics between actual isolated user domains
after ring-3 execution exists. This is the user-domain counterpart to the
kernel-owned v0.37.10 shared-mapping infrastructure, but it is still a
cooperative isolation proof until v0.46.2 preemption and CPU-budget enforcement
exist.

Deliverables:

- Two isolated user address spaces with distinct task/domain incarnations.
- Shared-buffer capability grant from an owning domain to a second domain.
- Read-only sealed shared-buffer mapping into both domains.
- Writable shared-buffer mapping only with `SHARE_WRITE`, a named
  synchronization protocol, and audit evidence.
- Writable cross-domain memory is exposed only as atomic fields, volatile byte
  regions, or audited protocol-specific wrappers. No safe `&mut T` or aliased
  non-atomic `&T` is constructed over concurrently writable shared storage.
- Volatile access is not synchronization. Non-atomic conflicting writers remain
  forbidden unless exclusive ownership has been transferred.
- Each writable-sharing protocol names permitted access widths, alignment,
  atomic orderings, ownership transitions, and recovery behavior.
- Non-atomic structured payloads require exclusive ownership transfer before
  access.
- `aesynx-rt` or the userspace SDK exposes safe shared-ring and shared-atomic
  types rather than arbitrary mutable shared slices.
- Strong-revocation teardown of user-domain shared mappings through the v0.37.9
  protocol.
- TLB shootdown acknowledgements that bind user address-space incarnation,
  ASID/PCID reuse generation, mapping generation, virtual range, operation,
  target-core incarnation, and revocation transaction ID.
- User-domain W^X alias checks proving no memory object is writable in one
  address space while executable in another.
- Cache-attribute consistency checks across every user-domain alias.

Verification:

- QEMU smoke maps a sealed read-only buffer into two actual ring-3 domains and
  proves both observe the same backing object.
- QEMU smoke proves one domain cannot write through a read-only shared mapping.
- Host tests prove shared writable regions cannot be wrapped in safe aliased
  non-atomic references through the public runtime APIs.
- QEMU smoke proves strong revoke tears down the mapping or quarantines the
  affected domain before success is reported.
- Host and QEMU tests reject W+X and cache-attribute alias conflicts across
  user address spaces.

Exit criteria:

- Aesynx proves user-domain shared mapping mechanics, memory isolation, and
  revocation safety. It must not claim hostile-domain scheduling progress or
  denial-of-service containment until v0.46.2 preemption is active.

### v0.46.2 - Preemptive Timer And CPU-Budget Enforcement

Goal:

Prevent ordinary hostile user domains from monopolizing a core before multiple
untrusted processes and AI policy services depend on scheduler progress.

Deliverables:

- Timer-driven forced kernel entry from ring 3.
- Full architectural context save and restore for preempted user domains.
- Per-core scheduler ownership rule for preemption decisions.
- Quantum and CPU-budget accounting with checked decrement and exhaustion
  handling.
- Preemption-disable nesting API for short kernel critical sections.
- IRQ/preemption-safe run-queue mutation discipline.
- Watchdog termination or reset path for non-yielding domains.
- Redacted telemetry for preemption, budget exhaustion, watchdog reset, and
  failed context restoration.
- Kani, Verus, or equivalent proof target for `ValidatedScheduleAction` or its
  successor when scheduler actions become externally advised.

Verification:

- QEMU smoke runs an infinite-loop user task while another user task continues
  to receive CPU time.
- Host tests prove run-queue mutation fails closed when attempted from the
  wrong owner core or unsafe IRQ/preemption context.
- Host tests prove CPU-budget underflow and quantum accounting overflow are
  rejected without mutating scheduler state.
- Fault-path tests prove preemption-disable nesting restores its previous state
  on success and failure.

Exit criteria:

- Aesynx has a concrete preemption baseline before normal multi-process
  userspace or AI policy services can claim progress isolation.

### v0.46.2.1 - Scheduling Contexts And Budget Donation

Goal:

Define who pays for synchronous service work, kernel parsing, faults, and
cross-core request execution before untrusted domains can use CALL/REPLY as a
resource-amplification path.

Deliverables:

- Scheduling-context object bound to principal/domain incarnation, task
  incarnation, priority ceiling, and CPU-budget accounting.
- Scheduling-context authority uses the central typed-right matrix:
  `DONATE`, `SET_CEILING`, `CANCEL_DONATION`, and `INSPECT_BUDGET`.
  `SET_CEILING` is non-delegable by default, `INSPECT_BUDGET` exposes only
  redacted accounting state unless a richer debug right is also present, and
  budget donation never implies authority donation.
- Synchronous `CALL` may transfer a bounded CPU budget to the callee for that
  transaction.
- Donation carries caller, callee endpoint, transaction ID, priority ceiling,
  donation depth, and expiry.
- Server spends donated budget only on work for that transaction.
- Unused budget returns on reply, cancellation, timeout, or server death.
- Donation chains have a strict maximum depth.
- Effective priority cannot exceed the endpoint manifest ceiling.
- Asynchronous messages consume receiver-owned service budgets and
  per-principal request credits.
- Kernel work performed on behalf of a request, including copying, parsing,
  page faults, capability lookup, and validation, is charged or strictly
  bounded.
- IRQ work is charged to the device or service domain where meaningful; any
  uncharged interrupt work must have a fixed bound and storm policy.
- Cross-core priority inheritance and donation cannot create cyclic waits.

Verification:

- Model tests prove conservation of scheduling budget across call, reply,
  cancellation, timeout, and server death.
- Model tests prove donation depth and endpoint priority ceilings are enforced.
- Host tests prove expensive usercopy, parser rejection, page-fault handling,
  and capability lookup paths are charged or bounded before mutation.
- Model tests prove cyclic donation across endpoints is rejected or broken by a
  documented rule.

Exit criteria:

- CALL/REPLY and service execution have explicit budget ownership instead of
  letting clients amplify work through privileged services.

### v0.46.3 - Transactional Task Migration

Goal:

Move runnable or waiting tasks between owner cores without duplicating
execution authority or losing scheduler state.

Deliverables:

- Transactional migration state machine:
  - source running/owned;
  - source frozen and destination pending;
  - destination accepted;
  - owner-issued commit certificate or journal decision;
  - ownership commit;
  - destination runnable and source empty;
  - abort restores source ownership.
- Migration transaction identity bound to task incarnation, domain
  incarnation, source core incarnation, destination core incarnation,
  scheduler epoch, and topology epoch.
- Transferred state includes CPU registers, SIMD/FPU state, address-space
  activation or residency permit, pending timers and wakeups, IPC reply/wait
  state, CPU budget, scheduler accounting, affinity/security-domain
  constraints, and outstanding per-core references.
- Destination queue-full, destination death, stale epoch, affinity violation,
  budget violation, or failed address-space activation aborts without losing or
  duplicating the task.
- Destination cannot execute the task merely because it accepted prepare.
- Source death after destination prepare cannot let the destination infer
  commit without the owner-issued commit certificate or journal decision.
- Destination death after commit is handled through domain/core recovery, not
  by blindly restoring the source copy.
- If the final decision is ambiguous and no trusted witness survives, the safe
  result is quarantine or explicit task loss, not running two copies.
- Duplicate prepare, accept, commit, abort, and acknowledgement messages are
  idempotent.
- Timer wakeups and reply completions are incarnation-stamped so they cannot
  awaken both source and destination.
- Migration journal capacity is reserved separately from ordinary scheduling
  traffic.
- Migration of a currently running task occurs only after a verified
  architectural quiescence point.
- Pinned IRQ, device, control-plane, and explicitly non-migratable tasks cannot
  migrate.
- Migration does not hold kernel locks or mutable owner-state guards while
  waiting for a remote core.

Verification:

- Model tests prove at most one core can execute a task.
- Model tests prove a committed runnable task is owned by exactly one core.
- Model tests prove abort restores source ownership without duplicating the
  task.
- Host tests prove stale migration messages cannot revive an earlier task
  incarnation.
- Failure-injection tests cover source death before and after commit,
  destination death before and after commit, commit-ack loss, timer expiry
  during prepare, and topology change during recovery.
- Host tests prove pinned/control tasks reject migration before mutation.

Exit criteria:

- Scheduler and AI policy work has a concrete task-ownership transfer protocol
  instead of treating migration as a local queue operation.

### v0.46.4 - Minimal Capability Syscall ABI

Goal:

Define the native syscall and IPC ABI dispatched after ring-3 entry before
`aesynx-abi`, `aesynx-rt`, init, or external commands depend on informal kernel
entry conventions.

Deliverables:

- Exact x86_64 register calling convention for syscall entry, return, and
  faulted return.
- Architecture-neutral ABI semantics are separated from architecture-specific
  entry mechanics:
  - syscall numbers, endpoint operation IDs, wire structures, error codes,
    capability handles, transaction states, and semantic behavior are identical
    fixed-width encodings on every supported architecture;
  - x86_64, future aarch64, and future RISC-V define their own trap
    instruction, argument registers, return registers, preserved-register set,
    stack rules, and red-zone or no-red-zone policy.
- Stack alignment contract at entry and return.
- ABI version, feature bitmap, and mandatory/optional feature negotiation.
- Fixed-width syscall numbers, endpoint operation codes, flags, capability
  handles, object handles, virtual addresses, lengths, timeouts, transaction
  IDs, and error codes.
- No Rust enum layout, trait object, pointer, reference, slice, `usize`-sized
  semantic value, or compiler-dependent layout crosses the boundary.
- Capability IDs are interpreted only in the caller's current capability-table
  incarnation and domain incarnation.
- Explicit split between true kernel syscalls and endpoint/service RPC:
  syscalls are the narrow entry mechanism for scheduling, domain lifecycle,
  memory/object/capability enforcement, and endpoint transport; rich policy
  remains service RPC.
- Minimal initial syscall set:
  - endpoint send;
  - endpoint receive;
  - endpoint call;
  - endpoint reply;
  - yield;
  - exit;
  - capability-table inspection where authorized;
  - controlled memory mapping/object operations needed by init and loader
    bring-up.
- Unknown syscall numbers, unsupported ABI versions, unknown mandatory flags,
  malformed handles, stale handle generations, and reserved fields are rejected
  before mutation.
- Copy-in/copy-out ownership rules:
  - copy-in produces a kernel-owned initialized snapshot before parsing;
  - no authority field is reread from user memory after validation;
  - copy-out completion is kernel-owned until the final user write;
  - copy-out failure has a defined pending-completion or cancellation result.
- Interruptible versus noninterruptible call classes.
- Cancellation, timeout, and restart behavior for every blocking call.
- Partial-result policy. A failed or interrupted syscall never leaves the caller
  guessing whether a committed mutation must be retried.
- Blocking transaction state:

```text
Entered -> Validated -> Pending -> Completed | Cancelled | Faulted
```

- User-visible retry semantics: retrying after interruption cannot repeat an
  already committed operation.
- Checked maximum kernel work per invocation, including copy, parse,
  capability lookup, queue work, and fault handling.
- Error namespace is Aesynx-native. POSIX `errno` values are not assumed unless
  a compatibility layer explicitly maps to them.
- No implicit parent process, current directory, file-descriptor table, global
  filesystem namespace, uid/gid, or ambient authority.
- Redacted syscall trace events that expose syscall class, result class, budget
  class, and incarnation mismatch class without leaking raw capability IDs,
  object IDs, pointers, or kernel addresses.

Verification:

- Host tests fuzz every register field, ABI version, syscall number, flag
  combination, handle generation, pointer/length pair, timeout encoding, and
  reserved field.
- Cross-architecture golden vectors prove x86_64 and future aarch64/RISC-V
  decoders interpret the same architecture-neutral request, response, handle,
  flag, timeout, and error structures identically.
- Host tests inject interruption at every blocking transaction state and prove
  retry cannot duplicate committed work.
- Host tests inject copy-in and copy-out faults and prove partial snapshots
  cannot authorize mutation while committed completions are not silently lost.
- Host tests prove unknown mandatory flags fail before mutation and optional
  unknown flags are ignored or rejected according to the negotiated feature
  policy.
- QEMU smoke exercises endpoint send/receive/call/reply, yield, exit, denied
  capability inspection, and one controlled mapping/object syscall through the
  stable ABI.

Exit criteria:

- Aesynx has a stable minimal syscall/endpoint ABI contract before runtime
  wrappers or init depend on it.

### v0.46.5 - User Fault Delivery Policy

Goal:

Define how faults from hostile user domains are classified, charged, delivered,
or converted into termination before native services rely on recoverable
exceptions or debugger behavior.

Deliverables:

- Initial fatal policy for page faults, invalid instructions, divide errors,
  protection violations, and explicit traps that are not covered by an
  authorized exception endpoint.
- Optional exception endpoints require explicit capability authority and are
  bound to domain/task incarnation.
- Kernel-stamped fault messages contain only redacted virtual-address class,
  fault type, access type, task/domain incarnation class, instruction-pointer
  class, and bounded register summary.
- Two fault outputs:
  - telemetry/audit fault records are always redacted and safe for unrelated
    observers;
  - capability-authorized same-domain exception messages may contain exact
    user-space fault addresses, user instruction pointers, and a defined
    register subset required for recovery, but never kernel addresses,
    physical addresses, page-table values, or unrelated-domain state.
- No raw kernel addresses, physical addresses, CR3 values, page-table roots, or
  raw page-table entries in user fault messages.
- Recoverable exception-delivery contract:
  - one-shot exception-resume capability;
  - resume token bound to fault transaction, task incarnation, domain
    incarnation, and saved-frame generation;
  - exact set of registers the handler may inspect or modify;
  - separate rights for inspect, modify, map, and resume;
  - resume rejected if the task was killed, migrated, restarted, or faulted
    again;
  - pager endpoint requires explicit address-space mapping authority;
  - handler cannot convert a protection fault into a mapping without the
    relevant memory-object and address-space capabilities;
  - timeout, recursive fault, or handler death consumes the resume token and
    enters teardown;
  - at most one live resume token per stopped fault frame.
- Recursive exception-delivery depth limit.
- Separate or guard-protected handler stack policy.
- Full or faulting exception endpoint falls back to deterministic termination
  through the domain teardown state machine.
- Debugger capability can suspend and inspect only the explicitly targeted
  domain and does not imply mapping, register-write, resume, or cross-domain
  authority unless separately granted.
- Fault delivery and repeated malicious faults are charged to the domain's
  scheduling/request budget.
- User fault handling integrates with domain termination and cannot leave a task
  executable on an unfenced core after fatal classification.
- "Same-domain exception message" means the handler has explicit authority over
  the target address space and fault frame. It does not mean the initial
  one-task domain runs its own handler task. In the first userspace profile,
  recoverable exception and pager endpoints normally target a separate service
  domain holding attenuated exception, pager, debug, memory, and address-space
  capabilities.

Verification:

- QEMU tests trigger each initial user exception class and prove the configured
  fatal or endpoint-delivery path.
- Host tests prove fault messages redact addresses and reject stale
  task/domain incarnations.
- Tests prove authorized exception messages expose exact user-space recovery
  data only to holders of the correct endpoint/debug/pager capabilities.
- Tests prove resume tokens are one-shot, frame-generation-bound, and rejected
  after kill, migration, restart, repeated fault, timeout, or handler death.
- Tests prove a pager cannot install mappings without the relevant memory and
  address-space capabilities.
- Tests prove recursive endpoint faults hit the depth limit and terminate
  deterministically.
- Tests prove debugger caps cannot inspect or resume unrelated domains.
- Budget tests prove repeated user faults consume the domain budget or trigger
  deterministic termination/reset.

Exit criteria:

- User exceptions have a deterministic policy before native services can expose
  exception handlers or debugger capabilities.

### v0.46.6 - Initial Task And Domain Model

Goal:

Define the first native userspace execution model before ABI/runtime work can
accidentally grow shared-address-space thread semantics.

Deliverables:

- One task per domain through the first native userspace milestones.
- `exit` terminates the current domain, not only a task.
- Fatal task faults enter the domain teardown protocol.
- Exception and pager endpoints target a separate service domain unless a later
  multi-task-domain milestone explicitly authorizes same-address-space handler
  tasks.
- No shared-address-space threads in the initial profile.
- No task-create, thread-create, task-join, or thread-join syscall numbers in
  the initial ABI.
- TLS is per task, but there is exactly one TLS instance per initial domain.
- Domain address-space mutation while the sole task can execute requires the
  same fencing and activation-generation checks as later multi-task mutation;
  there is no implicit "safe because one task exists" bypass.
- Task migration either migrates the whole domain in the initial profile or is
  rejected if future metadata says a domain has more than one task.
- Domain teardown revokes the capability table, stops the sole task, fences
  execution, and sanitizes private memory before cross-domain reuse.

Verification:

- Host ABI tests prove task-create, thread-create, task-join, and thread-join
  requests are undefined or rejected before mutation.
- Spawn tests prove a new initial domain contains exactly one runnable task.
- Exit and fatal-fault tests prove the domain teardown state machine is entered.
- Fault-delivery tests prove an exception handler task inside the same initial
  domain is rejected; an external pager/debug service with attenuated authority
  is accepted.
- TLS tests prove exactly one initial TLS instance and reset it during teardown.
- Address-space mutation tests prove the sole task is fenced before mappings
  change.

Exit criteria:

- Native userspace starts from a small domain model that cannot be confused
  with POSIX-style multithreaded processes.

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
- ABI constants and generated/centralized definitions for syscall numbers,
  endpoint operation IDs, flags, handles, object IDs, error codes, wire
  endianness, and value-schema IDs.
- Safe wrappers over the v0.46.4 syscall/endpoint ABI. Runtime wrappers do not
  expose kernel-private structs or raw unchecked capability IDs as authority.
- User TLS contract:
  - TLS is either explicitly unsupported in the first runtime profile or has a
    documented startup block layout;
  - user FS base is canonical and range-validated against the current user
    address space;
  - user GS is prohibited or reserved when kernel per-core state uses GS;
  - context switch and task migration save/restore user TLS base;
  - `SWAPGS` interactions are documented and tested;
  - FSGSBASE instructions are disabled, trapped, or validated according to the
    selected deployment profile;
  - process creation and teardown reset TLS state;
  - TLS descriptors cannot point into kernel space or another domain's address
    space.
- Startup information block with ABI version, runtime feature bits, initial
  capability bundle, stack/TLS layout, endpoint handles, and redaction policy.
- Startup block immutability:
  - kernel constructs it in owned memory;
  - it is mapped read-only into the new domain;
  - variable-length data is bounded and offset-based, not pointer-linked;
  - every offset and count is validated against the block size;
  - capability handles are table-local and invalid outside the new domain;
  - unused padding is zero;
  - no kernel pointers, physical addresses, raw object IDs, KASLR data, or
    other domains' handles are present;
  - the child may copy it but cannot treat it as live mutable kernel state;
  - later capability changes arrive through endpoints, not startup-block
    mutation;
  - restart produces a new startup-block generation.
- Runtime panic and exit reporting through explicit endpoint/capability
  authority, not ambient process-parent assumptions.

Verification:

- User program writes through console/log queue.
- Host tests verify ABI layout sizes, alignments, endianness, reserved-bit
  rejection, and no Rust-only enum layout in public ABI values.
- Host/QEMU tests cover TLS unsupported-mode rejection or TLS base validation,
  save/restore, migration handoff, teardown reset, and FSGSBASE policy.
- Host tests fuzz startup-block offsets, counts, padding, table-local handles,
  generation reuse, and forbidden pointer/address fields.
- Runtime tests prove debug output redacts capability handles and raw object
  identifiers.

Exit criteria:

- Native userspace is ergonomic enough to grow.

### v0.47.1 - Aesynx SDK And App Template

Goal:

Make it clear how developers build native and WASM apps for Aesynx.

Deliverables:

- `docs/sdk-roadmap.md`.
- Native Rust app target plan for `x86_64-unknown-aesynx`.
- Future `aarch64-unknown-aesynx` target placeholder.
- WASM component target/profile plan for `wasm32-wasip2-aesynx`.
- Userspace linker/startup rules owned by `aesynx-rt`.
- Minimal native command template.
- Minimal WASM component template.
- App/package manifest schema with artifact kind, target, entry point,
  exported commands, requested capabilities, SBOM, and provenance fields.
- Developer command flow for build, package, inspect, and QEMU-run smoke.
- Explicit rule that app developers use `aesynx-abi`, `aesynx-rt`, manifests,
  and capability handles rather than kernel-private headers or internals.

Verification:

- Template native app compiles against the SDK target plan or host-side
  placeholder until the native target is live.
- Template WASM component produces a manifest with no default authority.
- Manifest validation rejects undeclared ambient filesystem, network, device, or
  IPC authority.
- SDK docs include one complete hello-world flow and one capability-denied flow.

Exit criteria:

- External developers have a documented path for writing Aesynx apps without
  learning kernel internals.

### v0.47.2 - Executable Object Loader And User ASLR

Goal:

Load sealed executable objects into user address spaces through a bounded load
plan with user-space layout randomization, without making the kernel a broad
ELF policy engine.

Architecture:

- Full ELF parsing should live in a confined loader service where possible.
- Preferred trust model is a canonical signed load manifest: the executable
  object contains a canonical segment/load manifest covered by the object
  signature or boot-capsule signature, and the kernel checks the proposed load
  plan exactly matches it.
- The kernel consumes a bounded canonical load plan bound to the sealed
  executable-object hash and canonical load-manifest identity.
- The kernel independently validates every security-relevant mapping invariant
  before installing mappings.
- Initial `aesynx-init` may use a boot-capsule-provided, hash-bound load plan
  to avoid a circular dependency on `loaderd`, but that plan is covered by the
  verified boot capsule, not merely placed next to it.

Deliverables:

- Sealed executable object identity: exact bytes, content hash, architecture,
  ABI version, entry point, requested capability set, and manifest provenance.
- Content hash and sealed identity are mandatory for loader correctness.
  Signature authenticity is mandatory only for trust policies that claim
  publisher provenance. An unsigned executable may still be sandboxed if policy
  allows it, but a signature never grants capabilities automatically, and
  signed/unsigned artifacts cannot collide under one executable identity.
- Plan version, executable ABI, canonical load-manifest hash, executable object
  hash, and capability request manifest are part of the signed/hash-bound
  identity.
- Two distinct valid interpretations of the same executable bytes are not
  permitted unless they have distinct manifest identities.
- Bounded canonical load-plan format with fixed-width fields and no Rust layout
  dependency.
- Kernel load-plan checks:
  - every mapped byte range corresponds to a declared manifest segment;
  - file offsets, virtual ranges, access flags, entry point, and relocation
    records match the canonical load manifest;
  - no executable mapping can be manufactured from arbitrary data inside the
    same sealed object;
  - capability requests are bound to the same executable identity and load
    manifest;
  - boot-provided init load plans are covered by the verified boot capsule.
- ELF validation policy for the loader service:
  - supported class, endianness, machine, ABI, and file type;
  - static ELF only initially;
  - reject dynamic interpreter and unsupported dynamic linking;
  - bounded program-header count;
  - checked file offsets, lengths, and integer arithmetic;
  - `p_filesz <= p_memsz`;
  - canonical user virtual ranges;
  - page alignment and offset/virtual-address congruence requirements;
  - no overlapping or wraparound segments;
  - no writable-executable segment;
  - entry point lies inside an executable mapped segment;
  - no mapping into kernel, null, guard, reserved, queue, shared-control, or
    runtime-private regions;
  - BSS zero initialization;
  - no uninitialized padding disclosure;
  - strict supported relocation list if user PIE is allowed;
  - no text relocations;
  - RELRO or equivalent sealing where applicable.
- Executable backing cannot change between validation and mapping. The load
  plan is bound to the sealed executable hash, canonical load manifest, and
  capability request manifest.
- Executable transition protocol:
  - start from a sealed executable source object containing immutable
    canonical ELF/package bytes, load manifest, publisher provenance, and
    capability request identity;
  - create a private executable image instance for the target domain and load
    generation;
  - create private writable/non-executable staging mappings for that image
    instance;
  - copy file-backed bytes and zero BSS;
  - apply only declared and supported relocations;
  - validate instantiated bytes and manifest identity;
  - freeze every writable alias to executable content;
  - complete required local and remote TLB invalidation;
  - perform required architecture instruction-cache synchronization;
  - seal the executable image instance;
  - install final read/execute mappings for executable portions.
- Loader transition rules:
  - the immutable executable source object never becomes writable during
    loading;
  - relocated data belongs to the image instance, not the source identity;
  - publisher signatures cover the canonical source and load manifest, not
    ASLR-dependent relocated bytes;
  - post-relocation hashes are image-instance measurements, not replacement
    publisher identities;
  - two image instances from the same source may have different instance hashes
    and mapping generations;
  - relocation targets inside executable segments are rejected initially;
  - supported relocations may modify only declared writable relocation targets;
  - no physical frame may be writable in one address space while executable in
    another;
  - shared executable text is allowed only when its bytes are
    placement-independent and identical;
  - domain-private relocated text remains unsupported initially;
  - shared text and private relocated data are separate memory objects or
    separately tracked frame classes;
  - sealed source content and sealed image-instance content are immutable;
  - failed relocation, validation, sealing, or final mapping tears down staging
    and sanitizes staged frames before reuse;
  - final executable mappings use a generation distinct from writable staging;
  - revoking source execution authority prevents new image creation;
  - revoking one image instance tears down only that instance unless
    object-wide policy explicitly selects broader scope;
  - source deletion cannot reclaim backing pages while image instances or
    shared-text mappings remain pinned;
  - executable image instances are cross-object children linked from the source
    object through the generic `DerivedObjectEdge` and selective-revocation
    machinery rather than a loader-only side graph or an implicit same-object
    capability derivation;
  - every image instance records source object incarnation, source
    execution-authority lineage, load-manifest identity, image-instance
    incarnation, load generation, mapping lineage, and owning domain
    incarnation;
  - revoking a loader/execute capability prevents that principal from creating
    new instances but does not affect instances created through unrelated
    authority unless object-wide scope is selected;
  - revoking a source-lineage subtree finds and revokes every image derived
    through that lineage;
  - strong source-object revocation freezes new loads and tears down every
    derived image instance and shared-text mapping before it can report
    completion;
  - deleting a source object is allowed only after every image instance,
    shared-text mapping, load transaction, and executable pin is drained;
  - shared text cannot outlive the backing source incarnation unless it has
    been promoted into a separately owned immutable code object with its own
    authority lineage;
  - executable image revocation, downgrade, or replacement uses the same TLB
    and residency protocol as ordinary mapping revocation;
  - x86_64 cannot define the portable instruction-cache rule as a no-op for
    all architectures; aarch64 and RISC-V loaders must perform their explicit
    instruction-cache maintenance before claiming executable readiness.
- The canonical load manifest binds supported relocation types, relocation
  target ranges, post-relocation segment hashes where feasible, executable
  transition generation, and text shareability policy.
- Transactional mapping creation: any failure removes every partially installed
  segment, stack, heap, TLS, queue, and startup-block mapping.
- User ASLR:
  - prefer statically linked `ET_DYN`/PIE executables for randomized placement;
  - randomize executable base, stack, heap, and future mapping region through
    domain-separated DRBG labels;
  - maintain guard gaps around stack, heap, shared queues, TLS, startup block,
    and future mappings;
  - preserve low/null unmapped regions;
  - validate effective entropy after alignment and address-space constraints;
  - deterministic fixed layout is allowed only for tests/debug profiles with
    explicit non-production status;
  - failure to obtain randomness follows the selected deployment profile rather
    than silently claiming ASLR;
  - address randomization never substitutes for capability checks, W^X, SMAP,
    or usercopy discipline.
- Redacted loader diagnostics and World Service facts expose object hash class,
  source identity class, image-instance generation, mapping generation, segment
  counts, policy result, and entropy class without leaking raw user layout.

Verification:

- Coverage fuzzing for malformed ELF headers, program headers, overlapping
  segments, extreme counts, truncated files, invalid relocations, unsupported
  dynamic fields, entry-point confusion, and wraparound arithmetic.
- Differential tests against an independent host parser fixture for accepted
  and rejected ELF/load-plan cases.
- Host tests prove load-plan hash mismatch, canonical load-manifest mismatch,
  capability request manifest mismatch, backing-object mutation, and
  unsupported relocation fail before mapping.
- Host tests prove relocation targets inside executable segments are rejected
  initially and no final executable frame remains writable in another address
  space.
- Host tests prove failed relocation or post-relocation validation sanitizes
  staging frames and consumes/retires the staging generation.
- Host tests prove two ASLR placements can share immutable PIC text while
  retaining distinct private data image instances.
- Host tests prove image-instance hashes are not accepted as executable source
  signatures or publisher identities.
- Host tests prove source revocation blocks new image creation.
- Host tests prove image-instance revocation does not accidentally revoke
  unrelated instances unless object-wide scope was selected.
- Host tests prove revoking one principal's load authority does not affect
  instances created through unrelated authority unless object-wide scope was
  selected.
- Host tests prove source-lineage subtree revocation discovers every derived
  image instance and shared-text mapping.
- Host tests prove strong source-object revocation cannot complete while any
  derived image remains executable.
- Host tests prove an image instance cannot be rebound to a recreated source
  object with the same user-visible name.
- Host tests prove shared text cannot outlive the backing source incarnation
  unless it was promoted into a separately owned immutable code object with its
  own authority lineage.
- Host tests prove source backing cannot be reclaimed while any image instance
  or shared-text mapping remains live.
- Host tests prove executable downgrade/revocation follows the same TLB,
  residency, and generation rules as ordinary mapping revocation.
- Architecture tests prove aarch64 and RISC-V executable installs issue the
  required instruction-cache synchronization before the mapping is reported
  runnable.
- Host tests prove loaderd cannot select arbitrary object ranges, mark data as
  executable, choose a different entry point, omit security-relevant ELF
  metadata, or reinterpret the same executable object under a second identity.
- Boot-capsule tests prove init's load plan is covered by the verified capsule
  signature/manifest.
- Host tests prove transactional failure removes every partially installed
  segment and returns quotas/capability reservations.
- QEMU smoke loads init from a sealed boot-capsule load plan, then loads a
  second static executable through the loader service when available.
- ASLR tests prove independent domain-separated placement for executable, stack,
  heap, and mapping region, and prove deterministic mode is marked
  non-production.

Exit criteria:

- Executable loading and user ASLR are explicit, sealed, fuzzed, and
  transactional before external native commands run.

### v0.47.3 - Transactional Domain Spawn

Goal:

Create a process/domain only when every required authority, mapping, budget,
and scheduler component can be committed atomically.

Deliverables:

- Spawn transaction state machine for staging, validation, commit, abort, and
  recovery.
- Typed domain-lifecycle capabilities are entries in the central kind-to-right
  matrix, not local ad hoc enums:
  - `DomainFactoryRights::SPAWN`;
  - `DomainControlRights::{STOP, KILL, RESTART, INSPECT_STATUS,
    SET_EXCEPTION_ENDPOINT}`;
  - `DebugRights::{READ_REGISTERS, WRITE_REGISTERS, READ_MEMORY, WRITE_MEMORY,
    SUSPEND, RESUME}`.
- Executable-object possession alone does not authorize spawning a domain.
  `SPAWN` requires explicit quota, scheduling-context, address-space, and
  executable-object authority.
- Initial init authority comes from a boot-policy-issued capability bundle, not
  a hidden kernel exception.
- Staged resources:
  - new domain and task incarnations;
  - capability table and quota escrow;
  - address-space root;
  - executable mappings from a sealed load plan;
  - stack, TLS, startup block, and guard pages;
  - scheduling context and CPU budget;
  - initial endpoint and reply capabilities;
  - explicit initial capability bundle;
  - parent/creator launch-result capability if requested.
- Child cannot become runnable until every component is valid and the spawn
  commit record exists.
- Failed construction restores quotas, removes mappings, clears pending
  capabilities, retires journal records, and removes scheduler state.
- Initial capabilities are explicit grants. The child never inherits the
  creator's entire capability table, namespace, current directory, or ambient
  authority.
- Parent/creator identity does not imply authority over the child after launch.
  A creator controls a child only if it retains an explicit attenuated
  `DomainControl` capability.
- Self-exit is permitted through the caller's current execution context and
  does not imply control over another task or domain.
- `STOP` does not imply `KILL`, `INSPECT_STATUS`, debug authority, or
  `RESTART`. `INSPECT_STATUS` returns redacted lifecycle state only and does
  not imply memory/register inspection.
- `RESTART` creates a new domain incarnation and cannot silently reuse old
  endpoint, capability-table, scheduling-context, exception, or debug authority.
- Generic `ADMIN` does not satisfy domain factory, domain control, exception,
  or debug rights.
- Duplicate spawn commit cannot create two children from one transaction.
- Stale launch results cannot bind to recycled domain or task IDs.
- Spawn journal capacity is reserved separately from ordinary endpoint traffic.
- Redacted spawn telemetry reports result class and denial reason without raw
  capability or object identifiers.

Verification:

- Model tests prove no partially initialized child can execute.
- Model tests prove spawn has exactly one final outcome and duplicate
  prepare/commit/abort messages are idempotent.
- Host tests inject failures after every staged component and prove quotas,
  mappings, capability slots, scheduler records, TLS/startup memory, and
  endpoint state are reclaimed or retired according to policy.
- Host tests prove initial capability bundles are exact and no ambient creator
  authority leaks into the child.
- Host tests prove executable possession alone cannot spawn, `SPAWN` requires
  quota/scheduling/address-space authority, `ADMIN` does not satisfy typed
  lifecycle rights, and child control after launch requires an explicit
  attenuated `DomainControl` capability.
- Host tests prove stale launch results are rejected after domain/task
  incarnation reuse.

Exit criteria:

- Init and later spawn services can create domains without exposing partially
  initialized execution or ambient authority inheritance.

### v0.47.4 - Domain Termination And Resource Teardown

Goal:

Make the reverse lifecycle of domain creation explicit before separate
restartable processes, shell restart, driver services, or hostile domains become
normal.

Deliverables:

- Domain termination state machine:

```text
Running
  -> StopRequested
  -> ExecutionFenced
  -> AuthorityRevoking
  -> ResourcesDraining
  -> Dead
  -> Reclaimable
```

- Stop every task on every core and prevent new scheduling or migration.
- Fatal user fault, explicit exit, creator-requested kill, budget exhaustion,
  watchdog reset, and service restart all enter the same teardown protocol with
  typed reasons.
- Cancel or resolve pending calls and one-shot reply capabilities.
- Remove timers, wakeups, wait-queue records, and donated scheduling contexts.
- Revoke the domain capability table and resolve pending grants, move
  transactions, quota credits, and launch results.
- Tear down shared mappings and complete required TLB invalidation.
- Quiesce DMA and IRQ authority where present.
- Drain remote frees and allocator ownership.
- Remove World Service query leases, projections, and result-stream state.
- Cross-domain private-memory sanitization is unconditional for plaintext
  private memory. A private physical frame is never mapped into a different
  protection domain until all stale CPU/DMA observers are fenced and the frame
  has been zeroed or cryptographically erased under a proven memory-encryption
  key lifecycle.
- Narrow exceptions:
  - deliberately shared immutable content;
  - public executable/package content;
  - hardware-backed cryptographic erase where retiring the key demonstrably
    removes access;
  - same-domain reuse under the same surviving memory-confidentiality
    incarnation, if explicitly allowed.
- Sanitization coverage includes kernel stack pages, user stack and TLS pages,
  register/XSAVE save areas, secret-bearing capability-table slots, IPC and
  syscall snapshot buffers, page-table pages before reuse as ordinary data,
  allocator metadata that may contain prior object identities, and crash/fault
  emergency buffers.
- Zeroing happens after stale observers are fenced. It is not a substitute for
  TLB, IOMMU, DMA, or core-execution fencing.
- Publish exit result through an explicit capability rather than an ambient
  parent relationship.
- Bound retained exit records so dead or zombie domains cannot exhaust kernel
  memory.
- Change domain incarnation before identifiers, capability-table slots,
  endpoints, or task IDs can be reused.
- Strong termination cannot report success while a task remains executable on
  an unfenced core or while stale authority can still commit.

Verification:

- Model tests prove termination cannot report `Dead` before execution is fenced
  on every relevant core.
- Failure-injection tests cover task running during kill, pending reply,
  donated budget, timer wake, in-flight grant, shared mapping, TLB ack loss,
  remote-free backlog, world query lease, and stale exit-result delivery.
- Tests prove private frames, stack/TLS/register-save areas, IPC/syscall
  snapshots, page-table pages, allocator metadata, and emergency buffers are
  fenced then zeroed or cryptographically erased before cross-domain reuse.
- Tests prove zeroing before fencing is not accepted as teardown completion.
- Host tests prove duplicate kill/exit/fault events are idempotent.
- Host tests prove bounded exit records and zombie cleanup cannot exhaust
  kernel memory.
- Tests prove domain incarnation changes before ID reuse and stale launch/exit
  results are rejected.

Exit criteria:

- Aesynx has an auditable domain teardown path before normal process restart or
  hostile-domain lifecycle management.

### v0.48.0 - aesynx-init

Goal:

Start first native user process.

Deliverables:

- `aesynx-init`.
- Initial capability bundle.
- Boot object lookup.
- Init executable loaded from a sealed executable object or boot-capsule load
  plan validated by v0.47.2.
- Init domain created through the v0.47.3 spawn transaction.
- Init teardown, restart, and fatal fault paths use the v0.47.4 domain
  termination state machine.
- Init writes banner.

Expected serial:

```text
Aesynx userspace online
[TEST] init=ok
```

Verification:

- Kernel launches init through the sealed loader and transactional spawn path.

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

### v0.51.2 - Multi-Task Domain Model

Goal:

Add shared-address-space tasks only after the one-task native profile has real
end-to-end evidence through init boot, shell separation/restart, external
native command execution, single-task spawn/fault/exit/kill/restart stress,
single-task migration, and single-task teardown tests.

Until this milestone lands:

- Task-create, thread-create, task-join, and thread-join syscall numbers remain
  absent or rejected.
- The ABI feature bitmap reports no multi-task support.
- Domain manifests cannot request multiple initial tasks.
- Exception handlers remain external services.
- Scheduler and address-space internals may be multi-task-ready, but the public
  execution profile remains one task per domain.

Deliverables:

- Explicit ABI/runtime feature negotiation for multi-task domains. Enabling the
  feature cannot silently change existing exit, fault, TLS, or address-space
  semantics.
- Explicit `TaskFactory` or `ThreadCreate` authority. Possessing a domain,
  address-space, or executable capability does not imply authority to create a
  second task inside that domain.
- Creating a task requires task quota, stack/TLS memory, scheduling-context
  authority, and target-domain authority; `CREATE_TASK` cannot create a task in
  an unrelated domain.
- Per-task incarnation, stack, guard page, TLS base, scheduling context, fault
  frame state, exit state, and accounting.
- Domain-wide task table with thread-count quotas and fail-closed generation
  retirement on task-ID reuse.
- Address-space mutation synchronization across every task in the domain:
  freeze or rendezvous runnable tasks, invalidate stale activation permits,
  complete TLB shootdown/residency acknowledgements, then publish the new
  mapping generation.
- Clear task-local versus domain-wide exit semantics:
  - task exit produces a task result capability if requested;
  - task-local exit does not revoke the whole domain unless it is the last task
    under the configured policy;
  - domain exit tears down every task;
  - last-task exit terminates the domain unless policy explicitly keeps an
    empty supervisor domain alive.
- Join/result capabilities are task-incarnation-bound objects with
  operation-specific consumption semantics. `WAIT` is repeatable,
  `READ_RESULT` follows observer policy, and `CONSUME_RESULT` is one-shot.
- Join/result operation semantics:
  - `WAIT` is repeatable until completion and does not consume the join/result
    capability;
  - `READ_RESULT` returns a bounded immutable result and is repeatable only for
    observer policies that explicitly allow repeated reads;
  - `CONSUME_RESULT` is one-shot, has a named linearization point, and
    transitions the result to consumed for that holder;
  - cancellation and timeout never consume a result that completed
    concurrently unless the consume linearization point was reached.
- Result storage has a bounded retention deadline or quota and is reclaimed by
  policy after every authorized holder has consumed, expired, or been revoked.
- Reading a task result does not imply task control.
- Exit results cannot contain raw capability handles. Any capability returned
  by a task must transfer through a separate audited grant transaction.
- Fault containment rules define which faults terminate only the task, which
  terminate the domain, and which may be delivered to an authorized external
  pager/debug service.
- Domain teardown dominates every task-local capability and consumes or retires
  outstanding join/result objects.
- Cross-core teardown stops, fences, and drains every task before the domain can
  report `Dead`.
- Scheduler and budget accounting distinguish per-task budgets from domain
  aggregate ceilings.

Verification:

- One-task profile tests prove multi-task ABI feature bits are disabled before
  this milestone and task/thread creation or join requests are rejected.
- Manifest tests prove multiple initial tasks cannot be requested until the
  negotiated multi-task feature is active.
- Stress tests prove one-task spawn, fault, exit, kill, restart, migration, and
  teardown behavior remains unchanged before and after this feature is disabled.
- Host/model tests prove task creation requires explicit `TaskFactory` or
  `ThreadCreate` authority and respects task-count quotas.
- Tests prove `CREATE_TASK` cannot target an unrelated domain and cannot be
  derived from only domain, address-space, or executable possession.
- Tests prove task IDs, join capabilities, and result capabilities reject stale
  incarnations after task reuse.
- Model tests cover completion racing with wait timeout, cancellation,
  `READ_RESULT`, `CONSUME_RESULT`, domain teardown, result retention expiry,
  and task-incarnation reuse.
- Tests prove consuming one observer result does not destroy availability for
  other authorized observers unless single-consumer policy was explicitly
  selected.
- Tests prove exit results cannot carry raw capability handles and capability
  return uses a separate audited grant transaction.
- Tests prove `KILL` and exception-endpoint mutation rights are nondelegable by
  default and generic `ADMIN` does not satisfy task lifecycle rights.
- Address-space mutation tests prove no runnable task can execute with a stale
  mapping generation after mutation commits.
- Fault tests prove configured task-local faults do not leak authority or leave
  stale resume tokens, while domain-fatal faults enter the domain teardown
  protocol.
- Cross-core teardown tests prove a domain cannot report `Dead` until every
  task is fenced, every scheduling context is drained, and every join/result
  capability is resolved or retired.

Exit criteria:

- Shared-address-space tasks become an explicitly negotiated
  authority-bearing feature after the single-task userspace profile has proven
  itself end to end.

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
- Explicit non-claim that the verifier is not reachable by untrusted input
  until an interpreter/service call path is added.

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
- Real signature verification before loaded model bytes can influence advisory
  runtime policy. A placeholder is allowed only for offline fixtures that cannot
  affect scheduler, shell, or policy-service decisions.
- Supported signature scheme and canonical manifest encoding.
- Trust-root storage and update policy.
- Key rotation and signer revocation.
- Model-version anti-rollback.
- Policy-domain binding.
- Hash covers the exact bytes consumed by the evaluator, not a different
  packaging representation.
- Parser and manifest-version downgrade rejection.
- Failure behavior when clock, revocation status, or trust-root availability is
  unavailable.
- Safety limits.

Verification:

- Bad schema rejected.
- Bad hash rejected.
- Bad signature, revoked signer, stale model version, wrong policy domain, and
  parser downgrade are rejected before runtime policy use.
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

### v0.60.1 - OS World Trace Prototype

Goal:

Convert deterministic trace events into a host-side OS world prototype.

Deliverables:

- Trace-to-fact converter.
- Initial fact envelope for boot, memory, capabilities, objects, drivers, and
  tasks.
- Host-side world file with immutable fact records.
- Basic query examples for "why did this happen" and "what changed".
- Documentation that this is userspace tooling, not kernel query logic.

Verification:

- QEMU trace produces a deterministic world file.
- Host query can list boot facts, memory facts, and capability events.
- Redaction rules hide sensitive fields in public query mode.

Exit criteria:

- Aesynx has a concrete bridge from telemetry to the future world service.

### v0.60.2 - Live OS World Service

Goal:

Move from host-side trace conversion to a capability-scoped native `worldd`
service that can answer bounded questions without becoming a kernel authority
source.

Deliverables:

- Capability-scoped fact ingestion from kernel/service telemetry streams.
- Capability-scoped query API for native userspace.
- Canonical fact types versus derived/projection fact types.
- Source identity, boot/session nonce, core/domain incarnation, schema version,
  and provenance validation for every accepted fact.
- Query CPU, memory, result-size, and wall-clock budgets.
- Retention, compaction, and restart-reconstruction policy.
- Redaction before joins and projections, not only at final serialization.
- Projection invalidation rules when source facts are revoked, corrected, or
  superseded.
- Advisory-fact rule: no authority decision is made solely from advisory,
  incomplete, stale, or lossy facts.
- Query authorization creates a bounded live read lease or snapshot
  authorization.
- Long-running queries periodically observe cancellation/freeze state.
- Result publication revalidates query authority and classification.
- Revocation before result release prevents releasing newly unauthorized rows.
- Partial streaming results have explicit revocation semantics.
- Derived facts inherit the most restrictive source classification unless a
  deterministic declassification capability authorizes otherwise.
- Query and projection caches are partitioned by authorization and
  classification context.
- A result computed under one principal's authority cannot be replayed to
  another principal.
- Cancellation does not leave unbounded query memory or projection state.
- Per-core completeness frontiers:
  - complete through boot/session nonce, core incarnation, and sequence;
  - gap/loss after sequence `N`;
  - topology snapshot epoch `E`.
- Absence of an event is non-authoritative unless backed by a completeness
  certificate for the relevant source and epoch.

Verification:

- Host and userspace tests prove queries are filtered by caller capability.
- Tests prove redaction happens before joins can correlate restricted fields.
- Tests prove restart reconstruction preserves provenance and completeness
  frontier semantics.
- Tests prove missing telemetry, loss counters, and incomplete frontiers cannot
  be interpreted as "no event happened" for authority decisions.
- Tests prove revocation during a long-running query prevents release of newly
  unauthorized rows and frees bounded query/projection resources.

Exit criteria:

- Aesynx has a real userspace world service path that can support diagnostics
  and AI context packs without moving rich query logic into ring 0.

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

### v0.61.2 - Metered Asynchronous AI Policy Service

Goal:

Ensure AI advice can never block scheduler progress, bypass deterministic
validation, or become a hidden reference monitor.

Deliverables:

- AI/model execution runs outside ring 0 in a capability-confined policy
  service, not in the scheduler hot path.
- Preemptive timer enforcement is a hard dependency before Aesynx may claim an
  AI policy service cannot lock a core. Until preemption exists, AI policy
  services are cooperative/advisory only and must run on noncritical placement.
- Scheduler-controlled CPU budgets, watchdog termination, and domain reset for
  policy services.
- Policy services cannot own scheduler-critical locks, queues, or dedicated
  control cores.
- Model output is treated as hostile bytes. Public advice constructors are not
  authority: the scheduler computes a bounded validator over authoritative
  kernel state, including queues, running tasks, topology, task incarnations,
  affinity/security rules, budgets, and topology epoch.
- Model evaluation has explicit fuel, deadline, memory, and output-size limits.
- Manifest step and memory ceilings are consumed by the evaluator, not only
  stored as metadata.
- Runtime model authenticity check: loaded model bytes must match the signed
  manifest, model version, policy domain, and object identity. Safety must not
  depend on authenticity, but substitution and stale-model replay must still be
  rejected.
- Kernel scheduling path never waits synchronously for model output. Advice is
  accepted only if already available and still fresh.
- Advice records carry topology epoch, task incarnation, model version, expiry,
  and confidence bounded by the validated manifest.
- Raw `PolicyDecision` or `ScheduleAdvice` values are never directly
  executable. The scheduler accepts only a non-publicly-constructible
  `ValidatedScheduleAction` produced by the kernel validator.
- Kernel-side validator is total and bounded:
  - computes the finite admissible action set for the current scheduler state;
  - stays within an explicit complexity bound such as `O(C)` over a capped core
    count;
  - uses checked arithmetic and deterministic tie-breaking;
  - rejects stale topology/task epochs;
  - rejects invalid cores, ownership violations, affinity violations, and
    migration-budget violations;
  - executes deterministic fallback when advice is missing, stale, invalid, or
    over budget.
- Formal scheduler invariant statement: every admissible action preserves task
  ownership, queue membership, budget accounting, and security-domain placement
  constraints.
- Commit rechecks the scheduler/topology epoch after validation; stale advice
  falls back deterministically rather than mutating scheduler state.
- Task time budgets are decremented or explicitly documented as advisory until
  preemption lands.
- Telemetry buffer-full behavior cannot stop scheduling. Noncritical telemetry
  drops or overwrites records with a loss counter; security audit events use a
  separate reserved channel with a decision table:
  - authority creation, grant, executable mapping, DMA mapping, and policy
    expansion fail closed if required audit evidence cannot be recorded;
  - revoke, quarantine, and permission reduction proceed fail-safe, reserve
    emergency audit capacity, and record a sticky audit-loss digest or halt
    after authority is removed if even emergency evidence is exhausted;
  - noncritical scheduling telemetry may be lossy and increments loss counters;
  - operator/debug telemetry never blocks scheduler dispatch.
- OS-world trace emission targets zero allocation, zero locking, and zero copy
  on the normal event path: per-core single-writer binary rings, read-only
  collector mappings, release/acquire slot publication, slot generations,
  boot/session nonce, schema epoch, source-domain incarnation, explicit
  overwrite and multi-reader behavior, sequence gaps, redaction at export,
  replay/reordering detection, and user-space hash chaining or Merkle
  aggregation.
- Overwriteable telemetry records use a seqlock-style snapshot protocol:
  - writer publishes an odd/in-progress generation;
  - writer replaces the payload;
  - writer release-publishes an even/complete generation;
  - reader acquire-loads generation, copies payload, then reloads generation;
  - reader accepts only equal, even generations.
- Seqlock generation wrap retires or reinitializes the ring before reuse can
  make a stale even generation appear current.
- If a writer dies while a record generation is odd/in-progress, readers reject
  the slot, the collector records a torn-writer/lost-record event, and recovery
  must either complete, retire, or reinitialize the slot before reuse.
- Readers never form persistent references into overwriteable telemetry slots.
- Periodic tamper-evident chunk checkpoints, optionally anchored by TPM or a
  trusted service, for detecting suffix truncation or collector omission. The
  roadmap must not claim impossible zero overhead; the target is bounded
  constant overhead with zero allocation and zero locking on normal emission.

Verification:

- Host tests prove invalid, stale, over-budget, or unavailable advice falls
  back without blocking dispatch.
- Host tests prove a full noncritical telemetry buffer increments loss and does
  not prevent task dispatch.
- Host tests prove advice cannot select an invalid core or migrate a task
  outside allowed affinity/security-domain policy.
- Host tests prove raw advice cannot be executed without producing a
  `ValidatedScheduleAction`.
- Property tests prove arbitrary model bytes can only produce a valid sealed
  action or the exact deterministic fallback, and accepted actions preserve
  unique task ownership, current task/core incarnation, affinity,
  security-domain, manifest, priority, and budget ceilings.
- Host tests prove audit-buffer failure blocks authority creation/expansion
  while revoke, quarantine, and permission reduction proceed fail-safe with
  emergency audit-loss evidence; noncritical telemetry loss does not block
  dispatch.
- Host tests prove overwriteable telemetry snapshots reject odd generations,
  changed generations, and torn records.
- Host tests prove seqlock generation wrap retirement and odd-generation writer
  death recovery do not expose torn records as complete snapshots.
- QEMU smoke proves the scheduler continues with AI disabled, model timeout,
  and telemetry loss counters.

Exit criteria:

- AI is a bounded proposal source, and deterministic scheduler safety does not
  depend on model liveness or correctness. Claims about preventing a malicious
  policy service from monopolizing a core require preemptive CPU enforcement to
  be active.

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

- QEMU multicore boot through SMP/APIC hardware mechanisms.
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
- Disable PCI bus mastering before revoking DMA-capable devices.
- Device-specific reset or PCIe function-level reset where supported.
- IOMMU unmap followed by completed IOTLB invalidation before DMA authority is
  considered gone.
- PCIe ATS, PASID, and PRI invalidation, or explicit prohibition for devices
  where Aesynx cannot invalidate those translations safely.
- Interrupt-remapping, MSI, and MSI-X fencing so old interrupts cannot target a
  restarted or newly assigned domain.
- Device-memory ordering barriers around bus-master disable, reset or
  function-level reset, IOTLB completion observation, interrupt-remap teardown,
  and queue teardown.
- Device and interrupt-remapping incarnations included in driver restart and
  revoke transactions.
- No driver restart until old DMA identities and interrupt identities are
  unreachable.
- Fail-closed policy for devices that cannot be reliably quiesced, reset, or
  fenced from DMA/interrupt delivery.

### v1.3 - aarch64 QEMU Preview

- QEMU `virt` boot.
- PL011 serial.
- EL1 kernel entry with EL0 reserved for future userspace.
- Typed MAIR/TCR/SCTLR setup plan.
- TTBR1_EL1 kernel mapping policy and TTBR0_EL1 user address-space plan.
- ASID allocation and rollover policy.
- GICv3.
- Generic timer.
- PSCI secondary-core startup plan.
- Basic memory map.
- PXN, UXN, WXN, PAN, BTI, PAC, and MTE support policy with hardware feature
  gates and deterministic fallbacks.
- Device versus normal memory attributes validated before MMIO or DMA access.
- SMMUv3 DMA isolation roadmap for driver domains.
- Barrier policy for mapping publication, queue publication, MMIO, and TLB
  invalidation.

### v1.3.1 - RISC-V 64 QEMU Preview

- QEMU `virt` boot.
- Minimal firmware/SBI handoff strategy, with a clear split between any M-mode
  shim and the S-mode Aesynx kernel.
- UART console for QEMU.
- Sv39 address-space model.
- Timer and IPI path through SBI or a reviewed local interrupt-controller path.
- PLIC/AIA interrupt-controller roadmap.
- PMP/Smepmp policy for protection boundaries where available.
- Fixed-width, endian-defined fabric ABI conformance tests reused from x86_64
  and aarch64.
- Atomic-width and memory-ordering requirements documented before any shared
  fabric queue is enabled on RISC-V.
- Explicit note that RISC-V 32 is later work after the ABI and atomic
  requirements are proven portable.

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
