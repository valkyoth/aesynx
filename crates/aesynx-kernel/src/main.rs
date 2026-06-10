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

#[cfg(all(
    target_os = "none",
    feature = "timer-smoke",
    not(feature = "panic-smoke"),
    not(feature = "exception-smoke")
))]
const TIMER_SMOKE_MAX_SPINS: u64 = 100_000_000;
#[cfg(all(
    target_os = "none",
    feature = "timer-smoke",
    not(feature = "panic-smoke"),
    not(feature = "exception-smoke")
))]
const TIMER_SMOKE_HZ: u64 = 100;
#[cfg(all(
    target_os = "none",
    feature = "timer-smoke",
    not(feature = "panic-smoke"),
    not(feature = "exception-smoke")
))]
const TIMER_SMOKE_DELAY_TICKS: u64 = 2;

#[cfg(all(
    target_os = "none",
    not(feature = "panic-smoke"),
    not(feature = "exception-smoke"),
    not(feature = "timer-smoke")
))]
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
    let exception_status = aesynx_arch_x86_64::exceptions::init(descriptor_status.double_fault_ist);
    diagnostics::set_boot_phase(BootPhase::ExceptionSetup);
    write_diagnostic(LogLevel::Info, "idt initialized");
    aesynx_arch_x86_64::serial_println!(
        "exception setup=idt entries={} breakpoint={} page_fault={} double_fault={} df_ist={}",
        exception_status.idt_entries,
        exception_status.breakpoint_vector,
        exception_status.page_fault_vector,
        exception_status.double_fault_vector,
        exception_status.double_fault_ist.get()
    );
    aesynx_arch_x86_64::serial::write_str("[TEST] idt=ok\n");
    let interrupt_status = aesynx_arch_x86_64::interrupts::init();
    diagnostics::set_boot_phase(BootPhase::InterruptSetup);
    write_diagnostic(LogLevel::Info, "interrupt controller baseline initialized");
    aesynx_arch_x86_64::serial_println!(
        "interrupt setup=baseline legacy_pic_masked={} local_apic_present={} local_apic_mode={:?} irq_vector_base=0x{:x} irq_vector_count={}",
        interrupt_status.legacy_pic_masked,
        interrupt_status.local_apic_present,
        interrupt_status.local_apic_mode,
        interrupt_status.irq_vector_base,
        interrupt_status.irq_vector_count
    );
    aesynx_arch_x86_64::serial::write_str("[TEST] irq=ok\n");
    aesynx_arch_x86_64::exceptions::trigger_breakpoint_smoke();
    kernel_entry()
}

#[cfg(all(target_os = "none", feature = "panic-smoke"))]
fn kernel_entry() -> ! {
    panic_smoke_entry()
}

#[cfg(all(
    target_os = "none",
    feature = "exception-smoke",
    not(feature = "panic-smoke")
))]
fn kernel_entry() -> ! {
    exception_smoke_entry()
}

#[cfg(all(
    target_os = "none",
    not(feature = "panic-smoke"),
    not(feature = "exception-smoke"),
    not(feature = "timer-smoke")
))]
fn kernel_entry() -> ! {
    boot_entry()
}

#[cfg(all(
    target_os = "none",
    feature = "timer-smoke",
    not(feature = "panic-smoke"),
    not(feature = "exception-smoke")
))]
fn kernel_entry() -> ! {
    timer_smoke_entry()
}

#[cfg(all(target_os = "none", feature = "panic-smoke"))]
fn panic_smoke_entry() -> ! {
    diagnostics::set_boot_phase(BootPhase::PanicSmoke);
    trigger_panic_smoke();
}

#[cfg(all(
    target_os = "none",
    feature = "exception-smoke",
    not(feature = "panic-smoke")
))]
fn exception_smoke_entry() -> ! {
    diagnostics::set_boot_phase(BootPhase::ExceptionSmoke);
    write_diagnostic(LogLevel::Info, "exception smoke starting");
    aesynx_arch_x86_64::exceptions::trigger_page_fault_smoke()
}

#[cfg(all(
    target_os = "none",
    feature = "timer-smoke",
    not(feature = "panic-smoke"),
    not(feature = "exception-smoke")
))]
fn timer_smoke_entry() -> ! {
    diagnostics::set_boot_phase(BootPhase::TimerSmoke);
    write_diagnostic(LogLevel::Info, "timer smoke starting");
    match aesynx_arch_x86_64::timer::init_smoke_timer() {
        Ok(status) => {
            let rate = match aesynx_time::TickRate::new(TIMER_SMOKE_HZ) {
                Ok(rate) => rate,
                Err(error) => {
                    aesynx_arch_x86_64::serial_println!("timer setup=fail error={:?}", error);
                    aesynx_arch_x86_64::X86_64::halt_forever();
                }
            };
            let mut sleep_queue = aesynx_time::SleepQueue::<1>::new();
            let deadline = match rate.ticks_to_nanos(TIMER_SMOKE_DELAY_TICKS) {
                Ok(deadline) => deadline,
                Err(error) => {
                    aesynx_arch_x86_64::serial_println!("timer setup=fail error={:?}", error);
                    aesynx_arch_x86_64::X86_64::halt_forever();
                }
            };
            let sleep = aesynx_time::SleepRequest::new(
                aesynx_abi::TaskId::new(0),
                deadline,
                aesynx_time::WakeId::new(1),
            );
            if let Err(error) = sleep_queue.schedule(sleep) {
                aesynx_arch_x86_64::serial_println!("timer setup=fail error={:?}", error);
                aesynx_arch_x86_64::X86_64::halt_forever();
            }
            aesynx_arch_x86_64::serial_println!(
                "timer setup=pit irq={} vector=0x{:x} target_ticks={} hz={}",
                status.irq.get(),
                status.vector,
                status.target_ticks,
                rate.hz()
            );
            let _ = aesynx_arch_x86_64::X86_64::enable_interrupts();
            let mut spins = 0u64;
            while aesynx_arch_x86_64::timer::ticks() < aesynx_arch_x86_64::timer::target_ticks() {
                core::hint::spin_loop();
                let ticks = aesynx_arch_x86_64::timer::ticks();
                if let Ok(now) = rate.ticks_to_nanos(ticks)
                    && let Some(wake) = sleep_queue.pop_due(now)
                {
                    aesynx_arch_x86_64::serial_println!(
                        "timer delayed-log task={} wake_id={} at_ns={} ticks={}",
                        wake.task().get(),
                        wake.wake_id().get(),
                        now.nanos(),
                        ticks
                    );
                    aesynx_arch_x86_64::serial::write_str("[TEST] sleep=ok\n");
                }
                spins = spins.saturating_add(1);
                if spins >= TIMER_SMOKE_MAX_SPINS {
                    aesynx_arch_x86_64::serial_println!(
                        "timer timeout ticks={} target_ticks={} spins={}",
                        aesynx_arch_x86_64::timer::ticks(),
                        aesynx_arch_x86_64::timer::target_ticks(),
                        spins
                    );
                    break;
                }
            }
            let _ = aesynx_arch_x86_64::X86_64::disable_interrupts();
        }
        Err(error) => {
            aesynx_arch_x86_64::serial_println!("timer setup=fail error={:?}", error);
        }
    }
    aesynx_arch_x86_64::X86_64::halt_forever()
}

#[cfg(all(
    target_os = "none",
    not(feature = "panic-smoke"),
    not(feature = "exception-smoke"),
    not(feature = "timer-smoke")
))]
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
        Err(error) => {
            aesynx_arch_x86_64::serial::write_str("Aesynx: booting\n");
            write_diagnostic(LogLevel::Error, "bootinfo normalization failed");
            aesynx_arch_x86_64::serial_println!("bootinfo error={:?}", error);
            aesynx_arch_x86_64::serial::write_str("[TEST] bootinfo=fail\n");
        }
    }

    aesynx_arch_x86_64::X86_64::halt_forever()
}

#[cfg(all(target_os = "none", feature = "panic-smoke"))]
#[allow(clippy::panic)]
fn trigger_panic_smoke() -> ! {
    panic!("intentional v0.12.0 panic smoke");
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
