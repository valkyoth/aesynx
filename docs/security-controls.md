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
| Post-quantum readiness | Boot, package, update, entitlement, and identity metadata must stay crypto-agile and support future hybrid validation | Planned | `docs/post-quantum-readiness.md`, `docs/ARCHITECTURE_DECISIONS.md` |
| Static analysis | GitHub CodeQL default setup for Rust | Configured externally | GitHub code scanning default setup |
| Release pentest | Passing pentest report required before every tag | Configured | `scripts/validate-release-readiness.sh`, `security/pentest/README.md` |
| Capabilities | No ambient authority as design center | Model active | `crates/aesynx-cap`, `docs/IMPLEMENTATION_PLAN.md` |
| Capability audit | Derive/grant authority transfer must use audited call paths | Model active | `CapAuditLog`, `derive_with_audit`, `grant_with_audit` |
| Capability unforgeability | Capability fields are private and capability values are not `Copy` or `Clone` | Model active | `Capability` accessors |
| CI action integrity | Workflow actions must be pinned to commit SHA | Active | `scripts/validate-security-policy.sh`, `.github/workflows/ci.yml` |
| State transitions | Task and device state changes must use checked transition APIs | Model active | `Task::transition`, `DeviceObject::transition` |
| Boot address secrecy | KASLR-sensitive kernel image, RSDP, HHDM, device-tree, framebuffer, and memory-region physical starts are private or debug-redacted | Active candidate | `KernelImageInfo`, `MemoryRegion`, `BootInfo`, `BootMetadata`, `cargo xtask qemu` |
| BootInfo normalization | Bootloader-specific handoff data normalizes before generic kernel use; Limine response pointers are null/alignment/higher-half checked and request statics live in the RW handoff segment | Active candidate | `aesynx-boot`, `crates/aesynx-kernel/src/limine.rs`, `linker/kernel-x86_64.ld` |
| Physical memory accounting | Boot memory maps are checked for empty regions, region overflow, overlapping ranges, total/usable/reserved bytes, kind-specific reserved ranges, and full-frame counts before allocator work can trust them | Active candidate | `aesynx-boot::MemoryMap`, `cargo xtask qemu` |
| Frame allocation | Early bitmap allocator tracks known/free/used/reserved frame state, rejects overlapping seeds, detects double-free, validates region seeding and contiguous frees before committing bitmap mutations, rejects impossible private bitmap combinations on checked status reporting, keeps allocator status fields read-only through public accessors, avoids unchecked status arithmetic underflow, and QEMU-smokes the checked status path against a bounded usable boot-map window | Active candidate | `aesynx-mm::BitmapFrameAllocator`, `cargo xtask qemu` |
| Page-table mapping | Safe bounded mapper model validates canonical/aligned addresses, exposes a typed non-physical root identity, reports checked status only after audit validation, gates kernel address-space candidates through audit/root/status/non-empty/kernel-only/no-user-space/no-device/no-alias checks with a consolidated read-only shape walk, gates user address-space candidates through audit/root/status/non-empty/high-half-user/low-half-kernel/has-low-user/no-device/no-global/no-alias checks with a consolidated read-only shape walk, keeps candidate preflights read-only on success and failure, audits standalone mapping policy helpers before policy evaluation, rejects double maps, keeps device/cache flags behind builders/accessors, audits single-page map/protect/unmap before mutating existing mapper state, maps/unmaps 4 KiB pages, exposes audit-backed fail-closed single-address and contiguous byte-range translation plus read-only permission lookup, page-presence reporting, and range checks, redacts page-table debug output to aggregate counters, non-address fields, and private slot/range classifications, fails closed on malformed non-empty leaves, malformed non-empty raw slots, malformed next-table links, decoded root-table child links, dangling or out-of-range child links, invalid internal helper/table-path indices, invalid zero-page virtual-space/range-walk validation, empty-arena capacity/table-path checks, and invalid reclaim paths, validates reclaim spans and parent-child slot links before committing slot/table changes, checks mapped/unmapped/protected/kernel/user/executable/normal-memory/local ranges plus high-half kernel-space and low-half user-space ranges, rejects empty candidate address spaces, rejects kernel-only user candidates, rejects user-accessible mappings in the high-half kernel region, rejects kernel-privileged mappings in the low-half user region, rejects device mappings in candidate kernel address spaces, rejects device and global mappings in candidate user address spaces, rejects physical aliases when exclusive ownership is required, changes mapping permissions without changing frames, reclaims empty intermediate tables, carries generic page flags, and returns explicit TLB flush targets before any live CR3 activation claim | Active candidate | `aesynx-mm::PageTableMapper`, `cargo xtask qemu` |
| Object-native memory | Allocations and mappings should grow toward explicit owner, purpose, lifetime, sensitivity, sharing, snapshot, DMA, and revocation policy instead of anonymous ambient memory | Planned | `docs/memory-model-roadmap.md` |
| OS world model | Kernel and services emit bounded facts; native userspace world services provide policy-aware queries, context packs, projections, and AI-safe explanations without creating authority | Planned | `docs/os-world-roadmap.md`, `docs/userspace-vision.md` |
| Early diagnostics | Structured log-level records with validated components, escaped and bounded panic output, basename-only source location, boot phase, early core, message, arithmetic-only RFLAGS, redacted register summary, and QEMU panic marker | Active candidate | `aesynx-kernel::diagnostics`, `aesynx-arch-x86_64::registers`, `cargo xtask qemu --panic-smoke` |
| Kernel codegen boundary | Bare-metal kernel builds disable SSE/AVX until explicit FPU/SIMD context switching exists | Active candidate | `.cargo/config.toml`, `tools/xtask/src/kernel_flags.rs`, `cargo xtask build-kernel` |
| x86_64 descriptor tables | Early boot installs a private GDT, reloads CS/SS/DS/ES, nulls FS/GS selectors, loads TSS, and reserves a dedicated double-fault IST stack before generic kernel work | Active candidate | `aesynx-arch-x86_64::descriptors`, `cargo xtask qemu` |
| x86_64 exception tables | Early boot installs a private IDT and handlers for breakpoint, page fault, and double fault; page-fault smoke prints CR2 presence/page offset, CR3 low bits, RFLAGS summary, interrupt state, and decoded error bits | Active candidate | `aesynx-arch-x86_64::exceptions`, `cargo xtask qemu --exception-smoke` |
| x86_64 interrupt baseline | Legacy PIC is remapped out of exception vectors before masking; local APIC is detection-only until MMIO mapping exists; IRQ vectors and EOI paths are checked | Active candidate | `aesynx-arch-x86_64::interrupts`, `cargo xtask qemu` |
| x86_64 timer smoke | Opt-in PIT IRQ0 smoke installs a checked vector 0x20 handler, enables interrupts only for the controlled test, acknowledges each tick, and disables IRQ0 after the fixed three-tick target | Active candidate | `aesynx-arch-x86_64::timer`, `cargo xtask qemu --timer-smoke` |
| Monotonic time and sleep queue | Timer ticks convert into checked monotonic instants using the timer-provided rate; bounded sleep requests wake only when due and prefer the earliest due deadline; timeout checks fail closed on overflow | Active candidate | `aesynx-time`, `cargo xtask qemu --timer-smoke` |
| Telemetry integrity | Task telemetry uses append-only counters; core telemetry snapshots are advisory per-counter samples | Model active | `TaskTelemetry`, `CoreTelemetry` |
| Capability revocation | REVOKE authority check required before epoch mutation; epoch and generation counters must fail rather than wrap | Model active | `ensure_revoke_authority`, `RevocationEpochStore`, `Capability` |
| Early serial safety | COM1 output uses admitted ports, bounded polling, and a single-core marker type | Active for boot | `crates/aesynx-arch-x86_64/src/serial.rs`, `crates/aesynx-arch-x86_64/src/port.rs` |
| Drivers | Separate driver tree/packages, signed driver manifests, MMIO/IRQ/DMA caps, isolated driver services, and revocation lifecycle | Planned | `docs/driver-roadmap.md`, `docs/IMPLEMENTATION_PLAN.md` |
| WASM | Sandboxed extension model | Planned | `docs/userspace-vision.md` |
| AI | Bounded assistant and policy fallback | Planned | `docs/IMPLEMENTATION_PLAN.md`, `docs/userspace-vision.md` |
| QEMU testing | Boot smoke validates GDT/TSS/IDT setup, breakpoint handling, BootInfo, memory-map accounting, bounded frame-allocator smoke, bounded page-table mapper smoke, and Rust serial markers; panic smoke validates early panic diagnostics; exception smoke validates page-fault decoding; timer smoke validates three controlled PIT IRQ0 ticks plus delayed sleep wake markers | Active for boot | `cargo xtask qemu`, `cargo xtask qemu --panic-smoke`, `cargo xtask qemu --exception-smoke`, `cargo xtask qemu --timer-smoke`, `docs/RELEASE_PLAN.md` |
| Future bootloader | Rust UEFI bootloader must stay a minimal signed/measured security gateway | Planned | `docs/bootloader-roadmap.md` |
| Bootloader response pointers | Limine response references must be non-null, aligned, and higher-half kernel VMA pointers before Rust references are created | Active candidate | `crates/aesynx-kernel/src/limine.rs`, `cargo xtask qemu` |

## Admission Rule

Security-sensitive features should not graduate from planned to active until
they have:

- Tests or model checks.
- Documentation.
- Failure-mode analysis.
- Release-gate coverage.
- Clear non-claims.
