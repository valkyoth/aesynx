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
It is explicitly not planned as one huge OS binary: components should remain
separately identified, signed, versioned, updateable, and rollback-capable.

The first major goal is not a desktop OS and not a Unix clone. The first major
goal is a serious x86_64 QEMU research release with a coherent security model,
clear non-claims, and release gates that block tagging until checks and pentest
evidence are complete.

Aesynx is licensed under the European Union Public Licence 1.2.

## What Works Today

`v0.3.0` is the tagged QEMU image-skeleton line. `main` is currently carrying
the `v0.4.0` first Rust kernel serial-boot release candidate. The v0.4 pentest
follow-up is clear; the remaining pre-tag gate is GitHub CI and CodeQL green.
It builds a release-profile freestanding `x86_64-unknown-none` kernel ELF,
packages it into a Limine ISO, records build and boot tool versions in the
image manifest, boots it in QEMU, and verifies the kernel-owned serial marker.

| Area | Status | Notes |
| --- | --- | --- |
| Rust workspace | Active | Modular crate layout with no root `src/` implementation pile. |
| Toolchain | Active | Stable Rust `1.96.0`, edition 2024, resolver `3`, and `x86_64-unknown-none` for the first boot ELF. |
| Kernel crate policy | Active | Crates under `crates/` must be `no_std`, deny unsafe by default, and avoid external dependencies without exceptions. |
| Capability model | Model active | Private non-copy authority values, permission validation, audited derive/grant paths, generation/epoch validation, and revoke authority checks. |
| Memory model | Model active | Page flags make writable+executable and user-global mappings unrepresentable. |
| IPC model | Model active | Kernel-stamped message headers, caller requests, and bounded inline payloads. |
| Bytecode model | Model active | Fuel limit and capability-typed permission checks. |
| Logging model | Model active | Bounded single-record log messages. |
| Build path | Active | x86_64 target metadata, linker script, Cargo config validation, stable freestanding kernel ELF build, and an optional nightly custom-target probe. |
| QEMU first boot | Active | `cargo xtask image` creates a release-profile Limine ISO and `cargo xtask qemu` verifies `[TEST] boot=ok` from Rust `_start`. |
| Native snapshots | Planned | Content-addressed object roots make snapshots and rollback object-layer primitives rather than path-first filesystem features. |
| Future bootloader | Planned | Limine is current; a future Rust UEFI bootloader should be a minimal security gateway for signed/measured Aesynx boot capsules. |
| Supply-chain checks | Active | `cargo deny`, `cargo audit`, SBOM generation, Dependabot, SHA-pinned GitHub Actions, and CodeQL default Rust workflow. |
| Release gate | Active | Tags require local checks, SBOM, CodeQL on GitHub, and a passing pentest report for the exact commit. |

## Planned Next

| Area | Status | Target |
| --- | --- | --- |
| BootInfo normalization | Planned | `v0.5.0`; normalize Limine/bootloader metadata into generic Aesynx `BootInfo`. |
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

Validate the current kernel build path:

```bash
cargo xtask build-kernel
```

Create and smoke-test the v0.4 Limine QEMU image:

```bash
cargo xtask image
cargo xtask qemu
```

These commands require Limine 12.3.2 or newer, xorriso, and
`qemu-system-x86_64`. The generated manifest records the exact Rust, Limine,
xorriso, and QEMU version banners.

Try the documented custom-target experiment when a nightly toolchain is
available:

```bash
cargo xtask build-kernel --custom-target-probe
```

After a pentest report is completed for a tag:

```bash
cargo xtask release-ready v0.4.0
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
- [QEMU Image Skeleton](docs/qemu-image-skeleton.md)
- [First Serial Boot](docs/first-serial-boot.md)
- [Bootloader Roadmap](docs/bootloader-roadmap.md)
- [Storage Roadmap](docs/storage-roadmap.md)
- [Hosted Execution Roadmap](docs/hosted-execution-roadmap.md)
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
