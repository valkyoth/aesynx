# Aesynx Security Controls

Status: baseline control map

This document maps the security posture Aesynx should grow into. It is not a
claim that the controls are implemented today.

| Area | Control | Current Status | Evidence |
| --- | --- | --- | --- |
| Project naming | Single project name, no retired names | Active | `scripts/validate-security-policy.sh` |
| Dependency policy | License, source, advisory, and duplicate-version checks | Configured | `deny.toml` |
| Security reporting | Private-first vulnerability process | Configured | `SECURITY.md` |
| Unsafe code | Unsafe confined to documented boundaries | Policy only | `docs/unsafe-policy.md` |
| Kernel engineering | `no_std`, internal primitives, minimal unsafe, external dependency exceptions | Configured | `docs/kernel-engineering-policy.md`, `scripts/validate-kernel-policy.sh` |
| Modularity | Focused crates/modules, no giant source files | Configured | `docs/modularity-policy.md`, `scripts/validate-modularity-policy.sh` |
| Supply chain | Executable dependency and workflow changes require review | Policy only | `docs/supply-chain-security.md` |
| Static analysis | GitHub CodeQL default setup for Rust | Configured externally | GitHub code scanning default setup |
| Release pentest | Passing pentest report required before every tag | Configured | `scripts/validate-release-readiness.sh`, `security/pentest/README.md` |
| Capabilities | No ambient authority as design center | Model active | `crates/aesynx-cap`, `docs/IMPLEMENTATION_PLAN.md` |
| Capability audit | Derive/grant authority transfer must use audited call paths | Model active | `CapAuditLog`, `derive_with_audit`, `grant_with_audit` |
| Capability unforgeability | Capability fields are private and capability values are not `Copy` or `Clone` | Model active | `Capability` accessors |
| CI action integrity | Workflow actions must be pinned to commit SHA | Active | `scripts/validate-security-policy.sh`, `.github/workflows/ci.yml` |
| State transitions | Task and device state changes must use checked transition APIs | Model active | `Task::transition`, `DeviceObject::transition` |
| Boot address secrecy | KASLR-sensitive kernel image addresses are private and debug-redacted | Model active | `KernelImageInfo` |
| Telemetry integrity | Task telemetry uses append-only counters; core telemetry snapshots are advisory per-counter samples | Model active | `TaskTelemetry`, `CoreTelemetry` |
| Capability revocation | REVOKE authority check required before epoch mutation | Model active | `ensure_revoke_authority`, `RevocationEpochStore` |
| Drivers | MMIO/IRQ/DMA caps and revocation lifecycle | Planned | `docs/IMPLEMENTATION_PLAN.md` |
| WASM | Sandboxed extension model | Planned | `docs/userspace-vision.md` |
| AI | Bounded assistant and policy fallback | Planned | `docs/IMPLEMENTATION_PLAN.md`, `docs/userspace-vision.md` |
| QEMU testing | Boot/fault/security smoke tests | Planned | `docs/RELEASE_PLAN.md` |

## Admission Rule

Security-sensitive features should not graduate from planned to active until
they have:

- Tests or model checks.
- Documentation.
- Failure-mode analysis.
- Release-gate coverage.
- Clear non-claims.
