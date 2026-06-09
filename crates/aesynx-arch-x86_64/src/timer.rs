use core::arch::global_asm;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use aesynx_abi::IrqLine;
use aesynx_arch::{InterruptController, InterruptError};

use crate::interrupts::X86_64InterruptController;
use crate::port::{AdmittedPort, Port};

const TIMER_IRQ: u32 = 0;
pub const TIMER_VECTOR: u8 = 0x20;
const PIT_COMMAND_CHANNEL0_LO_HI_MODE2: u8 = 0x34;
const PIT_DIVISOR_100HZ: u16 = 11_932;
const TIMER_SMOKE_TARGET_TICKS: u64 = 3;

static INITIALIZED: AtomicBool = AtomicBool::new(false);
static TICKS: AtomicU64 = AtomicU64::new(0);

global_asm!(
    r#"
    .global aesynx_timer_irq0_stub
    .type aesynx_timer_irq0_stub, @function
aesynx_timer_irq0_stub:
    push rax
    push rbx
    push rcx
    push rdx
    push rbp
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15
    mov rbp, rsp
    and rsp, -16
    call aesynx_x86_64_timer_dispatch
    mov rsp, rbp
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rbp
    pop rdx
    pop rcx
    pop rbx
    pop rax
    iretq
    "#
);

unsafe extern "C" {
    fn aesynx_timer_irq0_stub();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimerStatus {
    pub vector: u8,
    pub irq: IrqLine,
    pub target_ticks: u64,
    pub ticks: u64,
}

pub fn init_smoke_timer() -> Result<TimerStatus, TimerError> {
    if !INITIALIZED.swap(true, Ordering::AcqRel) {
        install_and_start_timer().inspect_err(|_error| {
            INITIALIZED.store(false, Ordering::Release);
        })?;
    }

    Ok(status())
}

#[must_use]
pub fn status() -> TimerStatus {
    TimerStatus {
        vector: TIMER_VECTOR,
        irq: IrqLine::new(TIMER_IRQ),
        target_ticks: TIMER_SMOKE_TARGET_TICKS,
        ticks: ticks(),
    }
}

#[must_use]
pub fn ticks() -> u64 {
    TICKS.load(Ordering::Acquire)
}

#[must_use]
pub const fn target_ticks() -> u64 {
    TIMER_SMOKE_TARGET_TICKS
}

fn configure_pit_channel0() {
    Port::new(AdmittedPort::PitCommand).write_u8(PIT_COMMAND_CHANNEL0_LO_HI_MODE2);
    Port::new(AdmittedPort::PitChannel0).write_u8(PIT_DIVISOR_100HZ as u8);
    Port::new(AdmittedPort::PitChannel0).write_u8((PIT_DIVISOR_100HZ >> 8) as u8);
}

fn install_and_start_timer() -> Result<(), TimerError> {
    crate::exceptions::install_interrupt_gate(TIMER_VECTOR, aesynx_timer_irq0_stub)
        .map_err(|_| TimerError::IdtInstallFailed)?;
    configure_pit_channel0();
    X86_64InterruptController::enable_irq(IrqLine::new(TIMER_IRQ))
        .map_err(TimerError::InterruptController)?;
    Ok(())
}

#[unsafe(no_mangle)]
extern "C" fn aesynx_x86_64_timer_dispatch() {
    let tick = TICKS.fetch_add(1, Ordering::AcqRel) + 1;
    if tick <= TIMER_SMOKE_TARGET_TICKS {
        crate::serial_println!("timer tick {}", tick);
    }
    if tick == TIMER_SMOKE_TARGET_TICKS {
        let _ = X86_64InterruptController::disable_irq(IrqLine::new(TIMER_IRQ));
        crate::serial::write_str("[TEST] timer=ok\n");
    }
    let _ = X86_64InterruptController::acknowledge(IrqLine::new(TIMER_IRQ));
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimerError {
    IdtInstallFailed,
    InterruptController(InterruptError),
}

#[cfg(test)]
mod tests {
    use super::{
        PIT_COMMAND_CHANNEL0_LO_HI_MODE2, PIT_DIVISOR_100HZ, TIMER_IRQ, TIMER_SMOKE_TARGET_TICKS,
        TIMER_VECTOR, status, target_ticks,
    };

    #[test]
    fn timer_smoke_uses_remapped_irq0_vector() {
        assert_eq!(TIMER_IRQ, 0);
        assert_eq!(TIMER_VECTOR, 0x20);
        assert_eq!(target_ticks(), 3);
        assert_eq!(TIMER_SMOKE_TARGET_TICKS, 3);
    }

    #[test]
    fn pit_configuration_uses_channel0_rate_generator() {
        assert_eq!(PIT_COMMAND_CHANNEL0_LO_HI_MODE2, 0x34);
        assert_eq!(PIT_DIVISOR_100HZ, 11_932);
    }

    #[test]
    fn status_reports_current_counter_without_mutation() {
        let status = status();
        assert_eq!(status.vector, TIMER_VECTOR);
        assert_eq!(status.irq.get(), TIMER_IRQ);
        assert_eq!(status.target_ticks, TIMER_SMOKE_TARGET_TICKS);
    }
}
