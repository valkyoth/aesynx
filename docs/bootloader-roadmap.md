# Aesynx Bootloader Roadmap

Status: future design direction

Limine is the pragmatic current boot path. It lets Aesynx build the kernel,
BootInfo normalization, memory management, diagnostics, and userspace without
first building a bootloader.

Long term, Aesynx should grow its own minimal Rust bootloader. The design goal
is not a GRUB replacement with every possible feature. It is a small security
gateway that verifies and measures an Aesynx boot capsule, then hands off as
quickly as possible.

## Philosophy

The bootloader must do less than classic bootloaders:

- UEFI-first.
- Rust `no_std`.
- No scripting language.
- No shell.
- No network stack.
- No RAID, LVM, or complex storage logic.
- No custom filesystem drivers in the normal path.
- No driver ecosystem.
- No long-lived runtime after handoff.

The bootloader should rely on firmware-provided UEFI services to read a small
set of files from the EFI System Partition, then exit boot services and hand
off to Aesynx.

## Boot Capsule

The normal input should be an Aesynx boot capsule, similar in spirit to a
Unified Kernel Image but shaped for Aesynx:

- Kernel ELF.
- Initial object bundle.
- Boot manifest.
- Optional init/runtime/services.
- Optional model/policy objects.
- Signature metadata.
- Version and rollback metadata.
- Measurement metadata.

The capsule can be one signed delivery artifact while still preserving separate
component identities inside it. It must not imply a monolithic OS binary.

## Security Requirements

The future bootloader should:

- Verify signatures before executing any kernel code.
- Measure firmware state, bootloader state, capsule metadata, and loaded
  payload hashes into TPM PCRs where hardware supports it.
- Support sealed-secret flows where disk/object-store keys are released only
  when measurements match expected values.
- Treat TPM measurement as evidence, not magic enforcement. The bootloader
  measures; key release policy decides whether a changed system is trusted.
- Reject unsigned, malformed, stale, or policy-blocked capsules.
- Avoid parsing complex untrusted formats before verification.
- Avoid loading unbounded configuration.

## Configuration

Configuration should be declarative state, not code:

- Static boot manifest.
- No loops, conditionals, variables, shell commands, or eval-like behavior.
- Optional simple key-value policy.
- Prefer scanning a fixed ESP directory for signed capsule manifests.
- Boot menu selection, if needed, should choose among already verified
  manifests and must not execute scripts.

## UI Boundary

The bootloader should not become a graphical operating environment. If Aesynx
needs a rich boot menu, recovery UI, password entry, or network-assisted
recovery, prefer a verified Aesynx recovery capsule or minimal boot environment
after the security gateway has done signature and measurement work.

## Relationship To Limine

Current releases may continue using Limine until Aesynx has enough kernel and
diagnostic maturity to justify replacing the boot path.

The future Rust bootloader should become its own milestone only after:

- BootInfo normalization is stable.
- Kernel serial/panic diagnostics are reliable.
- Object bundles exist.
- Signed manifests exist.
- QEMU boot smoke is stable.
- The team can test UEFI behavior without risking daily-driver hardware.

## Non-Goals

- GRUB compatibility.
- BIOS-first design.
- Filesystem-driver collection.
- Boot-time scripting.
- General-purpose pre-boot shell.
- Direct support for every OS on the machine.
- Reimplementing Linux initramfs complexity inside the bootloader.
