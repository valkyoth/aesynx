<p align="center">
  <b>A clean-slate Rust operating system built around capabilities, objects, and native services.</b><br>
  Not Unix in new clothes. Not Windows rewritten. A fresh standalone OS path, built carefully from the first boot.
</p>

<div align="center">
  <a href="docs/IMPLEMENTATION_PLAN.md">Implementation Plan</a>
  |
  <a href="docs/RELEASE_PLAN.md">Release Plan</a>
  |
  <a href="docs/releases/README.md">Release Notes</a>
  |
  <a href="docs/security-controls.md">Security Controls</a>
  |
  <a href="SECURITY.md">Security</a>
</div>

<br>

<p align="center">
  <img src="./.github/images/aesynx.webp" alt="Aesynx overview">
</p>

# Aesynx

Aesynx is a Rust `no_std` operating-system project with a clean-slate goal: a
standalone OS that does not begin by copying Unix, Linux, or Windows. Its native
model is built around explicit capabilities, per-core ownership, service queues,
driver isolation, an immutable object graph, structured userspace, AI-ready
telemetry, and an AMP/multikernel direction from day one.

The long-term goal is a different kind of general-purpose system, not a compatibility
skin over old assumptions. Paths, processes, packages, drivers, snapshots, and
automation should be native Aesynx concepts first. Unix or Linux compatibility
can exist later as an isolated service, but it must not define the kernel,
userspace, or security model.

Aesynx is also explicitly not planned as one huge OS binary: components should
remain separately identified, signed, versioned, updateable, and
rollback-capable.

The first major milestone is a serious x86_64 QEMU release with a coherent
security model, clear non-claims, and release gates that block tagging until
checks and pentest evidence are complete. The project is early, but the direction
is intentionally standalone.

Aesynx is licensed under the European Union Public Licence 1.2.

## What Works Today

`v0.35.5` is the current AP startup dispatch candidate.

Current boot path:

- Builds a release-profile freestanding `x86_64-unknown-none` kernel ELF.
- Packages the kernel into a Limine ISO and records Rust, Limine, xorriso, and
  QEMU versions in the image manifest.
- Boots in QEMU and validates kernel-owned serial markers.
- Normalizes Limine handoff metadata into Aesynx `BootInfo`.
- Installs basic x86_64 GDT/TSS/IDT state, remaps and masks the legacy PIC,
  detects local APIC presence, and publishes checked IRQ vector allocation.
- Handles a returning breakpoint exception.

Diagnostics and timer smoke:

- Panic smoke emits bounded, escaped, redacted panic diagnostics.
- Exception smoke emits redacted CR2 presence/page-offset, CR3 low bits,
  public RFLAGS, interrupt state, and decoded page-fault bits.
- Timer smoke programs PIT IRQ0 in QEMU, observes three controlled ticks,
  converts ticks into monotonic nanosecond values, wakes one bounded sleep
  request, acknowledges each interrupt, and disables the smoke IRQ.

Memory and mapping model:

- Boot memory accounting reports checked total, usable, reserved, kernel,
  bootloader, framebuffer, ACPI, bad, and frame-count values before
  `[TEST] memory-map=ok`.
- The bounded bitmap frame allocator smoke verifies one-frame allocation/free,
  contiguous allocation/free, debug state, double-free detection, and atomic
  failure behavior before `[TEST] frame-allocator=ok`.
- The bounded x86_64-shaped page-table mapper model covers map, unmap, protect,
  contiguous range map/protect/unmap, typed and checked root-table identity,
  checked status accounting, fail-closed translation, checked byte-range
  translation, permission lookup/change, mapped/unmapped range checks,
  candidate kernel/user address-space checks, physical-alias prevention,
  redacted debug output, consistency audit, empty-table reclamation, and
  explicit TLB flush targets before `[TEST] page-table=ok`.

Kernel mapping policy:

- Linker-exported section boundaries feed a safe `aesynx-mm` policy descriptor.
- QEMU validates section layout, text RX, rodata read-only/NX, data RW/NX,
  reserved heap, guard page, and null-page invariants.
- Every v0.16 paging-policy-model `*_ok=true` marker plus
  `[TEST] paging-policy-model=ok` is required before normal boot success.

Address-space activation and CPU hardening:

- Audited mapper state streams into x86_64 hardware-shaped page tables in a
  static activation arena.
- The kernel switches to a private activation stack and loads an Aesynx-owned
  CR3 root before terminal boot success.
- The activation stack is mapped RW/NX with an unmapped guard page and QEMU
  requires `[TEST] kernel-stack-guard=ok`.
- Post-CR3 CPU hardening enables NX, write-protect, and CPUID-gated
  SMEP/SMAP/UMIP where supported, requests CPUID-gated Spectre-class
  `IA32_SPEC_CTRL` bits for IBRS/STIBP/SSBD where available, detects Intel and
  AMD IBPB support, reports boot-time IBPB attempt evidence separately from
  read-back-verifiable state, and keeps `ARCH_CAPABILITIES` evidence redacted
  before `[TEST] cpu-hardening=ok`.
- Early entropy policy classifies x86_64 `RDRAND`/`RDSEED` support behind
  CPUID checks, distinguishes deterministic anti-confusion generation counters
  from attacker-unpredictable random tokens, treats raw hardware entropy as seed
  evidence only, keeps random-token policy disabled until runtime hardware and
  DRBG self-tests exist, and reports only redacted booleans before
  `[TEST] entropy-policy=ok`.
- A bounded static kernel heap is initialized after CR3 activation and CPU
  hardening; fixed slab classes cover small allocations, page-sized runs cover
  larger allocations, and QEMU smokes `Box`, `Vec`, `BTreeMap`, slab reuse,
  page-run allocation/free, stress allocation/free, invalid-free telemetry,
  double-free detection, and explicit OOM rejection before `[TEST] heap=ok`.
- A fixed-capacity kernel capability table is smoke-tested after CR3 activation:
  root creation, permission checks, audited derivation, audited grant, audited
  revoke lifecycle, cross-owner child authority reduction, stale `CapId`
  rejection, redacted status telemetry, and cap-fault telemetry are required
  before `[TEST] cap=ok` and `[TEST] cap-audit=ok`.
- Memory capability enforcement now gates a mapper-facing checked mapping
  descriptor API: a derived subrange capability with `MAP|READ` can authorize
  one read-only mapping descriptor, while missing READ, missing WRITE, and
  range-escape requests fail before mapping construction and before
  `[TEST] memory-cap=ok`.
- Kernel task modeling now creates multiple task objects, validates local run
  queue admission, preserves FIFO pop order, moves runnable tasks into message
  and timer wait queues, wakes one message waiter back to runnable state, and
  rejects wrong-core and zero-task-ID admission before `[TEST] task-model=ok`.
- The cooperative executor smoke runs two kernel tasks through deterministic
  local round-robin dispatch, yields the current task back to the run queue,
  moves one task into the timer wait queue, wakes it back to runnable state, and
  verifies `[TEST] cooperative-sched=ok`.
- The scheduler telemetry smoke records bounded round-robin scheduler decision
  records, task scheduled-run counters, and a core run-queue counter before
  `[TEST] scheduler-telemetry=ok`.
- The telemetry event smoke records versioned boot-phase, capability-fault, and
  scheduler-decision events in a bounded per-core ring, then dumps a redacted
  serial summary plus schema-v1 `trace-event` records before
  `[TEST] telemetry-events=ok`.
- `tools/trace-decode` and `cargo xtask trace-decode <serial-log>` decode the
  QEMU serial trace into line-based offline analysis output while rejecting
  scheduler events that expose raw task identity.
- The AI policy interface now has a no_std model manifest, redacted manifest
  diagnostics, fixed-point scheduler feature validation, explicit safety
  limits, a deterministic fallback contract, and a non-AI fixed-point
  scheduler heuristic. QEMU accepts a fixed-point scheduler manifest, rejects
  one without fallback, verifies zero-confidence local fallback, and records
  bounded heuristic redacted score/core evidence before
  `[TEST] ai-policy=ok`.
- The concurrency discipline smoke validates safe early interrupt guards,
  nested interrupt masking behavior, guard-owned early locks, IRQ-masked lock
  acquisition, and lock-order rejection before `[TEST] concurrency=ok`.
- The AMP core smoke records the bootstrap core role, x86_64 capability
  metadata, owner-scoped per-core registry state, local telemetry, and sealed
  boot-barrier arrival before `[TEST] amp-core=ok`.
- The multicore topology smoke runs under QEMU `-smp 4` and models four visible
  cores with separate hardware-online and Aesynx role-assignment state before
  `[TEST] multicore-topology=ok`. The AP startup state table now audits the
  joint hardware/assignment/local-state model and reports `state_table_ok=true`
  before boot success. This is topology/ownership evidence; APs do not execute
  Rust code yet.

Fuzz and property gates:

- `cargo xtask fuzz-smoke` runs bounded BootInfo normalization fuzz seeds and
  deterministic byte-shaped mutations.
- Mapper property tests sweep map/unmap round trips, failed-operation
  atomicity, duplicate-frame rejection, range-walk bounds, and audit drift
  detection.

| Area | Status | Notes |
| --- | --- | --- |
| Rust workspace | Active | Modular crate layout with no root `src/` implementation pile. |
| Toolchain | Active | Stable Rust `1.96.0`, edition 2024, resolver `3`, and `x86_64-unknown-none` for the first boot ELF. |
| Kernel crate policy | Active | Crates under `crates/` must be `no_std`, deny unsafe by default, and avoid external dependencies without exceptions. |
| Capability model | Tagged | `v0.22.0`; private non-copy authority values, checked `CapId` slot/generation layout, fixed-capacity kernel capability table, permission validation, audited derive/grant/revoke paths, slot generation stale-id rejection, revoke authority checks, redacted capability/table/audit debug output, cap-fault telemetry, and memory-map authorization based on capability kind, range, and `MAP`/READ/WRITE/EXECUTE permissions. |
| Object model | Tagged | `v0.23.0`; host-side `aesynx-object-model` crate with nonzero redacted object IDs, explicit object kinds, immutable node metadata, duplicate/self-reference rejection, append-only graph insertion, missing-reference rejection, and reachability over references plus predecessor links. |
| Kernel object registry | Tagged | `v0.24.0`; no_std fixed-capacity `aesynx-object` registry with memory, endpoint, queue, and task-placeholder objects, local core ownership, create/list/delete operations, generation-backed slot recycling, redacted object debug output, and capability reference resolution against object ID, kind, generation, revocation epoch, and permission. |
| Kernel service queues | Tagged | `v0.26.0`; no_std `aesynx-ipc` service requests/completions, typed live-core validation before message-header routing, owner-core checks before queue mutation/inspection, fixed-capacity log/timer/object queue skeletons, fail-closed full/empty/unsupported-service behavior, release/acquire ordering evidence, redacted service debug output, and QEMU `[TEST] service-queue=ok`. |
| Kernel task model | Tagged | `v0.27.0`; no_std `aesynx-sched` task objects, checked state transitions, redacted task debug output, fixed-capacity local run queues, message/timer/object wait queues, fail-closed queue admission, and QEMU `[TEST] task-model=ok`. |
| Cooperative executor | Tagged | `v0.28.0`; local cooperative executor, deterministic round-robin dispatch, yield, timer sleep/wake, linear task ownership preservation, and QEMU `[TEST] cooperative-sched=ok`. |
| Scheduler telemetry | Tagged | `v0.29.0`; bounded scheduler decision records, deterministic round-robin decision reasons, task scheduled-run counters, core run-queue telemetry, redacted decision debug output, and QEMU `[TEST] scheduler-telemetry=ok`. |
| Telemetry event schema | Tagged | `v0.30.0`; versioned event IDs, event headers, boot-phase/capability/scheduler payloads, bounded per-core event ring, redacted scheduler event debug output, and QEMU `[TEST] telemetry-events=ok`. |
| Trace export tool | Tagged | `v0.31.0`; kernel emits schema-v1 `trace-event` serial records, `tools/trace-decode` produces line-based offline output, core IDs stay visible as local scheduling context, and scheduler selected-task export must remain `<redacted>`. |
| Scheduler policy model | Tagged | `v0.33.0`; no_std model manifests, redacted manifest diagnostics, metadata-presence hash/signature wrappers, fixed-point-only accepted model kinds, scheduler-domain metadata and fallback gates, fixed-point feature validation, manifest-enforced model confidence ceilings, deterministic fallback, bounded non-AI scheduler heuristic scoring, decision records, a disable switch, redacted heuristic serial evidence, and QEMU `[TEST] ai-policy=ok` prove fallback and heuristic evidence before any AI model can influence scheduling. |
| Concurrency discipline | Tagged | `v0.33.1`; safe `aesynx-sync` early-lock primitives, previous-state interrupt guards, nested interrupt masking behavior, guard-owned LIFO release with local poison on release-order violation, lock-rank validation, policy docs for lock-held behavior and AMP/multikernel-on-SMP-hardware migrations, and QEMU `[TEST] concurrency=ok`. |
| AMP core data structures | Tagged | `v0.34.0`; no_std `aesynx-core` models core roles, heterogeneous capability metadata, `CoreLocal`, owner-scoped core registries, per-core local telemetry, boot barriers, and QEMU `[TEST] amp-core=ok` for the bootstrap core without enabling multicore execution. |
| AP startup state table | Tagged | `v0.35.3`; xtask launches QEMU with `-smp 4`, manifests record `qemu_smp_cpus=4`, and `aesynx-core` now requires owner-issued `CoreStartupTicket` plus matching `CoreStartupArrival` evidence before a staged core can become hardware-online. AP launch resources are owner-scoped, staged-only, stack/watchdog checked, descriptor-table readiness is explicit, and topology mutation validates the joint hardware/assignment/local-state table before and after state changes. QEMU reports `state_table_ok=true`, `startup_evidence_ok=true`, `ap_preflight_ok=true`, and `ap_execution_blocked_ok=true` before `[TEST] multicore-topology=ok`. |
| Multi-domain hardening blockers | Tagged | `v0.35.4`; x86_64 CPU hardening now models CPUID-gated IBRS/IBPB, STIBP, SSBD, and `ARCH_CAPABILITIES`, admits `IA32_SPEC_CTRL` plus `IA32_PRED_CMD`, requests supported SPEC_CTRL bits, issues IBPB when available, verifies read-back state, and reports only redacted booleans in QEMU evidence. Pentest fixes also made the IRQ proof linear and smoke-policy gated, removed equality from hash/signature-bearing AI manifests, restored non-online quarantine semantics, split heap accounting-overflow telemetry from free-list corruption telemetry, cached per-class slab-page bounds for free-list scans, and hardened manifest field validation. |
| AP startup dispatch token | Active candidate | `v0.35.5`; `ApStartupPreflight` now mints a sealed, non-`Copy` dispatch token only for the topology owner and only when AP execution is allowed. Shared-bootstrap descriptor tables still block token creation, and QEMU requires `ap_dispatch_token_blocked_ok=true` beside `ap_execution_blocked_ok=true` before `[TEST] multicore-topology=ok`. |
| Memory model | Model active | Page flags make writable+executable and user-global mappings unrepresentable; long-term memory should become object-native, purpose-tagged, capability-scoped, and snapshot-aware. |
| OS world model | Planned | Kernel-stamped facts should feed a native world service so Aesynx can explain boot, memory, packages, drivers, capabilities, snapshots, and policy decisions without putting a database in ring 0. |
| IPC model | Model active | Kernel-stamped message headers, caller requests, and bounded inline payloads. |
| Bytecode model | Model active | Fuel limit and capability-typed permission checks. |
| Logging model | Model active | Bounded single-record log messages. |
| Build path | Active | x86_64 target metadata, linker script, Cargo config validation, stable freestanding kernel ELF build, and an optional nightly custom-target probe. |
| QEMU first boot | Active | `cargo xtask image` creates a release-profile Limine ISO and `cargo xtask qemu` launches QEMU with `-smp 4`; the smoke verifies descriptor/IRQ setup, checked memory-map/frame-allocator/page-table markers, every v0.16 paging-policy-model `*_ok=true` marker, `[TEST] paging-policy-model=ok`, `[TEST] kernel-cr3=ok`, `[TEST] kernel-stack-guard=ok`, `[TEST] bootinfo=ok`, `[TEST] boot=ok`, post-CR3 CPU hardening, `[TEST] cpu-hardening=ok`, v0.18.1 entropy policy evidence with `[TEST] entropy-policy=ok`, the v0.18 kernel heap smoke with `[TEST] heap=ok`, the v0.20 kernel capability-table smoke with `[TEST] cap=ok`, the v0.21 memory-capability mapping-descriptor gate with `[TEST] memory-cap=ok`, the v0.22 capability audit/telemetry gate with `[TEST] cap-audit=ok`, the v0.26 kernel service queue smoke with `[TEST] service-queue=ok`, the v0.27 task-model smoke with `[TEST] task-model=ok`, the v0.28 cooperative scheduler smoke with `[TEST] cooperative-sched=ok`, the v0.29 scheduler telemetry smoke with `[TEST] scheduler-telemetry=ok`, the v0.30 telemetry event schema smoke with `[TEST] telemetry-events=ok`, v0.31 decodable `trace-event` serial records, v0.33 AI policy heuristic/fallback evidence with `[TEST] ai-policy=ok`, v0.33.1 concurrency discipline evidence with `[TEST] concurrency=ok`, v0.34 AMP core evidence with `[TEST] amp-core=ok`, and v0.35 four-core topology/AP-preflight evidence with `[TEST] multicore-topology=ok` from Rust `_start`. |
| Fuzz/property smoke | Active candidate | `v0.16.1`; `cargo xtask fuzz-smoke` runs BootInfo fuzz seeds, deterministic BootInfo byte mutations, and mapper property sweeps before live CR3 activation. |
| BootInfo normalization | Tagged | Limine memory map, executable address, HHDM, RSDP, and framebuffer metadata normalize into dependency-free `aesynx-boot` structures. |
| Early diagnostics | Tagged | Boot phase tracking and `cargo xtask qemu --panic-smoke` verify readable panic output with `[TEST] panic=ok`. |
| GDT and TSS | Tagged | Early x86_64 boot installs an Aesynx-owned GDT, TSS, and double-fault IST stack, verified with `[TEST] gdt=ok`. |
| IDT and exceptions | Tagged | Early x86_64 boot installs an IDT with deterministic halt-and-log catch-all entries for every vector, handles breakpoint, page-fault, and double-fault vectors, and verifies `[TEST] exception=ok`. |
| Fault decoding | Tagged | `v0.9.0`; page-fault smoke prints redacted CR2 presence/page offset, CR3 low bits, public RFLAGS, interrupt state, and decoded error bits. |
| Interrupt controller baseline | Tagged | `v0.10.0`; remaps/masks legacy PIC IRQs, detects local APIC presence, defines checked IRQ vectors, and exposes an EOI path. |
| Timer ticks | Tagged | `v0.11.0`; opt-in QEMU timer smoke programs PIT IRQ0, records a tick counter, and verifies `timer tick 1..3` plus `[TEST] timer=ok`. |
| Monotonic time and sleeps | Tagged | `v0.12.0`; converts timer ticks into monotonic instants, schedules a bounded sleep request, and verifies `timer delayed-log`, `[TEST] sleep=ok`, and `[TEST] timer=ok`. |
| Physical memory map | Tagged | `v0.13.0`; rejects invalid/overlapping regions and reports checked total/usable/reserved bytes, frame counts, and kernel/bootloader reserved accounting with `[TEST] memory-map=ok`. |
| Bitmap frame allocator | Tagged | `v0.14.0`; safe `aesynx-mm` bitmap allocator model plus QEMU smoke for bounded early alloc/free, contiguous allocation, debug states, double-free detection, and atomic failure behavior with `[TEST] frame-allocator=ok`. |
| Page table mapper | Tagged | `v0.15.0`; safe bounded `aesynx-mm` page-table mapper model with x86_64-shaped tables, mapper-issued typed root-table identity, checked root-table identity, checked status accounting, non-empty kernel and user address-space candidate preflights, audit-backed map/unmap/protect, fail-closed translation, checked contiguous byte-range translation, audit-backed permission lookup, contiguous range map/protect/unmap plus lookup, upfront range validation, bounded range walks, audit-backed unmapped range checks, audit-backed mapped-range checks, page-presence checks, kernel-only policy checks, kernel high-half user-access guard checks, user low-half kernel-privilege guard checks, no-user-space policy checks, no-executable policy checks, no-writable policy checks, no-device policy checks, no-global policy checks, map-time no-physical-alias policy checks with const-capacity bounded side-index audit, audit-backed kernel-range policy checks, audit-backed user-range policy checks, write-protected range checks, non-executable range checks, executable range checks, normal-memory range checks, local range checks, high-half kernel-space checks, low-half user-space checks, read-only mapping visit, redacted mapping summaries, redacted page-table debug output, virtual range permission verification, fail-closed leaf decoding including hardware Accessed/Dirty bits, permission lookup/change, consistency audit, empty-table reclamation, explicit TLB flush targets, conservative TLB flush merging, and QEMU smoke with `[TEST] page-table=ok`. |
| Kernel mapping policy | Tagged | `v0.16.0`; linker-exported section boundaries feed a safe `aesynx-mm` policy descriptor that verifies section layout, text RX, rodata read-only/NX, data RW/NX, reserved heap, guard page, and null-page invariants before `[TEST] paging-policy-model=ok`. |
| Kernel-owned address space | Tagged | `v0.16.2`; audited mapper state now streams redacted x86_64 hardware-shaped page-table entries using Limine's normalized kernel physical placement, copies used tables into a static activation arena, switches to a private kernel activation stack, loads an Aesynx-owned CR3 root, and QEMU requires `hardware_copied=true` plus `[TEST] kernel-cr3=ok`. |
| CPU hardening and stack guards | Tagged | `v0.16.3`; CPUID-gated EFER.NXE, CR0.WP, SMEP, SMAP, and UMIP policy is host-tested and QEMU-smoked with redacted read-back `cpu-hardening` booleans; the terminal activation stack is mapped separately with an unmapped guard page and `[TEST] kernel-stack-guard=ok`. |
| Limine handoff module split | Tagged | `v0.16.4`; Limine ABI structs, constants, request statics, link-section markers, and ABI assertions now live in a private `limine/abi.rs` module while normalization flow remains in `limine.rs`. |
| Early heap | Tagged | `v0.17.0`; bounded static bump allocator, global allocator wrapper, post-CR3 `Box`/`Vec`/`BTreeMap` smoke, and explicit OOM rejection before `[TEST] heap=ok`. |
| Slab/page heap | Tagged | `v0.18.0`; bounded static reusable kernel heap with fixed slab classes, page-sized runs, aggregate stats, invalid-free and free-while-free double-free telemetry, zero-before-reuse host coverage, and QEMU allocation/free stress before `[TEST] heap=ok`; allocation-epoch stale raw-pointer detection remains future work. |
| Early entropy semantics | Tagged | `v0.18.1`; safe entropy policy crate, x86_64 CPUID classification for `RDRAND`/`RDSEED`, explicit runtime hardware and DRBG self-test evidence, deterministic anti-confusion generation counters, random-token gating that rejects CPUID-only evidence and raw hardware entropy without DRBG output, and redacted QEMU telemetry before `[TEST] entropy-policy=ok`. |
| Native snapshots | Planned | Content-addressed object roots make snapshots and rollback object-layer primitives rather than path-first filesystem features. |
| Native package manager | Planned | Content-addressed package objects, declarative generations, explicit tracks, SBOM/provenance, and capability manifests. |
| Future bootloader | Planned | Limine is current; a future Rust UEFI bootloader should be a minimal security gateway for signed/measured Aesynx boot capsules. |
| Post-quantum readiness | Planned | Crypto-agile boot, package, update, and identity metadata with room for hybrid classical plus post-quantum validation. |
| Supply-chain checks | Active | `cargo deny`, `cargo audit`, SBOM generation, Dependabot, SHA-pinned GitHub Actions, and CodeQL default Rust workflow. |
| Release gate | Active | Tags require local checks, SBOM, CodeQL on GitHub, and a passing pentest report for the exact commit. |

## Planned Next

| Area | Status | Target |
| --- | --- | --- |
| Real arch mechanisms | Planned | Core identity, timestamp, production page tables, and CPU setup. |
| Capability services | Planned | Concrete revocation epoch store, audit backend, object registry, and authenticated call paths. |
| Native userspace | Planned | `aesh`, structured pipelines, WASM components, and capability-scoped command execution. |
| OS world service | Planned | Signed/versioned facts, branchable worlds, policy-aware queries, context packs, and AI-safe explanations over deterministic OS evidence. |
| Package manager | Planned | `aepkg`/`aepkgd` roadmap for search, install, update, rollback, repair, and future store UI. |
| Post-quantum readiness | Planned | Crypto-agile signature envelopes and trust policy before signed boot capsules, package registries, or update metadata. |

## Local Checks

Run the full repository gate:

```bash
scripts/checks.sh
```

Generate the source SBOM:

```bash
scripts/generate-sbom.sh
```

Validate the current kernel build path:

```bash
cargo xtask build-kernel
```

Create and smoke-test the current Limine QEMU image:

```bash
cargo xtask image
cargo xtask qemu
```

Run the full current QEMU smoke suite:

```bash
cargo xtask qemu-suite
```

Run the deliberate panic diagnostics smoke:

```bash
cargo xtask qemu --panic-smoke
```

Run the deliberate exception smoke:

```bash
cargo xtask qemu --exception-smoke
```

Run the controlled timer smoke:

```bash
cargo xtask qemu --timer-smoke
```

These commands require Limine 12.3.2 or newer, xorriso, and
`qemu-system-x86_64`. The generated manifest records the exact Rust, Limine,
xorriso, and QEMU version banners.

Try the documented custom-target experiment when a nightly toolchain is
available:

```bash
cargo xtask build-kernel --custom-target-probe
```

After a pentest report is completed for a tag:

```bash
cargo xtask release-ready v0.35.5
```

## Security Posture

Aesynx treats boot, memory, capabilities, IPC, driver authority, userspace ABI,
WASM execution, telemetry, AI policy, build tooling, GitHub workflows, and
dependency metadata as high-risk. The project prefers internal kernel
primitives, narrow unsafe boundaries, no ambient authority, explicit
capabilities, and small modules that can be reviewed and tested.

Every release tag is blocked until the exact commit being tagged has a passing
pentest report in `security/pentest/<tag>.md`.

## Documentation

- [Implementation Plan](docs/IMPLEMENTATION_PLAN.md)
- [Userspace Vision](docs/userspace-vision.md)
- [SDK Roadmap](docs/sdk-roadmap.md)
- [Memory Model Roadmap](docs/memory-model-roadmap.md)
- [OS World Roadmap](docs/os-world-roadmap.md)
- [Package Manager Roadmap](docs/package-manager-roadmap.md)
- [Driver Roadmap](docs/driver-roadmap.md)
- [Multikernel Fabric Roadmap](docs/multikernel-fabric-roadmap.md)
- [Concurrency Policy](docs/concurrency-policy.md)
- [Release Plan](docs/RELEASE_PLAN.md)
- [Architecture Decisions](docs/ARCHITECTURE_DECISIONS.md)
- [Build Skeleton](docs/build-skeleton.md)
- [QEMU Image Skeleton](docs/qemu-image-skeleton.md)
- [First Serial Boot](docs/first-serial-boot.md)
- [BootInfo Normalization](docs/bootinfo-normalization.md)
- [Early Diagnostics](docs/early-diagnostics.md)
- [Release Candidate Notes Archive](docs/releases/README.md)
- [Telemetry Event Schema](docs/telemetry-event-schema.md)
- [v0.35.3 Release Candidate Notes](docs/releases/v0.35.3-rc.md)
- [v0.35.4 Release Candidate Notes](docs/releases/v0.35.4-rc.md)
- [v0.35.5 Release Candidate Notes](docs/releases/v0.35.5-rc.md)
- [Bootloader Roadmap](docs/bootloader-roadmap.md)
- [Storage Roadmap](docs/storage-roadmap.md)
- [Hosted Execution Roadmap](docs/hosted-execution-roadmap.md)
- [Post-Quantum Readiness](docs/post-quantum-readiness.md)
- [Security Policy](SECURITY.md)
- [Threat Model](docs/threat-model.md)
- [Security Controls](docs/security-controls.md)
- [Supply-Chain Security](docs/supply-chain-security.md)
- [Kernel Engineering Policy](docs/kernel-engineering-policy.md)
- [Unsafe Policy](docs/unsafe-policy.md)
- [Modularity Policy](docs/modularity-policy.md)
- [Licensing Notes](docs/licensing.md)
- [License](LICENSE)
- [Initial Idea](docs/initial-idea.md)
