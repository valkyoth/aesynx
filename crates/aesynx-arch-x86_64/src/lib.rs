#![no_std]
#![deny(unsafe_code)]

pub mod serial;

use aesynx_abi::CoreId;
use aesynx_arch::{ArchCpu, ArchError};

#[allow(unsafe_code)]
mod port;

pub struct X86_64;

impl ArchCpu for X86_64 {
    fn arch_name() -> &'static str {
        "x86_64"
    }

    fn wait_for_interrupt() {
        core::hint::spin_loop();
    }

    fn halt_forever() -> ! {
        loop {
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
