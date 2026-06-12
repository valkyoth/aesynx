#![no_std]
#![deny(unsafe_code)]

pub mod serial;

use aesynx_abi::CoreId;
use aesynx_arch::{ArchCpu, ArchError};

pub(crate) const RFLAGS_PUBLIC_MASK: u64 = 0x0000_0000_0000_0cd5;

#[allow(unsafe_code)]
pub mod descriptors;
#[allow(unsafe_code)]
pub mod exceptions;
#[allow(unsafe_code)]
pub mod interrupts;
#[allow(unsafe_code)]
mod port;
#[allow(unsafe_code)]
pub mod registers;
#[allow(unsafe_code)]
pub mod timer;

pub struct X86_64;

impl ArchCpu for X86_64 {
    fn arch_name() -> &'static str {
        "x86_64"
    }

    #[allow(unsafe_code)]
    fn wait_for_interrupt() {
        // SAFETY: `hlt` idles the current CPU until the next external
        // interrupt, NMI, SMI, or reset. Callers must only use this when an
        // interrupt source is expected or the CPU is intentionally idle.
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }

    #[allow(unsafe_code)]
    fn halt_forever() -> ! {
        loop {
            // SAFETY: `hlt` is the x86_64 architectural idle instruction. This
            // path is used only for terminal halt states and does not access
            // Rust memory, stack data, or I/O ports.
            unsafe {
                core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
            }
        }
    }

    #[allow(unsafe_code)]
    fn enable_interrupts() -> Result<(), ArchError> {
        // SAFETY: `sti` enables maskable interrupts for the current CPU. Aesynx
        // calls this only after installing IDT entries and configuring the
        // interrupt controller for the active smoke path.
        unsafe {
            core::arch::asm!("sti", options(nomem, nostack, preserves_flags));
        }
        Ok(())
    }

    #[allow(unsafe_code)]
    fn disable_interrupts() -> Result<(), ArchError> {
        // SAFETY: `cli` disables maskable interrupts for the current CPU and
        // does not access Rust memory.
        unsafe {
            core::arch::asm!("cli", options(nomem, nostack, preserves_flags));
        }
        Ok(())
    }

    #[allow(unsafe_code)]
    fn interrupts_enabled() -> Result<bool, ArchError> {
        let rflags: u64;
        // SAFETY: `pushfq; pop` copies RFLAGS through the current stack without
        // creating Rust references or dereferencing untrusted pointers.
        unsafe {
            core::arch::asm!(
                "pushfq",
                "pop {rflags}",
                rflags = lateout(reg) rflags,
                options(preserves_flags)
            );
        }
        Ok(rflags & (1 << 9) != 0)
    }

    fn current_core_id() -> Result<CoreId, ArchError> {
        Err(ArchError::Unsupported)
    }

    #[allow(unsafe_code)]
    fn read_timestamp() -> Result<u64, ArchError> {
        let low: u32;
        let high: u32;
        // This is intentionally the cheap telemetry timestamp path. `rdtsc` is
        // not serializing; callers that need precise security or expiry bounds
        // must use a future fenced timestamp API or issue an `lfence` before
        // reading the counter.
        // SAFETY: `rdtsc` reads the architectural timestamp counter into EDX:EAX
        // and does not touch Rust memory.
        unsafe {
            core::arch::asm!(
                "rdtsc",
                out("eax") low,
                out("edx") high,
                options(nomem, nostack, preserves_flags)
            );
        }
        Ok((u64::from(high) << 32) | u64::from(low))
    }
}
