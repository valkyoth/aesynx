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
- Minimum accepted protocol version per endpoint or service.
- Negotiated version and feature set bound to peer/domain incarnations and
  recorded in authority transactions/audit events.
- Required versus optional extension fields; unknown required extensions fail
  closed.
- Canonical wire encoding that rejects duplicate, noncanonical, or malformed
  fields.
- No silent downgrade after authenticated negotiation.
- Negotiation results live in a kernel-managed channel/session object with
  protocol ID, version, feature-set hash, peer/domain incarnations, session
  generation, and negotiation transcript hash. Later messages inherit the
  session instead of choosing their own version or extension set. Peer restart,
  service-owner transfer, or route replacement invalidates the session and
  requires renegotiation.

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
- Consumer acknowledgement must have its own reverse release/acquire edge:
  the consumer finishes reading, release-stores cursor/ack advancement, and the
  producer acquire-loads that acknowledgement before scrubbing or reusing the
  slot.
- Enqueue, dequeue completion, cancellation, acknowledgement, and slot reuse
  each need a named linearization point.
- Hostile userspace queue producers are different from trusted kernel fabric
  producers. Published userspace slots are untrusted bytes: the kernel performs
  one fault-contained raw byte copy into owned storage, treats that owned
  snapshot as arbitrary attacker input even if userspace mutated it during the
  copy, validates only the owned snapshot, stamps source identity and traffic
  class itself, and never rereads authority-bearing fields from the shared slot
  after validation. User-controlled generation counters are cooperative race
  diagnostics only, not a security proof.
- Slot generation/sequence numbers for wraparound and reuse.
- Scrub-on-vacate before authority-bearing payload storage can be observed by a
  different trust domain.
- Doorbell bitmaps, IPI coalescing, batching, and traffic-class separation so a
  noisy telemetry path cannot delay revocation or topology-control messages.
- Doorbells/IPIs are hints; the queue state is authoritative. Safety and
  liveness are separate obligations:
  - safety: duplicated, stale, coalesced, or early notifications never duplicate
    message consumption or authorize stale payload reuse;
  - liveness: under documented scheduler and hardware fairness assumptions, a
    published message is eventually observed even if the first notification is
    lost.
- A live endpoint needs one reliable progress mechanism: a persistent pending
  bit acknowledged only after observed work, producer retransmission after a
  bounded local timeout, receiver watchdog/periodic polling of inbound
  summaries, a level-triggered notification source, or a rule that deep idle is
  unavailable while no reliable wake source exists. Acknowledgements are bound
  to doorbell generation, link incarnation, receiver incarnation, and observed
  cursor so old acknowledgements cannot clear newer work.
- Traffic class is not an untrusted message field. It is derived from endpoint
  or protocol capability, stamped by the local kernel/CPU driver, rate-limited,
  and backed by reserved control capacity. Ordinary service traffic still needs
  progress guarantees so control floods cannot starve it forever.

Pairwise queues are isolation-friendly but scale quadratically. Aesynx should
create links sparsely, keep dedicated direct links or reserved traffic classes
for revocation/control messages, define a core-count threshold for direct
pairwise links, and use cluster-local or NUMA-local routers above that
threshold. Doorbell state should be sharded or hierarchical; a single
many-writer bitmap is treated as a cache-coherency hotspot until measurement
proves otherwise.

Shared queue pages preserve the shared-nothing rule only when write ownership is
explicit:

- Producer owns and writes payload and slot-publication pages.
- Consumer maps producer pages read-only.
- Consumer owns and writes acknowledgement/cursor pages.
- Producer scrubs payload storage after observing consumption and before reuse.

The implementation must choose either fixed-width wire frames encoded entirely
through atomics or a tiny audited unsafe queue-storage island. If unsafe storage
is used, it needs a local safety proof, Miri/model wrappers, and no general
unsafe exposure from IPC callers.

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

The prospective-revoke linearization point is distributed. It is reached only
after every core capable of local authorization has installed a deny/freeze
fence for the new epoch, or that core has been fenced, quarantined, or reset.
Every cached authorization proof must either be invalidated or represented by a
registry-visible lease that revocation can drain, and new proof issuance is
blocked during the transition. Local epoch caches are advisory; they do not
establish liveness unless backed by a still-valid owner-issued lease or a
locally installed revocation fence.

Strong revocation requires more than an epoch bump. It needs freeze/ack/commit
or equivalent agreement, mapping teardown, local and remote TLB invalidation
acknowledgements, DMA quiesce/cancel/drain, in-flight IPC cancellation or
replay rules, and timeout handling for failed peers.

Selective revocation needs lineage semantics. A strong-revoke transaction names
whether it revokes one entry, a delegation subtree, a revocation domain, or the
whole object. Capability derivation carries lineage identity; descendants may
create independent revocation domains only under explicit policy. Lineage nodes
carry object incarnation, lineage generation, parent reference, revocation
domain reference, bounded child count, maximum depth, and a retirement rule so
stale descendants cannot bind to recycled nodes. Mappings, DMA records, leases,
queued operations, and pending grants must be indexed by lineage for selective
revocation.

Normal audit-buffer exhaustion must not preserve stale authority. Authority
creation, grant, executable mapping, DMA mapping, and policy expansion can fail
closed before mutation when required evidence cannot be recorded. Revoke,
quarantine, and permission reduction proceed fail-safe, reserve emergency audit
capacity, and set a sticky audit-loss digest or halt after authority is removed
if even emergency evidence cannot be retained.

TLB shootdown acknowledgements bind address-space incarnation, ASID/PCID and
reuse generation, mapping generation, virtual range, operation class, target
core incarnation, and revocation transaction ID. An acknowledgement is valid
only after the invalidation instruction and required architectural
serialization have completed. A core that is unresponsive but still executing
with possible stale TLB reachability prevents strong-revoke success unless it is
hardware-reset, fenced from execution, or the system halts.

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

AP startup and restart need incarnations just like services. Every startup
attempt carries a startup generation; late AP arrivals after timeout cannot
satisfy a later generation. APIC-ID reuse, duplicate hardware IDs, topology
snapshot epochs, boot-parameter publication/consumption barriers, and
read-only/zeroed AP parameter pages after consumption are all part of the
fabric safety model before live routing trusts a core.

Live endpoint messages also need stamped identity before payload parsing:
machine boot/session nonce, topology epoch, sender and receiver logical core
IDs, sender and receiver core incarnations, endpoint incarnation, link
generation, protocol version, and negotiated extension set. A validated core ID
without the topology epoch and core incarnation is only advisory; it must not
authorize a future message after the topology snapshot that validated it has
expired.

Until AP incarnation fencing exists, live AP-backed queues must prohibit restart
and hotplug. A failed or timed-out AP stays quarantined until reboot rather than
reusing a core identity that may still hold stale endpoint, routing, or
capability-table messages.

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

Endpoint RPC needs one-shot reply capabilities. A reply cap is minted by the
kernel, bound to caller, callee endpoint, transaction, boot/domain incarnation,
and timeout/cancellation state, and can complete exactly once. Servers cannot
redirect it to an unrelated caller or delegate it unless an endpoint type
explicitly permits that behavior. Server death resolves outstanding reply caps
through typed cancellation or retryable failure and cleans authority as part of
the server-restart transaction.

Cross-core deadlines are local decisions unless Aesynx has a synchronized clock
with a documented skew bound. Fabric messages should carry relative TTLs;
receivers stamp local deadlines on authenticated receipt, coordinators decide
timeouts using their local monotonic clock, and epoch/incarnation changes
invalidate old deadlines.

Authority-moving protocols use a bounded preallocated transaction journal. The
journal records transaction ID, participant incarnations, source/destination
capability identities, frozen source generation, prepared/committed/aborted
state, witness acknowledgements, commit certificate or decision epoch, recovery
owner, timeout owner, journal generation, torn-record integrity evidence, and
bounded replay window. Journal ownership is one-writer; it must not become a
globally mutated shared structure. Each protocol must specify whether records
survive coordinator-domain restart, core reset, or machine reboot, and which
replication or witness acknowledgements are required before receiver authority
becomes active. Terminal records are reclaimed only after every participant has
acknowledged the outcome and the replay window has retired. Coordinator
restart recovers from the journal; if no authoritative coordinator record or
trusted commit witness survives, availability is not guaranteed. If commit
might have been observed but evidence is lost, the safe result is quarantine or
explicit resource loss, never reconstruction from participant-controlled
claims or blind abort that restores sender authority.

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
- Security-grade timebase rules before timeouts become enforcement inputs:
  per-core clock-source selection, invariant/nonstop TSC detection where
  relevant, checked calibration, core-to-core skew measurement, rollover and
  suspend/resume/AP-restart behavior, clock generations attached to deadlines,
  fail-closed backward-jump handling, independent watchdog sources where
  coordinator failure matters, and an authenticated synchronized-clock
  capability before comparing absolute times across cores. Timeouts can abort,
  retry, cancel, or quarantine, but cannot manufacture a commit.

Authority-transfer protocols need transaction semantics. A grant proposal should
reserve a pending receiver slot, carry a transaction ID, wait for acceptance,
then commit or abort. Retried proposals must be idempotent, and timed-out grants
must not leave usable receiver authority behind. Audit records should preserve
proposal, acknowledgement, commit, abort, and revoke linkage without exposing
raw authority identifiers.

Service/RPC protocols also need a confused-deputy contract. Caller identity is
context and audit evidence, not authorization. Each request schema declares the
capability arguments that authorize the operation, nested service calls carry
only explicitly delegated authority, scheduling-budget donation is not authority
donation, reply capabilities only authorize the matching result path, and
service-local caches preserve caller authority and classification context.

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
- Microcode-revision and mixed-core mitigation policy before speculative
  controls are treated as valid evidence.
- Rate-limited and quantized telemetry exports.
- Capability-gated high-resolution clocks and performance counters.
- Cache/LLC partitioning through CAT/MPAM or equivalent where available.
- Memory-bandwidth allocation or rate limiting where supported.
- Page coloring only when measured and included in allocator policy.
- Shared-buffer, shared-cache, shared-core, SMT-sibling, and shared-device
  relationships recorded as deliberate covert-channel edges.
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
- Writable cross-domain memory is represented only through atomic fields,
  volatile byte regions, or audited protocol wrappers. Safe `&mut T` and
  aliased non-atomic `&T` are not constructed over concurrently writable shared
  storage.
- Every writable-sharing protocol names permitted access widths, alignment,
  atomic orderings, ownership transitions, and recovery behavior. Non-atomic
  structured payloads require exclusive ownership transfer before access.
- The mapper distinguishes declared shared-buffer aliasing from accidental
  physical double ownership.
- Kernel-owned shared mapping infrastructure may be proven first using real
  page-table roots, CR3 switching, and TLB shootdown in QEMU. Hostile
  cross-domain user mappings are a later proof that requires actual isolated
  ring-3 address spaces; model dispatchers are not evidence for that claim.
- Page-table pages are protected after installation. Later updates require a
  narrow owner-core edit window or dedicated temporary mapping protocol that
  restores the protected state before returning.

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
- TLA+ or Quint models for grant, revoke, coordinator failure, AP restart, and
  strong-revocation linearization.
- TLA+ or Quint models for derived-edge creation, promotion/detachment, v1
  single-parent publication, parent-owner coordinator locality, owner-local
  reservation manifests, canonical required-reservation plans with versioned
  domain-separated identities, parent-local preparing-record persistence before
  remote side effects, transaction-bound reservation consumption/release,
  deadlock/livelock-free reservation acquisition, deterministic priority
  conflict resolution, permit consumption at the parent-owned journal commit
  transition, parent-local audit placeholder finalization, recoverable
  quarantine, `ResourceLost` terminal tombstones, provenance recording, and
  cascading revocation. Future multi-parent support requires its own
  `ParentSetManifest` model before any implementation accepts more than one
  parent.
- Kani, Verus, or equivalent bounded proofs for permission attenuation, range
  containment, generation retirement, and scheduler action validation.
- Loom models for SPSC publication, slot reuse, cursor caching, and
  scrub-before-reuse ordering.
- Architecture litmus tests for x86_64, aarch64, and RISC-V ordering
  assumptions, especially where MMIO, IPI, DMA, TLB, or cache maintenance is
  involved.
- Cross-endian fabric golden vectors and decoder fuzzing for every stable wire
  format.
- Differential and metamorphic BootInfo normalization tests so parser behavior
  is checked beyond fuzz-generated cases.
- Test-only fault injection for dropped, duplicated, delayed, and reordered
  messages; lost IPIs; coordinator death; AP late arrival; TLB-ack loss; full
  queues; and epoch exhaustion.
- Global W^X alias property tests across memory objects and address spaces.
- Refinement tests showing executable Rust state machines agree with the
  formal transition models they implement.
- Required safety properties: no authority amplification, authority
  resurrection, split-brain commit, W+X alias, stale-core acceptance, usable
  orphan derived child, promotion-based revocation escape, multi-parent derived
  child use before the `ParentSetManifest` feature gate, provenance-as-authority,
  cross-core transmission of an internal derivation permit, ordinary allocation
  required after derived-edge commit, stale permit replay, delayed commit/abort
  resurrection after `ResourceLost`, missing audit evidence for an
  authority-creating commit, reservation-plan substitution, persistent
  reservation deadlock/livelock, remote reservation before recoverable
  parent-local prepare state, inconsistent transaction-priority conflict
  handling, or cycle under concurrent edge transactions.
- Required bounded-liveness properties: healthy grant/revoke transactions
  eventually commit or abort, revocation/control traffic is not starved by
  telemetry floods, coordinator restart converges to one final result, and
  queues progress under explicit producer/consumer scheduling assumptions.
- TLA+/Quint models must state their fairness assumptions explicitly and have
  negative variants or refinement tests that intentionally disagree with Rust
  state-machine behavior.

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
