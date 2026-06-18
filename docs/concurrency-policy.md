# Aesynx Concurrency Policy

Status: v0.35.1 AP startup evidence candidate

This document defines the synchronization contract that future multicore work
must follow. Current Aesynx boot remains single-core, but shared-state code must
not grow by accident before the rules exist.

Important terminology:

- **SMP hardware bring-up:** the x86_64 mechanism used to discover and start
  additional cores through APIC/IPI/platform topology support.
- **AMP kernel policy:** the intended Aesynx model after cores are online. Cores
  have explicit roles and local ownership rather than acting as interchangeable
  peers over one shared kernel state.
- **Multikernel fabric:** cross-core communication by bounded messages,
  capability-aware handoff, and directed interrupts instead of broad global
  locks.
- **Replicated authority state:** capability revocation, service ownership,
  routing, and global policy changes use epochs and message agreement rather
  than a hidden writable singleton.

Locks in this document are for unavoidable bootstrap, local owner-core,
hardware-control, and temporary shared-state boundaries. They are not permission
to evolve Aesynx into a classic shared-everything SMP kernel.

Production Aesynx must not use a cross-core shared lock to make mutable OS state
coherent. If a structure can be mutated by more than one core, the default fix is
an owner-core message, a replicated-state protocol, or an explicitly
capability-scoped shared-buffer protocol. A shared lock is only acceptable as a
documented bootstrap or hardware-control exception with a removal/migration
story in the release notes.

## Primitive Rules

- Use `aesynx-sync` early-lock primitives for pre-multicore model work.
- Locks must be guard-owned. Public unlock APIs must not exist without a
  non-forgeable guard token.
- Interrupt-masked sections must restore the previous interrupt state, not
  blindly enable interrupts on exit.
- Nested interrupt guards must leave interrupts disabled until the outer guard
  exits.
- Guard release must be strict LIFO. A non-LIFO release is a corrupted local
  synchronization state and must fail closed instead of rewriting tracker or
  interrupt-mask state.
- Poisoned synchronization state has no local unpoison path. A future real
  integration that can hit this condition must define recovery as resetting the
  owning core/domain or rebooting, not silently reusing the tracker or mask.
- `LocalInterruptMask` is a software model for host tests and policy evidence.
  It does not disable hardware interrupts by itself; real IRQ-safe locking needs
  an architecture-backed proof token that records actual interrupt masking on
  the owning core.
- Lock acquisition must follow the global rank order. Acquiring an equal or
  lower-ranked lock while a higher-ranked lock is held is a policy violation.
- Lock failures must not partially mutate protected state.
- A lock that is visible to more than one core must document why owner-core
  messaging is insufficient and why the exception does not become a permanent
  shared-kernel design.

## Lock Rank Order

Locks must be acquired from lower rank to higher rank:

| Rank | Class |
| --- | --- |
| 10 | Interrupt controller |
| 20 | Descriptor tables |
| 30 | Address space |
| 40 | Frame allocator |
| 50 | Kernel heap |
| 60 | Scheduler |
| 70 | IPC |
| 80 | Telemetry |
| 90 | AI policy |

The rank order is intentionally conservative. A subsystem may avoid taking a
lock, but if it takes more than one lock it must follow this order or split the
operation into phases.

## While Holding A Lock

Code holding a kernel lock must not:

- Block or wait for another task.
- Allocate from the kernel heap.
- Emit serial/log output.
- Call into AI policy, package, storage, driver, or userspace code.
- Invoke callbacks supplied by another subsystem.

The narrow exception is a fatal path that is about to halt and cannot safely
release state first; that path must be documented at the call site.

## Per-Core Versus Shared State

Before a subsystem becomes multicore-aware, its release notes must answer:

- Is the state per-core, role-owned, immutable after boot, or shared?
- Which core owns mutation?
- If more than one core can observe it, why is message passing insufficient?
- Which lock rank protects shared mutation?
- If cross-core mutation is proposed, why is it not modeled as a message to the
  owner core?
- Can interrupts preempt mutation on the owning core?
- Can an IRQ handler acquire the same lock?
- Are lock-held sections bounded by a fixed small capacity?
- Does `Debug` output redact authority-bearing identifiers?

## AMP Role Rules

Every online core should eventually have an explicit Aesynx role. Early roles
may be coarse, but the ownership must be visible:

- Bootstrap/control-plane core owns early topology and global handoff.
- Driver/service cores own their device queues and IRQ routing.
- Scheduler/application cores own local runnable queues.
- Idle/reserve cores are explicit capacity, not hidden scheduler spillover.

Cross-core mutation should be modeled as a message to the owning core. Direct
shared mutation needs a release-note justification and a bounded synchronization
contract.

The long-term per-core privileged component should be CPU-driver-like: local
trap/interrupt dispatch, local protection checks, local address-space switching,
and local message delivery. Complex distributed policy belongs in monitor or
service domains above that local kernel boundary, not in a broad ring-0
cross-core subsystem.

## Replicated State Rules

State that must be visible on more than one core must not become an untracked
global variable. Before adding replicated state, the release notes must define:

- The owner or coordinator.
- The epoch/version field.
- The prepare, commit, abort, or equivalent transition.
- Which peers must acknowledge before commit.
- What happens on timeout.
- What happens when a peer is dead or quarantined.
- How stale replicas fail closed.
- Which audit event links the proposal to the final state.

Full quorum consensus is not required for early machine-local releases, but
critical authority changes such as capability revoke, service-owner transfer,
and routing-policy update need an explicit agreement protocol before they can
affect multiple cores.

## Fabric Protocol Rules

Fabric messages must be versioned and bounded. Any message that may cross a
future heterogeneous boundary must avoid Rust-specific layout assumptions and
must document:

- Endianness.
- Alignment.
- Maximum payload size.
- Sequence handling.
- Rejection/dead-letter behavior.
- Redaction rules for diagnostics.

Direct function calls are not a fabric protocol.

## Service Queues

Current service queues are local fixed-capacity structures. Any future
shared-memory or multi-core queue must name the producer/consumer owner roles,
enforce owner identity before mutation or inspection, and scrub payload storage
before a vacated slot can be observed outside the current trust domain.
Release/acquire ordering evidence must be proven on the real shared
slot-validity or head/tail atomics, not only on descriptive metadata.

## Shared Memory Windows

Shared memory is allowed only as explicit capability-based sharing. It is not a
license to bypass the multikernel ownership model.

- Applications and services request shared-buffer objects, not raw physical
  frames.
- Each dispatcher or address space receives its own derived mapping capability.
- Read-only sealed buffers are the default for large zero-copy assets.
- Writable shared windows require `SHARE_WRITE`, a named synchronization
  protocol, audit events, and a revocation/TLB-shootdown plan.
- The owner core remains responsible for mutation unless ownership is
  transferred by message.
- Mapper policy must treat declared shared-buffer aliasing separately from
  accidental physical double mapping.

## Descriptor Tables

Current GDT, TSS, IDT, and double-fault IST storage is single-core. Before a
secondary core can observe or mutate descriptor state:

- GDT/TSS/IDT storage must move to per-core ownership.
- IDT gate installation must avoid non-atomic shared descriptor writes visible
  to another core.
- Each core must own its TSS and IST stacks.
- `set_ring0_stack()`-style mutation must require interrupts masked on the
  owning core and must not race task switching.

The existing `smp` feature tripwire must remain until this migration exists.

## Activation Storage

Static activation arenas and stacks are single-core boot scaffolding. Before
multi-core activation, they must move to explicit interior mutability such as a
stable `SyncUnsafeCell` equivalent with a documented owner, or to per-core
owned storage.

## Kernel Heap

The current heap remains a bounded static heap. Before multicore activation or
material heap growth:

- The backing store must move away from the current `static mut` raw-address
  pattern.
- IRQ-masked heap sections must have bounded latency, or bulk work must move
  outside the lock in a two-phase design.
- Membership checks for free-list/double-free detection must become O(1) or
  stay bounded by a documented small static capacity.
- Per-core heaps or ownership-partitioned arenas should be preferred over one
  global hot lock.

## IRQ Routing

Production interrupt routing should follow AMP ownership:

- Device IRQs route to the core that owns the driver/service domain.
- Other cores receive work through messages, not surprise device interrupts.
- Load-balancing IRQs across all cores is a fallback requiring explicit
  justification.
- A future IOMMU/DMA policy must match the owning driver core and memory domain.

## Fault Containment

Restartable services need a fault-domain contract before they become live:

- Heartbeat interval and timeout.
- Quarantine state.
- Capability revoke-on-fault.
- In-flight message cancel or replay rule.
- DMA/IOMMU cleanup before restart.
- Service rebinding rule.
- Restart budget and escalation path.

Some faults still require a full halt. The policy goal is containment for
isolated driver/service failure, not pretending corrupted kernel memory is
recoverable.

## Release Rule

Multicore milestones cannot graduate until their release notes reference this
policy and identify which single-core assumptions were removed or deliberately
kept as tripwires. Milestones that use the word SMP must state whether they mean
hardware bring-up or shared-kernel architecture; the latter is not the Aesynx
goal.
