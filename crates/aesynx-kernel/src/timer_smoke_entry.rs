use aesynx_arch::ArchCpu;
use aesynx_kernel::diagnostics::{self, BootPhase};
use aesynx_log::LogLevel;

const TIMER_SMOKE_MAX_SPINS: u64 = 100_000_000;
const TIMER_SMOKE_DELAY_TICKS: u64 = 2;

pub fn run() -> ! {
    diagnostics::set_boot_phase(BootPhase::TimerSmoke);
    crate::write_diagnostic(LogLevel::Info, "timer smoke starting");
    match aesynx_arch_x86_64::timer::init_smoke_timer() {
        Ok(status) => {
            let rate = match aesynx_time::TickRate::new(status.tick_rate_hz) {
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
                aesynx_arch_x86_64::X86_64::wait_for_interrupt();
                let ticks = aesynx_arch_x86_64::timer::ticks();
                match rate.ticks_to_nanos(ticks) {
                    Ok(now) => {
                        if let Some(wake) = sleep_queue.pop_due(now) {
                            aesynx_arch_x86_64::serial_println!(
                                "timer delayed-log task={} wake_id={} at_ns={} ticks={}",
                                wake.task().get(),
                                wake.wake_id().get(),
                                now.nanos(),
                                ticks
                            );
                            aesynx_arch_x86_64::serial::write_str("[TEST] sleep=ok\n");
                        }
                    }
                    Err(error) => {
                        aesynx_arch_x86_64::serial_println!(
                            "timer monotonic=fail error={:?}",
                            error
                        );
                    }
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
