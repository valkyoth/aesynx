#![no_std]
#![deny(unsafe_code)]

use aesynx_abi::{CoreId, ROOT_CORE};
use aesynx_arch::ArchCpu;

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

    fn enable_interrupts() {}

    fn disable_interrupts() {}

    fn interrupts_enabled() -> bool {
        false
    }

    fn current_core_id() -> CoreId {
        ROOT_CORE
    }

    fn read_timestamp() -> u64 {
        0
    }
}
