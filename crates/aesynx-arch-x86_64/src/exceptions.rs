use core::arch::global_asm;
use core::mem::size_of;
use core::sync::atomic::{AtomicBool, Ordering};

use crate::descriptors::{InterruptStackTableIndex, SegmentSelector};

const IDT_ENTRIES: usize = 256;
const BREAKPOINT_VECTOR: usize = 3;
const DOUBLE_FAULT_VECTOR: usize = 8;
const PAGE_FAULT_VECTOR: usize = 14;
const INTERRUPT_GATE_PRESENT: u16 = 0x8e00;

static INITIALIZED: AtomicBool = AtomicBool::new(false);

static mut IDT: [IdtEntry; IDT_ENTRIES] = [IdtEntry::missing(); IDT_ENTRIES];

global_asm!(
    r#"
    .global aesynx_exception_breakpoint_stub
    .type aesynx_exception_breakpoint_stub, @function
aesynx_exception_breakpoint_stub:
    push 0
    push 3
    jmp aesynx_exception_common_return

    .global aesynx_exception_page_fault_stub
    .type aesynx_exception_page_fault_stub, @function
aesynx_exception_page_fault_stub:
    push 14
    jmp aesynx_exception_common_halt

    .global aesynx_exception_double_fault_stub
    .type aesynx_exception_double_fault_stub, @function
aesynx_exception_double_fault_stub:
    push 8
    jmp aesynx_exception_common_halt

aesynx_exception_common_return:
    mov rdi, rsp
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
    call aesynx_x86_64_exception_dispatch
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
    add rsp, 16
    iretq

aesynx_exception_common_halt:
    mov rdi, rsp
    and rsp, -16
    call aesynx_x86_64_exception_dispatch
1:
    hlt
    jmp 1b
    "#
);

unsafe extern "C" {
    fn aesynx_exception_breakpoint_stub();
    fn aesynx_exception_page_fault_stub();
    fn aesynx_exception_double_fault_stub();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExceptionTableStatus {
    pub idt_entries: usize,
    pub breakpoint_vector: u8,
    pub page_fault_vector: u8,
    pub double_fault_vector: u8,
    pub double_fault_ist: InterruptStackTableIndex,
}

#[must_use]
pub fn init(double_fault_ist: InterruptStackTableIndex) -> ExceptionTableStatus {
    if !INITIALIZED.swap(true, Ordering::AcqRel) {
        // SAFETY: IDT setup runs during early single-core boot before Aesynx
        // enables external interrupts. The IDT static is private and remains
        // valid after `lidt`; handler symbols are fixed assembly stubs in this
        // module.
        unsafe {
            init_idt(double_fault_ist);
            load_idt();
        }
    }

    ExceptionTableStatus {
        idt_entries: IDT_ENTRIES,
        breakpoint_vector: BREAKPOINT_VECTOR as u8,
        page_fault_vector: PAGE_FAULT_VECTOR as u8,
        double_fault_vector: DOUBLE_FAULT_VECTOR as u8,
        double_fault_ist,
    }
}

pub fn trigger_breakpoint_smoke() {
    // SAFETY: The breakpoint handler is installed by `init()` before this smoke
    // path is called. `int3` raises vector 3 and then resumes at the next
    // instruction through the returning exception stub.
    unsafe {
        core::arch::asm!("int3", options(nomem, nostack));
    }
}

#[allow(clippy::empty_loop)]
pub fn trigger_page_fault_smoke() -> ! {
    // SAFETY: This deliberately reads an unmapped null pointer only in the
    // opt-in exception smoke path. The installed page-fault handler prints the
    // marker and halts instead of returning to this faulting instruction.
    unsafe {
        let _ = core::ptr::read_volatile(core::ptr::null::<u64>());
    }

    loop {}
}

unsafe fn init_idt(double_fault_ist: InterruptStackTableIndex) {
    // SAFETY: The private IDT is initialized exactly once before it is loaded
    // into the CPU. The handler addresses are canonical kernel text addresses.
    unsafe {
        IDT[BREAKPOINT_VECTOR] = IdtEntry::interrupt_gate(aesynx_exception_breakpoint_stub, 0);
        IDT[PAGE_FAULT_VECTOR] = IdtEntry::interrupt_gate(aesynx_exception_page_fault_stub, 0);
        IDT[DOUBLE_FAULT_VECTOR] = IdtEntry::interrupt_gate(
            aesynx_exception_double_fault_stub,
            double_fault_ist.get() as u8,
        );
    }
}

unsafe fn load_idt() {
    let pointer = DescriptorTablePointer {
        limit: (size_of::<[IdtEntry; IDT_ENTRIES]>() - 1) as u16,
        base: core::ptr::addr_of!(IDT) as u64,
    };

    // SAFETY: The pointer references the private static IDT initialized above.
    // `lidt` loads the architectural descriptor-table register and does not
    // create Rust references or access untrusted memory.
    unsafe {
        core::arch::asm!(
            "lidt [{pointer}]",
            pointer = in(reg) &pointer,
            options(readonly, nostack, preserves_flags)
        );
    }
}

#[unsafe(no_mangle)]
extern "C" fn aesynx_x86_64_exception_dispatch(frame: *const RawExceptionFrame) {
    let Some(frame) = ExceptionFrame::from_raw(frame) else {
        crate::serial::write_str("exception frame=invalid\n");
        return;
    };

    match frame.vector {
        BREAKPOINT_VECTOR_U8 => {
            crate::serial::write_str("exception vector=breakpoint\n");
            crate::serial::write_str("[TEST] exception=ok\n");
        }
        PAGE_FAULT_VECTOR_U8 => {
            crate::serial_println!("exception vector=page-fault error=0x{:x}", frame.error_code);
            crate::serial::write_str("[TEST] pagefault=ok\n");
            crate::serial::write_str("[TEST] exception=ok\n");
        }
        DOUBLE_FAULT_VECTOR_U8 => {
            crate::serial_println!(
                "exception vector=double-fault error=0x{:x}",
                frame.error_code
            );
            crate::serial::write_str("[TEST] doublefault=ok\n");
            crate::serial::write_str("[TEST] exception=ok\n");
        }
        vector => {
            crate::serial_println!("exception vector={} error=0x{:x}", vector, frame.error_code);
        }
    }
}

const BREAKPOINT_VECTOR_U8: u8 = BREAKPOINT_VECTOR as u8;
const PAGE_FAULT_VECTOR_U8: u8 = PAGE_FAULT_VECTOR as u8;
const DOUBLE_FAULT_VECTOR_U8: u8 = DOUBLE_FAULT_VECTOR as u8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ExceptionFrame {
    vector: u8,
    error_code: u64,
}

impl ExceptionFrame {
    fn from_raw(raw: *const RawExceptionFrame) -> Option<Self> {
        if raw.is_null() || raw.align_offset(core::mem::align_of::<RawExceptionFrame>()) != 0 {
            return None;
        }

        // SAFETY: The assembly stubs pass a pointer to the active exception
        // stack frame. Only value fields needed for bounded diagnostics are
        // copied, and no reference escapes this function.
        let raw = unsafe { raw.read() };
        let vector = u8::try_from(raw.vector).ok()?;
        Some(Self {
            vector,
            error_code: raw.error_code,
        })
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RawExceptionFrame {
    vector: u64,
    error_code: u64,
    instruction_pointer: u64,
    code_segment: u64,
    rflags: u64,
}

#[repr(C, packed)]
struct DescriptorTablePointer {
    limit: u16,
    base: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    options: u16,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

impl IdtEntry {
    const fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            options: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    fn interrupt_gate(handler: unsafe extern "C" fn(), ist: u8) -> Self {
        let address = handler as *const () as usize as u64;
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

#[cfg(test)]
mod tests {
    use core::mem::size_of;

    use super::{
        DOUBLE_FAULT_VECTOR, ExceptionFrame, IDT_ENTRIES, INTERRUPT_GATE_PRESENT, IdtEntry,
        PAGE_FAULT_VECTOR, RawExceptionFrame,
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
    fn exception_frame_rejects_invalid_pointer() {
        assert_eq!(ExceptionFrame::from_raw(core::ptr::null()), None);
    }
}
