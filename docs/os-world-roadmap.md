# Aesynx OS World Roadmap

Status: design direction

Aesynx should grow a native OS world model: a signed, versioned,
policy-bound graph of facts about the running system. This is the layer that
can make the OS feel like it understands itself without turning the privileged
kernel into a database, search engine, or AI runtime.

The kernel remains the trust anchor. It creates and enforces capabilities,
memory mappings, interrupt ownership, driver authority, object references, and
security decisions. A native userspace service records those facts, indexes
them, answers questions, builds context packs, and exposes them to the shell,
GUI, package manager, diagnostics, and bounded AI helpers.

The goal is not a Linux-style pile of logs plus package databases plus tracing
tools bolted together later. The goal is a first-class Aesynx concept: the OS
has a causal memory of what happened, why it happened, who had authority, which
object versions were involved, and which policy allowed or denied an action.

## Core Decision

The kernel is not the brain. The kernel is the root of truth.

The OS brain should be a native service layer:

- `worldd`: append-only OS fact/world service.
- `queryd`: policy-aware query planner and executor.
- `contextd`: context-pack builder for diagnostics and AI helpers.
- `auditd`: tamper-evident audit export and verification.
- `projectiond`: rebuildable indexes, views, search, and UI projections.

These names are placeholders, but the boundary is not: high-level world
queries do not belong in ring 0.

## Fact Model

OS facts should be structured, signed or kernel-stamped where appropriate, and
causally linked.

Examples:

- Boot capsule `A` started kernel image `B`.
- Kernel image `B` exposed memory map summary `C`.
- Driver package `D` requested MMIO range `E`.
- Capability `F` granted read-only access to object `G`.
- Capability `H` was revoked by actor/service `I`.
- Package generation `J` replaced generation `K`.
- Snapshot root `L` was published from object root `M`.
- Service `N` crashed after fault `O`.
- Policy `P` denied DMA because IOMMU scope `Q` was missing.
- AI helper `R` suggested command `S`, but user approval `T` was required.

Every fact should carry enough metadata to be useful later:

- Fact identity.
- Subject, predicate, object/value.
- Source service.
- Policy epoch.
- Capability or redacted authority reference.
- Object root or generation where relevant.
- Time or monotonic event id.
- Causal parents.
- Classification/sensitivity label.
- Confidence only for derived/advisory facts.
- Signature, kernel stamp, or provenance marker.

## Worlds

The OS should use worlds as branchable system states:

- `boot`: facts from firmware, bootloader, and kernel handoff.
- `runtime`: current live OS facts.
- `audit`: append-only security history.
- `package-generation`: package/profile state.
- `snapshot`: retained object roots and rollback metadata.
- `simulation`: proposed changes before they execute.
- `ai-scratch`: bounded derived context that is never authoritative.
- `user-local`: per-user view filtered by capabilities.

This makes simulation and explanation native. For example, a package manager can
ask what would change in a new generation before publishing it, or a driver
installer can show which capabilities would be granted before the user accepts.

## Queries The OS Should Answer

Aesynx should eventually be able to answer questions like:

```text
why was this driver denied DMA?
what changed since snapshot 42?
which package introduced this service?
which objects can this app read?
show all memory objects owned by service X
why did this service restart?
which policy denied this IPC call?
which facts caused this security decision?
can this AI helper summarize object Y?
what would happen if generation N became active?
```

These answers should come from deterministic facts and policy proofs first.
AI may summarize the answer, but it must not invent the underlying evidence.

## Kernel Responsibilities

The kernel should produce minimal, structured, bounded facts and enforce
authority. It should not store the whole world graph or run rich queries.

Kernel responsibilities:

- Stamp security-critical events.
- Emit capability grant, derive, revoke, and denial events.
- Emit memory-object and mapping events.
- Emit driver MMIO, IRQ, DMA, and restart events.
- Emit task/service lifecycle events.
- Emit object-root publication events.
- Enforce that event emission cannot bypass capability checks.
- Keep event records bounded and non-leaking.
- Preserve deterministic behavior if the world service is absent.

The kernel must continue working if `worldd`, projections, or AI helpers are
disabled, corrupted, or rolled back.

## Userspace Responsibilities

Native userspace owns the rich OS brain:

- Append-only fact storage.
- Branchable worlds and diffs.
- Tamper-evident manifests.
- Query planning and redaction.
- Rebuildable search, graph, and UI projections.
- Context packs for diagnostics and AI.
- Human-facing explanations in shell and GUI.
- Retention, export, backup, and audit workflows.

This matches the Aesynx componentization rule: the intelligence layer is
updateable, restartable, inspectable, and rollback-capable.

## Security Rules

The world model is security-sensitive because it describes the system.

Hard rules:

- The world service does not create authority.
- Query permission is capability-scoped.
- AI sees only facts the caller can read.
- Derived facts are never authoritative until promoted by a deterministic,
  signed, or user-approved path.
- Denial reasons must avoid leaking secret labels, compartments, physical
  addresses, raw capability tokens, or private object ids.
- Security-critical facts must be append-only.
- Projections are caches and can be rebuilt from canonical facts.
- Rollback must not cross policy epochs without explicit authority.
- Tamper evidence must be available for audit/export paths.

## Relationship To Memory

The memory roadmap and OS world roadmap should reinforce each other.

Purpose-tagged memory objects become facts:

- who owns the memory,
- why it exists,
- whether it can execute,
- whether it can be shared,
- whether it can be snapshotted,
- whether it can be exposed to DMA,
- when it was revoked or freed.

This gives the OS a queryable memory history without requiring the kernel to
inspect arbitrary program bytes.

## Relationship To Storage

The storage roadmap provides the durable substrate. World facts, projections,
package generations, snapshots, audit proofs, and context packs should all be
object-store records.

Canonical facts should be immutable objects. Mutable state should be represented
by versioned roots. Search, graph, vector, and UI indexes are projections that
can be rebuilt from canonical facts.

## Relationship To AI

AI belongs above deterministic facts and policy proofs.

Allowed AI roles:

- Summarize world facts the caller can already read.
- Explain why a capability, package, driver, or memory decision happened.
- Build proposed query plans.
- Draft remediation steps.
- Compare snapshots in human language.
- Help construct context packs.

Forbidden AI roles:

- Grant authority.
- Bypass capability checks.
- Rewrite canonical facts.
- Hide audit facts.
- Publish package generations or snapshots without deterministic approval.
- Treat model output as truth.

## Practical Build Order

1. Define kernel event/fact envelope.
2. Add event ids for boot, memory, capability, object, driver, task, and package
   facts.
3. Export facts over the existing telemetry/trace path.
4. Build a host-side trace-to-world prototype.
5. Add a minimal RAM `worldd` service after userspace starts.
6. Store canonical facts as immutable object records.
7. Add simple world queries to `aesh`.
8. Add redaction and capability-scoped query planning.
9. Add snapshot/package generation facts.
10. Add rebuildable graph/search projections.
11. Add context-pack generation for diagnostics.
12. Add bounded AI explanation over capability-filtered context.
13. Add tamper-evident persistent world segments after storage persistence.

## Non-Claims

This roadmap does not claim Aesynx currently has an OS world service,
production fact storage, query execution, search, graph indexes, or AI
explanations. The current kernel work should only emit small deterministic
facts as the lower layers mature. Rich world behavior belongs in native
userspace services.
