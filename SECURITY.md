# Security Policy

Aesynx is a security-sensitive operating-system project. Treat changes to boot,
memory, page tables, capabilities, IPC, userspace ABI, driver authority,
WebAssembly execution, telemetry, AI policy, build tooling, and dependency
metadata as high-risk until reviewed and tested.

## Supported Versions

Aesynx has no released version yet. Security fixes apply to the main development
line until the first release branch exists.

## Reporting

Do not publish exploitable security details before a fix is available. Once the
repository has public security channels configured, use a private security
advisory. Until then, contact the maintainers directly.

Include:

- Affected commit or tag.
- Target architecture and QEMU/hardware configuration.
- Reproducer, trace, or input corpus when safe to share.
- Whether the issue affects boot, memory, capabilities, IPC, drivers,
  userspace, WASM, AI policy, or supply chain.
- Whether secrets, object capabilities, DMA authority, or kernel memory can be
  exposed or modified.

## Security Bar

Required before release candidates:

```bash
scripts/checks.sh
cargo deny check
cargo audit
scripts/generate-sbom.sh
scripts/validate-release-readiness.sh vX.Y.Z
```

GitHub security automation must include CodeQL default setup for Rust before
tagging a release. Do not add a custom CodeQL workflow while default setup is
enabled, because GitHub rejects SARIF from mixed default and advanced
configurations.

No release tag is allowed until the exact commit being tagged has a completed
pentest report with `Status: PASS`. The report belongs in
`security/pentest/<tag>.md`, and the release-readiness script verifies it.
Temporary pentest findings are exchanged through ignored root `PENTEST.md`;
release-scope findings must be reviewed, addressed where appropriate, and
`PENTEST.md` deleted before committing.

As implementation matures, the gate must expand to include:

- QEMU boot smoke tests.
- Panic/fault smoke tests.
- Capability model tests.
- IPC ring model tests.
- Object graph model tests.
- Bytecode verifier tests.
- Fuzz targets for parsers, object manifests, command manifests, and bytecode.
- Miri on host model crates where useful.
- Kani or equivalent bounded verification for critical pure logic.
- Reproducible build checks for release artifacts.
- Modularity checks that prevent giant source files and force subsystem splits.

## Dependency Policy

The dependency policy lives in `deny.toml`.

Rules:

- Unknown registries are denied.
- Unknown git sources are denied.
- Git dependencies require a pinned revision.
- License exceptions must be narrow, named, and documented.
- Build scripts, procedural macros, `*-sys` crates, vendored native code,
  GitHub workflow edits, and release script edits are executable supply-chain
  changes.
- Advisory ignores must include a reason, exposure analysis, and removal
  condition.

## Unsafe Rust Policy

The default stance is safe Rust. Unsafe code is allowed only at reviewed
boundaries:

- Architecture entry and CPU setup.
- Interrupt/trap entry.
- Context switching.
- Page table manipulation.
- MMIO and port I/O.
- Atomic queue internals.
- Allocator internals.
- Bootloader handoff parsing.

Every unsafe block must have a local `SAFETY:` explanation. Every unsafe module
must be listed in `docs/unsafe-policy.md` before release.

## Modularity Policy

Aesynx must be split into focused crates and modules. Large one-file
implementations are a security risk because they are hard to review, fuzz,
prove, and safely modify. `docs/modularity-policy.md` defines the file-size and
crate-boundary rules, and `scripts/checks.sh` runs the modularity validator.

## Kernel Security Priorities

Aesynx security work focuses on:

- Explicit capabilities over ambient authority.
- Generation-checked and revocable handles.
- Per-core ownership and message passing.
- No global mutable object registry as the final design.
- Driver MMIO/IRQ/DMA capabilities.
- IOMMU or explicit trusted/degraded mode for DMA-capable devices.
- Structured, redacted telemetry.
- Object graph integrity and rollback.
- Deterministic fallback for every AI-assisted policy.

## Userspace Security Priorities

Native userspace is not Unix-compatible by default.

Rules:

- Commands receive explicit capabilities.
- Command manifests declare required authority and output types.
- WASM components are sandboxed by default.
- AI assistants cannot gain capabilities, run commands, or inspect objects
  without explicit authority.
- Text is a display/fallback format, not the primary authority or data model.

## Non-Claims

Aesynx does not yet claim:

- Production-ready isolation.
- Real hardware safety.
- Formal verification.
- Stable ABI.
- Secure boot or measured boot.
- Complete driver sandboxing.
- Constant-time behavior for all security-sensitive paths.
- Safe execution of untrusted WASM modules.
- AI policy safety beyond the documented design constraints.
