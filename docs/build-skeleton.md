# Aesynx Build Skeleton

Status: v0.2 foundation

The repository contains the first x86_64 kernel build shape:

- `targets/x86_64-unknown-aesynx.json`
- `linker/kernel-x86_64.ld`
- `.cargo/config.toml`
- `cargo xtask build-kernel`
- `cargo xtask image`
- `cargo xtask qemu`

## Stable Rust Rule

Aesynx targets Rust stable `1.96.0`. Custom JSON targets usually require a
`build-std` path for `core`, and that is not enabled as the default project
path yet. Until the boot pipeline is ready, `cargo xtask build-kernel` performs
the stable host validation for `aesynx-kernel` and verifies that the custom
target and linker files exist.

Nightly-only build paths must be documented as exceptions before they are used.

## Current Commands

```bash
cargo xtask build-kernel
```

Validates the kernel crate and build skeleton.

```bash
cargo xtask image
cargo xtask qemu
```

These commands intentionally return a controlled "not implemented until
v0.3.0" failure until the boot image pipeline exists.

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
