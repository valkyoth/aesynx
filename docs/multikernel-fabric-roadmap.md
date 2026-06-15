# Aesynx Multikernel Fabric Roadmap

Status: planning document

Aesynx uses x86_64 SMP/APIC mechanisms to bring cores online, but the
operational model should become a software-defined AMP/multikernel fabric.
Once cores are online, they should behave more like explicit service nodes than
interchangeable threads inside one shared kernel.

This document tracks requirements that must exist before Aesynx can honestly
claim a mature multikernel design.

## Core Principles

- Cores and execution domains have explicit roles and local ownership.
- Cross-core authority moves through capabilities and audited messages.
- Shared memory is explicit, object-backed, capability-scoped, and revocable.
- Global state is replicated or owned; it is not a hidden shared variable.
- Device interrupts route to owner cores or service domains.
- Failed components are isolated, revoked, and restarted where possible.
- Heterogeneous peers are represented by capability metadata, not hidden behind
  a fake "all cores are identical" abstraction.

## Fabric Peers

A fabric peer can be more than an x86_64 CPU core. Long-term candidates include:

- x86_64 cores.
- aarch64 cores on future ports.
- Performance and efficiency cores on heterogeneous CPUs.
- Driver service cores.
- GPU, NPU, DSP, smartNIC, or secure-enclave service domains where the hardware
  can speak an Aesynx-compatible protocol through a trusted bridge.

The first implementation should stay conservative: QEMU x86_64 cores only. The
protocol must still avoid x86_64-only assumptions in message layout,
endianness, object identity, capability IDs, and version negotiation.

## Fabric Protocol

The fabric protocol is the internal network of Aesynx. It needs:

- Versioned message headers.
- Explicit sender and receiver identities.
- Core/domain role metadata.
- Capability transfer records.
- Sequence numbers.
- Bounded payload sizes.
- Backpressure and retry policy.
- Timeout policy.
- Dead-letter or rejection records.
- Redacted diagnostics.
- Endianness and alignment rules suitable for heterogeneous peers.

No Rust-specific memory layout should cross the fabric boundary unless the
sender and receiver are proven to use the same ABI and trust domain.

## Replicated Authority State

A true multikernel cannot rely on one global lock for authority state.
Capability revocation, service routing, driver ownership, and system policy
updates need replicated-state rules:

- Every replicated record has an owner or coordinator.
- Updates carry monotonic epochs.
- Stale epochs fail closed.
- Critical authority changes use prepare/commit or an equivalent two-phase
  protocol.
- Participants acknowledge readiness before commit.
- Timeouts lead to abort, quarantine, or fail-closed degraded mode.
- Audit records link proposal, acknowledgement, commit, and revoke events.

This does not require full cloud-style consensus in early releases. The first
machine-local design can use owner-core coordination plus two-phase commit for
critical state. Quorum/consensus algorithms are later work if Aesynx supports
fault-tolerant peer groups or machine-to-machine clusters.

## Topology And Routing

Early Aesynx can send direct core-to-core messages. A mature fabric should learn
and use topology facts:

- NUMA node.
- Core cluster.
- Cache locality.
- Inter-core latency.
- Queue depth.
- Service load.
- Device locality.
- Heterogeneous core capability.

Routing decisions must remain deterministic and auditable. AI may advise later,
but a bounded non-AI policy must always exist.

## Naming And Discovery

Fabric peers and services need stable names that are not raw core numbers:

- Peer identity.
- Service identity.
- Role identity.
- Generation/epoch.
- Owner core or domain.
- Current route.
- Health state.

A peer restart must not accidentally inherit stale authority from a previous
generation. Service discovery must return capability-scoped handles, not ambient
global pointers.

## Ordering, Timeouts, And Backpressure

The fabric cannot assume one global clock or one global lock. Message protocols
need:

- Per-channel sequence numbers.
- Epoch checks for authority-bearing operations.
- Timeout classes for best-effort, authority-critical, and fatal-bootstrap
  messages.
- Backpressure before queues overflow.
- Explicit drop, retry, cancel, or dead-letter behavior.
- Bounded memory use during retry storms.

## Admission Control And Quotas

A malicious or broken peer must not exhaust the machine-local fabric:

- Per-peer queue limits.
- Per-service outstanding request limits.
- Shared-buffer size limits.
- Capability grant rate limits where needed.
- Restart-rate limits for crashing services.
- Telemetry for throttling and denial.

These controls should become capability policy, not hardcoded hidden globals.

## Fault Containment

Driver and service failure must not automatically become whole-kernel failure.
The fabric needs:

- Heartbeats.
- Watchdogs.
- Fault domains.
- Service quarantine.
- Capability revoke on fault.
- DMA/IOMMU cleanup before restart.
- In-flight message cancellation or replay rules.
- Service rebinding.
- Restart budget and escalation policy.
- Operator-visible telemetry.

Fatal CPU or memory corruption may still require a full halt. The roadmap goal
is containment for isolated service/domain failure, not pretending all hardware
faults are recoverable.

## Shared Memory

Zero-copy sharing is useful, but it must stay explicit:

- Applications ask for shared-buffer objects, not raw physical frames.
- Each dispatcher receives a derived mapping capability.
- Read-only sealed buffers are preferred for large assets.
- Writable sharing requires `SHARE_WRITE`, a named synchronization protocol,
  audit, revocation, and TLB shootdown.
- The mapper distinguishes declared shared-buffer aliasing from accidental
  physical double ownership.

## Security Gates

Before any fabric milestone graduates, release notes must answer:

- Which core/domain owns the state?
- Which messages can transfer authority?
- Which capabilities are required?
- What is the revocation path?
- What happens on timeout?
- What happens if the destination core is dead?
- What audit evidence proves the operation?
- Which identifiers are redacted in diagnostics?
- Does the implementation fail closed on stale epochs?
- What are the queue, memory, and retry bounds?
- How is service discovery scoped by capabilities?

## Non-Claims

The current kernel does not yet implement:

- Live multicore scheduling.
- Heterogeneous ISA execution.
- Distributed consensus.
- Topology-aware routing.
- Service restart after core failure.
- Shared-buffer user mappings.

Those are roadmap items, not present controls.
