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

## ADR-008: Object Graph Instead Of Files As Core Model

Decision:

The core storage model is an immutable object graph. Human-friendly names are name-index objects, not a traditional filesystem requirement.

Rationale:

Immutable objects fit capability security, rollback, auditability, signed updates, and future AI trace/model objects.

Implications:

- Boot bundle is an object bundle.
- Shell can expose `/bin`-like names, but internally they resolve to object IDs.
- POSIX path semantics are not a kernel requirement.

## ADR-009: Per-Core Ownership Is The Scalability Model

Decision:

Long-term kernel state is owned by cores. Cross-core interaction uses messages.

Rationale:

This matches the multikernel direction and future many-core/heterogeneous machines.

Implications:

- No global scheduler lock as final design.
- No global object registry as final design.
- No global allocator lock as final design.
- Early global bootstrap state must be explicitly temporary.
