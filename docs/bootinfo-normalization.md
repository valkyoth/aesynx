# Aesynx BootInfo Normalization

Status: v0.5 implementation candidate

`v0.5.0` starts the real bootloader handoff path. Limine remains the bootloader,
but kernel code normalizes Limine-specific responses into dependency-free
Aesynx `BootInfo` structures before the generic kernel uses them.

## Current Path

`crates/aesynx-kernel/src/limine.rs` owns the raw Limine request and response
boundary. It requests:

- Base revision 6.
- Memory map.
- Executable address.
- Higher-half direct map.
- Framebuffer metadata.
- RSDP metadata.

The parser copies Limine memory-map entries into a fixed early stack buffer and
then calls `BootInfo::normalize` from `crates/aesynx-boot`. The public boot
crate remains `no_std`, dependency-free, and safe Rust.

After normalization, `_start` uses the generic `aesynx-kernel::boot_summary`
API for serial output. The generic kernel therefore consumes only Aesynx
BootInfo, not Limine response structures.

Limine base revision 6 returns pointer-style handoff data through HHDM virtual
addresses. Aesynx records RSDP, HHDM, device-tree, and framebuffer addresses in
private BootInfo fields, exposes only presence checks to generic kernel code,
and uses redacted Debug implementations for handoff metadata. Memory-map region
starts remain physical addresses.

The Limine boundary rejects null and misaligned response pointers before
creating Rust references, pins the transcribed framebuffer ABI with compile-time
layout assertions, and validates kernel-image base alignment plus the x86_64
kernel-half virtual address invariant.

## Serial Contract

`cargo xtask qemu` now requires both markers:

```text
[TEST] bootinfo=ok
[TEST] boot=ok
```

Expected v0.5 serial shape:

```text
Aesynx: booting
arch=x86_64 platform=qemu
memmap regions=<n> usable=<n> usable_bytes=<n>
rsdp=present
[TEST] bootinfo=ok
[TEST] boot=ok
```

## Boundaries

This milestone proves:

- Limine request sections are retained in the kernel ELF.
- QEMU boots with `kaslr: yes`.
- Limine handoff metadata is available at `_start`.
- Memory-map and kernel-image metadata normalize into Aesynx `BootInfo`.
- Synthetic BootInfo unit tests validate memory-map summaries, KASLR-sensitive
  Debug redaction, and rejection of empty maps or invalid kernel-image bases.

This milestone does not prove:

- Ownership of bootloader memory after reclaim.
- Page-table ownership or remapping.
- Heap allocation.
- Interrupts or exceptions.
- SMP topology discovery.
- Use of framebuffer output.
