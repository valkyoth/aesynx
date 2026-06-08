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
mod port;
#[allow(unsafe_code)]
pub mod registers;

pub struct X86_64;

impl ArchCpu for X86_64 {
    fn arch_name() -> &'static str {
        "x86_64"
    }

    fn wait_for_interrupt() {
        core::hint::spin_loop();
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
