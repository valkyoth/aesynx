# Aesynx Supply-Chain Security

Status: policy

Aesynx is an operating-system project. Build tools, dependencies, CI workflows,
boot assets, generated images, and release scripts are part of the security
boundary.

## Dependency Rules

- Dependencies must come from crates.io unless explicitly reviewed.
- Git dependencies require pinned revisions.
- Build scripts and procedural macros are executable code and require review.
- `*-sys` crates and vendored native code require architecture and license
  review.
- License exceptions must be narrow and documented in `deny.toml`.
- Advisory ignores must include an exposure analysis and removal condition.
- Unknown registries and unknown git sources are denied.

## Required Checks

Baseline:

```bash
scripts/checks.sh
cargo deny check
cargo audit
scripts/generate-sbom.sh
```

Future release gates:

- Reproducible kernel/image build check.
- QEMU boot smoke with serial markers.
- SBOM for source workspace and generated release artifacts.
- Release notes with artifact checksums.
- Signed tag verification.
- Toolchain and bootloader version capture.

## Review Triggers

Require security review for:

- New dependencies.
- Dependency updates that add build scripts or proc macros.
- New CI actions.
- New release scripts.
- New bootloader/image-generation tools.
- New WASM runtime dependencies.
- New cryptography, random number, or signature dependencies.
- New AI model loading or inference dependencies.

## Non-Claims

The current repository has planning docs and security scaffolding. It does not
yet provide a reproducible kernel image, signed release artifacts, or verified
dependency closure.

