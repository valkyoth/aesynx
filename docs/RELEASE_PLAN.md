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
- Re-run the kernel mapping policy verifier against the hardware-shaped table
  image before loading CR3.
- Switch CR3 to the Aesynx-owned root table.
- Read back CR3 in redacted form and verify that execution continues under the
  Aesynx-owned address space.
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
  response use, relocation assumptions, and QEMU evidence. If full KASLR is not
  implemented in this milestone, it remains a tagged blocker before ring 3.
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

Prove the first pairwise multikernel message-fabric contract. Until secondary
cores execute Aesynx code, this is a model-backed QEMU smoke that the later live
hardware path must preserve.

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
- No secondary core executes Aesynx code yet.

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
- Root minting restricted to registry-issued mint tickets or bootstrap-only
  audited paths; normal code must not supply arbitrary object ID, generation,
  and epoch tuples as authority.
- Enforcement APIs that return short-lived checked proof types, such as a live
  capability lease, endpoint send permit, or address-space map permit, instead
  of letting callers combine `check()` plus optional registry validation by
  convention.
- Rename or restrict table-only permission checks so they cannot be mistaken
  for complete live authority validation.
- Endpoint objects with typed `SEND`, `RECV`, call/reply, and notification
  rights; queue objects remain transport mechanisms, not the authority
  boundary.
- Kernel-stamped endpoint metadata: source principal/domain incarnation,
  protocol ID/version, sequence number, transaction ID, and bounded payload
  schema.
- Transactional capability grant protocol shape:
  - reserve pending receiver slot;
  - send grant proposal with transaction ID;
  - receiver accepts or rejects;
  - commit makes authority usable;
  - abort/timeout expires pending authority;
  - retries are idempotent.
- Mapping-authority split between memory-object capability, destination
  address-space capability, and optional executable/JIT policy authority.
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
- Host tests prove endpoint send/receive checks require endpoint rights and
  kernel-stamped source metadata.
- Host tests prove map requests require both memory-object and address-space
  authority.

Exit criteria:

- Capability IPC has a hardened identity and endpoint foundation suitable for
  the later shared-memory and multikernel fabric milestones.

### v0.37.2 - Capability-Based Shared Memory Windows

Goal:

Allow explicit zero-copy sharing between dispatchers without weakening the
AMP/multikernel rule that cores do not casually share kernel state.

Design rule:

Shared memory is never raw physical authority at the user API. A caller asks for
a typed shared-buffer object or derives authority from an existing memory
object. The kernel decides the physical backing internally and returns
capabilities with bounded range, permission, lifetime, and revocation metadata.

Deliverables:

- Shared-buffer object kind or typed memory-object mode.
- Shared memory capability derivation:
  - `SHARE_READ` for read-only shared mappings.
  - `SHARE_WRITE` only with an explicit synchronization protocol.
  - `MAP` still required before any address-space mapping is created.
- Multi-address-space mapping request that maps the same backing object into
  multiple dispatchers through separate capability grants.
- Read-only seal/freeze operation for large asset buffers, such as geometry,
  texture, model, or package-block data.
- Revocation epoch integration so unmapping/revoking a shared buffer
  invalidates every derived mapping capability.
- TLB shootdown plan for every core/address space that observed the mapping.
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

- Host tests prove read-only shared-buffer grants can be mapped by two
  dispatchers without copying.
- Host tests reject writable sharing without `SHARE_WRITE` and a declared
  synchronization protocol.
- Revocation invalidates every dispatcher mapping descriptor.
- Mapper tests distinguish allowed shared-frame aliasing from accidental
  physical-frame double ownership.

Exit criteria:

- Zero-copy shared assets are possible through explicit capabilities, while
  accidental physical aliasing still fails closed.

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
- Per-peer queue, retry, and outstanding-request bounds.
- Rejection/dead-letter message shape.
- Redacted debug output for peer identities and authority-bearing fields.

Verification:

- Host tests encode/decode fabric headers without raw pointer layout.
- Host tests reject unknown versions, oversized payloads, invalid peer roles,
  and non-monotonic sequence use where tracked.

Exit criteria:

- Aesynx has one documented internal fabric ABI before adding more cross-core
  protocols.

### v0.37.4 - Replicated Authority State Protocol

Goal:

Handle global authority changes without a hidden global lock.

Deliverables:

- Owner/coordinator rule for replicated authority records.
- Monotonic epoch records for capability revocation, service ownership, routing
  table, and policy updates.
- Prepare/commit/abort message types for critical authority changes.
- Fail-closed stale-epoch handling.
- Timeout and participant-dead handling.
- Audit events linking proposal, acknowledgement, commit, abort, and revoke.
- Explicit non-goal that full quorum/distributed consensus is later work unless
  Aesynx grows fault-tolerant peer groups.

Verification:

- Host model tests prove a revoke proposal cannot commit if a required
  participant rejects or times out.
- Host model tests prove stale epochs cannot regain authority after commit.
- Audit logs preserve proposal-to-commit linkage without exposing raw object
  IDs.

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
- Formal-verification target list for local capability checks, fabric message
  decoding, shared-buffer alias rules, and replicated authority protocols.
- Updated security controls that distinguish current QEMU scaffolding from
  future production TCB claims.

Verification:

- Documentation gate proves every planned fabric authority path names its
  privileged local mechanism and its monitor/service policy owner.
- Host model tests or static checks reject new fabric protocol definitions that
  lack an owner, timeout, stale-epoch behavior, and redaction rule.

Exit criteria:

- Aesynx has a documented path to a small per-core kernel plus isolated
  monitor/services before distributed policy becomes live.

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

- Hardware SPSC queue design that removes shared mutable `len` from producer
  and consumer hot paths.
- Monotonic producer and consumer cursors, each written by exactly one endpoint.
- Cached remote-cursor observations that are explicitly advisory and refreshed
  through acquire loads.
- Producer and consumer metadata separated onto distinct cache lines, with an
  option to place endpoint metadata on separate pages when permissions differ.
- Slot publication protocol:
  - producer writes payload;
  - producer scrubs or initializes authority-bearing padding;
  - producer performs a release store of slot sequence or tail;
  - consumer performs an acquire load before reading payload.
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
- Per-principal/service credits, deadlines, retry budgets, cancellation, and
  dead-letter records for noisy or stalled peers.
- Explicit memory-ordering tests and model checks for x86_64, aarch64, and
  RISC-V assumptions. Release/acquire evidence must correspond to actual atomic
  stores/loads, not metadata fields.

Verification:

- Host model tests prove full, empty, wraparound, stale-slot, and retry cases
  fail closed without payload reuse.
- Loom/Kani-style or equivalent bounded model tests prove the SPSC publication
  protocol does not permit payload reads before release publication.
- Cache-line layout tests prove producer and consumer hot metadata do not share
  a cache line.
- QEMU smoke reports direct-link evidence only after the atomic queue path is
  used by the kernel smoke.

Exit criteria:

- Aesynx has a queue implementation shape that can be wired to live APs without
  pretending model `Ordering` evidence is a hardware happens-before relation.

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

Deliverables:

- Two documented revoke classes:
  - Prospective revoke: no operation beginning after the revoke linearization
    point may succeed.
  - Strong revoke: when revoke returns, no stale operation, mapping, DMA
    request, delegated entry, or in-flight endpoint operation can still commit.
- Revocation messages carry object incarnation, previous epoch, new epoch,
  transaction ID, reason code, coordinator proof, and affected authority class.
- Distributed prepare/freeze/ack/commit/abort model for strong revocation.
- Mapping teardown protocol that unmaps every affected address space before
  strong revoke commit.
- Mandatory local and remote TLB invalidation acknowledgements before a
  permission reduction or unmap is reported complete.
- DMA quiesce/cancel/drain requirement before strong revocation of device-visible
  memory.
- In-flight IPC cancellation or replay policy for operations authorized before
  the revoke linearization point.
- Failure handling for dead cores or domains: timeout leads to quarantine,
  degraded fail-closed state, or system halt depending on authority class.
- Audit records linking proposal, freeze, acknowledgement, TLB/DMA cleanup,
  commit, abort, and timeout.
- Redacted diagnostics that expose counts, classes, and reason codes without
  raw object IDs, physical frames, or table slots.

Verification:

- Host model tests prove prospective revoke rejects every operation starting
  after the linearization point.
- Host model tests prove strong revoke cannot complete until modeled mappings,
  TLB acknowledgements, DMA ownership, and pending grants are resolved.
- Timeout tests prove dead participants cannot let stale authority remain
  silently usable.
- Mapper tests prove permission reduction/unmap cannot be acknowledged until
  the required flush obligation is consumed.

Exit criteria:

- Revocation semantics are precise enough for live shared memory, driver DMA,
  and cross-core capability transfer to build on them.

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
- Redacted audit events for rejected user memory access.

Expected serial:

```text
[TEST] usercopy=ok
```

Verification:

- Host tests cover valid copy, invalid pointer, cross-page copy, noncanonical
  pointer, overflow, unmapped page, and permission mismatch cases.
- QEMU smoke exercises at least one rejected user-memory access without
  corrupting kernel state.

Exit criteria:

- The kernel has one reviewed path for user memory access before userspace can
  pass pointers into kernel services.

### v0.44.5 - Domain Transition And Speculation Hardening

Goal:

Define and smoke-test the CPU and address-space hardening required before ring
3, mutually distrusting domains, or context switches can be treated as a real
security boundary.

Deliverables:

- Correct x86_64 CPUID feature detection for Intel and AMD speculative controls,
  including AMD extended-leaf IBRS bit 14, IBPB bit 12, STIBP bit 15, and SSBD
  bit 24.
- `IA32_ARCH_CAPABILITIES` field decoding plan for eIBRS/IBRS_ALL, RDCL_NO,
  MDS_NO, TAA_NO, RSBA, and newer vendor-documented controls where available.
- Context-transition mitigation policy for switching between mutually
  distrusting domains:
  - when IBPB is required;
  - when STIBP is redundant or required;
  - when RSB stuffing or BHB/BHI mitigation is required;
  - when VERW-based MDS/RFDS buffer clearing is required;
  - when L1D flush policy is required.
- Explicit SMT-domain policy for high-assurance workloads.
- KPTI or dual-root page-table plan for processors affected by rogue data-cache
  load, with a documented QEMU/general-mode fallback.
- L1TF hygiene policy: non-present PTEs are all-zero unless a reviewed
  exception exists, and physical page zero must never contain secrets.
- Hardened syscall/sysret or interrupt-return entry/exit plan with per-core
  TSS/RSP0, swapgs fencing if used, contained user faults, and no reliance on
  compiler-generated prologues for critical transition assembly.
- Full architectural state-switch plan including SIMD/FPU ownership before
  SSE/AVX is enabled in kernel or user contexts.
- Trampoline or boot-order policy that enables compatible NX/WP/SMEP/SMAP/UMIP
  protections before untrusted code or APs can execute with the final CR3.
- Redacted serial markers for supported, requested, applied, and deferred
  hardening controls.

Verification:

- Host tests cover Intel/AMD CPUID matrices, including the AMD IBRS bit-14
  regression and unrelated-bit rejection.
- Host tests cover `ARCH_CAPABILITIES` decode cases and strict/general policy
  selection.
- QEMU smoke reports boolean hardening evidence without raw MSR values.
- Documentation states which mitigations are active, which are planned, and
  which are not relevant on the current QEMU CPU model.

Exit criteria:

- The ring-3 path has an explicit security-transition hardening plan and tests
  before user address spaces become a hostile input boundary.

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
- Model evaluation has explicit fuel, deadline, memory, and output-size limits.
- Manifest step and memory ceilings are consumed by the evaluator, not only
  stored as metadata.
- Kernel scheduling path never waits synchronously for model output. Advice is
  accepted only if already available and still fresh.
- Advice records carry topology epoch, task incarnation, model version, expiry,
  and confidence bounded by the validated manifest.
- Kernel-side validator is total and bounded:
  - computes the finite admissible action set for the current scheduler state;
  - rejects stale topology/task epochs;
  - rejects invalid cores, ownership violations, affinity violations, and
    migration-budget violations;
  - executes deterministic fallback when advice is missing, stale, invalid, or
    over budget.
- Formal scheduler invariant statement: every admissible action preserves task
  ownership, queue membership, budget accounting, and security-domain placement
  constraints.
- Task time budgets are decremented or explicitly documented as advisory until
  preemption lands.
- Telemetry buffer-full behavior cannot stop scheduling. Noncritical telemetry
  drops or overwrites records with a loss counter; security audit events use a
  separate reserved channel with an explicit fail-open/fail-closed policy.
- OS-world trace emission targets zero allocation, zero locking, and zero copy
  on the normal event path: per-core single-writer binary rings, read-only
  collector mappings, sequence gaps, redaction at export, and user-space hash
  chaining or Merkle aggregation for persistent tamper evidence.

Verification:

- Host tests prove invalid, stale, over-budget, or unavailable advice falls
  back without blocking dispatch.
- Host tests prove a full noncritical telemetry buffer increments loss and does
  not prevent task dispatch.
- Host tests prove advice cannot select an invalid core or migrate a task
  outside allowed affinity/security-domain policy.
- QEMU smoke proves the scheduler continues with AI disabled, model timeout,
  and telemetry loss counters.

Exit criteria:

- AI is a bounded proposal source, and deterministic scheduler safety does not
  depend on model liveness or correctness.

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
