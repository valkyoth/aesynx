#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]
#![cfg_attr(target_os = "none", allow(unsafe_code))]

#[cfg(target_os = "none")]
use core::panic::PanicInfo;

#[cfg(target_os = "none")]
use aesynx_arch::ArchCpu;

#[cfg(target_os = "none")]
mod limine;

#[cfg(target_os = "none")]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    aesynx_arch_x86_64::serial::init();

    let mut scratch = limine::EarlyBootScratch::new();
    match limine::normalize(&mut scratch) {
        Ok(info) => {
            let summary = aesynx_kernel::boot_summary(&info);
            aesynx_arch_x86_64::serial::write_str("Aesynx: booting\n");
            aesynx_arch_x86_64::serial::write_str(summary.arch_label);
            aesynx_arch_x86_64::serial::write_str(" ");
            aesynx_arch_x86_64::serial::write_str(summary.platform_label);
            aesynx_arch_x86_64::serial::write_str("\n");
            aesynx_arch_x86_64::serial_println!(
                "memmap regions={} usable={} usable_bytes={}",
                summary.memory_regions,
                summary.usable_regions,
                summary.usable_bytes
            );
            if summary.rsdp_present {
                aesynx_arch_x86_64::serial::write_str("rsdp=present\n");
            } else {
                aesynx_arch_x86_64::serial::write_str("rsdp=absent\n");
            }
            aesynx_arch_x86_64::serial::write_str("[TEST] bootinfo=ok\n");
            aesynx_arch_x86_64::serial::write_str("[TEST] boot=ok\n");
        }
        Err(_error) => {
            aesynx_arch_x86_64::serial::write_str("Aesynx: booting\n");
            aesynx_arch_x86_64::serial::write_str("[TEST] bootinfo=fail\n");
        }
    }

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
