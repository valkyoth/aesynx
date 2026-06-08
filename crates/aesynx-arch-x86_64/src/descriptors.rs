use core::mem::size_of;
use core::sync::atomic::{AtomicBool, Ordering};

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

static INITIALIZED: AtomicBool = AtomicBool::new(false);

static mut GDT: [u64; GDT_ENTRIES] = [0; GDT_ENTRIES];
static mut TSS: TaskStateSegment = TaskStateSegment::new();
static mut DOUBLE_FAULT_STACK: AlignedStack = AlignedStack::new();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DescriptorTableStatus {
    pub gdt_entries: usize,
    pub tss_selector: SegmentSelector,
    pub double_fault_ist: InterruptStackTableIndex,
    pub double_fault_stack_bytes: usize,
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
    if !INITIALIZED.swap(true, Ordering::AcqRel) {
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
    }

    DescriptorTableStatus {
        gdt_entries: GDT_ENTRIES,
        tss_selector: SegmentSelector::TSS,
        double_fault_ist: InterruptStackTableIndex::DOUBLE_FAULT,
        double_fault_stack_bytes: DOUBLE_FAULT_STACK_BYTES,
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
/// while ring 3 execution is enabled.
pub unsafe fn set_ring0_stack(stack_top: u64) {
    // SAFETY: The public unsafe contract above requires the caller to provide a
    // valid current-CPU kernel stack pointer before privilege transitions use it.
    unsafe {
        TSS.rsp[0] = stack_top;
    }
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
        GDT_ENTRIES, InterruptStackTableIndex, KERNEL_CODE_DESCRIPTOR, KERNEL_DATA_DESCRIPTOR,
        SegmentSelector, TaskStateSegment, tss_descriptor,
    };

    #[test]
    fn segment_selectors_match_gdt_layout() {
        assert_eq!(SegmentSelector::KERNEL_CODE.bits(), 0x08);
        assert_eq!(SegmentSelector::KERNEL_DATA.bits(), 0x10);
        assert_eq!(SegmentSelector::TSS.bits(), 0x18);
        assert_eq!(GDT_ENTRIES, 5);
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
}
