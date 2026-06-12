use super::host_tools::{HostToolVersions, MIN_LIMINE_VERSION_TEXT};
use super::smoke::{
    BOOT_DIAGNOSTIC_MARKER, BOOTINFO_MARKER, CPU_SETUP_MARKER, EXCEPTION_MARKER,
    EXCEPTION_SETUP_MARKER, FAULT_ADDRESS_MARKER, FAULT_ADDRESS_PRESENT_MARKER, FAULT_CR3_MARKER,
    FAULT_ERROR_DECODE_MARKER, FAULT_INTERRUPTS_MARKER, FAULT_RFLAGS_MARKER,
    FRAME_ALLOCATOR_MARKER, FRAME_ALLOCATOR_STATUS_MARKER, IRQ_SETUP_MARKER, MEMORY_MAP_MARKER,
    MEMORY_RESERVED_MARKER, MEMORY_TOTAL_MARKER, MEMORY_USABLE_MARKER, PAGE_FAULT_MARKER,
    PAGE_TABLE_AUDIT_MARKER, PAGE_TABLE_CHECKED_ROOT_MARKER, PAGE_TABLE_CHECKED_STATUS_MARKER,
    PAGE_TABLE_CHECKED_TRANSLATE_MARKER, PAGE_TABLE_EXECUTABLE_RANGE_MARKER,
    PAGE_TABLE_FLAGS_MARKER, PAGE_TABLE_FLUSH_PAGE_MARKER, PAGE_TABLE_KERNEL_CANDIDATE_MARKER,
    PAGE_TABLE_KERNEL_ONLY_MARKER, PAGE_TABLE_KERNEL_RANGE_MARKER,
    PAGE_TABLE_KERNEL_SPACE_RANGE_MARKER, PAGE_TABLE_KERNEL_USER_GUARD_MARKER,
    PAGE_TABLE_LOCAL_RANGE_MARKER, PAGE_TABLE_LOOKUP_MARKER, PAGE_TABLE_MAPPED_RANGE_MARKER,
    PAGE_TABLE_MARKER, PAGE_TABLE_NO_ALIAS_MARKER, PAGE_TABLE_NO_DEVICE_MARKER,
    PAGE_TABLE_NO_EXECUTABLE_MARKER, PAGE_TABLE_NO_GLOBAL_MARKER, PAGE_TABLE_NO_USER_SPACE_MARKER,
    PAGE_TABLE_NO_WRITABLE_MARKER, PAGE_TABLE_NON_EXECUTABLE_RANGE_MARKER,
    PAGE_TABLE_NORMAL_MEMORY_RANGE_MARKER, PAGE_TABLE_PRESENCE_MARKER, PAGE_TABLE_PROTECT_MARKER,
    PAGE_TABLE_PROTECT_RANGE_MARKER, PAGE_TABLE_RANGE_LOOKUP_MARKER, PAGE_TABLE_RANGE_MARKER,
    PAGE_TABLE_RANGE_TRANSLATE_MARKER, PAGE_TABLE_RECLAIM_MARKER, PAGE_TABLE_ROOT_MARKER,
    PAGE_TABLE_STATUS_MARKER, PAGE_TABLE_TRANSLATE_OFFSET_MARKER, PAGE_TABLE_UNMAPPED_RANGE_MARKER,
    PAGE_TABLE_USER_CANDIDATE_MARKER, PAGE_TABLE_USER_RANGE_MARKER,
    PAGE_TABLE_USER_SPACE_RANGE_MARKER, PAGE_TABLE_VISIT_MARKER,
    PAGE_TABLE_WRITE_PROTECTED_RANGE_MARKER, PAGING_POLICY_MODEL_DATA_RW_NX_MARKER,
    PAGING_POLICY_MODEL_GUARD_PAGE_MARKER, PAGING_POLICY_MODEL_HEAP_RESERVED_MARKER,
    PAGING_POLICY_MODEL_MARKER, PAGING_POLICY_MODEL_NULL_PAGE_MARKER,
    PAGING_POLICY_MODEL_RODATA_READ_ONLY_MARKER, PAGING_POLICY_MODEL_SECTION_LAYOUT_MARKER,
    PAGING_POLICY_MODEL_STATUS_MARKER, PAGING_POLICY_MODEL_TEXT_RX_MARKER, PANIC_DIAGNOSTIC_MARKER,
    PANIC_MARKER, PANIC_REGISTERS_MARKER, SERIAL_MARKER, SLEEP_MARKER, SmokeKind,
    TIMER_DELAYED_LOG_MARKER, TIMER_MARKER, TIMER_SETUP_MARKER, TIMER_TICK_1_MARKER,
    TIMER_TICK_2_MARKER, TIMER_TICK_3_MARKER,
};
use super::{KERNEL_PROFILE, KERNEL_TARGET};

use std::fs;
use std::path::Path;

pub(super) fn write_manifest(
    manifest: &Path,
    image: &Path,
    kernel_elf: &Path,
    host_tools: &HostToolVersions,
    smoke: SmokeKind,
) -> Result<(), String> {
    let manifest_contents = format!(
        "name=Aesynx v0.16.0 kernel mapping policy\nsmoke={}\nimage={}\nformat=iso\nbootloader=limine\nkernel={}\nkernel_target={KERNEL_TARGET}\nkernel_profile={KERNEL_PROFILE}\ncpu_setup_marker={CPU_SETUP_MARKER}\nexception_setup_marker={EXCEPTION_SETUP_MARKER}\nirq_setup_marker={IRQ_SETUP_MARKER}\nexception_marker={EXCEPTION_MARKER}\nboot_diagnostic_marker={BOOT_DIAGNOSTIC_MARKER}\npage_fault_marker={PAGE_FAULT_MARKER}\nfault_address_present_marker={FAULT_ADDRESS_PRESENT_MARKER}\nfault_address_marker={FAULT_ADDRESS_MARKER}\nfault_cr3_marker={FAULT_CR3_MARKER}\nfault_rflags_marker={FAULT_RFLAGS_MARKER}\nfault_interrupts_marker={FAULT_INTERRUPTS_MARKER}\nfault_error_decode_marker={FAULT_ERROR_DECODE_MARKER}\nmemory_total_marker={MEMORY_TOTAL_MARKER}\nmemory_usable_marker={MEMORY_USABLE_MARKER}\nmemory_reserved_marker={MEMORY_RESERVED_MARKER}\nmemory_map_marker={MEMORY_MAP_MARKER}\nframe_allocator_status_marker={FRAME_ALLOCATOR_STATUS_MARKER}\nframe_allocator_marker={FRAME_ALLOCATOR_MARKER}\npage_table_status_marker={PAGE_TABLE_STATUS_MARKER}\npage_table_root_marker={PAGE_TABLE_ROOT_MARKER}\npage_table_checked_root_marker={PAGE_TABLE_CHECKED_ROOT_MARKER}\npage_table_checked_status_marker={PAGE_TABLE_CHECKED_STATUS_MARKER}\npage_table_kernel_candidate_marker={PAGE_TABLE_KERNEL_CANDIDATE_MARKER}\npage_table_user_candidate_marker={PAGE_TABLE_USER_CANDIDATE_MARKER}\npage_table_translate_offset_marker={PAGE_TABLE_TRANSLATE_OFFSET_MARKER}\npage_table_checked_translate_marker={PAGE_TABLE_CHECKED_TRANSLATE_MARKER}\npage_table_lookup_marker={PAGE_TABLE_LOOKUP_MARKER}\npage_table_presence_marker={PAGE_TABLE_PRESENCE_MARKER}\npage_table_protect_marker={PAGE_TABLE_PROTECT_MARKER}\npage_table_protect_range_marker={PAGE_TABLE_PROTECT_RANGE_MARKER}\npage_table_range_lookup_marker={PAGE_TABLE_RANGE_LOOKUP_MARKER}\npage_table_range_translate_marker={PAGE_TABLE_RANGE_TRANSLATE_MARKER}\npage_table_mapped_range_marker={PAGE_TABLE_MAPPED_RANGE_MARKER}\npage_table_unmapped_range_marker={PAGE_TABLE_UNMAPPED_RANGE_MARKER}\npage_table_kernel_range_marker={PAGE_TABLE_KERNEL_RANGE_MARKER}\npage_table_user_range_marker={PAGE_TABLE_USER_RANGE_MARKER}\npage_table_write_protected_range_marker={PAGE_TABLE_WRITE_PROTECTED_RANGE_MARKER}\npage_table_non_executable_range_marker={PAGE_TABLE_NON_EXECUTABLE_RANGE_MARKER}\npage_table_executable_range_marker={PAGE_TABLE_EXECUTABLE_RANGE_MARKER}\npage_table_normal_memory_range_marker={PAGE_TABLE_NORMAL_MEMORY_RANGE_MARKER}\npage_table_local_range_marker={PAGE_TABLE_LOCAL_RANGE_MARKER}\npage_table_kernel_space_range_marker={PAGE_TABLE_KERNEL_SPACE_RANGE_MARKER}\npage_table_user_space_range_marker={PAGE_TABLE_USER_SPACE_RANGE_MARKER}\npage_table_no_user_space_marker={PAGE_TABLE_NO_USER_SPACE_MARKER}\npage_table_no_executable_marker={PAGE_TABLE_NO_EXECUTABLE_MARKER}\npage_table_no_writable_marker={PAGE_TABLE_NO_WRITABLE_MARKER}\npage_table_no_device_marker={PAGE_TABLE_NO_DEVICE_MARKER}\npage_table_no_global_marker={PAGE_TABLE_NO_GLOBAL_MARKER}\npage_table_no_alias_marker={PAGE_TABLE_NO_ALIAS_MARKER}\npage_table_kernel_user_guard_marker={PAGE_TABLE_KERNEL_USER_GUARD_MARKER}\npage_table_kernel_only_marker={PAGE_TABLE_KERNEL_ONLY_MARKER}\npage_table_audit_marker={PAGE_TABLE_AUDIT_MARKER}\npage_table_visit_marker={PAGE_TABLE_VISIT_MARKER}\npage_table_flags_marker={PAGE_TABLE_FLAGS_MARKER}\npage_table_reclaim_marker={PAGE_TABLE_RECLAIM_MARKER}\npage_table_range_marker={PAGE_TABLE_RANGE_MARKER}\npage_table_flush_page_marker={PAGE_TABLE_FLUSH_PAGE_MARKER}\npage_table_marker={PAGE_TABLE_MARKER}\npaging_policy_model_status_marker={PAGING_POLICY_MODEL_STATUS_MARKER}\npaging_policy_model_section_layout_marker={PAGING_POLICY_MODEL_SECTION_LAYOUT_MARKER}\npaging_policy_model_text_rx_marker={PAGING_POLICY_MODEL_TEXT_RX_MARKER}\npaging_policy_model_rodata_read_only_marker={PAGING_POLICY_MODEL_RODATA_READ_ONLY_MARKER}\npaging_policy_model_data_rw_nx_marker={PAGING_POLICY_MODEL_DATA_RW_NX_MARKER}\npaging_policy_model_heap_reserved_marker={PAGING_POLICY_MODEL_HEAP_RESERVED_MARKER}\npaging_policy_model_guard_page_marker={PAGING_POLICY_MODEL_GUARD_PAGE_MARKER}\npaging_policy_model_null_page_marker={PAGING_POLICY_MODEL_NULL_PAGE_MARKER}\npaging_policy_model_marker={PAGING_POLICY_MODEL_MARKER}\nbootinfo_marker={BOOTINFO_MARKER}\nserial_marker={SERIAL_MARKER}\npanic_diagnostic_marker={PANIC_DIAGNOSTIC_MARKER}\npanic_registers_marker={PANIC_REGISTERS_MARKER}\npanic_marker={PANIC_MARKER}\ntimer_setup_marker={TIMER_SETUP_MARKER}\ntimer_tick_1_marker={TIMER_TICK_1_MARKER}\ntimer_tick_2_marker={TIMER_TICK_2_MARKER}\ntimer_tick_3_marker={TIMER_TICK_3_MARKER}\ntimer_delayed_log_marker={TIMER_DELAYED_LOG_MARKER}\nsleep_marker={SLEEP_MARKER}\ntimer_marker={TIMER_MARKER}\nrustc_version={}\ncargo_version={}\nlimine_version={}\nlimine_min_version={}\nxorriso_version={}\nqemu_version={}\n",
        smoke.name(),
        image.display(),
        kernel_elf.display(),
        host_tools.rustc,
        host_tools.cargo,
        host_tools.limine,
        MIN_LIMINE_VERSION_TEXT,
        host_tools.xorriso,
        host_tools.qemu
    );
    fs::write(manifest, manifest_contents)
        .map_err(|error| format!("failed to write manifest: {error}"))
}
