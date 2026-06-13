use core::arch::global_asm;
use core::mem::size_of;
use core::sync::atomic::{AtomicBool, Ordering};

use aesynx_arch::ArchCpu;

use crate::descriptors::InterruptStackTableIndex;
use crate::registers::FaultRegisterSnapshot;

mod frame;
mod idt;

use frame::{ExceptionFrame, PageFaultErrorCode, RawExceptionFrame};
use idt::{DescriptorTablePointer, IdtEntry};

const IDT_ENTRIES: usize = 256;
const BREAKPOINT_VECTOR: usize = 3;
const DOUBLE_FAULT_VECTOR: usize = 8;
const PAGE_FAULT_VECTOR: usize = 14;
const INTERRUPT_GATE_PRESENT: u16 = 0x8e00;
const EXCEPTION_STUB_BYTES: usize = 16;
const RETURNING_EXCEPTION_GPR_SAVE_COUNT: usize = 15;
const RETURNING_EXCEPTION_GPR_SAVE_BYTES: usize =
    RETURNING_EXCEPTION_GPR_SAVE_COUNT * size_of::<u64>();

static INITIALIZED: AtomicBool = AtomicBool::new(false);

static mut IDT: [IdtEntry; IDT_ENTRIES] = [IdtEntry::missing(); IDT_ENTRIES];

global_asm!(
    r#"
    .global aesynx_exception_stub_table
    .type aesynx_exception_stub_table, @function
aesynx_exception_stub_table:
    .set vector, 0
    .rept 256
        .set has_error_code, 0
        .if vector == 8
            .set has_error_code, 1
        .endif
        .if vector == 10
            .set has_error_code, 1
        .endif
        .if vector == 11
            .set has_error_code, 1
        .endif
        .if vector == 12
            .set has_error_code, 1
        .endif
        .if vector == 13
            .set has_error_code, 1
        .endif
        .if vector == 14
            .set has_error_code, 1
        .endif
        .if vector == 17
            .set has_error_code, 1
        .endif
        .if vector == 21
            .set has_error_code, 1
        .endif
        .if vector == 29
            .set has_error_code, 1
        .endif
        .if vector == 30
            .set has_error_code, 1
        .endif
        .if has_error_code
            push vector
        .else
            push 0
            push vector
        .endif
        jmp aesynx_exception_common_halt
        .org aesynx_exception_stub_table + ((vector + 1) * 16), 0x90
        .set vector, vector + 1
    .endr

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
    lea rdi, [rsp + {return_frame_offset}]
    mov rbp, rsp
    and rsp, -16
    call aesynx_x86_64_exception_dispatch
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
    add rsp, 16
    iretq

aesynx_exception_common_halt:
    mov rdi, rsp
    and rsp, -16
    call aesynx_x86_64_exception_dispatch
1:
    hlt
    jmp 1b
    "#,
    return_frame_offset = const RETURNING_EXCEPTION_GPR_SAVE_BYTES,
);

unsafe extern "C" {
    static aesynx_exception_stub_table: u8;
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
    pub initialized_this_call: bool,
}

#[must_use]
pub fn init(double_fault_ist: InterruptStackTableIndex) -> ExceptionTableStatus {
    let initialized_this_call = !INITIALIZED.swap(true, Ordering::AcqRel);
    if initialized_this_call {
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
        initialized_this_call,
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

pub(crate) fn install_interrupt_gate(
    vector: u8,
    handler: unsafe extern "C" fn(),
) -> Result<(), IdtError> {
    let index = usize::from(vector);
    if !(0x20..IDT_ENTRIES).contains(&index) {
        return Err(IdtError::InvalidInterruptVector);
    }

    let interrupts_were_enabled = interrupts_were_enabled_before_masking();
    debug_assert!(
        !interrupts_were_enabled,
        "install_interrupt_gate called with interrupts enabled"
    );
    if interrupts_were_enabled {
        let _ = crate::X86_64::enable_interrupts();
        return Err(IdtError::CpuStateUnavailable);
    }

    // SAFETY: The IDT is private static storage initialized during early boot
    // and this installer is limited to non-exception interrupt vectors. The
    // 16-byte descriptor write is not architecturally atomic. This function
    // masks IF before the write and rejects callers that had already enabled
    // maskable interrupts. NMIs are still a future live-IDT-mutation concern
    // and require a separate platform-specific exclusion strategy before
    // drivers can update gates.
    unsafe {
        IDT[index] = IdtEntry::interrupt_gate(handler, 0);
    }
    Ok(())
}

fn interrupts_were_enabled_before_masking() -> bool {
    let rflags: u64;
    // SAFETY: `pushfq; cli; pop` stores the previous RFLAGS value on the
    // current stack, masks IF for the current CPU, then copies the saved value
    // into a general-purpose register. It does not create Rust references or
    // touch untrusted memory.
    unsafe {
        core::arch::asm!(
            "pushfq",
            "cli",
            "pop {rflags}",
            rflags = lateout(reg) rflags,
        );
    }
    rflags & (1 << 9) != 0
}

#[allow(clippy::empty_loop)]
pub fn trigger_page_fault_smoke() -> ! {
    // SAFETY: This deliberately issues an assembly load from virtual address
    // zero only in the opt-in exception smoke path. The installed page-fault
    // handler prints the marker and halts instead of returning to this faulting
    // instruction. The fault is produced by the CPU load instruction rather
    // than by constructing an invalid Rust pointer.
    unsafe {
        core::arch::asm!(
            "mov {scratch}, qword ptr [{address}]",
            address = in(reg) 0usize,
            scratch = lateout(reg) _,
            options(nostack, readonly)
        );
    }

    loop {}
}

unsafe fn init_idt(double_fault_ist: InterruptStackTableIndex) {
    // SAFETY: The private IDT is initialized exactly once before it is loaded
    // into the CPU. The handler addresses are canonical kernel text addresses.
    unsafe {
        let mut vector = 0usize;
        while vector < IDT_ENTRIES {
            IDT[vector] = IdtEntry::interrupt_gate_address(exception_stub_address(vector), 0);
            vector += 1;
        }
        IDT[BREAKPOINT_VECTOR] = IdtEntry::interrupt_gate(aesynx_exception_breakpoint_stub, 0);
        IDT[PAGE_FAULT_VECTOR] = IdtEntry::interrupt_gate(aesynx_exception_page_fault_stub, 0);
        IDT[DOUBLE_FAULT_VECTOR] = IdtEntry::interrupt_gate(
            aesynx_exception_double_fault_stub,
            double_fault_ist.get() as u8,
        );
    }
}

fn exception_stub_address(vector: usize) -> u64 {
    let table = core::ptr::addr_of!(aesynx_exception_stub_table) as usize as u64;
    table + (vector as u64 * EXCEPTION_STUB_BYTES as u64)
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
    let fault_address = FaultRegisterSnapshot::capture_fault_address();

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
            let error = PageFaultErrorCode::new(frame.error_code);
            let registers = FaultRegisterSnapshot::capture_with_fault_address(fault_address);
            crate::serial_println!(
                "exception vector=page-fault error=0x{:x} rip_present={} rip_offset=0x{:x} cs=0x{:x} frame_rflags=0x{:x} cr2_present={} cr2_offset=0x{:x} cr3_offset=0x{:x} rflags=0x{:x} interrupts_enabled={} present={} write={} user={} reserved={} instruction={} protection_key={} shadow_stack={} sgx={}",
                frame.error_code,
                frame.instruction_pointer_present(),
                frame.instruction_pointer_offset(),
                frame.code_segment,
                frame.public_rflags(),
                registers.fault_address_present(),
                registers.fault_address_page_offset(),
                registers.cr3_page_offset(),
                registers.public_rflags(),
                registers.interrupts_enabled(),
                error.present(),
                error.write(),
                error.user(),
                error.reserved_bit(),
                error.instruction_fetch(),
                error.protection_key(),
                error.shadow_stack(),
                error.sgx()
            );
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
pub enum IdtError {
    InvalidInterruptVector,
    CpuStateUnavailable,
}

const PAGE_OFFSET_MASK: u64 = 0xfff;

#[cfg(test)]
mod tests;
