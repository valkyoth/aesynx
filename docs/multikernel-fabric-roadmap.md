# Aesynx Multikernel Fabric Roadmap

Status: planning document

Aesynx uses x86_64 SMP/APIC mechanisms to bring cores online, but the
operational model should become a software-defined AMP/multikernel fabric.
Once cores are online, they should behave more like explicit service nodes than
interchangeable threads inside one shared kernel.

This document tracks requirements that must exist before Aesynx can honestly
claim a mature multikernel design.

The SOSP 2009 Barrelfish paper is useful evidence for this direction, but
Aesynx is not trying to clone Barrelfish. The lessons we should keep are:
explicit inter-core messages, hardware-neutral protocols, replicated state with
agreement, topology-aware routing, user-space service domains, and a small
per-core privileged component. The mistakes we should avoid are turning physical
RAM allocation into a hot global capability protocol and letting distributed
policy bloat ring 0.

## Core Principles

- Cores and execution domains have explicit roles and local ownership.
- Cross-core authority moves through capabilities and audited messages.
- Shared memory is explicit, object-backed, capability-scoped, and revocable.
- Global state is replicated or owned; it is not a hidden shared variable.
- Device interrupts route to owner cores or service domains.
- Failed components are isolated, revoked, and restarted where possible.
- Heterogeneous peers are represented by capability metadata, not hidden behind
  a fake "all cores are identical" abstraction.
- Ring 0 stays local and minimal; distributed policy runs in isolated monitor or
  service domains.

## CPU Driver And Monitor Boundary

The long-term privileged component on each core should look more like a local
CPU driver than a monolithic shared kernel. Its job is to enforce protection and
perform local mechanism:

- Trap, exception, and interrupt dispatch for the owning core.
- Local address-space switch and page-table install.
- Local capability checks for operations it directly enforces.
- Local scheduler dispatch primitives.
- Local message endpoint delivery and doorbell/IPI handling.
- Local hardware access mediation for APIC/MMU/CPU state.

The following do not belong in the permanent ring-0 TCB:

- LLM/model execution or rich AI scoring.
- Package/store policy.
- World graph queries or search.
- Global capability agreement protocols.
- Rich telemetry aggregation and projections.
- Driver policy beyond local hardware enforcement.
- Distributed routing policy beyond validated local message delivery.

Those functions should run as monitor, driver, world, telemetry, package, or AI
service domains with explicit capabilities. Current in-kernel smokes and models
are allowed as scaffolding while there is no userspace, but each authority
surface must have a migration path out of ring 0 before it becomes a production
claim.

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

Live core-to-core queues are not the same as the current sequential model. A
production SPSC link needs:

- No shared hot-path `len` field written by both producer and consumer.
- Monotonic cursors where each cursor has exactly one writer.
- Cached remote cursor observations refreshed through acquire loads.
- Producer and consumer metadata on separate cache lines, or separate pages
  when endpoint permissions differ.
- Payload write followed by release publication, and acquire observation before
  payload read.
- Slot generation/sequence numbers for wraparound and reuse.
- Scrub-on-vacate before authority-bearing payload storage can be observed by a
  different trust domain.
- Doorbell bitmaps, IPI coalescing, batching, and traffic-class separation so a
  noisy telemetry path cannot delay revocation or topology-control messages.

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

The coordinator should usually be a monitor/service domain, not arbitrary kernel
code on every core. The local kernel validates and applies the final local
mechanism only after the authority protocol has produced a bounded, auditable
decision.

Revocation has two classes:

- Prospective revocation: no operation beginning after the linearization point
  may succeed.
- Strong revocation: when revoke returns, no stale operation, mapping, DMA
  request, delegated entry, or in-flight endpoint operation may still commit.

Strong revocation requires more than an epoch bump. It needs freeze/ack/commit
or equivalent agreement, mapping teardown, local and remote TLB invalidation
acknowledgements, DMA quiesce/cancel/drain, in-flight IPC cancellation or
replay rules, and timeout handling for failed peers.

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

These facts are Aesynx's system-knowledge layer. They should be deterministic,
bounded, and queryable by monitor/world services. The kernel emits and enforces
facts; it must not become a general-purpose knowledge database.

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

Authority-bearing identities are minted by trusted registries or dispatchers,
not by request payloads. A message may contain claimed peer, core, service,
object, or endpoint IDs for logging and routing, but enforcement must use
kernel-stamped execution context and registry-issued incarnations. This avoids
turning public constructors for ID-shaped values into security boundaries.

Object identity is especially sensitive. A visible object name, package object
ID, or content hash is not enough to authorize access. Capability targets need a
stable logical incarnation that cannot be recreated by deleting an object and
later placing the same visible ID in another registry slot. Stale capability
tests must cover slot reuse, migration between slots, deletion/recreation,
generation exhaustion, and revocation-epoch changes.

Capability tables and endpoints also need owners. A production table is bound
to a domain or principal incarnation, and an endpoint send/receive operation is
authorized through endpoint capabilities plus kernel-stamped source metadata.
Raw queue access is never the authority boundary.

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

Authority-transfer protocols need transaction semantics. A grant proposal should
reserve a pending receiver slot, carry a transaction ID, wait for acceptance,
then commit or abort. Retried proposals must be idempotent, and timed-out grants
must not leave usable receiver authority behind. Audit records should preserve
proposal, acknowledgement, commit, abort, and revoke linkage without exposing
raw authority identifiers.

## Admission Control And Quotas

A malicious or broken peer must not exhaust the machine-local fabric:

- Per-peer queue limits.
- Per-service outstanding request limits.
- Shared-buffer size limits.
- Capability grant rate limits where needed.
- Restart-rate limits for crashing services.
- Telemetry for throttling and denial.

These controls should become capability policy, not hardcoded hidden globals.

## Side-Channel And Denial Boundaries

The current threat model does not claim general side-channel resistance, but the
fabric must not make future isolation impossible. Before mutually distrusting
domains share cores or high-resolution telemetry, Aesynx needs explicit policy
for:

- Security-domain-aware placement and scheduling.
- SMT disablement or partitioning for high-assurance workloads.
- Speculation-control configuration where the architecture supports it.
- Rate-limited and quantized telemetry exports.
- Per-principal CPU, queue, memory, grant, and restart budgets.
- Bounded lookup structures on syscall-hot or fabric-hot paths.
- Cache-line separation for producer/consumer queue metadata.
- Backpressure rules that prevent one endpoint from starving unrelated
  channels.
- Revocation epoch checks at the final enforcement boundary, so delayed
  messages cannot preserve authority.

These controls should be staged as measurable policy gates, not retrofitted
after user domains start carrying hostile workloads.

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

Raw physical frame allocation should remain local allocator work wherever
possible. Capabilities authorize memory objects, mapping rights, sharing, DMA,
ownership transfer, and revocation. They must not turn every hot frame
allocation/free into a global cross-core agreement path.

## Formal Verification Targets

Testing and pentesting remain required, but they are not enough for
authority-bearing multikernel primitives. The fabric roadmap should keep the
core protocols shaped so they can be model-checked or formally verified in
small pieces.

Priority proof targets:

- AP/core state-machine valid combinations.
- Local capability checks and unforgeable authority transitions.
- Fabric message decode/reject behavior.
- Grant/revoke agreement protocols and stale-epoch failure.
- Shared-buffer alias rules.
- Owner-core mutation and quarantine transitions.
- Topology route selection invariants.

Host model tests should come first; Kani, Verus, Prusti, Coq, or similar tools
can then be applied where the code shape is stable enough.

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
- User-space monitor domains.
- Formal proofs of fabric/capability protocols.

Those are roadmap items, not present controls.
