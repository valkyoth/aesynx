#![no_std]
#![deny(unsafe_code)]

use aesynx_abi::{CoreId, IrqLine, PhysAddr, VirtAddr};
use aesynx_mm::{AddressSpace, GenericPageFlags};

pub trait ArchCpu {
    fn arch_name() -> &'static str;
    fn wait_for_interrupt();
    fn halt_forever() -> !;
    fn enable_interrupts();
    fn disable_interrupts();
    fn interrupts_enabled() -> bool;
    fn current_core_id() -> CoreId;
    fn read_timestamp() -> u64;
}

pub trait ArchMemory {
    fn create_address_space() -> Result<AddressSpace, MemoryError>;
    fn map_page(
        space: &mut AddressSpace,
        virt: VirtAddr,
        phys: PhysAddr,
        flags: GenericPageFlags,
    ) -> Result<(), MemoryError>;
    fn unmap_page(space: &mut AddressSpace, virt: VirtAddr) -> Result<PhysAddr, MemoryError>;
    fn translate(space: &AddressSpace, virt: VirtAddr) -> Option<PhysAddr>;
    fn activate_address_space(space: &AddressSpace);
    fn flush_tlb(addr: Option<VirtAddr>);
}

pub trait InterruptController {
    fn init() -> Result<(), InterruptError>;
    fn enable_irq(irq: IrqLine) -> Result<(), InterruptError>;
    fn disable_irq(irq: IrqLine) -> Result<(), InterruptError>;
    fn acknowledge(irq: IrqLine);
    fn send_ipi(target: CoreId, vector: IpiVector) -> Result<(), InterruptError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IpiVector(u8);

impl IpiVector {
    #[must_use]
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryError {
    Unsupported,
    InvalidAddress,
    AlreadyMapped,
    NotMapped,
    OutOfMemory,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterruptError {
    Unsupported,
    InvalidIrq,
    ControllerUnavailable,
}
