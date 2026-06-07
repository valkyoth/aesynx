# Aesynx Build Skeleton

Status: v0.3 image-skeleton implementation complete

The repository contains the first x86_64 kernel build shape:

- `targets/x86_64-unknown-aesynx.json`
- `linker/kernel-x86_64.ld`
- `.cargo/config.toml`
- `cargo xtask build-kernel`
- `cargo xtask build-kernel --custom-target-probe`
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

Validates the kernel crate and build skeleton.

```bash
cargo xtask build-kernel --custom-target-probe
```

Attempts the custom JSON target with nightly Cargo `build-std`. This is an
explicit probe for the future kernel-object path, not a stable requirement.

```bash
cargo xtask image
cargo xtask qemu
```

`cargo xtask image` creates `build/qemu/aesynx-v0.3.0.raw` with a temporary
stage-0 boot probe. `cargo xtask qemu` starts QEMU, captures serial output, and
expects `[TEST] bootloader=skeleton`.

The v0.3 image proves that image generation and QEMU launch work. It does not
claim a Rust kernel entry point; that starts in `v0.4.0`.

## Target Shape

The first target is x86_64 QEMU with:

- Little-endian 64-bit pointers.
- Red zone disabled.
- Static relocation model.
- Kernel code model.
- Abort panics.
- `rust-lld` as linker.

The target file is version-controlled so future linker, bootloader, and QEMU
changes are reviewable.
