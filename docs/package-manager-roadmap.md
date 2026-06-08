# Aesynx Package Manager Roadmap

Status: future design direction

Aesynx should not copy destructive package managers that mutate a shared global
filesystem. The native package manager should be a capability-aware front end
to the same content-addressed object model used by storage, snapshots,
userspace components, and boot bundles.

Working names:

- `aepkgd`: privileged package-management service.
- `aepkg`: CLI client.
- `aesh pkg`: shell-integrated command surface.
- Aesynx Store: future graphical or TUI client using the same service API.

The names are placeholders. The architecture is the important decision.

## Core Position

Package management in Aesynx should be:

- Content-addressed: package payloads are stored by verified content identity.
- Immutable: installed artifacts are read-only objects, not mutable files.
- Declarative: system and user package state is a generation object.
- Atomic: install, remove, update, and rollback publish a new generation root.
- Capability-native: packages declare requested authority but receive only
  explicit grants.
- Track-aware: core, official, community, market, sovereign, and vendor
  packages are separated by trust policy.
- SBOM-native: software bill of materials metadata is part of the signed
  package identity.
- GUI-ready: CLI and graphical store clients use the same daemon API.
- Offline-capable: installed generations remain usable without network access.
- Repairable: corrupted store objects can be detected and fetched again from a
  verified source.

## Non-Goals

- Installing into `/usr/bin`, `/lib`, or another shared mutable tree.
- Running package install scripts as root.
- Granting ambient filesystem/network/device authority to packages.
- Treating paid-license status as a kernel permission.
- Making community or market tracks equivalent to core OS trust.
- Requiring a single central registry.
- Making package names the security identity. Hashes, signatures, and policy
  are the security identity.

## System Shape

```text
User interfaces
|-- aepkg CLI
|-- aesh pkg built-ins
`-- Aesynx Store GUI/TUI
       |
       | typed local RPC over a capability-protected service endpoint
       v
aepkgd
|-- transaction planner
|-- resolver
|-- registry client
|-- signature and transparency verifier
|-- content-store writer
|-- profile/generation publisher
|-- health and repair worker
|-- garbage collector
|-- entitlement broker for paid packages
`-- audit/provenance emitter
       |
       v
Object-backed package store
|-- immutable package objects
|-- Merkle block objects
|-- signed manifests
|-- SBOM/provenance objects
|-- AOT WASM/native cache objects
`-- generation/profile root objects
```

The daemon is privileged only for package-management capabilities. It should
not become an all-powerful system daemon. Its authority should be split into
separate capabilities for:

- Reading registry configuration.
- Writing immutable store objects.
- Publishing system generation roots.
- Publishing user profile roots.
- Reading license receipts from the secure local vault.
- Running repair and garbage collection.

## Package Artifact Types

Aesynx package artifacts should be typed from day one:

| Type | Purpose | Default authority |
| --- | --- | --- |
| `native-command` | Trusted Aesynx command built for the target architecture | None until launched with caps |
| `wasm-component` | Portable sandboxed command, plugin, or automation module | None until launched with caps |
| `service` | Long-running userspace service | Manifest-bounded service caps |
| `driver-service` | Out-of-kernel driver process | Device-specific caps only |
| `data` | Fonts, themes, schemas, model data, docs | Read-only object cap |
| `policy` | Capability, scheduler, AI, or security policy objects | Policy-management caps |
| `bundle` | Atomic group of artifacts released together | Bundle-defined child caps |

This keeps the project rule intact: Aesynx should not become one giant binary.
The boot capsule or release image may package many objects together, but the
installed system remains independently identifiable, signed, updateable, and
rollback-capable.

## Package Manifest

Every package has a signed immutable manifest. A possible shape:

```toml
[package]
name = "log-view"
version = "1.4.2"
track = "community"
kind = "wasm-component"
summary = "Structured log viewer"
publisher = "did:aesynx:pub:example"

[artifact]
hash = "sha256:..."
merkle_root = "sha256:..."
target = "wasm32-wasip2-aesynx"
entry = "component:log-view"

[exports]
commands = ["log-view"]
schemas = ["aesynx.log.entry.v1"]

[dependencies]
aesynx_value = { version = "1", hash = "sha256:..." }

[capabilities]
network = []
storage_read = ["object://logs/*"]
storage_write = []
ipc = []
graphics = false
device = []

[supply_chain]
sbom = "spdx:sha256:..."
source = "https://example.invalid/source/log-view"
builder = "did:aesynx:builder:community-ci"
provenance = "slsa:sha256:..."
transparency_entry = "rekor-like:..."

[[signatures]]
algorithm = "policy-selected"
key_id = "did:aesynx:pub:example#key-1"
value = "base64:..."

[license]
software = "EUPL-1.2"
entitlement = "none"
```

Manifest rules:

- The manifest is signed by the publisher.
- Signatures are represented as versioned envelopes with algorithm identifiers,
  not as one fixed signature type or fixed-size field.
- The registry track signs or countersigns index inclusion.
- SBOM and provenance are immutable dependencies of the package identity.
- Capability declarations are requests, not grants.
- Package activation must be declarative. No root post-install scripts.
- Migration logic, if ever needed, must be a constrained component with an
  explicit capability grant and audit record.

## Tracks

Aesynx should separate package trust with tracks:

| Track | Owner | Intended contents | Verification policy |
| --- | --- | --- | --- |
| `core` | Aesynx OS | Kernel-adjacent services, init, shell, core tools | Aesynx release keys, strict reproducibility target, mandatory pentest evidence |
| `official` | Aesynx project | First-party non-core apps, docs, tools | Aesynx signing keys, SBOM, CI provenance |
| `community` | External open-source publishers | WASM components, user tools, themes | Publisher signatures, registry inclusion, automated SBOM/advisory checks |
| `market` | Marketplace operators | Paid or proprietary apps | Publisher signature plus signed entitlement receipt |
| `sovereign` | User, company, or institution | Private/internal packages | Local PKI or explicitly configured trust root |
| `vendor` | Hardware/software vendors | Driver services, firmware helpers, vendor libraries | Vendor key plus device/capability policy review |

Track policy is local state. A user can enable or disable tracks, pin packages
to tracks, require manual approval for track changes, and set different update
cadence per track.

Core system updates must not be blocked by community or market registry health.
Community packages must not be allowed to shadow core package names without an
explicit namespace or policy override.

## Registries

A repository URL still exists, but it is a verifiable object registry rather
than an untrusted tarball bucket.

Registry responsibilities:

- Serve signed index snapshots.
- Serve package manifests.
- Serve Merkle blocks or full artifact blobs.
- Serve SBOM and provenance objects.
- Serve transparency-log inclusion proofs where configured.
- Advertise mirrors and local cache peers.
- For market packages, validate entitlement requests without receiving broad OS
  authority.

Client verification order:

1. Load local track policy.
2. Fetch signed registry root/snapshot metadata.
3. Verify signatures against the track trust root.
4. Apply track policy for accepted algorithms and hybrid-signature
   requirements.
5. Verify transparency inclusion when required by track policy.
6. Resolve package names to immutable manifest hashes.
7. Fetch Merkle blocks from any mirror or peer.
8. Hash every block and the completed artifact.
9. Verify publisher signature and SBOM/provenance references.
10. Store the verified objects immutably.
11. Publish a new generation root only after all verification succeeds.

Mirror agnosticism is mandatory. A mirror is only a source of bytes. It is not a
source of truth.

## Local Store And Generations

The conceptual human namespace may look like this:

```text
/system/store/<hash>-<name>-<version>
/system/profiles/system/gen-000142
/system/profiles/users/<user>/gen-000031
```

Internally these should be object-store roots, not path-first filesystem state.

Installing a package:

1. Resolve requested name, track, version, and policy.
2. Build a transaction plan.
3. Fetch and verify all missing objects.
4. Create a new profile generation object.
5. Atomically publish the generation root.
6. Emit audit/provenance events.

Removing a package:

1. Create a new generation that omits the package.
2. Check reverse dependencies and protected packages.
3. Publish the new generation root.
4. Leave unreachable store objects for garbage collection.

Updating packages:

1. Resolve available updates per track policy.
2. Prefer block-delta fetches by Merkle identity.
3. Create a new generation.
4. Keep previous generations for rollback.
5. Garbage collect only after retention policy allows it.

Rollback:

```text
aepkg rollback system --to gen-000141
aesh> pkg rollback system --to gen-000141
```

Rollback changes the active generation root. It does not mutate package files.

## CLI Surface

The CLI should feel familiar, but the implementation is declarative:

```text
aepkg search telemetry
aepkg show community/log-view
aepkg install community/log-view
aepkg install market/pixel-editor
aepkg remove log-view
aepkg update
aepkg update --track core
aepkg list
aepkg generations
aepkg rollback system --to gen-000141
aepkg health
aepkg health --repair
aepkg gc --dry-run
aepkg track list
aepkg track enable community
aepkg track disable market
aepkg pin core/aesh --version 1.2.0
aepkg receipts list
```

Search should support:

- Local indexed search for offline use.
- Remote registry search when enabled.
- Track filtering.
- Package kind filtering.
- Capability filtering.
- License and price filtering.
- SBOM/advisory status filtering.
- GUI-friendly structured result records.

The shell can expose the same model as structured pipelines:

```text
aesh> pkg search log | where track == community && kind == wasm-component | view
aesh> pkg health | where status != Ok | view
```

## GUI Store

The future store UI should be a client, not a privileged package manager.

It can provide:

- Search and discovery.
- Screenshots, descriptions, changelogs, ratings, and publisher identity.
- Track badges: core, official, community, market, sovereign, vendor.
- Capability review before install.
- SBOM/provenance display.
- Update history and generation rollback.
- Health/repair status.
- Paid checkout flow for market packages.

The store UI talks to `aepkgd` through typed local RPC. It should never write
the system store directly.

## Paid Packages

Paid packages belong in the `market` track or an explicitly configured
sovereign/private track. Licensing must not weaken OS security.

Rules:

- A signed entitlement receipt permits fetch/decryption. It does not grant
  runtime capabilities.
- The decrypted payload must still match the signed manifest hash.
- The publisher signature and registry policy must still verify.
- The package runs under the same capability model as free packages.
- License checks should be handled by userland services, not kernel DRM.
- Offline behavior must be explicit in the receipt policy.
- Refund/revocation behavior must never remove user data silently.

Market flow:

1. Store UI starts checkout with a payment provider.
2. Registry issues a signed entitlement receipt.
3. `aepkgd` stores the receipt in a secure local vault object.
4. Registry authorizes encrypted block fetch.
5. `aepkgd` decrypts blocks locally, verifies content identity, and stores
   immutable objects.
6. Runtime capabilities are still approved separately.

## Lazy Execution

`aesh` can integrate with package lookup:

```text
aesh> trace-route --target example.net
```

If `trace-route` is missing locally:

1. `aesh` asks `aepkgd` for matching command exports.
2. The user sees track, publisher, capabilities, and persistence choice.
3. The package may run once from a verified temporary generation.
4. The user can pin it for offline use or discard it after execution.

Lazy execution must be policy-controlled. Core/admin contexts should be able to
disable network lookup entirely.

## Health, Repair, And Garbage Collection

Self-healing is a package-manager responsibility backed by the object store.

Health checks:

- Verify active generation manifests.
- Verify active package object hashes.
- Verify Merkle block hashes.
- Verify signatures and track policy.
- Verify SBOM/advisory cache freshness.
- Report missing mirrors or expired receipts.

Repair:

- Quarantine corrupted objects.
- Re-fetch verified blocks from any configured mirror/cache.
- Rebuild derived AOT caches from immutable inputs.
- Refuse to repair if policy, signature, or entitlement validation fails.
- Never overwrite user data as part of package repair.

Garbage collection:

- Mark from active and retained generation roots.
- Keep pinned packages and offline cache policy roots.
- Delete unreachable immutable objects only after retention policy allows it.
- Emit a dry-run plan before destructive GC.

## Security Rules

- No package install script runs with ambient root authority.
- Every registry trust root is explicit local policy.
- Track changes are auditable state changes.
- Names are advisory; hashes and signatures are authoritative.
- Signature and key formats are crypto-agile. See
  [Post-Quantum Readiness Roadmap](post-quantum-readiness.md).
- Capability manifests are upper bounds, not automatic grants.
- Paid-license receipts do not override capability policy.
- Package metadata is untrusted until signatures and transparency policy pass.
- Search results from remote registries are untrusted display data.
- A package cannot replace core commands without explicit policy.
- A package cannot mutate another package's immutable store objects.
- Store repair must verify before publish, never after publish.

## Implementation Order

Host-model first:

1. Add an `aesynx-pkg` model crate with manifest, track, package kind, and
   generation types.
2. Add strict manifest parsing tests with no network.
3. Add a local fixture registry format for tests.
4. Add a content-addressed in-memory store model.
5. Add generation diff/install/remove/update planning.
6. Add capability-manifest validation against local policy.
7. Add SBOM/provenance reference types.
8. Add `aepkg` host CLI prototype for search/list/plan only.

Aesynx userspace integration:

9. Store package manifests as object-store objects.
10. Add profile generation roots.
11. Add `aesh pkg search/show/install/remove` built-ins.
12. Add WASM component package install and launch.
13. Add health and repair against the local object store.
14. Add rollback and GC commands.

Registry and market:

15. Add signed registry index support.
16. Add mirror verification.
17. Add transparency-log policy hooks.
18. Add sovereign/private registry configuration.
19. Add store UI RPC.
20. Add market entitlement receipts.
21. Add encrypted block fetch for paid packages.

This order keeps the security model testable before adding network, GUI, or
payments.

## Relationship To 1.0

The first QEMU `v1.0.0` release should not require the full package manager.
The minimum useful target before 1.0 is:

- Package manifest model exists.
- Boot/userspace bundles can be described as signed component objects.
- The object store and generation model do not block a future package manager.

The full package manager belongs after native userspace, persistent object
storage, and initial registry/provenance infrastructure are real.
