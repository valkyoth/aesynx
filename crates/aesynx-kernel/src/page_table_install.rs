use aesynx_arch::ArchCpu;
use core::sync::atomic::{Ordering, compiler_fence};

#[cfg(feature = "smp")]
compile_error!(
    "ACTIVATION_ARENA and ACTIVATION_STACK are single-core statics; \
     move activation storage to per-core ownership before enabling smp"
);

pub const ACTIVATION_TABLES: usize = aesynx_mm::PAGE_TABLE_LEVELS;
const ACTIVATION_STACK_BYTES: usize = 16 * 1024;
const ACTIVATION_STACK_GUARD_PAGES: u64 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageTableInstallStatus {
    pub tables_copied: u64,
    pub entries_copied: u64,
    pub root_copied: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivationStackLayout {
    pub guard_start: aesynx_abi::VirtAddr,
    pub guard_pages: u64,
    pub stack_start: aesynx_abi::VirtAddr,
    pub stack_end: aesynx_abi::VirtAddr,
    pub stack_pages: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PageTableInstallError {
    ActiveCr3Overlap,
    ActivationStackRange,
    Mapper(aesynx_mm::PageTableError),
    KernelImageRange,
    UnexpectedImage,
}

pub fn activation_root_phys(
    info: &aesynx_boot::BootInfo<'_>,
) -> Result<aesynx_abi::PhysAddr, PageTableInstallError> {
    let arena = activation_arena_virt();
    info.kernel_image
        .phys_for_virt(arena)
        .ok_or(PageTableInstallError::KernelImageRange)
}

pub fn activation_stack_layout() -> Result<ActivationStackLayout, PageTableInstallError> {
    let guard_start = linker_symbol_virt(activation_stack_guard_start);
    let guard_end = linker_symbol_virt(activation_stack_guard_end);
    let stack_start = linker_symbol_virt(activation_stack_start);
    let stack_end = linker_symbol_virt(activation_stack_end);
    let guard_bytes = guard_end
        .get()
        .checked_sub(guard_start.get())
        .ok_or(PageTableInstallError::ActivationStackRange)?;
    let stack_bytes = stack_end
        .get()
        .checked_sub(stack_start.get())
        .ok_or(PageTableInstallError::ActivationStackRange)?;

    if !page_aligned(guard_start)
        || !page_aligned(guard_end)
        || !page_aligned(stack_start)
        || !page_aligned(stack_end)
        || guard_end != stack_start
        || guard_bytes != aesynx_mm::FRAME_SIZE
        || stack_bytes != ACTIVATION_STACK_BYTES as u64
    {
        return Err(PageTableInstallError::ActivationStackRange);
    }

    Ok(ActivationStackLayout {
        guard_start,
        guard_pages: ACTIVATION_STACK_GUARD_PAGES,
        stack_start,
        stack_end,
        stack_pages: stack_bytes / aesynx_mm::FRAME_SIZE,
    })
}

pub fn copy_mapper_to_activation_arena<const TABLES: usize, const MAPPED_FRAMES: usize>(
    root_phys: aesynx_abi::PhysAddr,
    mapper: &aesynx_mm::PageTableMapper<TABLES, MAPPED_FRAMES>,
) -> Result<PageTableInstallStatus, PageTableInstallError> {
    if TABLES > ACTIVATION_TABLES {
        return Err(PageTableInstallError::UnexpectedImage);
    }
    if aesynx_arch_x86_64::registers::EarlyRegisterSnapshot::capture().cr3_page_matches(root_phys) {
        return Err(PageTableInstallError::ActiveCr3Overlap);
    }

    // SAFETY: `ACTIVATION_ARENA` is a private, page-aligned kernel `.bss`
    // object. During the v0.16.2 single-core boot smoke no Rust references to
    // the arena are created; it is written only through raw volatile stores
    // before any future CR3 switch can consume it.
    let arena = activation_arena_ptr();
    // SAFETY: `arena` points at `ACTIVATION_TABLES` contiguous 4 KiB page-table
    // frames owned by the kernel image. The helper writes exactly that bounded
    // table area with volatile stores.
    unsafe {
        zero_activation_arena(arena);
    }

    let mut table_index = 0usize;
    let mut tables_copied = 0u64;
    let mut entries_copied = 0u64;
    let mut entries = [0u64; aesynx_mm::PAGE_TABLE_ENTRIES];
    while table_index < TABLES {
        if mapper
            .export_x86_64_hardware_table_entries(root_phys, table_index, &mut entries)
            .map_err(PageTableInstallError::Mapper)?
        {
            // SAFETY: The arena was validated above and `table_index` is
            // bounded by `TABLES <= ACTIVATION_TABLES`.
            unsafe {
                write_table_volatile(arena, table_index, &entries);
            }
            tables_copied += 1;
            entries_copied += aesynx_mm::PAGE_TABLE_ENTRIES as u64;
        }
        table_index += 1;
    }

    if tables_copied == 0 {
        return Err(PageTableInstallError::UnexpectedImage);
    }

    compiler_fence(Ordering::Release);
    Ok(PageTableInstallStatus {
        tables_copied,
        entries_copied,
        root_copied: true,
    })
}

pub fn activate_kernel_address_space_and_halt(
    root_phys: aesynx_abi::PhysAddr,
    allocator: &'static crate::kernel_heap::KernelHeapAllocator,
) -> Result<core::convert::Infallible, PageTableInstallError> {
    if aesynx_arch_x86_64::registers::EarlyRegisterSnapshot::capture().cr3_page_matches(root_phys) {
        return Err(PageTableInstallError::ActiveCr3Overlap);
    }
    let stack_top =
        activation_stack_top_virt().ok_or(PageTableInstallError::ActivationStackRange)?;
    compiler_fence(Ordering::Release);

    // SAFETY: The activation stack is a private kernel `.bss` object covered by
    // the just-installed data/BSS mapping. `root_phys` identifies the copied
    // static activation arena, which maps the current text and data sections.
    unsafe { switch_to_activation_stack(root_phys.get(), stack_top.get(), allocator) }
}

fn activation_arena_virt() -> aesynx_abi::VirtAddr {
    // SAFETY: Taking the raw address of the private static does not create a
    // Rust reference or read/write the arena. The address is used only as a
    // value so BootInfo can derive the corresponding kernel-image physical
    // address.
    let arena = activation_arena_ptr() as u64;
    aesynx_abi::VirtAddr::new(arena)
}

#[inline(never)]
fn activation_arena_ptr() -> *mut u64 {
    // SAFETY: Taking the raw address of the private static does not create a
    // Rust reference or access memory. `black_box` keeps the address in a
    // runtime value so volatile stores use a register base instead of fragile
    // absolute addressing forms for the high-half kernel.
    let arena = unsafe { core::ptr::addr_of_mut!(ACTIVATION_ARENA.tables) as *mut u64 };
    core::hint::black_box(arena)
}

fn activation_stack_top_virt() -> Option<aesynx_abi::VirtAddr> {
    let layout = activation_stack_layout().ok()?;
    layout
        .stack_start
        .get()
        .checked_add(ACTIVATION_STACK_BYTES as u64)
        .map(aesynx_abi::VirtAddr::new)
}

fn linker_symbol_virt(symbol: fn() -> *const u8) -> aesynx_abi::VirtAddr {
    aesynx_abi::VirtAddr::new(core::hint::black_box(symbol() as u64))
}

fn page_aligned(virt: aesynx_abi::VirtAddr) -> bool {
    virt.get() & (aesynx_mm::FRAME_SIZE - 1) == 0
}

fn activation_stack_guard_start() -> *const u8 {
    unsafe extern "C" {
        static __kernel_activation_stack_guard_start: u8;
    }

    // SAFETY: The symbol is provided by the kernel linker script. Taking its
    // raw address does not read memory or create aliases.
    core::ptr::addr_of!(__kernel_activation_stack_guard_start)
}

fn activation_stack_guard_end() -> *const u8 {
    unsafe extern "C" {
        static __kernel_activation_stack_guard_end: u8;
    }

    // SAFETY: The symbol is provided by the kernel linker script. Taking its
    // raw address does not read memory or create aliases.
    core::ptr::addr_of!(__kernel_activation_stack_guard_end)
}

fn activation_stack_start() -> *const u8 {
    unsafe extern "C" {
        static __kernel_activation_stack_start: u8;
    }

    // SAFETY: The symbol is provided by the kernel linker script. Taking its
    // raw address does not read memory or create aliases.
    core::ptr::addr_of!(__kernel_activation_stack_start)
}

fn activation_stack_end() -> *const u8 {
    unsafe extern "C" {
        static __kernel_activation_stack_end: u8;
    }

    // SAFETY: The symbol is provided by the kernel linker script. Taking its
    // raw address does not read memory or create aliases.
    core::ptr::addr_of!(__kernel_activation_stack_end)
}

unsafe fn switch_to_activation_stack(
    root_phys: u64,
    stack_top: u64,
    allocator: &'static crate::kernel_heap::KernelHeapAllocator,
) -> ! {
    // SAFETY: The caller guarantees that `stack_top` is the one-past-end
    // address of the private activation stack and that `root_phys` points to a
    // page table that maps both this function's text and that stack. The stack
    // is aligned to the SysV function-entry convention before jumping to the
    // terminal activation continuation.
    unsafe {
        core::arch::asm!(
            "mov rsp, {stack_top}",
            "and rsp, -16",
            "sub rsp, 8",
            "jmp {entry}",
            stack_top = in(reg) stack_top,
            in("rdi") root_phys,
            in("rsi") allocator,
            entry = sym activate_on_kernel_stack,
            options(noreturn)
        );
    }
}

extern "C" fn activate_on_kernel_stack(
    root_phys: u64,
    allocator: &'static crate::kernel_heap::KernelHeapAllocator,
) -> ! {
    let load_result = {
        // SAFETY: The caller switched to the private activation stack and
        // passes the physical root of the static activation arena populated
        // from the audited mapper immediately before this terminal handoff.
        unsafe { aesynx_arch_x86_64::registers::load_cr3(aesynx_abi::PhysAddr::new(root_phys)) }
    };
    if let Err(error) = load_result {
        aesynx_arch_x86_64::serial_println!("kernel-cr3 load_error={:?}", error);
        aesynx_arch_x86_64::serial::write_str("[TEST] kernel-cr3=fail\n");
        aesynx_arch_x86_64::X86_64::halt_forever()
    }
    compiler_fence(Ordering::SeqCst);
    match aesynx_arch_x86_64::cpu_hardening::init() {
        Ok(status) => {
            aesynx_arch_x86_64::serial_println!(
                "cpu-hardening nx={} wp={} smep={} smap={} umip={} ibrs={} ibpb_supported={} ibpb_attempted={} stibp={} ssbd={} arch_capabilities={}",
                status.nx_enabled,
                status.wp_enabled,
                status.smep_enabled,
                status.smap_enabled,
                status.umip_enabled,
                status.ibrs_enabled,
                status.ibpb_supported,
                status.ibpb_attempted,
                status.stibp_enabled,
                status.ssbd_enabled,
                status.arch_capabilities_supported
            );
            aesynx_arch_x86_64::serial::write_str("[TEST] cpu-hardening=ok\n");
        }
        Err(error) => {
            aesynx_arch_x86_64::serial_println!("cpu-hardening error={:?}", error);
            aesynx_arch_x86_64::serial::write_str("[TEST] cpu-hardening=fail\n");
            aesynx_arch_x86_64::X86_64::halt_forever()
        }
    }
    match crate::entropy_smoke::run() {
        Ok(status) => {
            aesynx_arch_x86_64::serial_println!(
                "entropy-policy rdrand={} rdseed={} hardware_self_test={} drbg_self_test={} hardware_present={} fallback_used={} generation_counter_ok={} random_tokens_available={} source={:?}",
                status.rdrand_supported,
                status.rdseed_supported,
                status.hardware_self_test_passed,
                status.drbg_self_test_passed,
                status.hardware_entropy_present,
                status.fallback_used,
                status.generation_counter_ok,
                status.random_tokens_available,
                status.primary_source
            );
            aesynx_arch_x86_64::serial::write_str("[TEST] entropy-policy=ok\n");
        }
        Err(error) => {
            aesynx_arch_x86_64::serial_println!("entropy-policy error={:?}", error);
            aesynx_arch_x86_64::serial::write_str("[TEST] entropy-policy=fail\n");
            aesynx_arch_x86_64::X86_64::halt_forever()
        }
    }
    match crate::kernel_heap::smoke(allocator) {
        Ok(status) => {
            aesynx_arch_x86_64::serial_println!(
                "heap bytes={} allocated={} peak={} slab_classes={} slab_allocations={} page_allocations={} frees={} double_free_detected={} invalid_free_detected={} accounting_overflow_detected={} corrupt_free_list_detected={} box_ok={} vec_ok={} btree_ok={} slab_reuse_ok={} page_run_ok={} stress_ok={} oom_rejected={}",
                status.heap_bytes,
                status.allocated_bytes,
                status.peak_allocated_bytes,
                status.slab_classes,
                status.slab_allocations,
                status.page_allocations,
                status.frees,
                status.double_free_detected,
                status.invalid_free_detected,
                status.accounting_overflow_detected,
                status.corrupt_free_list_detected,
                status.box_ok,
                status.vec_ok,
                status.btree_ok,
                status.slab_reuse_ok,
                status.page_run_ok,
                status.stress_ok,
                status.oom_rejected
            );
            aesynx_arch_x86_64::serial::write_str("[TEST] heap=ok\n");
        }
        Err(error) => {
            aesynx_arch_x86_64::serial_println!("heap error={:?}", error);
            aesynx_arch_x86_64::serial::write_str("[TEST] heap=fail\n");
            aesynx_arch_x86_64::X86_64::halt_forever()
        }
    }
    match crate::capability_smoke::run() {
        Ok(status) => {
            aesynx_arch_x86_64::serial_println!(
                "cap-table capacity={} occupied_before_revoke={} occupied_after_revoke={} root_read_ok={} child_read_ok={} grant_read_ok={} grant_regrant_denied={} child_write_denied={} stale_root_denied={} stale_child_denied={} audit_events={} revoked_slots={}",
                status.capacity,
                status.occupied_before_revoke,
                status.occupied_after_revoke,
                status.root_read_ok,
                status.child_read_ok,
                status.grant_read_ok,
                status.grant_regrant_denied,
                status.child_write_denied,
                status.stale_root_denied,
                status.stale_child_denied,
                status.audit_events,
                status.revoked_slots
            );
            aesynx_arch_x86_64::serial_println!(
                "memory-cap map_allowed={} mapping_descriptor_ok={} read_denied={} write_denied={} range_escape_denied={}",
                status.memory_map_allowed,
                status.memory_mapping_descriptor_ok,
                status.memory_read_denied,
                status.memory_write_denied,
                status.memory_range_escape_denied
            );
            aesynx_arch_x86_64::serial_println!(
                "cap-audit events={} mint_seen={} derive_seen={} grant_seen={} revoke_seen={} revoke_slots={} cap_faults={}",
                status.audit_events,
                status.mint_audit_seen,
                status.derive_audit_seen,
                status.grant_audit_seen,
                status.revoke_audit_seen,
                status.revoke_audit_slots,
                status.cap_fault_events
            );
            aesynx_arch_x86_64::serial::write_str("[TEST] cap=ok\n");
            aesynx_arch_x86_64::serial::write_str("[TEST] memory-cap=ok\n");
            aesynx_arch_x86_64::serial::write_str("[TEST] cap-audit=ok\n");
        }
        Err(error) => {
            aesynx_arch_x86_64::serial_println!("cap-table error={:?}", error);
            aesynx_arch_x86_64::serial::write_str("[TEST] cap=fail\n");
            aesynx_arch_x86_64::serial::write_str("[TEST] memory-cap=fail\n");
            aesynx_arch_x86_64::serial::write_str("[TEST] cap-audit=fail\n");
            aesynx_arch_x86_64::X86_64::halt_forever()
        }
    }
    match crate::ipc_pingpong_smoke::run() {
        Ok(status) => {
            aesynx_arch_x86_64::serial_println!(
                "ipc-pingpong ping_seq={} pong_seq={} backpressure_events={} ipc_backpressure_ok={} ipc_release_acquire_ok={} ipc_pairwise_route_ok={}",
                status.ping_seq,
                status.pong_seq,
                status.backpressure_events,
                status.backpressure_ok,
                status.release_acquire_ok,
                status.pairwise_route_ok
            );
            aesynx_arch_x86_64::serial::write_str("[TEST] ipc-pingpong=ok\n");
        }
        Err(error) => {
            aesynx_arch_x86_64::serial_println!("ipc-pingpong error={:?}", error);
            aesynx_arch_x86_64::serial::write_str("[TEST] ipc-pingpong=fail\n");
            aesynx_arch_x86_64::X86_64::halt_forever()
        }
    }
    super::capability_ipc_report::run_or_halt();
    match crate::service_queue_smoke::run() {
        Ok(status) => {
            aesynx_arch_x86_64::serial_println!(
                "service-queue log_submitted={} log_observed={} completion_observed={} timer_pending={} object_pending={} release_acquire_ok={} unsupported_denied={} unsupported_pending_denied={}",
                status.log_submitted,
                status.log_observed,
                status.completion_observed,
                status.timer_pending,
                status.object_pending,
                status.release_acquire_ok,
                status.unsupported_denied,
                status.unsupported_pending_denied
            );
            aesynx_arch_x86_64::serial::write_str("[TEST] service-queue=ok\n");
        }
        Err(error) => {
            aesynx_arch_x86_64::serial_println!("service-queue error={:?}", error);
            aesynx_arch_x86_64::serial::write_str("[TEST] service-queue=fail\n");
            aesynx_arch_x86_64::X86_64::halt_forever()
        }
    }
    crate::execution_smoke::run();
    aesynx_arch_x86_64::serial::write_str("kernel-cr3 active=true stack=kernel\n");
    aesynx_arch_x86_64::serial::write_str("[TEST] kernel-cr3=ok\n");
    aesynx_arch_x86_64::X86_64::halt_forever()
}

unsafe fn zero_activation_arena(arena: *mut u64) {
    let mut index = 0usize;
    while index < ACTIVATION_TABLES * aesynx_mm::PAGE_TABLE_ENTRIES {
        // SAFETY: The caller guarantees that `arena` spans the complete static
        // activation table area and `index` is bounded by that area.
        unsafe {
            arena.add(index).write_volatile(0);
        }
        index += 1;
    }
}

unsafe fn write_table_volatile(
    arena: *mut u64,
    table_index: usize,
    entries: &[u64; aesynx_mm::PAGE_TABLE_ENTRIES],
) {
    let base = table_index * aesynx_mm::PAGE_TABLE_ENTRIES;
    let mut index = 0usize;
    while index < aesynx_mm::PAGE_TABLE_ENTRIES {
        // SAFETY: The caller guarantees that `table_index` selects a table in
        // the static activation arena and `index` is bounded by one table.
        unsafe {
            arena.add(base + index).write_volatile(entries[index]);
        }
        index += 1;
    }
}

#[repr(C, align(4096))]
struct AlignedActivationArena {
    tables: [[u64; aesynx_mm::PAGE_TABLE_ENTRIES]; ACTIVATION_TABLES],
}

impl AlignedActivationArena {
    const ZERO: Self = Self {
        tables: [[0; aesynx_mm::PAGE_TABLE_ENTRIES]; ACTIVATION_TABLES],
    };
}

static mut ACTIVATION_ARENA: AlignedActivationArena = AlignedActivationArena::ZERO;

#[repr(C, align(4096))]
struct AlignedActivationStack {
    bytes: [u8; ACTIVATION_STACK_BYTES],
}

impl AlignedActivationStack {
    const ZERO: Self = Self {
        bytes: [0; ACTIVATION_STACK_BYTES],
    };
}

#[unsafe(link_section = ".aesynx_activation_stack")]
#[used]
static mut ACTIVATION_STACK: AlignedActivationStack = AlignedActivationStack::ZERO;
