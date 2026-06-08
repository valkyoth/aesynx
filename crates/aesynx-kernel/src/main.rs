#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]
#![cfg_attr(target_os = "none", allow(unsafe_code))]

#[cfg(target_os = "none")]
use core::panic::PanicInfo;

#[cfg(target_os = "none")]
use aesynx_arch::ArchCpu;

#[cfg(target_os = "none")]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    aesynx_arch_x86_64::serial::init();
    aesynx_arch_x86_64::serial::write_str("Aesynx: booting\n");
    aesynx_arch_x86_64::serial::write_str("arch=x86_64 platform=qemu\n");
    aesynx_arch_x86_64::serial::write_str("[TEST] boot=ok\n");
    aesynx_arch_x86_64::X86_64::halt_forever()
}

#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    aesynx_arch_x86_64::serial::init();
    aesynx_arch_x86_64::serial::write_str("Aesynx: panic during early boot\n");
    aesynx_arch_x86_64::X86_64::halt_forever()
}

#[cfg(not(target_os = "none"))]
fn main() {}
