# Aesynx Security Controls

Status: baseline control map

This document maps the security posture Aesynx should grow into. It is not a
claim that the controls are implemented today.

| Area | Control | Current Status | Evidence |
| --- | --- | --- | --- |
| Project naming | Single project name, no retired names | Active | `scripts/validate-security-policy.sh` |
| Dependency policy | License, source, advisory, and duplicate-version checks | Configured | `deny.toml` |
| Security reporting | Private-first vulnerability process | Configured | `SECURITY.md` |
| Unsafe code | Unsafe confined to documented boundaries | Active boundaries documented | `docs/unsafe-policy.md`, `crates/aesynx-arch-x86_64/src/port.rs`, `crates/aesynx-kernel/src/limine.rs` |
| Kernel engineering | `no_std`, internal primitives, minimal unsafe, external dependency exceptions | Configured | `docs/kernel-engineering-policy.md`, `scripts/validate-kernel-policy.sh` |
| Modularity | Focused crates/modules, no giant source files | Configured | `docs/modularity-policy.md`, `scripts/validate-modularity-policy.sh` |
| Componentization | System must not collapse into one huge OS binary; components remain independently versioned and rollback-capable | Policy active | `docs/modularity-policy.md`, `docs/ARCHITECTURE_DECISIONS.md` |
| Supply chain | Executable dependency and workflow changes require review | Policy only | `docs/supply-chain-security.md` |
| Static analysis | GitHub CodeQL default setup for Rust | Configured externally | GitHub code scanning default setup |
| Release pentest | Passing pentest report required before every tag | Configured | `scripts/validate-release-readiness.sh`, `security/pentest/README.md` |
| Capabilities | No ambient authority as design center | Model active | `crates/aesynx-cap`, `docs/IMPLEMENTATION_PLAN.md` |
| Capability audit | Derive/grant authority transfer must use audited call paths | Model active | `CapAuditLog`, `derive_with_audit`, `grant_with_audit` |
| Capability unforgeability | Capability fields are private and capability values are not `Copy` or `Clone` | Model active | `Capability` accessors |
| CI action integrity | Workflow actions must be pinned to commit SHA | Active | `scripts/validate-security-policy.sh`, `.github/workflows/ci.yml` |
| State transitions | Task and device state changes must use checked transition APIs | Model active | `Task::transition`, `DeviceObject::transition` |
| Boot address secrecy | KASLR-sensitive kernel image, RSDP, HHDM, device-tree, framebuffer, and memory-region physical starts are private or debug-redacted | Active candidate | `KernelImageInfo`, `MemoryRegion`, `BootInfo`, `BootMetadata`, `cargo xtask qemu` |
| BootInfo normalization | Bootloader-specific handoff data normalizes before generic kernel use; Limine response pointers are null/alignment checked and request statics live in the RW handoff segment | Active candidate | `aesynx-boot`, `crates/aesynx-kernel/src/limine.rs`, `linker/kernel-x86_64.ld` |
| Early diagnostics | Structured log-level records with validated components, escaped and bounded panic output, basename-only source location, boot phase, early core, message, arithmetic-only RFLAGS, redacted register summary, and QEMU panic marker | Active candidate | `aesynx-kernel::diagnostics`, `aesynx-arch-x86_64::registers`, `cargo xtask qemu --panic-smoke` |
| x86_64 descriptor tables | Early boot installs a private GDT, TSS, and dedicated double-fault IST stack before generic kernel work | Active candidate | `aesynx-arch-x86_64::descriptors`, `cargo xtask qemu` |
| Telemetry integrity | Task telemetry uses append-only counters; core telemetry snapshots are advisory per-counter samples | Model active | `TaskTelemetry`, `CoreTelemetry` |
| Capability revocation | REVOKE authority check required before epoch mutation; epoch increment must fail rather than wrap | Model active | `ensure_revoke_authority`, `RevocationEpochStore` |
| Early serial safety | COM1 output uses admitted ports, bounded polling, and a single-core marker type | Active for boot | `crates/aesynx-arch-x86_64/src/serial.rs`, `crates/aesynx-arch-x86_64/src/port.rs` |
| Drivers | MMIO/IRQ/DMA caps and revocation lifecycle | Planned | `docs/IMPLEMENTATION_PLAN.md` |
| WASM | Sandboxed extension model | Planned | `docs/userspace-vision.md` |
| AI | Bounded assistant and policy fallback | Planned | `docs/IMPLEMENTATION_PLAN.md`, `docs/userspace-vision.md` |
| QEMU testing | Boot smoke validates GDT/TSS setup, BootInfo, and Rust serial markers; panic smoke validates descriptor setup and early panic diagnostics | Active for boot | `cargo xtask qemu`, `cargo xtask qemu --panic-smoke`, `docs/RELEASE_PLAN.md` |
| Future bootloader | Rust UEFI bootloader must stay a minimal signed/measured security gateway | Planned | `docs/bootloader-roadmap.md` |

## Admission Rule

Security-sensitive features should not graduate from planned to active until
they have:

- Tests or model checks.
- Documentation.
- Failure-mode analysis.
- Release-gate coverage.
- Clear non-claims.
