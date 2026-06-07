#![no_std]
#![deny(unsafe_code)]

use aesynx_abi::CoreId;
use aesynx_arch::{ArchCpu, ArchError};

pub struct Aarch64;

impl ArchCpu for Aarch64 {
    fn arch_name() -> &'static str {
        "aarch64"
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
