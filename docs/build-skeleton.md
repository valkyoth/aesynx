# Aesynx Build Skeleton

Status: v0.6 early diagnostics implementation candidate

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
```

`cargo xtask image` creates `build/qemu/aesynx-v0.6.0.iso` with Limine and the
release Rust kernel ELF. The image manifest records the Rust, Limine, xorriso,
and QEMU version banners. `cargo xtask qemu` starts QEMU, captures serial
output, and expects `[TEST] bootinfo=ok` and `[TEST] boot=ok`.

`cargo xtask qemu --panic-smoke` creates a separate
`build/qemu/aesynx-v0.6.0-panic.iso`, enables the kernel `panic-smoke` feature,
and expects `[TEST] panic=ok`.

Xtask kernel builds pass `--remap-path-prefix <workspace>=.` through encoded
Rust flags so embedded panic source paths do not expose the local workspace
root. The panic handler still emits only an escaped filename basename.

The v0.6 image proves that Limine can load the Rust kernel ELF, reach `_start`,
provide handoff metadata that normalizes into Aesynx `BootInfo`, and produce
readable early panic diagnostics. It does not claim page-table ownership,
interrupts, memory allocation, exception handling, or bootloader memory
reclamation.

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
