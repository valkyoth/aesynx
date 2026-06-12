use alloc::format;

use aesynx_abi::PhysAddr;

use crate::page_table::{PAGE_TABLE_ENTRIES, PageTableError, PageTableMapper};
use crate::{GenericPageFlags, PageAccess};

use super::{KERNEL_PHYS, KERNEL_VIRT};

const ROOT_PHYS: PhysAddr = PhysAddr::new(0x0100_0000);
const PRESENT: u64 = 1 << 0;
const WRITABLE: u64 = 1 << 1;
const USER: u64 = 1 << 2;
const SOFTWARE_NEXT_TABLE: u64 = 1 << 9;
const ADDRESS_MASK: u64 = 0x000f_ffff_ffff_f000;

#[test]
fn hardware_image_reencodes_model_next_slots_as_physical_entries() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadWrite),
    )?;

    let image = mapper.export_x86_64_hardware_image(ROOT_PHYS)?;
    let mut root = [0u64; PAGE_TABLE_ENTRIES];
    image.copy_table_entries(0, &mut root)?;
    let next = root[((KERNEL_VIRT.get() >> 39) & 0x1ff) as usize];

    assert_eq!(image.root_phys(), ROOT_PHYS);
    assert_eq!(image.mapped_pages(), 1);
    assert_eq!(image.table_count(), 8);
    assert_eq!(image.used_tables(), 4);
    assert!(image.table_used(0));
    assert!(image.table_used(1));
    assert!(image.table_used(2));
    assert!(image.table_used(3));
    assert!(!image.table_used(4));
    assert_eq!(next & PRESENT, PRESENT);
    assert_eq!(next & WRITABLE, WRITABLE);
    assert_eq!(next & USER, 0);
    assert_eq!(next & SOFTWARE_NEXT_TABLE, 0);
    assert_eq!(next & ADDRESS_MASK, ROOT_PHYS.get() + crate::FRAME_SIZE);
    Ok(())
}

#[test]
fn hardware_image_sets_user_permission_on_user_subtrees() -> Result<(), PageTableError> {
    let user_virt = aesynx_abi::VirtAddr::new(0x0000_0000_4000_0000);
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_page(
        user_virt,
        KERNEL_PHYS,
        GenericPageFlags::user(PageAccess::ReadOnly),
    )?;

    let image = mapper.export_x86_64_hardware_image(ROOT_PHYS)?;
    let mut root = [0u64; PAGE_TABLE_ENTRIES];
    image.copy_table_entries(0, &mut root)?;
    let next = root[((user_virt.get() >> 39) & 0x1ff) as usize];

    assert_eq!(next & USER, USER);
    assert_eq!(next & WRITABLE, 0);
    Ok(())
}

#[test]
fn hardware_table_streaming_export_matches_image_table() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadWrite),
    )?;

    let image = mapper.export_x86_64_hardware_image(ROOT_PHYS)?;
    let mut image_root = [0u64; PAGE_TABLE_ENTRIES];
    let mut streamed_root = [0u64; PAGE_TABLE_ENTRIES];
    let mut unused = [1u64; PAGE_TABLE_ENTRIES];

    image.copy_table_entries(0, &mut image_root)?;
    assert!(mapper.export_x86_64_hardware_table_entries(ROOT_PHYS, 0, &mut streamed_root)?);
    assert!(!mapper.export_x86_64_hardware_table_entries(ROOT_PHYS, 4, &mut unused)?);

    assert_eq!(streamed_root, image_root);
    assert_eq!(unused, [0u64; PAGE_TABLE_ENTRIES]);
    Ok(())
}

#[test]
fn hardware_image_rejects_unaligned_or_overflowing_table_arena() -> Result<(), PageTableError> {
    let mapper = PageTableMapper::<2>::new()?;

    assert_eq!(
        mapper.export_x86_64_hardware_image(PhysAddr::new(ROOT_PHYS.get() + 1)),
        Err(PageTableError::UnalignedPhysicalAddress)
    );
    assert_eq!(
        mapper.export_x86_64_hardware_image(PhysAddr::new(0x000f_ffff_ffff_f000)),
        Err(PageTableError::InvalidPhysicalAddress)
    );
    Ok(())
}

#[test]
fn hardware_image_debug_redacts_physical_addresses_and_entries() -> Result<(), PageTableError> {
    let mut mapper = PageTableMapper::<8>::new()?;
    mapper.map_page(
        KERNEL_VIRT,
        KERNEL_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )?;
    let debug = format!("{:?}", mapper.export_x86_64_hardware_image(ROOT_PHYS)?);

    assert!(debug.contains("X86_64PageTableImage"));
    assert!(debug.contains("root_phys: \"<redacted>\""));
    assert!(!debug.contains("16777216"));
    assert!(!debug.contains("entries"));
    Ok(())
}
