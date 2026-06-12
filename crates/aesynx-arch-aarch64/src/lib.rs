#![no_std]
#![deny(unsafe_code)]

use aesynx_abi::CoreId;
use aesynx_arch::{ArchCpu, ArchError};

pub struct Aarch64;

impl ArchCpu for Aarch64 {
    fn arch_name() -> &'static str {
        "aarch64"
    }

    #[allow(unsafe_code)]
    fn wait_for_interrupt() {
        #[cfg(target_arch = "aarch64")]
        {
            // SAFETY: `wfi` is the AArch64 architectural idle instruction.
            // This does not access Rust memory, stack data, or device
            // registers, and returns when the CPU observes an interrupt/event.
            unsafe {
                core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
            }
        }

        #[cfg(not(target_arch = "aarch64"))]
        core::hint::spin_loop();
    }

    #[allow(unsafe_code)]
    fn halt_forever() -> ! {
        loop {
            #[cfg(target_arch = "aarch64")]
            {
                // SAFETY: `wfi` is the AArch64 architectural idle instruction.
                // This terminal halt path does not touch Rust memory, stack
                // data, or device registers.
                unsafe {
                    core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
                }
            }

            #[cfg(not(target_arch = "aarch64"))]
            core::hint::spin_loop();
        }
    }

    fn enable_interrupts() -> Result<(), ArchError> {
        Err(ArchError::Unsupported)
    }

    fn disable_interrupts() -> Result<(), ArchError> {
        Err(ArchError::Unsupported)
    }

    fn interrupts_enabled() -> Result<bool, ArchError> {
        Err(ArchError::Unsupported)
    }

    fn current_core_id() -> Result<CoreId, ArchError> {
        Err(ArchError::Unsupported)
    }

    fn read_timestamp() -> Result<u64, ArchError> {
        Err(ArchError::Unsupported)
    }
}
