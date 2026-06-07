# Aesynx QEMU Image Skeleton

Status: v0.3 candidate

`v0.3.0` introduces the first generated QEMU boot image. The image is a raw
1.44 MiB BIOS-bootable disk containing a temporary Aesynx stage-0 serial probe.

This is intentionally not the final kernel boot path. It exists to make image
generation and QEMU smoke testing real before the Rust kernel entry point lands
in `v0.4.0`.

## Commands

```bash
cargo xtask image
```

Creates:

```text
build/qemu/aesynx-v0.3.0.raw
build/qemu/aesynx-v0.3.0.manifest
```

```bash
cargo xtask qemu
```

Creates the image if needed, starts `qemu-system-x86_64`, captures serial
output, and expects:

```text
Aesynx v0.3.0 boot image skeleton
[TEST] bootloader=skeleton
```

The runner stops QEMU after the marker is observed.

## Tracked Boot Config

The stage-0 boot probe is configured by:

```text
boot/qemu/stage0.toml
```

The `xtask` image builder validates this file before writing generated image
artifacts.

## Boundary

This milestone proves:

- The repository can create a QEMU-consumable image.
- QEMU can boot the generated image.
- Serial capture works.
- The image/QEMU commands are no longer placeholders.

This milestone does not prove:

- Rust kernel `_start`.
- Limine or UEFI handoff.
- BootInfo parsing.
- Page-table setup.
- Kernel panic handling.

Those begin in later milestones.
