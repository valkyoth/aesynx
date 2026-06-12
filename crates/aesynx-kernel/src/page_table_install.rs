use core::sync::atomic::{Ordering, compiler_fence};

pub const ACTIVATION_TABLES: usize = aesynx_mm::PAGE_TABLE_LEVELS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageTableInstallStatus {
    pub tables_copied: u64,
    pub entries_copied: u64,
    pub root_copied: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PageTableInstallError {
    ActiveCr3Overlap,
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
