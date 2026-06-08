#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]
#![cfg_attr(target_os = "none", allow(unsafe_code))]

#[cfg(target_os = "none")]
use core::panic::PanicInfo;

#[cfg(target_os = "none")]
use aesynx_arch::ArchCpu;

#[cfg(target_os = "none")]
use aesynx_kernel::diagnostics::{self, BootPhase, DiagnosticComponent};

#[cfg(target_os = "none")]
use aesynx_log::{LogLevel, LogMessage};

#[cfg(all(target_os = "none", not(feature = "panic-smoke")))]
mod limine;

#[cfg(target_os = "none")]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    aesynx_arch_x86_64::serial::init();
    diagnostics::set_boot_phase(BootPhase::Entry);
    write_diagnostic(LogLevel::Info, "serial initialized");
    let descriptor_status = aesynx_arch_x86_64::descriptors::init();
    diagnostics::set_boot_phase(BootPhase::CpuSetup);
    write_diagnostic(LogLevel::Info, "gdt and tss initialized");
    aesynx_arch_x86_64::serial_println!(
        "cpu setup=gdt_tss entries={} tss=0x{:x} df_ist={} df_stack_bytes={}",
        descriptor_status.gdt_entries,
        descriptor_status.tss_selector.bits(),
        descriptor_status.double_fault_ist.get(),
        descriptor_status.double_fault_stack_bytes
    );
    aesynx_arch_x86_64::serial::write_str("[TEST] gdt=ok\n");
    kernel_entry()
}

#[cfg(all(target_os = "none", feature = "panic-smoke"))]
fn kernel_entry() -> ! {
    panic_smoke_entry()
}

#[cfg(all(target_os = "none", not(feature = "panic-smoke")))]
fn kernel_entry() -> ! {
    boot_entry()
}

#[cfg(all(target_os = "none", feature = "panic-smoke"))]
fn panic_smoke_entry() -> ! {
    diagnostics::set_boot_phase(BootPhase::PanicSmoke);
    trigger_panic_smoke();
}

#[cfg(all(target_os = "none", not(feature = "panic-smoke")))]
fn boot_entry() -> ! {
    let mut scratch = limine::EarlyBootScratch::new();
    diagnostics::set_boot_phase(BootPhase::BootloaderHandoff);
    match limine::normalize(&mut scratch) {
        Ok(info) => {
            diagnostics::set_boot_phase(BootPhase::BootInfoNormalized);
            write_diagnostic(LogLevel::Info, "bootinfo normalized");
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
            diagnostics::set_boot_phase(BootPhase::Running);
        }
        Err(_error) => {
            aesynx_arch_x86_64::serial::write_str("Aesynx: booting\n");
            write_diagnostic(LogLevel::Error, "bootinfo normalization failed");
            aesynx_arch_x86_64::serial::write_str("[TEST] bootinfo=fail\n");
        }
    }

    aesynx_arch_x86_64::X86_64::halt_forever()
}

#[cfg(all(target_os = "none", feature = "panic-smoke"))]
#[allow(clippy::panic)]
fn trigger_panic_smoke() -> ! {
    panic!("intentional v0.7.0 panic smoke");
}

#[cfg(target_os = "none")]
#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    aesynx_arch_x86_64::serial::init();
    let snapshot = diagnostics::panic_snapshot();
    let registers = aesynx_arch_x86_64::registers::EarlyRegisterSnapshot::capture();
    aesynx_arch_x86_64::serial::write_str("Aesynx: panic during early boot\n");
    write_diagnostic(LogLevel::Fatal, "panic handler entered");
    aesynx_arch_x86_64::serial_println!(
        "panic core={} phase={}",
        snapshot.core.get(),
        snapshot.phase.label()
    );
    let mut serial = aesynx_arch_x86_64::serial::Com1::new();
    if let Some(location) = info.location() {
        let _ = diagnostics::write_panic_location(
            &mut serial,
            location.file(),
            location.line(),
            location.column(),
        );
    } else {
        aesynx_arch_x86_64::serial::write_str("panic location=unknown\n");
    }
    let _ = diagnostics::write_panic_message(&mut serial, format_args!("{}", info.message()));
    aesynx_arch_x86_64::serial_println!(
        "panic registers=rsp_present={} rbp_present={} rsp_align={} rbp_align={} rflags=0x{:x} cr3_offset=0x{:x}",
        registers.stack_pointer_present(),
        registers.frame_pointer_present(),
        registers.stack_pointer_alignment(),
        registers.frame_pointer_alignment(),
        registers.public_rflags(),
        registers.cr3_page_offset()
    );
    diagnostics::set_boot_phase(BootPhase::Panic);
    #[cfg(feature = "panic-smoke")]
    aesynx_arch_x86_64::serial::write_str("[TEST] panic=ok\n");
    aesynx_arch_x86_64::X86_64::halt_forever()
}

#[cfg(target_os = "none")]
fn write_diagnostic(level: LogLevel, message: &'static str) {
    let message = LogMessage::new(message).unwrap_or(LogMessage::REJECTED);
    let record =
        diagnostics::DiagnosticRecord::current(level, DiagnosticComponent::KERNEL, message);
    let mut serial = aesynx_arch_x86_64::serial::Com1::new();
    let _ = record.write_to(&mut serial);
}

#[cfg(not(target_os = "none"))]
fn main() {}
