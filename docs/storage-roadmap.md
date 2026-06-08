# Aesynx Storage Roadmap

Status: design direction

Aesynx should not grow a traditional filesystem as its native storage model.
The native model is the object graph already used throughout the kernel and
userspace design. Disk persistence should preserve that model instead of
translating it into path-first filesystem semantics.

## Core Decision

The native persistent format should be a content-addressed object store:

- Immutable objects are stored by content hash.
- Object payloads are deduplicated by construction.
- Mutable state is represented by versioned references to immutable objects.
- Human-friendly names are name-index objects, not primary storage identity.
- Rollback is a root reference change, not a special filesystem feature.
- Capability checks authorize access to object IDs and name-index objects.

This makes integrity verification, rollback, provenance, and deduplication
properties of the storage design rather than later bolt-ons.

## Snapshot And Rollback Model

Aesynx snapshotting should be native to the object layer rather than copied
from a traditional filesystem design. The kernel/storage layer provides safe
primitive operations; userspace provides policy, retention, presentation, and
operator workflow.

Kernel/object-layer responsibilities:

- Retain immutable object roots as snapshots.
- Atomically publish a new named root reference.
- Atomically roll a named root reference back to a retained object root.
- Verify content hashes before exposing persisted objects.
- Enforce capabilities on object IDs, root references, and name-index objects.
- Preserve provenance and parent/root metadata for audit.
- Refuse rollback across policy boundaries unless the caller has explicit
  authority.

Userspace responsibilities:

- Commands such as `snapshot`, `rollback`, `diff`, `roots`, and `gc`.
- Retention policy.
- Human-readable names and descriptions.
- Confirmation flows for destructive rollbacks.
- Visualization of object graph differences.
- AI-assisted explanations only as advisory summaries of deterministic object
  metadata.

This gives Aesynx Btrfs-like operator value without making Btrfs or POSIX
filesystem semantics the native storage model.

## Package Store Relationship

The native package manager should be built on the same object-store model. See
[Aesynx Package Manager Roadmap](package-manager-roadmap.md).

Package payloads, manifests, SBOMs, provenance, cached AOT artifacts, and
profile generations should be immutable objects. Installing or updating a
package publishes a new generation root. Removing a package omits it from the
next generation and lets garbage collection reclaim unreachable immutable
objects after retention policy allows it.

This means package rollback, system snapshotting, and self-healing are the same
kind of operation: verify immutable objects, then atomically publish or restore
a named root reference.

## Object Identity

`ObjectId(u128)` remains the primary OS-facing object identity. For early RAM
objects, object IDs may be allocated by the boot-local object registry. For
persistent immutable payloads, each object records a content hash and can be
looked up through storage indexes.

The implementation does not have to make `ObjectId` equal to the hash. Keeping
them distinct leaves room for compact object handles, generation metadata,
capability tables, and future object migration while still making integrity
verification mandatory for persisted bytes.

## Names Are Index Objects

Paths such as `/bin/aesh` and `/system/root` are human-friendly names. They
resolve through versioned name-index objects:

```text
"/bin/aesh" -> ObjectId(...)
"system-root" -> ObjectId(...)
```

The name index is itself an object, protected by capabilities, versioned like
other objects, and rollback-capable.

## Practical Boot Compatibility

UEFI boot requires an EFI System Partition that firmware can read, normally
FAT32. Aesynx should support the minimum read-only FAT32 path needed for EFI
boot and bootloader/module loading.

That is a compatibility shim. It must not define the native storage model.

## Build Order

1. Implement the in-memory object graph.
2. Load a boot object bundle into RAM.
3. Run kernel, init, shell, and core commands from RAM objects.
4. Add object-store APIs that are independent of the backing store.
5. Add a persistent append-log backend.
6. Add checkpoints and root references.
7. Add snapshot retention and atomic rollback of named roots.
8. Add crash recovery and integrity verification.
9. Add garbage collection for unreachable immutable objects.
10. Add virtio-block as the first QEMU persistence backend.
11. Add NVMe as the first serious modern hardware storage target.

This keeps filesystem persistence out of the critical boot path while the OS
model is still forming.

## Non-Goals

- POSIX filesystem semantics as the native kernel model.
- Ambient filesystem authority for components.
- Path strings as the primary object identity.
- Writable in-place object mutation.
- Designing the OS around FAT32, ext4, NTFS, or another legacy disk format.
