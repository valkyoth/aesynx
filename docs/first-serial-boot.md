# Aesynx First Serial Boot

Status: v0.4 implementation candidate

`v0.4.0` replaces the temporary v0.3 stage-0 probe with a Rust kernel entry.
The image path now builds a freestanding `x86_64-unknown-none` ELF, packages it
with Limine, boots it in QEMU, and validates serial output from Rust `_start`.

## Commands

```bash
cargo xtask build-kernel
cargo xtask image
cargo xtask qemu
```

`cargo xtask image` creates:

```text
build/qemu/aesynx-v0.4.0.iso
build/qemu/aesynx-v0.4.0.manifest
```

`cargo xtask qemu` expects:

```text
Aesynx: booting
arch=x86_64 platform=qemu
[TEST] boot=ok
```

## Boot Path

- `boot/qemu/limine.conf` configures Limine.
- `crates/aesynx-kernel/src/main.rs` exports `_start`.
- `crates/aesynx-arch-x86_64/src/serial.rs` provides the safe COM1 writer.
- `crates/aesynx-arch-x86_64/src/port.rs` contains the reviewed port-I/O unsafe
  boundary.
- `linker/kernel-x86_64.ld` keeps load segments page-separated so Limine does
  not load mixed-permission program headers onto the same page.

## Boundary

This milestone proves:

- Stable Rust can build the first freestanding kernel ELF.
- Limine can load the ELF in QEMU.
- Rust `_start` runs.
- COM1 serial output works.
- The QEMU smoke test validates a kernel-owned marker.

This milestone does not prove:

- BootInfo parsing.
- Memory-map normalization.
- Page-table ownership.
- Interrupts or exceptions.
- Panic diagnostics beyond a minimal serial fallback.
- Allocator setup.
