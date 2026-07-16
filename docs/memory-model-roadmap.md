# Aesynx Memory Model Roadmap

Status: design direction

Aesynx should treat memory as a first-class system object, not as anonymous
bytes hidden behind process-private heaps. Linux and Windows already provide
virtual memory, page permissions, and process isolation. Aesynx should build on
those lessons, but its native model should make authority, purpose, sharing,
revocation, and lifetime explicit from the first real allocator onward.

The goal is not that the kernel magically understands every byte in every
program. The realistic goal is stronger: every meaningful allocation and mapping
should be created through an API that records why it exists, who can use it,
how it may be shared, whether devices may access it, whether it can execute,
whether it contains secrets, and whether it participates in snapshots.

## Core Principle

Memory is an object with declared authority.

A memory object should have:

- Object identity.
- Owner service or task.
- Capability-controlled permissions.
- Purpose classification.
- Lifetime policy.
- Sharing policy.
- Revocation generation.
- Snapshot policy.
- Device/DMA policy.
- Optional sensitivity label.

The default memory policy should be restrictive:

- Private.
- Non-executable.
- Non-DMA.
- Not shared.
- Zeroed before reuse.
- Excluded from persistence and snapshots unless declared.
- Revocable through capability epochs.

Anything more powerful should require an explicit capability.

## Memory Classes

The kernel should avoid a single generic allocation bucket for everything. The
initial class set should be small, but semantically clear:

| Class | Purpose | Default Policy |
| --- | --- | --- |
| `kernel` | Kernel-owned internal state | Kernel-only, non-user, non-DMA |
| `code` | Executable text | Read/execute, never writable after load |
| `rodata` | Immutable data | Read-only, non-executable |
| `data` | Mutable service data | Read/write, non-executable |
| `stack` | Kernel or task stack | Guarded, non-executable |
| `heap` | General mutable allocation | Private, non-executable |
| `ipc` | Message or shared transfer buffer | Capability-scoped, revocable |
| `dma` | Device-visible memory | IOMMU-scoped, never default |
| `secret` | Keys, tokens, credentials | Zero-on-free, non-snapshot, non-core-dump |
| `object-cache` | Cached object-store blocks | Integrity checked, reclaimable |
| `wasm-linear` | WASM component linear memory | Sandboxed, bounded, non-executable |
| `device-mmio` | Device registers | Driver-capability only |
| `framebuffer` | Display memory | Graphics authority only |

These classes are policy handles. They do not need to become over-engineered
types on day one, but the allocator and mapper should grow toward this shape.

## Capabilities

Mapping memory should be an authority decision, not an ambient right.

Separate capabilities should exist for:

- Allocating physical frames.
- Creating a memory object.
- Mapping a memory object into an address space.
- Changing mapping permissions.
- Sharing memory with another service.
- Transferring ownership.
- Revoking a mapping.
- Marking code executable.
- Creating DMA-visible memory.
- Including memory in a snapshot.
- Persisting memory-backed state.

No service should gain executable, DMA, persistent, or cross-service sharing
rights by accident.

Production mapping is one reference-monitor operation. A caller should not
separately authorize a `MemoryMapRequest`, extract raw addresses, and then call
the mapper with ordinary physical/virtual addresses by convention. The live map
path must consume a checked proof that composes:

- memory-object authority over backing object offset and length;
- address-space authority over the destination virtual range;
- requested access rights and cache/device/confidential attributes;
- executable/JIT policy authority where applicable;
- current object generation and revocation epoch;
- address-space incarnation and ASID/PCID context.

Permission reduction and unmap also create TLB obligations. A live mapper must
not report success to callers until required local and remote invalidation
acknowledgements have completed or the operation has failed closed into a
documented quarantine/degraded state.

W^X is a memory-object invariant, not only a single-PTE invariant. A physical
frame or memory object must not be writable in one address space while
executable in another. Executable transition requires freezing writable
mappings, completing TLB invalidation, performing any architecture-required
instruction-cache synchronization, sealing the memory object, and then creating
executable mappings.

Aliases of the same memory object must also agree on cacheability/device memory
attributes unless an architecture-specific reviewed exception exists. Page-table
pages themselves are protected objects: once installed, they must not be mapped
user-accessible, DMA-visible, or ordinary writable kernel data.

Low-level raw frame allocation is different from authority to use memory. The
Barrelfish experience is a warning here: making every physical-memory operation
a fine-grained, globally coordinated capability protocol adds complexity and
coordination latency. Aesynx should keep per-core or owner-local frame
allocation fast and simple, while capabilities govern higher-level memory
objects, mappings, sharing, DMA visibility, ownership transfer, executable
permission, and revocation.

## Required Invariants

These invariants should become release-gated as the memory subsystem matures:

- Writable and executable must not be enabled on the same live mapping.
- User and global must not be combined for user mappings.
- Freed memory must not be reused without zeroing or proven overwrite.
- Secret memory must not be snapshotted, logged, dumped, or exposed to DMA.
- DMA buffers must be bounded by IOMMU policy before real hardware support.
- Revoked mappings must fail on the next validation boundary.
- Shared memory must name both participants and the transfer mode.
- Object-store cache memory must be verifiable against content hashes.
- WASM linear memory must stay bounded by the component's declared limits.
- Memory accounting must fail closed on integer overflow.

## Page-Table And Address-Space Roadmap

The v0.15 mapper is intentionally a bounded model, not the final production
address-space implementation. Future mapper work should preserve the strict
audit-first shape while adding hardware features only when they have a clear
security purpose and tests.

Priority items:

1. Replace the bounded sorted frame side index with a scalable `no_std`
   structure once map/unmap volume grows. The v0.15 index has an explicit
   mapper const-generic capacity, with a conservative default for QEMU smoke,
   and gives bounded duplicate-frame lookup plus audit-time table/index
   agreement checks. Its sorted array still shifts entries on insert/remove, so
   a future intrusive tree or fixed-capacity B-tree should make
   exclusive-frame checks fast enough for syscall and address-space activation
   paths.
2. Add typed huge-page support for 2 MiB and 1 GiB leaves. This must use
   explicit page-size types, strict alignment checks, mixed-size alias checks,
   and reviewed split/unmap semantics. Huge pages should reduce TLB pressure
   and audit surface area, but they must not weaken the 4 KiB mapper
   invariants.
3. Add address-space identifiers with PCID and INVPCID support on x86_64 once
   real address spaces exist. `TlbFlush::AddressSpace` is currently a
   conservative whole-address-space shape; future arch code should map it to
   tagged TLB invalidation instead of flushing more than necessary.
4. Enable CR4 hardening features such as SMEP, SMAP, and UMIP when supported.
   The mapper's `USER` flag plumbing is only the prerequisite. The arch crate
   must still enforce that the kernel cannot execute user pages and cannot
   access user pages without an explicit audited access window.
5. Add optional protection-key support. PKU and PKS are page-granularity
   hardware domains, not sub-page isolation, but they are a strong fit for fast
   intra-address-space isolation between WASM linear memories, runtime
   compartments, and kernel subsystems without rewriting page tables or
   flushing the TLB.
6. Add confidential-computing memory attributes without exposing raw
   vendor-specific bits as the generic ABI. Aesynx should model attributes such
   as private, shared, encrypted, or confidential, and let the arch/backend
   translate those into AMD SEV-SNP C-bit handling, Intel TDX private/shared GPA
   rules, or a no-op on unsupported hardware.
7. Investigate Linear Address Masking and related pointer-tagging support only
   as an optional fast-path hint. Pointer tags may carry capability-generation
   or diagnostic metadata, but they must never become the sole security
   boundary; authoritative capability/object generation checks remain the
   source of truth.
8. Add property-based and model-checked verification for mapper invariants.
   The first proof targets should be map/unmap round trips, failed-operation
   atomicity, duplicate-physical-frame exclusion, audit detection of raw table
   corruption, and table/index agreement. Host property tests should come
   first, then Kani/CBMC-style bounded proofs where practical.

## Copy-Free IPC

Aesynx should prefer transferring memory object authority over copying bytes.

For small messages, bounded inline IPC is fine. For larger data, the intended
model is:

1. Sender creates or owns a memory object.
2. Sender grants a receiver a scoped capability to that object.
3. Kernel records the transfer in an audit path.
4. Receiver maps the object with only the granted permissions.
5. Sender may retain, share, transfer, or revoke according to the capability.

This supports fast IPC without making shared mutable memory the default.

## Snapshot-Aware Memory

Snapshots should be explicit object-root state, not raw dumps of arbitrary RAM.
Memory classes should decide whether live pages are eligible for snapshot
capture.

Default snapshot behavior:

- Code and immutable object references may be represented by content hashes.
- Mutable object state must opt in.
- Secret memory is excluded.
- DMA and MMIO memory is excluded.
- IPC buffers are excluded unless a service declares them as durable state.
- Rebuildable caches are excluded and can be repopulated after restore.

This keeps snapshots deterministic and avoids accidentally freezing keys,
device state, or temporary authority into a rollback point.

## AI And Telemetry Readiness

AI-assisted diagnostics can be useful only if memory events are structured and
bounded. The kernel should expose deterministic telemetry first and AI summaries
only later.

Useful non-sensitive events:

- Allocation class and size bucket.
- Owner service/task.
- Capability id or redacted object id.
- Mapping permission changes.
- Revocation events.
- Failed permission checks.
- Pressure and reclaim decisions.
- Snapshot inclusion/exclusion reason.

Telemetry must not include raw memory contents, secrets, full physical addresses,
or unredacted authority tokens.

## Practical Build Order

The memory roadmap should advance in narrow, testable layers:

1. Physical memory accounting.
2. Bitmap frame allocator.
3. Page-table mapper.
4. Kernel mapping policy.
5. BootInfo fuzzing and mapper property tests.
6. Kernel-owned page-table activation and CR3 switch.
7. CPU hardening bits and kernel stack guards.
8. CET/shadow-stack investigation behind explicit CPUID and model-specific
   register gates.
9. Kernel memory-object type.
10. Address-space type.
11. Capability-controlled map/unmap.
12. Purpose-tagged allocation classes.
13. Revocable mappings with epoch validation.
14. Secret memory class.
15. Usercopy discipline before ring-3 syscall pointers.
16. IOMMU-backed DMA memory class.
17. Shared-memory IPC transfer.
18. Snapshot-aware memory classification.
19. WASM linear-memory object integration.
20. Memory pressure and self-healing policy.

Each step should add tests or QEMU smoke evidence before becoming a release
claim.

## Non-Claims

This document does not claim that Aesynx already has a production allocator,
general post-switch kernel services, process address spaces, object-memory
integration, IOMMU support, or snapshot-aware memory. `v0.16.2` builds on
checked boot memory-map accounting, a bounded bitmap frame allocator model, a
bounded page-table mapper model, BootInfo fuzz/property evidence, and the v0.16
kernel mapping policy by adding an audited x86_64 hardware-shaped page-table
image export, streaming the used tables into a page-aligned static kernel
activation arena, switching to a private kernel activation stack, loading the
Aesynx-owned CR3 root, and requiring post-switch QEMU evidence. The required
next step is explicit: enable CPU hardening and stack guards, then keep moving
heap and later address-space work behind live hardware-enforced memory
isolation. The roadmap exists so the next allocator and mapper decisions move
toward the clean-slate Aesynx model instead of copying old process/file
assumptions by default.
