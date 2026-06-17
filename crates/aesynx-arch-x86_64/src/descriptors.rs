use core::mem::size_of;
use core::sync::atomic::{AtomicU8, Ordering};

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
use aesynx_arch::ArchCpu;

#[cfg(feature = "smp")]
compile_error!(
    "x86_64 GDT/TSS/double-fault-stack storage is single-core static backing \
     storage; move descriptor tables and IST stacks to per-core ownership \
     before enabling smp"
);

const GDT_ENTRIES: usize = 5;
const DOUBLE_FAULT_STACK_BYTES: usize = 16 * 1024;
/// 0-indexed position in `TSS.ist`. The architectural IST number for IDT gates
/// is this value + 1 because IST field 0 means "no IST". Use
/// [`InterruptStackTableIndex::DOUBLE_FAULT`] when writing IDT gate descriptors.
const DOUBLE_FAULT_IST_SLOT: usize = 0;

const KERNEL_CODE_DESCRIPTOR: u64 = 0x00af_9a00_0000_ffff;
const KERNEL_DATA_DESCRIPTOR: u64 = 0x00af_9200_0000_ffff;
const TSS_DESCRIPTOR_TYPE: u64 = 0x9;
const PRESENT_BIT: u64 = 1 << 47;
const X86_64_KERNEL_VMA_MIN: u64 = 0xffff_8000_0000_0000;
const INIT_UNINITIALIZED: u8 = 0;
const INIT_IN_PROGRESS: u8 = 1;
const INIT_READY: u8 = 2;
const MSR_FS_BASE: u32 = 0xc000_0100;
const MSR_GS_BASE: u32 = 0xc000_0101;
const MSR_KERNEL_GS_BASE: u32 = 0xc000_0102;

static INIT_STATE: AtomicU8 = AtomicU8::new(INIT_UNINITIALIZED);

// TODO(smp): move GDT storage to per-core ownership before enabling SMP.
static mut GDT: [u64; GDT_ENTRIES] = [0; GDT_ENTRIES];
// TODO(smp): move TSS storage to per-core ownership before enabling SMP.
static mut TSS: TaskStateSegment = TaskStateSegment::new();
// TODO(smp): allocate a dedicated double-fault IST stack per core and place an
// unmapped guard page below each stack once paging owns per-core IST mappings.
static mut DOUBLE_FAULT_STACK: AlignedStack = AlignedStack::new();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DescriptorTableStatus {
    pub gdt_entries: usize,
    pub tss_selector: SegmentSelector,
    pub double_fault_ist: InterruptStackTableIndex,
    pub double_fault_stack_bytes: usize,
    pub initialized_this_call: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SegmentBaseMsr {
    Fs,
    Gs,
    KernelGs,
}

impl SegmentBaseMsr {
    const fn index(self) -> u32 {
        match self {
            Self::Fs => MSR_FS_BASE,
            Self::Gs => MSR_GS_BASE,
            Self::KernelGs => MSR_KERNEL_GS_BASE,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ring0StackError {
    InvalidStackTop,
    InterruptsEnabled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SegmentSelector(u16);

impl SegmentSelector {
    pub const KERNEL_CODE: Self = Self(0x08);
    pub const KERNEL_DATA: Self = Self(0x10);
    pub const TSS: Self = Self(0x18);

    #[must_use]
    pub const fn bits(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InterruptStackTableIndex(u16);

impl InterruptStackTableIndex {
    /// Architectural IDT-gate IST value for the double-fault handler.
    ///
    /// This is 1-indexed by the CPU and maps to `TSS.ist[0]`.
    pub const DOUBLE_FAULT: Self = Self(1);

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[must_use]
pub fn init() -> DescriptorTableStatus {
    let initialized_this_call = match INIT_STATE.compare_exchange(
        INIT_UNINITIALIZED,
        INIT_IN_PROGRESS,
        Ordering::AcqRel,
        Ordering::Acquire,
    ) {
        Ok(_) => {
            // SAFETY: Descriptor-table initialization runs during early single-core
            // boot before interrupts are enabled by Aesynx. The statics are private
            // to this module, initialized once, and then treated as read-only CPU
            // tables for the rest of early boot.
            unsafe {
                init_tables();
                load_gdt();
                reload_segment_registers();
                load_task_register(SegmentSelector::TSS);
            }
            INIT_STATE.store(INIT_READY, Ordering::Release);
            true
        }
        Err(INIT_READY) => false,
        Err(_) => {
            fail_closed_init("descriptor table init re-entered\n");
        }
    };

    DescriptorTableStatus {
        gdt_entries: GDT_ENTRIES,
        tss_selector: SegmentSelector::TSS,
        double_fault_ist: InterruptStackTableIndex::DOUBLE_FAULT,
        double_fault_stack_bytes: DOUBLE_FAULT_STACK_BYTES,
        initialized_this_call,
    }
}

/// Sets the kernel stack pointer used on ring 3 to ring 0 transitions.
///
/// This must be set to a valid per-core kernel stack before Aesynx enables
/// ring 3 execution. v0.7.0 does not run userspace yet, so the field remains
/// zero until the future privilege-transition setup can provide the real stack.
/// The current boot descriptor tables are deliberately single-core only: SMP
/// or ring 3 support must replace this global TSS/GDT storage with per-CPU
/// tables before any secondary core or userspace transition can call this.
///
/// # Safety
///
/// The caller must guarantee that `stack_top` is a canonical one-past-end
/// pointer to a writable kernel stack that remains valid for the current CPU
/// while ring 3 execution is enabled. The caller must also mask interrupts
/// before calling so no TSS consumer can observe an in-progress `rsp0` update.
/// This remains single-core-only storage; SMP must move to per-CPU TSS storage
/// before secondary cores can enter userspace.
pub unsafe fn set_ring0_stack(stack_top: u64) -> Result<(), Ring0StackError> {
    if !valid_ring0_stack_top(stack_top) {
        return Err(Ring0StackError::InvalidStackTop);
    }
    if !ring0_stack_update_interrupt_contract_holds() {
        return Err(Ring0StackError::InterruptsEnabled);
    }

    // NOTE: this is a single aligned 64-bit store, which is atomic with
    // respect to interrupt/NMI observation on x86_64. Do not split it into
    // multiple writes.
    // SAFETY: The public unsafe contract above requires a valid current-CPU
    // kernel stack pointer and masked interrupts before privilege transitions
    // or TSS/IST consumers can observe the update.
    unsafe {
        TSS.rsp[0] = stack_top;
    }
    Ok(())
}

fn ring0_stack_update_interrupt_contract_holds() -> bool {
    #[cfg(all(target_arch = "x86_64", target_os = "none"))]
    {
        matches!(crate::X86_64::interrupts_enabled(), Ok(false))
    }

    #[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
    {
        true
    }
}

const fn is_canonical_address(address: u64) -> bool {
    let sign_bit = (address >> 47) & 1;
    let upper = address >> 48;
    (sign_bit == 0 && upper == 0) || (sign_bit == 1 && upper == 0xffff)
}

const fn valid_ring0_stack_top(stack_top: u64) -> bool {
    stack_top != 0
        && stack_top >= X86_64_KERNEL_VMA_MIN
        && stack_top & 0xf == 0
        && is_canonical_address(stack_top)
}

unsafe fn init_tables() {
    // SAFETY: Private early-boot statics are initialized once before they are
    // loaded into CPU registers. The stack top points one byte past a private
    // aligned byte array, which is the architectural stack pointer convention.
    unsafe {
        let stack_base = core::ptr::addr_of!(DOUBLE_FAULT_STACK.bytes) as u64;
        let stack_top = stack_base + DOUBLE_FAULT_STACK_BYTES as u64;
        let tss_base = core::ptr::addr_of!(TSS) as u64;

        TSS.ist[DOUBLE_FAULT_IST_SLOT] = stack_top;
        GDT[0] = 0;
        GDT[1] = KERNEL_CODE_DESCRIPTOR;
        GDT[2] = KERNEL_DATA_DESCRIPTOR;
        let descriptor = tss_descriptor(tss_base, size_of::<TaskStateSegment>() - 1);
        GDT[3] = descriptor.low;
        GDT[4] = descriptor.high;
    }
}

unsafe fn load_gdt() {
    let pointer = DescriptorTablePointer {
        limit: (size_of::<[u64; GDT_ENTRIES]>() - 1) as u16,
        base: core::ptr::addr_of!(GDT) as u64,
    };

    // SAFETY: The pointer references the private static GDT initialized above.
    // `lgdt` only loads the CPU descriptor-table register and does not
    // dereference Rust references or modify Rust-managed memory.
    unsafe {
        core::arch::asm!(
            "lgdt [{pointer}]",
            pointer = in(reg) &pointer,
            options(readonly, nostack, preserves_flags)
        );
    }
}

unsafe fn reload_segment_registers() {
    const CODE_SELECTOR: u64 = SegmentSelector::KERNEL_CODE.bits() as u64;
    const DATA_SELECTOR: u16 = SegmentSelector::KERNEL_DATA.bits();

    // SAFETY: The selectors reference descriptors just installed in the
    // private GDT. The far return reloads CS, the data selector refreshes
    // SS/DS/ES, and FS/GS are reset to null selectors. Future per-CPU and TLS
    // support must set FS/GS bases separately through model-specific registers.
    unsafe {
        core::arch::asm!(
            "push {code_selector}",
            "lea {return_address}, [rip + 2f]",
            "push {return_address}",
            "retfq",
            "2:",
            "mov ax, {data_selector}",
            "mov ss, ax",
            "mov ds, ax",
            "mov es, ax",
            "xor ax, ax",
            "mov fs, ax",
            "mov gs, ax",
            code_selector = const CODE_SELECTOR,
            data_selector = const DATA_SELECTOR,
            return_address = lateout(reg) _,
            options(preserves_flags)
        );
        clear_fs_gs_bases();
    }
}

unsafe fn clear_fs_gs_bases() {
    // SAFETY: These are architectural x86_64 segment-base MSRs. Clearing them
    // removes bootloader/firmware residual state after FS/GS selectors are
    // reset and before Aesynx has any TLS/per-CPU base policy.
    unsafe {
        write_segment_base_msr(SegmentBaseMsr::Fs, 0);
        write_segment_base_msr(SegmentBaseMsr::Gs, 0);
        write_segment_base_msr(SegmentBaseMsr::KernelGs, 0);
    }
}

unsafe fn write_segment_base_msr(msr: SegmentBaseMsr, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    let index = msr.index();
    // SAFETY: The caller admits the architectural MSR and preserves its
    // reserved-bit requirements.
    unsafe {
        core::arch::asm!(
            "wrmsr",
            in("ecx") index,
            in("eax") low,
            in("edx") high,
            options(nomem, nostack, preserves_flags)
        );
    }
}

fn fail_closed_init(message: &str) -> ! {
    #[cfg(all(target_arch = "x86_64", target_os = "none"))]
    {
        crate::serial::write_str(message);
        crate::X86_64::halt_forever()
    }

    #[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
    {
        let _ = message;
        loop {
            core::hint::spin_loop();
        }
    }
}

unsafe fn load_task_register(selector: SegmentSelector) {
    let selector = selector.bits();

    // SAFETY: The selector names the available TSS descriptor installed in the
    // private GDT above. The TSS is static and remains valid after `ltr`.
    unsafe {
        core::arch::asm!("ltr ax", in("ax") selector, options(nostack, preserves_flags));
    }
}

const fn tss_descriptor(base: u64, limit: usize) -> SystemDescriptor {
    let limit = limit as u64;
    let low = (limit & 0xffff)
        | ((base & 0x00ff_ffff) << 16)
        | (TSS_DESCRIPTOR_TYPE << 40)
        | PRESENT_BIT
        | (((limit >> 16) & 0x0f) << 48)
        | (((base >> 24) & 0xff) << 56);
    let high = base >> 32;

    SystemDescriptor { low, high }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SystemDescriptor {
    low: u64,
    high: u64,
}

#[repr(C, packed)]
struct DescriptorTablePointer {
    limit: u16,
    base: u64,
}

#[repr(C, packed)]
struct TaskStateSegment {
    reserved_0: u32,
    rsp: [u64; 3],
    reserved_1: u64,
    ist: [u64; 7],
    reserved_2: u64,
    reserved_3: u16,
    io_map_base: u16,
}

impl TaskStateSegment {
    const fn new() -> Self {
        Self {
            reserved_0: 0,
            // RSP0 must be set with `set_ring0_stack()` before any ring 3 to
            // ring 0 transition is permitted.
            rsp: [0; 3],
            reserved_1: 0,
            ist: [0; 7],
            reserved_2: 0,
            reserved_3: 0,
            io_map_base: size_of::<Self>() as u16,
        }
    }
}

#[repr(align(16))]
struct AlignedStack {
    bytes: [u8; DOUBLE_FAULT_STACK_BYTES],
}

impl AlignedStack {
    const fn new() -> Self {
        Self {
            bytes: [0; DOUBLE_FAULT_STACK_BYTES],
        }
    }
}

#[cfg(test)]
mod tests {
    use core::mem::{align_of, size_of};

    use super::{
        AlignedStack, DOUBLE_FAULT_IST_SLOT, DOUBLE_FAULT_STACK_BYTES, DescriptorTablePointer,
        GDT_ENTRIES, INIT_IN_PROGRESS, INIT_READY, INIT_UNINITIALIZED, InterruptStackTableIndex,
        KERNEL_CODE_DESCRIPTOR, KERNEL_DATA_DESCRIPTOR, MSR_FS_BASE, MSR_GS_BASE,
        MSR_KERNEL_GS_BASE, SegmentBaseMsr, SegmentSelector, TaskStateSegment, tss_descriptor,
        valid_ring0_stack_top,
    };

    #[test]
    fn segment_selectors_match_gdt_layout() {
        assert_eq!(SegmentSelector::KERNEL_CODE.bits(), 0x08);
        assert_eq!(SegmentSelector::KERNEL_DATA.bits(), 0x10);
        assert_eq!(SegmentSelector::TSS.bits(), 0x18);
        assert_eq!(GDT_ENTRIES, 5);
    }

    #[test]
    fn init_state_values_are_ordered() {
        assert_eq!(INIT_UNINITIALIZED, 0);
        assert_eq!(INIT_IN_PROGRESS, 1);
        assert_eq!(INIT_READY, 2);
    }

    #[test]
    fn admitted_segment_base_msrs_are_explicit() {
        assert_eq!(SegmentBaseMsr::Fs.index(), MSR_FS_BASE);
        assert_eq!(SegmentBaseMsr::Gs.index(), MSR_GS_BASE);
        assert_eq!(SegmentBaseMsr::KernelGs.index(), MSR_KERNEL_GS_BASE);
    }

    #[test]
    fn double_fault_ist_uses_first_architectural_slot() {
        assert_eq!(DOUBLE_FAULT_IST_SLOT, 0);
        assert_eq!(InterruptStackTableIndex::DOUBLE_FAULT.get(), 1);
        assert_eq!(DOUBLE_FAULT_STACK_BYTES, 16 * 1024);
        assert_eq!(align_of::<AlignedStack>(), 16);
    }

    #[test]
    fn code_and_data_descriptors_are_present_ring_zero_long_mode_entries() {
        assert_eq!(KERNEL_CODE_DESCRIPTOR, 0x00af_9a00_0000_ffff);
        assert_eq!(KERNEL_DATA_DESCRIPTOR, 0x00af_9200_0000_ffff);
    }

    #[test]
    fn tss_descriptor_encodes_base_limit_type_and_present_bit() {
        let base = 0x1234_5678_9abc_def0;
        let limit = size_of::<TaskStateSegment>() - 1;
        let descriptor = tss_descriptor(base, limit);

        assert_eq!(descriptor.low & 0xffff, limit as u64 & 0xffff);
        assert_eq!((descriptor.low >> 16) & 0x00ff_ffff, base & 0x00ff_ffff);
        assert_eq!((descriptor.low >> 40) & 0x0f, 0x9);
        assert_eq!((descriptor.low >> 47) & 1, 1);
        assert_eq!((descriptor.low >> 48) & 0x0f, (limit as u64 >> 16) & 0x0f);
        assert_eq!((descriptor.low >> 56) & 0xff, (base >> 24) & 0xff);
        assert_eq!(descriptor.high, base >> 32);
    }

    #[test]
    fn tss_layout_matches_x86_64_size() {
        assert_eq!(size_of::<TaskStateSegment>(), 104);
        assert_eq!(size_of::<DescriptorTablePointer>(), 10);
    }

    #[test]
    fn ring0_stack_validator_accepts_aligned_canonical_kernel_stack_top() {
        assert!(valid_ring0_stack_top(0xffff_ffff_8000_1000));
    }

    #[test]
    fn ring0_stack_validator_rejects_invalid_stack_tops() {
        assert!(!valid_ring0_stack_top(0));
        assert!(!valid_ring0_stack_top(0x0000_7fff_ffff_f000));
        assert!(!valid_ring0_stack_top(0xffff_ffff_8000_1008));
        assert!(!valid_ring0_stack_top(0x0000_8000_0000_0000));
    }
}
