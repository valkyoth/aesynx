use crate::descriptors::SegmentSelector;

use super::INTERRUPT_GATE_PRESENT;

#[repr(C, packed)]
pub(super) struct DescriptorTablePointer {
    pub(super) limit: u16,
    pub(super) base: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct IdtEntry {
    pub(super) offset_low: u16,
    pub(super) selector: u16,
    pub(super) options: u16,
    pub(super) offset_mid: u16,
    pub(super) offset_high: u32,
    pub(super) reserved: u32,
}

impl IdtEntry {
    pub(super) const fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            options: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    pub(super) fn interrupt_gate(handler: unsafe extern "C" fn(), ist: u8) -> Self {
        let address = handler as *const () as usize as u64;
        Self::interrupt_gate_address(address, ist)
    }

    pub(super) fn interrupt_gate_address(address: u64, ist: u8) -> Self {
        let options = INTERRUPT_GATE_PRESENT | u16::from(ist & 0x07);

        Self {
            offset_low: address as u16,
            selector: SegmentSelector::KERNEL_CODE.bits(),
            options,
            offset_mid: (address >> 16) as u16,
            offset_high: (address >> 32) as u32,
            reserved: 0,
        }
    }
}
