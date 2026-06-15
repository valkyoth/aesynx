# Aesynx Concurrency Policy

Status: v0.33.1 concurrency discipline candidate

This document defines the synchronization contract that future SMP work must
follow. Current Aesynx boot remains single-core, but shared-state code must not
grow by accident before the rules exist.

## Primitive Rules

- Use `aesynx-sync` early-lock primitives for pre-SMP model work.
- Locks must be guard-owned. Public unlock APIs must not exist without a
  non-forgeable guard token.
- Interrupt-masked sections must restore the previous interrupt state, not
  blindly enable interrupts on exit.
- Nested interrupt guards must leave interrupts disabled until the outer guard
  exits.
- Lock acquisition must follow the global rank order. Acquiring an equal or
  lower-ranked lock while a higher-ranked lock is held is a policy violation.
- Lock failures must not partially mutate protected state.

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

Before a subsystem becomes SMP-aware, its release notes must answer:

- Is the state per-core, immutable after boot, or shared?
- Which core owns mutation?
- Which lock rank protects shared mutation?
- Can interrupts preempt mutation on the owning core?
- Can an IRQ handler acquire the same lock?
- Are lock-held sections bounded by a fixed small capacity?
- Does `Debug` output redact authority-bearing identifiers?

## Service Queues

Current service queues are local fixed-capacity structures. Any future
shared-memory or multi-core queue must scrub payload storage before a vacated
slot can be observed outside the current trust domain. Release/acquire ordering
evidence must be proven on the real shared slot-validity or head/tail atomics,
not only on descriptive metadata.

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

The current heap remains a bounded static heap. Before SMP or material heap
growth:

- The backing store must move away from the current `static mut` raw-address
  pattern.
- IRQ-masked heap sections must have bounded latency, or bulk work must move
  outside the lock in a two-phase design.
- Membership checks for free-list/double-free detection must become O(1) or
  stay bounded by a documented small static capacity.
- Per-core heaps or ownership-partitioned arenas should be preferred over one
  global hot lock.

## Release Rule

SMP milestones cannot graduate until their release notes reference this policy
and identify which single-core assumptions were removed or deliberately kept as
tripwires.
