use crate::{GenericPageFlags, PageAccess, PagePrivilege};

use super::KERNEL_PHYS;
use crate::page_table::{PageMapping, PageTableError, PageTableSlot, X86_64PageTableEntry};

#[test]
fn x86_64_entry_encodes_safe_mapping_flags() -> Result<(), PageTableError> {
    let flags = GenericPageFlags::kernel(PageAccess::ReadExecute)
        .with_global()
        .map_err(|_error| PageTableError::CorruptTable)?;
    let entry = X86_64PageTableEntry::from_mapping(PageMapping::new(KERNEL_PHYS, flags))?;

    assert_eq!(entry.raw() & 1, 1);
    assert_eq!(entry.raw() & (1 << 1), 0);
    assert_eq!(entry.raw() & (1 << 2), 0);
    assert_eq!(entry.raw() & (1 << 8), 1 << 8);
    assert_eq!(entry.raw() & (1 << 63), 0);
    Ok(())
}

#[test]
fn x86_64_entry_encodes_user_nx_device_mapping() -> Result<(), PageTableError> {
    let flags = GenericPageFlags::user(PageAccess::ReadOnly).device();
    let entry = X86_64PageTableEntry::from_mapping(PageMapping::new(KERNEL_PHYS, flags))?;

    assert_eq!(flags.privilege, PagePrivilege::User);
    assert_eq!(entry.raw() & (1 << 2), 1 << 2);
    assert_eq!(entry.raw() & (1 << 3), 1 << 3);
    assert_eq!(entry.raw() & (1 << 4), 1 << 4);
    assert_eq!(entry.raw() & (1 << 63), 1 << 63);
    Ok(())
}

#[test]
fn x86_64_entry_rejects_executable_device_mapping() {
    let mut flags = GenericPageFlags::kernel(PageAccess::ReadExecute);
    flags.device_memory = true;
    flags.cacheable = false;

    assert_eq!(
        X86_64PageTableEntry::from_mapping(PageMapping::new(KERNEL_PHYS, flags)),
        Err(PageTableError::InvalidMappingFlags)
    );
}

#[test]
fn x86_64_entry_rejects_user_global_mapping() {
    let mut flags = GenericPageFlags::user(PageAccess::ReadOnly);
    flags.global = true;

    assert_eq!(
        X86_64PageTableEntry::from_mapping(PageMapping::new(KERNEL_PHYS, flags)),
        Err(PageTableError::InvalidMappingFlags)
    );
}

#[test]
fn raw_slot_decode_rejects_write_execute_corruption() {
    let slot = PageTableSlot {
        raw: KERNEL_PHYS.get() | 1 | (1 << 1),
    };

    assert_eq!(slot.mapping(), Err(PageTableError::CorruptTable));
}

#[test]
fn raw_slot_decode_rejects_user_global_corruption() {
    let slot = PageTableSlot {
        raw: KERNEL_PHYS.get() | 1 | (1 << 2) | (1 << 8) | (1 << 63),
    };

    assert_eq!(slot.mapping(), Err(PageTableError::CorruptTable));
}

#[test]
fn raw_slot_decode_rejects_executable_device_corruption() {
    let slot = PageTableSlot {
        raw: KERNEL_PHYS.get() | 1 | (1 << 4),
    };

    assert_eq!(slot.mapping(), Err(PageTableError::CorruptTable));
}

#[test]
fn raw_slot_decode_rejects_partial_cache_policy_corruption() {
    let write_through_only = PageTableSlot {
        raw: KERNEL_PHYS.get() | 1 | (1 << 3) | (1 << 63),
    };
    let cache_disable_only = PageTableSlot {
        raw: KERNEL_PHYS.get() | 1 | (1 << 4) | (1 << 63),
    };

    assert_eq!(
        write_through_only.mapping(),
        Err(PageTableError::CorruptTable)
    );
    assert_eq!(
        cache_disable_only.mapping(),
        Err(PageTableError::CorruptTable)
    );
}

#[test]
fn raw_slot_decode_rejects_unknown_leaf_bits() {
    let slot = PageTableSlot {
        raw: KERNEL_PHYS.get() | 1 | (1 << 10) | (1 << 63),
    };

    assert_eq!(slot.mapping(), Err(PageTableError::CorruptTable));
}

#[test]
fn raw_slot_decode_rejects_nonpresent_nonempty_slot() {
    let slot = PageTableSlot {
        raw: KERNEL_PHYS.get(),
    };

    assert_eq!(slot.mapping(), Err(PageTableError::CorruptTable));
}

#[test]
fn next_table_slot_rejects_missing_present_bit() {
    let slot = PageTableSlot { raw: 1 << 9 };

    assert_eq!(slot.next_index(), Err(PageTableError::CorruptTable));
    assert_eq!(slot.mapping(), Err(PageTableError::CorruptTable));
}

#[test]
fn next_table_slot_rejects_leaf_flag_corruption() {
    let slot = PageTableSlot {
        raw: 1 | (1 << 1) | (1 << 9) | 0x1000,
    };

    assert_eq!(slot.next_index(), Err(PageTableError::CorruptTable));
    assert_eq!(slot.mapping(), Err(PageTableError::CorruptTable));
}

#[test]
fn raw_slot_decode_ignores_empty_and_next_slots() -> Result<(), PageTableError> {
    assert_eq!(PageTableSlot::EMPTY.mapping(), Ok(None));
    assert_eq!(PageTableSlot::next(1)?.mapping(), Ok(None));
    Ok(())
}

#[test]
fn next_table_slot_rejects_unencodable_index() {
    assert_eq!(
        PageTableSlot::next(usize::MAX),
        Err(PageTableError::AddressOverflow)
    );
}

#[test]
fn next_table_slot_rejects_root_table_index() {
    assert_eq!(PageTableSlot::next(0), Err(PageTableError::CorruptTable));
}

#[test]
fn raw_next_table_slot_rejects_root_table_index() {
    let slot = PageTableSlot { raw: 1 | (1 << 9) };

    assert_eq!(slot.next_index(), Err(PageTableError::CorruptTable));
    assert_eq!(slot.mapping(), Err(PageTableError::CorruptTable));
}
