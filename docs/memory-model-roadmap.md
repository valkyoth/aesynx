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
5. Kernel memory-object type.
6. Address-space type.
7. Capability-controlled map/unmap.
8. Purpose-tagged allocation classes.
9. Revocable mappings with epoch validation.
10. Secret memory class.
11. IOMMU-backed DMA memory class.
12. Shared-memory IPC transfer.
13. Snapshot-aware memory classification.
14. WASM linear-memory object integration.
15. Memory pressure and self-healing policy.

Each step should add tests or QEMU smoke evidence before becoming a release
claim.

## Non-Claims

This document does not claim that Aesynx already has a production allocator,
page tables, address spaces, object-memory integration, IOMMU support, or
snapshot-aware memory. `v0.13.0` only establishes checked boot memory-map
accounting. The roadmap exists so the next allocator and mapper decisions move
toward the clean-slate Aesynx model instead of copying old process/file
assumptions by default.
