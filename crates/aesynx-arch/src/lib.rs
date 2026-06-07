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
    fn acknowledge(irq: IrqLine) -> Result<(), InterruptError>;
    fn send_ipi(target: CoreId, vector: IpiVector) -> Result<(), InterruptError>;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IpiVector(u8);

impl IpiVector {
    pub const fn new(value: u8) -> Result<Self, ArchError> {
        if value < 0x20 {
            return Err(ArchError::ReservedVector);
        }

        Ok(Self(value))
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
    ReservedVector,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterruptError {
    Unsupported,
    InvalidIrq,
    ControllerUnavailable,
}

#[cfg(test)]
mod tests {
    use super::{ArchError, IpiVector};

    #[test]
    fn ipi_vector_rejects_reserved_exception_vectors() {
        assert_eq!(IpiVector::new(0x1f), Err(ArchError::ReservedVector));
        assert_eq!(IpiVector::new(0x20).map(IpiVector::get), Ok(0x20));
    }
}
