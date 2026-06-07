<p align="center">
  <b>Rust no_std research OS with explicit capabilities, native userspace, and security gates from day one.</b><br>
  Modular by design. Capability-first. Built toward a serious QEMU release.
</p>

<div align="center">
  <a href="docs/IMPLEMENTATION_PLAN.md">Implementation Plan</a>
  |
  <a href="docs/RELEASE_PLAN.md">Release Plan</a>
  |
  <a href="docs/security-controls.md">Security Controls</a>
  |
  <a href="SECURITY.md">Security</a>
</div>

<br>

<p align="center">
  <img src="./.github/images/aesynx.webp" alt="Aesynx overview">
</p>

# Aesynx

Aesynx is a Rust `no_std` operating-system research project built around
explicit capabilities, per-core ownership, service queues, driver isolation, an
immutable object graph, native userspace, and AI-ready telemetry from day one.

The first major goal is not a desktop OS and not a Unix clone. The first major
goal is a serious x86_64 QEMU research release with a coherent security model,
clear non-claims, and release gates that block tagging until checks and pentest
evidence are complete.

Aesynx is licensed under the European Union Public Licence 1.2.

## What Works Today

`v0.1.0` is the tagged repository foundation line. `main` is currently carrying
the `v0.2.0` build-skeleton candidate. It is not bootable yet, but the project
structure, security baseline, and kernel build-shape checks are active.

| Area | Status | Notes |
| --- | --- | --- |
| Rust workspace | Active | Modular crate layout with no root `src/` implementation pile. |
| Toolchain | Active | Stable Rust `1.96.0`, edition 2024, resolver `3`. |
| Kernel crate policy | Active | Crates under `crates/` must be `no_std`, deny unsafe by default, and avoid external dependencies without exceptions. |
| Capability model | Model active | Permission validation, derive/grant checks, audited variants, generation/epoch validation, and revoke authority checks. |
| Memory model | Model active | Page flags make writable+executable access unrepresentable. |
| IPC model | Model active | Message types plus bounded inline payloads. |
| Bytecode model | Model active | Fuel limit and capability-typed permission checks. |
| Logging model | Model active | Bounded single-record log messages. |
| Build skeleton | Active | x86_64 target metadata, linker script, Cargo config validation, `cargo xtask build-kernel`, and an optional nightly custom-target probe. |
| Supply-chain checks | Active | `cargo deny`, `cargo audit`, SBOM generation, Dependabot, and CodeQL default Rust workflow. |
| Release gate | Active | Tags require local checks, SBOM, CodeQL on GitHub, and a passing pentest report for the exact commit. |

## Planned Next

| Area | Status | Target |
| --- | --- | --- |
| Boot image | Planned | `v0.3.0`; bootloader/image pipeline and controlled QEMU boot attempt. |
| First serial boot | Planned | Phase 1; print an Aesynx boot marker over serial. |
| Real arch mechanisms | Planned | Interrupt control, core identity, timestamp, page tables, and CPU setup. |
| Capability services | Planned | Concrete revocation epoch store, audit backend, object registry, and authenticated call paths. |
| Native userspace | Planned | `aesh`, structured pipelines, WASM components, and capability-scoped command execution. |

## Local Checks

Run the full repository gate:

```bash
scripts/checks.sh
```

Generate the source SBOM:

```bash
scripts/generate-sbom.sh
```

Validate the current kernel build skeleton:

```bash
cargo xtask build-kernel
```

Try the documented custom-target experiment when a nightly toolchain is
available:

```bash
cargo xtask build-kernel --custom-target-probe
```

After a pentest report is completed for a tag:

```bash
cargo xtask release-ready v0.1.0
```

## Security Posture

Aesynx treats boot, memory, capabilities, IPC, driver authority, userspace ABI,
WASM execution, telemetry, AI policy, build tooling, GitHub workflows, and
dependency metadata as high-risk. The project prefers internal kernel
primitives, narrow unsafe boundaries, no ambient authority, explicit
capabilities, and small modules that can be reviewed and tested.

Every release tag is blocked until the exact commit being tagged has a passing
pentest report in `security/pentest/<tag>.md`.

## Documentation

- [Implementation Plan](docs/IMPLEMENTATION_PLAN.md)
- [Userspace Vision](docs/userspace-vision.md)
- [Release Plan](docs/RELEASE_PLAN.md)
- [Architecture Decisions](docs/ARCHITECTURE_DECISIONS.md)
- [Build Skeleton](docs/build-skeleton.md)
- [Security Policy](SECURITY.md)
- [Threat Model](docs/threat-model.md)
- [Security Controls](docs/security-controls.md)
- [Supply-Chain Security](docs/supply-chain-security.md)
- [Kernel Engineering Policy](docs/kernel-engineering-policy.md)
- [Unsafe Policy](docs/unsafe-policy.md)
- [Modularity Policy](docs/modularity-policy.md)
- [Licensing Notes](docs/licensing.md)
- [License](LICENSE)
- [Initial Idea](docs/initial-idea.md)
