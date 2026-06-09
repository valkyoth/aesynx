use core::sync::atomic::{AtomicBool, Ordering};

use aesynx_abi::{CoreId, IrqLine};
use aesynx_arch::{InterruptController, InterruptError, IpiVector};

use crate::port::{AdmittedPort, Port};

const LEGACY_PIC_IRQS: u32 = 16;
const IRQ_VECTOR_BASE: u8 = 0x20;
const IRQ_VECTOR_COUNT: u8 = LEGACY_PIC_IRQS as u8;
const PIC_MASTER_VECTOR_BASE: u8 = IRQ_VECTOR_BASE;
const PIC_SLAVE_VECTOR_BASE: u8 = IRQ_VECTOR_BASE + 8;
const PIC_MASK_ALL: u8 = 0xff;
const PIC_EOI: u8 = 0x20;
const PIC_ICW1_INIT_WITH_ICW4: u8 = 0x11;
const PIC_ICW3_MASTER_HAS_SLAVE_ON_IRQ2: u8 = 0x04;
const PIC_ICW3_SLAVE_ID_2: u8 = 0x02;
const PIC_ICW4_8086_MODE: u8 = 0x01;
const PIC_OCW3_READ_ISR: u8 = 0x0b;
const PIC_SPURIOUS_MASTER_IRQ: u32 = 7;
const PIC_SPURIOUS_SLAVE_IRQ: u32 = 15;
const CPUID_FEATURE_EDX_APIC: u32 = 1 << 9;

static INITIALIZED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InterruptControllerStatus {
    pub legacy_pic_masked: bool,
    pub local_apic_present: bool,
    pub local_apic_mode: LocalApicMode,
    pub legacy_pic_remapped: bool,
    pub irq_vector_base: u8,
    pub irq_vector_count: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalApicMode {
    Unavailable,
    DeferredUntilMmioMapping,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IrqVector(u8);

impl IrqVector {
    pub const fn from_irq(irq: IrqLine) -> Result<Self, InterruptError> {
        let line = irq.get();
        if line >= LEGACY_PIC_IRQS {
            return Err(InterruptError::InvalidIrq);
        }

        Ok(Self(IRQ_VECTOR_BASE + line as u8))
    }

    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

pub struct X86_64InterruptController;

impl InterruptController for X86_64InterruptController {
    fn init() -> Result<(), InterruptError> {
        let _ = init();
        Ok(())
    }

    fn enable_irq(irq: IrqLine) -> Result<(), InterruptError> {
        let _ = IrqVector::from_irq(irq)?;
        Err(InterruptError::ControllerUnavailable)
    }

    fn disable_irq(irq: IrqLine) -> Result<(), InterruptError> {
        mask_legacy_pic_irq(irq)
    }

    fn acknowledge(irq: IrqLine) -> Result<(), InterruptError> {
        acknowledge_legacy_pic_irq(irq)
    }

    fn send_ipi(_target: CoreId, _vector: IpiVector) -> Result<(), InterruptError> {
        Err(InterruptError::ControllerUnavailable)
    }
}

#[must_use]
pub fn init() -> InterruptControllerStatus {
    if !INITIALIZED.load(Ordering::Acquire) {
        reprogram_legacy_pic();
        mask_all_legacy_pic_irqs();
        INITIALIZED.store(true, Ordering::Release);
    }

    let local_apic_present = local_apic_present();
    InterruptControllerStatus {
        legacy_pic_masked: true,
        local_apic_present,
        legacy_pic_remapped: true,
        local_apic_mode: if local_apic_present {
            LocalApicMode::DeferredUntilMmioMapping
        } else {
            LocalApicMode::Unavailable
        },
        irq_vector_base: IRQ_VECTOR_BASE,
        irq_vector_count: IRQ_VECTOR_COUNT,
    }
}

pub fn irq_vector(irq: IrqLine) -> Result<IrqVector, InterruptError> {
    IrqVector::from_irq(irq)
}

fn reprogram_legacy_pic() {
    Port::new(AdmittedPort::PicMasterCommand).write_u8(PIC_ICW1_INIT_WITH_ICW4);
    Port::new(AdmittedPort::PicSlaveCommand).write_u8(PIC_ICW1_INIT_WITH_ICW4);
    Port::new(AdmittedPort::PicMasterData).write_u8(PIC_MASTER_VECTOR_BASE);
    Port::new(AdmittedPort::PicSlaveData).write_u8(PIC_SLAVE_VECTOR_BASE);
    Port::new(AdmittedPort::PicMasterData).write_u8(PIC_ICW3_MASTER_HAS_SLAVE_ON_IRQ2);
    Port::new(AdmittedPort::PicSlaveData).write_u8(PIC_ICW3_SLAVE_ID_2);
    Port::new(AdmittedPort::PicMasterData).write_u8(PIC_ICW4_8086_MODE);
    Port::new(AdmittedPort::PicSlaveData).write_u8(PIC_ICW4_8086_MODE);
}

fn mask_all_legacy_pic_irqs() {
    Port::new(AdmittedPort::PicMasterData).write_u8(PIC_MASK_ALL);
    Port::new(AdmittedPort::PicSlaveData).write_u8(PIC_MASK_ALL);
}

fn mask_legacy_pic_irq(irq: IrqLine) -> Result<(), InterruptError> {
    let line = irq.get();
    if line >= LEGACY_PIC_IRQS {
        return Err(InterruptError::InvalidIrq);
    }

    let port = if line < 8 {
        Port::new(AdmittedPort::PicMasterData)
    } else {
        Port::new(AdmittedPort::PicSlaveData)
    };
    let bit = 1u8 << (line % 8);
    let mask = port.read_u8() | bit;
    port.write_u8(mask);
    Ok(())
}

fn acknowledge_legacy_pic_irq(irq: IrqLine) -> Result<(), InterruptError> {
    let line = irq.get();
    if line >= LEGACY_PIC_IRQS {
        return Err(InterruptError::InvalidIrq);
    }

    if line == PIC_SPURIOUS_MASTER_IRQ && !legacy_pic_irq_in_service(false, PIC_SPURIOUS_MASTER_IRQ)
    {
        return Ok(());
    }

    if line == PIC_SPURIOUS_SLAVE_IRQ && !legacy_pic_irq_in_service(true, 7) {
        Port::new(AdmittedPort::PicMasterCommand).write_u8(PIC_EOI);
        return Ok(());
    }

    if line >= 8 {
        Port::new(AdmittedPort::PicSlaveCommand).write_u8(PIC_EOI);
    }
    Port::new(AdmittedPort::PicMasterCommand).write_u8(PIC_EOI);
    Ok(())
}

fn legacy_pic_irq_in_service(slave: bool, irq_bit: u32) -> bool {
    let command = if slave {
        Port::new(AdmittedPort::PicSlaveCommand)
    } else {
        Port::new(AdmittedPort::PicMasterCommand)
    };
    command.write_u8(PIC_OCW3_READ_ISR);
    command.read_u8() & (1u8 << irq_bit) != 0
}

fn local_apic_present() -> bool {
    cpuid_leaf_1_edx() & CPUID_FEATURE_EDX_APIC != 0
}

#[cfg(target_arch = "x86_64")]
fn cpuid_leaf_1_edx() -> u32 {
    core::arch::x86_64::__cpuid(1).edx
}

#[cfg(not(target_arch = "x86_64"))]
const fn cpuid_leaf_1_edx() -> u32 {
    0
}

#[cfg(test)]
mod tests {
    use aesynx_abi::IrqLine;
    use aesynx_arch::{InterruptController, InterruptError};

    use super::{
        CPUID_FEATURE_EDX_APIC, IRQ_VECTOR_BASE, IrqVector, LEGACY_PIC_IRQS, LocalApicMode,
        PIC_ICW1_INIT_WITH_ICW4, PIC_ICW3_MASTER_HAS_SLAVE_ON_IRQ2, PIC_ICW3_SLAVE_ID_2,
        PIC_ICW4_8086_MODE, PIC_MASTER_VECTOR_BASE, PIC_OCW3_READ_ISR, PIC_SLAVE_VECTOR_BASE,
        PIC_SPURIOUS_MASTER_IRQ, PIC_SPURIOUS_SLAVE_IRQ,
    };

    #[test]
    fn irq_vector_allocator_rejects_out_of_range_lines() {
        assert_eq!(
            IrqVector::from_irq(IrqLine::new(0)).map(IrqVector::get),
            Ok(IRQ_VECTOR_BASE)
        );
        assert_eq!(
            IrqVector::from_irq(IrqLine::new(15)).map(IrqVector::get),
            Ok(IRQ_VECTOR_BASE + 15)
        );
        assert_eq!(
            IrqVector::from_irq(IrqLine::new(LEGACY_PIC_IRQS)),
            Err(InterruptError::InvalidIrq)
        );
    }

    #[test]
    fn local_apic_modes_are_explicit() {
        assert_ne!(
            LocalApicMode::Unavailable,
            LocalApicMode::DeferredUntilMmioMapping
        );
        assert_eq!(CPUID_FEATURE_EDX_APIC, 1 << 9);
    }

    #[test]
    fn legacy_pic_is_remapped_out_of_exception_vector_range() {
        assert_eq!(PIC_ICW1_INIT_WITH_ICW4, 0x11);
        assert_eq!(PIC_MASTER_VECTOR_BASE, 0x20);
        assert_eq!(PIC_SLAVE_VECTOR_BASE, 0x28);
        assert_eq!(PIC_ICW3_MASTER_HAS_SLAVE_ON_IRQ2, 0x04);
        assert_eq!(PIC_ICW3_SLAVE_ID_2, 0x02);
        assert_eq!(PIC_ICW4_8086_MODE, 0x01);
        assert_eq!(PIC_OCW3_READ_ISR, 0x0b);
        assert_eq!(PIC_SPURIOUS_MASTER_IRQ, 7);
        assert_eq!(PIC_SPURIOUS_SLAVE_IRQ, 15);
    }

    #[test]
    fn deferred_enable_still_validates_irq_line() {
        assert_eq!(
            super::X86_64InterruptController::enable_irq(IrqLine::new(LEGACY_PIC_IRQS)),
            Err(InterruptError::InvalidIrq)
        );
        assert_eq!(
            super::X86_64InterruptController::enable_irq(IrqLine::new(0)),
            Err(InterruptError::ControllerUnavailable)
        );
    }
}
