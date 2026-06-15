# Aesynx Architecture Decisions

This file records initial decisions that should shape implementation. These are not permanent laws, but changing them requires a conscious design review.

Naming rule: the project, kernel, and system are named `Aesynx`.

## ADR-001: 1.0 Is A QEMU Research OS

Decision:

Aesynx 1.0 targets QEMU x86_64 as a serious research OS release.

Rationale:

Real hardware support explodes scope. QEMU gives repeatable hardware, deterministic tests, and enough virtio devices to build a meaningful OS.

Implications:

- Real hardware support is deferred.
- Virtio is preferred before physical NICs/storage.
- Serial-based smoke tests are mandatory.

## ADR-002: Native Aesynx Userspace, Not Unix Compatibility

Decision:

The 1.0 userspace is native Aesynx. It uses capabilities, objects, service queues, native runtime, native init, and native shell.

The shell and command model should follow [Aesynx Userspace Vision](userspace-vision.md): structured pipelines, capability manifests, object-native commands, WASM components for sandboxed extension, and bounded AI assistance.

Rationale:

Unix compatibility would dominate the project and force early design compromises around files, file descriptors, fork, signals, terminals, and POSIX edge cases.

Implications:

- Bash is out of scope.
- POSIX compatibility is out of scope.
- Native shell and commands are in scope.
- Toolchains may be ported later using native ABI support.
- WASM components are the preferred untrusted extension and automation format.
- Text output is a display/fallback format, not the primary data model.

## ADR-003: Architecture-Neutral Policy From Day One

Decision:

Generic kernel policy must be separated from architecture-specific mechanism.

Rationale:

The project should be future-ready for Intel, AMD, and Arm. x86_64 is first, but it must not define the whole kernel.

Implications:

- `aesynx-arch` traits come early.
- `aesynx-arch-x86_64` is the first real backend.
- `aesynx-arch-aarch64` exists as a planned/stub backend.
- Generic scheduler, capability, object, IPC, driver, and AI policy code must not contain raw x86 assumptions.

## ADR-004: Focused Crates And Small Modules

Decision:

Aesynx uses focused crates and modules from the beginning. Large one-file crates
or giant `.rs` files are not acceptable as the normal implementation style.

Rationale:

Security review, unsafe-code audit, fuzzing, model checking, and future fixes
all get harder when behavior is hidden in huge files. The project should follow
the workspace style used by mature Rust projects: split by ownership,
subsystem, and reason to change.

Implications:

- `src/` at the repository root is disallowed.
- Implementation lives under `crates/`, `models/`, `tools/`, and `tests/`.
- `lib.rs` wires modules and exports APIs.
- Large files must be split before they exceed policy thresholds.
- `scripts/validate-modularity-policy.sh` is part of the local check gate.

## ADR-005: Capabilities Are The Authority Model

Decision:

Capabilities are the core security and authority model.

Rationale:

Capabilities give explicit authority, revocation points, auditability, and a path toward CHERI-like hardware support later.

Implications:

- Protected kernel APIs should accept `CapId`, not raw authority.
- Driver resources are represented as caps.
- User boot info grants caps.
- Cross-core authority transfer is a grant operation, not an integer copy.

## ADR-006: Deterministic First, AI-Assisted Later

Decision:

AI is prepared from day one through telemetry, schemas, policy interfaces, and model objects. AI does not make mandatory kernel decisions in 1.0.

Rationale:

An opaque scheduler or driver policy would be dangerous early. The right foundation is instrumentation and deterministic fallback.

Implications:

- Structured telemetry is required before AI policy.
- Every AI policy has fallback.
- Model objects are immutable and safety-checked.
- AI cannot bypass capability checks.

## ADR-007: Drivers Become Services

Decision:

Drivers should move toward isolated, restartable, capability-limited services. Bootstrap drivers can be trusted in-kernel.

Rationale:

Driver isolation is one of the strongest reasons to build the OS. Classic unrestricted kernel modules are not the desired model.

Implications:

- Device resources are caps.
- Driver lifecycle has quiesce/drain/revoke/reset stages.
- No untrusted native kernel modules by default.
- IOMMU/DMA isolation is a long-term requirement.
- Driver code belongs in a top-level `drivers/` area or external signed driver
  packages, not inside the core kernel crate.
- Community and vendor drivers should install through the package/driver
  manager as generation-published driver services.
- Closed-source vendor drivers may exist only behind the same service ABI,
  signed package policy, explicit capability grants, and visible trust/taint
  state.

Reference: [Aesynx Driver Roadmap](driver-roadmap.md).

## ADR-008: Object Graph Instead Of Files As Core Model

Decision:

The core storage model is an immutable object graph. Human-friendly names are name-index objects, not a traditional filesystem requirement.

Rationale:

Immutable objects fit capability security, rollback, auditability, signed updates, and future AI trace/model objects.

Implications:

- Boot bundle is an object bundle.
- Shell can expose `/bin`-like names, but internally they resolve to object IDs.
- POSIX path semantics are not a kernel requirement.
- Persistent storage should be content-addressed immutable objects with
  versioned root/name-index references.
- Snapshots are retained object roots. Rollback is an atomic root-reference
  change authorized by capabilities, not a path-first filesystem operation.
- FAT32 may be used as a read-only EFI boot shim, but it is not the native
  storage model.

## ADR-009: Componentized System, Not One Huge Binary

Decision:

Aesynx must preserve independently versioned components. A signed boot capsule
or image may package many objects together, but the running design must not
collapse kernel, drivers, services, commands, policies, and models into one
monolithic binary.

Rationale:

Independent update and rollback are security features. A monolithic binary
makes driver replacement, service restart, component attestation, object-root
rollback, and audit harder.

Implications:

- Kernel internals can evolve behind stable service/ABI contracts.
- Drivers move toward isolated services.
- Commands and services carry manifests.
- Object roots can be snapshotted and rolled back.
- Boot bundles preserve internal component identity.
- Release tooling should treat monolithic growth as a regression.

## ADR-010: AMP/Multikernel Ownership Is The Scalability Model

Decision:

Aesynx uses platform SMP mechanisms to bring cores online, but the long-term
kernel model is software-defined AMP and multikernel ownership. Kernel state is
owned by cores or explicit service domains. Cross-core interaction uses bounded
messages, versioned fabric protocols, replicated authority epochs, and
capability-aware handoff.

Rationale:

This matches the multikernel direction and future many-core/heterogeneous
machines. It also avoids designing the kernel around one shared global state
protected by locks.

Implications:

- No global scheduler lock as final design.
- No global object registry as final design.
- No global allocator lock as final design.
- Device IRQs should route to the driver/service core that owns the device.
- Heterogeneous cores should be represented by role/capability metadata rather
  than hidden behind a fake "all cores are equal" abstraction.
- Global authority changes must use owner/coordinator and epoch protocols, not
  hidden singleton mutation.
- Routing, backpressure, and service failure become fabric concerns.
- Restartable driver/service domains require heartbeat, quarantine, revoke, DMA
  cleanup, and rebinding policy before zero-downtime claims.
- Early global bootstrap state must be explicitly temporary.

## ADR-011: Future Bootloader As Minimal Security Gateway

Decision:

Aesynx may eventually replace Limine with a Rust `no_std`, UEFI-first
bootloader, but only as a small security gateway.

Rationale:

GRUB-style feature breadth creates a large pre-boot attack surface. Aesynx
should avoid filesystem-driver collections, scripting languages, shells, and
complex UI in the bootloader.

Implications:

- Current milestones may use Limine.
- Future Aesynx bootloader reads a signed boot capsule from the ESP through
  UEFI services.
- It verifies signatures, measures boot state into TPM PCRs where available,
  and hands off quickly.
- Rich recovery UI belongs in a verified Aesynx recovery capsule, not the
  bootloader itself.
- Bootloader configuration is declarative state, not executable code.

## ADR-012: Capsules Instead Of Linux-Style Containers As The Native Model

Decision:

Aesynx should support container-like workflows through native capsules:
isolated object roots, explicit capabilities, resource budgets, and virtualized
service endpoints. A hosted runtime may later run Aesynx userspace on another
host kernel for development and CI.

Rationale:

Linux containers are tightly coupled to Linux namespaces, cgroups, mounts,
signals, and filesystem semantics. Copying that model would pull Aesynx toward
Unix compatibility before the native object/capability model is mature.

Implications:

- Capsules are native Aesynx isolation units, not OCI/Linux containers.
- Hosted Aesynx execution is useful, but it maps to the Aesynx component/object
  ABI rather than defining a POSIX ABI.
- Micro-VM or Linux compatibility support is a later service layer, not a 1.0
  kernel goal.

## ADR-013: Package Management Uses Immutable Object Generations

Decision:

Aesynx package management should use content-addressed immutable package
objects, declarative profile generations, signed registry tracks, SBOM and
provenance objects, and capability manifests. See
[Aesynx Package Manager Roadmap](package-manager-roadmap.md).

Rationale:

Traditional package managers mutate shared global filesystem trees and often
execute privileged install scripts. That conflicts with Aesynx's object-native,
capability-native, rollback-capable design.

Implications:

- Installing, removing, updating, and rolling back packages publishes generation
  roots instead of mutating shared directories.
- Core, official, community, market, sovereign, and vendor packages are
  separated by local track policy and trust roots.
- Package manifests describe requested capabilities, but launch policy grants
  actual authority.
- The future store UI is a client of the package service, not a privileged disk
  mutator.
- Paid entitlements permit fetch/decryption only; they do not grant runtime
  authority.

## ADR-014: Post-Quantum Readiness Through Crypto Agility

Decision:

Aesynx must be post-quantum ready by design. Trust metadata should use
versioned, algorithm-identified signature and key-establishment envelopes
instead of assuming one permanent public-key algorithm. See
[Post-Quantum Readiness Roadmap](post-quantum-readiness.md).

Rationale:

Quantum risk affects boot trust, package trust, updates, remote attestation,
secure channels, paid entitlements, and signed model/policy objects before it
affects direct operating-system support for quantum processors. A future
quantum processor is an accelerator/driver problem; quantum-resistant security
is a trust-format and migration problem.

Implications:

- Boot capsules and package manifests use signature envelopes, not single
  signature fields.
- Stable ABIs avoid fixed public-key, signature, KEM ciphertext, and
  certificate-size assumptions.
- Critical trust paths can require hybrid classical plus post-quantum
  validation once crypto providers exist.
- Unknown algorithms are rejected by default unless local policy admits them.
- Cryptographic migration is represented as signed generation state with audit
  evidence, not in-place mutation.
