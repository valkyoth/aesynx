# Aesynx Build Skeleton

Status: v0.15 page-table-mapper implementation candidate

The repository contains the first x86_64 kernel build shape:

- `targets/x86_64-unknown-aesynx.json`
- `linker/kernel-x86_64.ld`
- `.cargo/config.toml`
- `cargo xtask build-kernel`
- `cargo xtask build-kernel --custom-target-probe`
- `x86_64-unknown-none` stable boot target
- `cargo xtask image`
- `cargo xtask qemu`

## Stable Rust Rule

Aesynx targets Rust stable `1.96.0`. Custom JSON targets usually require a
`build-std` path for `core`, and that is not enabled as the default project
path yet. Until the boot pipeline is ready, `cargo xtask build-kernel` performs
the stable host validation for `aesynx-kernel` and verifies that the custom
target, linker, and Cargo config files contain the required release markers.

Confirmed with Cargo `1.96.0`: `cargo build -Z build-std=core --target
targets/x86_64-unknown-aesynx.json -p aesynx-kernel` is still rejected on the
stable channel. The project should not rely on that path unless a future
milestone explicitly documents a nightly exception or a stable alternative.

Nightly-only build paths must be documented as exceptions before they are used.
For experimentation, `cargo xtask build-kernel --custom-target-probe` attempts
the explicit nightly build-std path with `cargo +nightly`. That command is not
the v0.2 release gate.

## Current Commands

```bash
cargo xtask build-kernel
```

Validates the kernel crate and build skeleton, then builds the release-profile
freestanding `x86_64-unknown-none` kernel ELF used by the QEMU image.

```bash
cargo xtask build-kernel --custom-target-probe
```

Attempts the custom JSON target with nightly Cargo `build-std`. This is an
explicit probe for the future kernel-object path, not a stable requirement.

```bash
cargo xtask image
cargo xtask qemu
cargo xtask qemu --panic-smoke
cargo xtask qemu --exception-smoke
cargo xtask qemu --timer-smoke
```

`cargo xtask image` creates `build/qemu/aesynx-v0.15.0.iso` with Limine and the
release Rust kernel ELF. The image manifest records the Rust, Limine, xorriso,
and QEMU version banners. `cargo xtask qemu` starts QEMU, captures serial
output, and expects `[TEST] gdt=ok`, `[TEST] idt=ok`,
`[TEST] irq=ok`, `[TEST] exception=ok`, `memory total_bytes=`,
`memory usable_bytes=`, `memory reserved_bytes=`, `[TEST] memory-map=ok`,
`frame-allocator total_frames=`, `[TEST] frame-allocator=ok`,
`page-table total_tables=`, `root_ok=true`, `checked_root_ok=true`,
`checked_status_ok=true`, `kernel_candidate_ok=true`,
`user_candidate_ok=true`,
`translate_offset_ok=true`, `checked_translate_ok=true`,
`mapping_lookup_ok=true`, `presence_ok=true`, `protect_ok=true`,
`protect_range_ok=true`, `range_lookup_ok=true`, `range_translate_ok=true`,
`mapped_range_ok=true`, `unmapped_range_ok=true`, `audit_ok=true`,
`kernel_range_ok=true`, `user_range_ok=true`, `write_protected_range_ok=true`,
`non_executable_range_ok=true`, `executable_range_ok=true`,
`normal_memory_range_ok=true`, `local_range_ok=true`,
`kernel_space_range_ok=true`, `user_space_range_ok=true`,
`no_user_space_ok=true`, `no_executable_ok=true`, `no_writable_ok=true`,
`no_device_ok=true`, `no_global_ok=true`, `no_alias_ok=true`,
`kernel_user_guard_ok=true`, `kernel_only_ok=true`, `visit_ok=true`,
`flags_ok=true`, `reclaim_ok=true`, `range_ok=true`, `flush_page=true`,
`[TEST] page-table=ok`,
`[TEST] bootinfo=ok`, and `[TEST] boot=ok`.

`cargo xtask qemu --panic-smoke` creates a separate
`build/qemu/aesynx-v0.15.0-panic.iso`, enables the kernel `panic-smoke` feature,
and expects `[TEST] idt=ok`, `[TEST] irq=ok`, `[TEST] exception=ok`, and
`[TEST] panic=ok`.

`cargo xtask qemu --exception-smoke` creates a separate
`build/qemu/aesynx-v0.15.0-exception.iso`, enables the kernel
`exception-smoke` feature, and expects `[TEST] pagefault=ok`,
`[TEST] irq=ok`, `[TEST] exception=ok`, `cr2_present=`, `cr2_offset=0x`,
`cr3_offset=0x`, `rflags=0x`, `interrupts_enabled=`, and decoded page-fault
error fields.

`cargo xtask qemu --timer-smoke` creates a separate
`build/qemu/aesynx-v0.15.0-timer.iso`, enables the kernel `timer-smoke` feature,
programs PIT IRQ0 as the chosen QEMU timer source, enables interrupts only for
that controlled smoke path, converts ticks into monotonic instants, wakes one
bounded sleep request, and expects `timer tick 1`, `timer tick 2`,
`timer delayed-log`, `[TEST] sleep=ok`, `timer tick 3`, and `[TEST] timer=ok`.

The tracked `.cargo/config.toml` uses a repo-local Rust compiler wrapper that
computes the workspace root dynamically and passes
`--remap-path-prefix <workspace>=.` for direct workspace builds. Xtask kernel
builds also pass the same remap through encoded Rust flags as portable
defense-in-depth for the release image path. Kernel rustflags also disable
SSE/AVX code generation until Aesynx owns explicit FPU/SIMD context
management. The panic handler still emits only an escaped filename basename.

The v0.15 image proves that Limine can load the Rust kernel ELF, reach `_start`,
install basic x86_64 GDT/TSS/IDT state, remap and mask legacy PIC IRQs, detect
local APIC availability for the deferred MMIO path, handle a returning breakpoint
exception, catch and decode an opt-in page fault, run a controlled PIT-backed
timer IRQ0 smoke test, convert ticks into monotonic time, wake a bounded sleep
request for a delayed log event, provide handoff metadata that normalizes into
Aesynx `BootInfo`, and emit checked physical memory accounting with total,
usable, reserved, and frame counts. It seeds a bounded early bitmap allocator
from a usable memory-map window and verifies one-frame allocation/free,
contiguous allocation/free, debug state, and double-free detection. It also
exercises a bounded x86_64-shaped page-table mapper model with typed root-table
identity, checked status, map, fail-closed single-address translation, checked
byte-range translation, permission lookup, contiguous range lookup, permission
change, contiguous range map/protect/unmap,
unmapped range checks, read-only mapping visit, virtual range permission
verification, kernel-space and user-space virtual range policy, high-half
kernel user-access guard policy, low-half user kernel-privilege guard policy,
non-empty kernel/user address-space candidate preflights, no-alias policy,
fail-closed malformed leaf decoding, unmap, consistency audit,
empty-table reclamation, and explicit TLB flush targets. It does not claim
active CR3 replacement, production page-table
ownership, APIC MMIO activation, global physical-memory ownership, heap
allocation, page-fault recovery, a calibrated production clock service,
scheduler preemption, or bootloader memory reclamation.

## Target Shape

The first target is x86_64 QEMU with:

- Little-endian 64-bit pointers.
- Red zone disabled.
- Static relocation model.
- Kernel code model.
- Abort panics.
- `rust-lld` as linker.
- Limine page-permission-compatible ELF load segments.

The target file is version-controlled so future linker, bootloader, and QEMU
changes are reviewable.
