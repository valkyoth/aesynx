use core::mem::size_of;

use super::{
    DOUBLE_FAULT_VECTOR, ExceptionFrame, IDT_ENTRIES, INTERRUPT_GATE_PRESENT, IdtEntry,
    PAGE_FAULT_VECTOR, PageFaultErrorCode, RETURNING_EXCEPTION_GPR_SAVE_BYTES,
    RETURNING_EXCEPTION_GPR_SAVE_COUNT, RawExceptionFrame,
};
use crate::descriptors::{InterruptStackTableIndex, SegmentSelector};

#[test]
fn idt_entry_encodes_handler_selector_and_ist() {
    unsafe extern "C" fn handler() {}

    let entry =
        IdtEntry::interrupt_gate(handler, InterruptStackTableIndex::DOUBLE_FAULT.get() as u8);
    let address = handler as *const () as usize as u64;

    assert_eq!(entry.offset_low, address as u16);
    assert_eq!(entry.offset_mid, (address >> 16) as u16);
    assert_eq!(entry.offset_high, (address >> 32) as u32);
    assert_eq!(entry.selector, SegmentSelector::KERNEL_CODE.bits());
    assert_eq!(
        entry.options,
        INTERRUPT_GATE_PRESENT | InterruptStackTableIndex::DOUBLE_FAULT.get()
    );
    assert_eq!(entry.reserved, 0);
}

#[test]
fn idt_shape_matches_x86_64_descriptor_size() {
    assert_eq!(size_of::<IdtEntry>(), 16);
    assert_eq!(size_of::<RawExceptionFrame>(), 40);
    assert_eq!(IDT_ENTRIES, 256);
    assert_eq!(DOUBLE_FAULT_VECTOR, 8);
    assert_eq!(PAGE_FAULT_VECTOR, 14);
}

#[test]
fn returning_exception_frame_offset_tracks_saved_gprs() {
    assert_eq!(RETURNING_EXCEPTION_GPR_SAVE_COUNT, 15);
    assert_eq!(
        RETURNING_EXCEPTION_GPR_SAVE_BYTES,
        RETURNING_EXCEPTION_GPR_SAVE_COUNT * size_of::<u64>()
    );
    assert_eq!(RETURNING_EXCEPTION_GPR_SAVE_BYTES, 120);
}

#[test]
fn exception_frame_rejects_invalid_pointer() {
    assert_eq!(ExceptionFrame::from_raw(core::ptr::null()), None);
}

#[test]
fn exception_frame_copies_interrupt_frame_fields() {
    let raw = RawExceptionFrame {
        vector: 14,
        error_code: 0b101,
        instruction_pointer: 0xffff_ffff_8000_1234,
        code_segment: SegmentSelector::KERNEL_CODE.bits() as u64,
        rflags: 0xffff_ffff_0000_0ed7,
    };

    let frame = ExceptionFrame::from_raw(core::ptr::addr_of!(raw));

    assert_eq!(
        frame,
        Some(ExceptionFrame {
            vector: 14,
            error_code: 0b101,
            instruction_pointer: 0xffff_ffff_8000_1234,
            code_segment: SegmentSelector::KERNEL_CODE.bits() as u64,
            rflags: 0xffff_ffff_0000_0ed7,
        })
    );
    assert_eq!(frame.map(ExceptionFrame::public_rflags), Some(0x0cd5));
    assert_eq!(
        frame.map(ExceptionFrame::instruction_pointer_present),
        Some(true)
    );
    assert_eq!(
        frame.map(ExceptionFrame::instruction_pointer_offset),
        Some(0x234)
    );
}

#[test]
fn page_fault_error_code_decodes_architectural_bits() {
    let error = PageFaultErrorCode::new(
        (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 4) | (1 << 5) | (1 << 6) | (1 << 15),
    );

    assert!(error.present());
    assert!(error.write());
    assert!(error.user());
    assert!(error.reserved_bit());
    assert!(error.instruction_fetch());
    assert!(error.protection_key());
    assert!(error.shadow_stack());
    assert!(error.sgx());
}
