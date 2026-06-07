#![no_std]
#![deny(unsafe_code)]

use aesynx_abi::{CoreId, IrqLine, PhysAddr, VirtAddr};
use aesynx_mm::{AddressSpace, GenericPageFlags};

pub trait ArchCpu {
    fn arch_name() -> &'static str;
    fn wait_for_interrupt();
    fn halt_forever() -> !;
    fn enable_interrupts() -> Result<(), ArchError>;
    fn disable_interrupts() -> Result<(), ArchError>;
    fn interrupts_enabled() -> Result<bool, ArchError>;
    fn current_core_id() -> Result<CoreId, ArchError>;
    fn read_timestamp() -> Result<u64, ArchError>;
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
    fn activate_address_space(space: &AddressSpace) -> Result<(), MemoryError>;
    fn flush_tlb(addr: Option<VirtAddr>) -> Result<(), MemoryError>;
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
pub enum ArchError {
    Unsupported,
    NotInitialized,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterruptError {
    Unsupported,
    InvalidIrq,
    ControllerUnavailable,
}
