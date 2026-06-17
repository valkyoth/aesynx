use super::host_tools::{HostToolVersions, MIN_LIMINE_VERSION_TEXT};
use super::smoke::{
    AI_POLICY_HEURISTIC_CORE_MARKER, AI_POLICY_HEURISTIC_DISABLED_MARKER,
    AI_POLICY_HEURISTIC_ENABLED_MARKER, AI_POLICY_HEURISTIC_SCORE_MARKER, AI_POLICY_MARKER,
    AI_POLICY_METADATA_GATE_MARKER, AI_POLICY_STATUS_MARKER, AMP_CORE_BARRIER_MARKER,
    AMP_CORE_BOOTSTRAP_ROLE_MARKER, AMP_CORE_CAPABILITIES_MARKER, AMP_CORE_MARKER,
    AMP_CORE_REGISTRY_MARKER, AMP_CORE_STATUS_MARKER, AMP_CORE_TELEMETRY_MARKER,
    BOOT_DIAGNOSTIC_MARKER, BOOTINFO_MARKER, CAP_AUDIT_MARKER, CAP_AUDIT_STATUS_MARKER,
    CAP_TABLE_MARKER, CAP_TABLE_STATUS_MARKER, CONCURRENCY_IRQ_LOCK_MARKER, CONCURRENCY_IRQ_MARKER,
    CONCURRENCY_LOCK_MARKER, CONCURRENCY_MARKER, CONCURRENCY_NESTED_IRQ_MARKER,
    CONCURRENCY_ORDER_MARKER, CONCURRENCY_STATUS_MARKER, COOPERATIVE_SCHED_MARKER,
    COOPERATIVE_SCHED_STATUS_MARKER, CPU_HARDENING_MARKER, CPU_HARDENING_STATUS_MARKER,
    CPU_SETUP_MARKER, ENTROPY_POLICY_FALLBACK_MARKER, ENTROPY_POLICY_GENERATION_MARKER,
    ENTROPY_POLICY_HARDWARE_MARKER, ENTROPY_POLICY_MARKER, ENTROPY_POLICY_RANDOM_TOKEN_MARKER,
    ENTROPY_POLICY_SELF_TEST_MARKER, ENTROPY_POLICY_SOURCE_MARKER, ENTROPY_POLICY_STATUS_MARKER,
    EXCEPTION_MARKER, EXCEPTION_SETUP_MARKER, FAULT_ADDRESS_MARKER, FAULT_ADDRESS_PRESENT_MARKER,
    FAULT_CR3_MARKER, FAULT_ERROR_DECODE_MARKER, FAULT_INTERRUPTS_MARKER, FAULT_RFLAGS_MARKER,
    FRAME_ALLOCATOR_MARKER, FRAME_ALLOCATOR_STATUS_MARKER, HEAP_CORRUPT_FREE_LIST_MARKER,
    HEAP_DOUBLE_FREE_MARKER, HEAP_INVALID_FREE_MARKER, HEAP_MARKER, HEAP_PAGE_RUN_MARKER,
    HEAP_SLAB_CLASSES_MARKER, HEAP_SLAB_REUSE_MARKER, HEAP_STATUS_MARKER, HEAP_STRESS_MARKER,
    IRQ_SETUP_MARKER, KERNEL_CR3_ACTIVE_MARKER, KERNEL_CR3_MARKER, KERNEL_STACK_GUARD_MARKER,
    KERNEL_STACK_GUARD_STATUS_MARKER, KERNEL_STACK_PAGES_MARKER, MEMORY_CAP_MARKER,
    MEMORY_CAP_STATUS_MARKER, MEMORY_MAP_MARKER, MEMORY_RESERVED_MARKER, MEMORY_TOTAL_MARKER,
    MEMORY_USABLE_MARKER, PAGE_FAULT_MARKER, PAGE_TABLE_AUDIT_MARKER,
    PAGE_TABLE_CHECKED_ROOT_MARKER, PAGE_TABLE_CHECKED_STATUS_MARKER,
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
    PAGING_POLICY_MODEL_GUARD_PAGE_MARKER, PAGING_POLICY_MODEL_HARDWARE_ARENA_MARKER,
    PAGING_POLICY_MODEL_HARDWARE_COPIED_MARKER, PAGING_POLICY_MODEL_HARDWARE_IMAGE_MARKER,
    PAGING_POLICY_MODEL_HARDWARE_ROOT_MARKER, PAGING_POLICY_MODEL_HARDWARE_TABLES_MARKER,
    PAGING_POLICY_MODEL_HEAP_RESERVED_MARKER, PAGING_POLICY_MODEL_MARKER,
    PAGING_POLICY_MODEL_NULL_PAGE_MARKER, PAGING_POLICY_MODEL_RODATA_READ_ONLY_MARKER,
    PAGING_POLICY_MODEL_SECTION_LAYOUT_MARKER, PAGING_POLICY_MODEL_STATUS_MARKER,
    PAGING_POLICY_MODEL_TEXT_RX_MARKER, PANIC_DIAGNOSTIC_MARKER, PANIC_MARKER,
    PANIC_REGISTERS_MARKER, SCHEDULER_TELEMETRY_MARKER, SCHEDULER_TELEMETRY_STATUS_MARKER,
    SERIAL_MARKER, SERVICE_QUEUE_MARKER, SERVICE_QUEUE_STATUS_MARKER, SLEEP_MARKER, SmokeKind,
    TASK_MODEL_MARKER, TASK_MODEL_STATUS_MARKER, TELEMETRY_EVENTS_MARKER,
    TELEMETRY_EVENTS_SCHEMA_MARKER, TELEMETRY_EVENTS_STATUS_MARKER, TIMER_DELAYED_LOG_MARKER,
    TIMER_MARKER, TIMER_SETUP_MARKER, TIMER_TICK_1_MARKER, TIMER_TICK_2_MARKER,
    TIMER_TICK_3_MARKER, TRACE_EVENT_BOOT_MARKER, TRACE_EVENT_CAPABILITY_MARKER,
    TRACE_EVENT_SCHEDULER_MARKER, TRACE_EVENT_TASK_REDACTED_MARKER,
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
        "name=Aesynx v0.34.0 AMP core data structures candidate\nsmoke={}\nimage={}\nformat=iso\nbootloader=limine\nkernel={}\nkernel_target={KERNEL_TARGET}\nkernel_profile={KERNEL_PROFILE}\ncpu_setup_marker={CPU_SETUP_MARKER}\nexception_setup_marker={EXCEPTION_SETUP_MARKER}\nirq_setup_marker={IRQ_SETUP_MARKER}\nexception_marker={EXCEPTION_MARKER}\nboot_diagnostic_marker={BOOT_DIAGNOSTIC_MARKER}\npage_fault_marker={PAGE_FAULT_MARKER}\nfault_address_present_marker={FAULT_ADDRESS_PRESENT_MARKER}\nfault_address_marker={FAULT_ADDRESS_MARKER}\nfault_cr3_marker={FAULT_CR3_MARKER}\nfault_rflags_marker={FAULT_RFLAGS_MARKER}\nfault_interrupts_marker={FAULT_INTERRUPTS_MARKER}\nfault_error_decode_marker={FAULT_ERROR_DECODE_MARKER}\nmemory_total_marker={MEMORY_TOTAL_MARKER}\nmemory_usable_marker={MEMORY_USABLE_MARKER}\nmemory_reserved_marker={MEMORY_RESERVED_MARKER}\nmemory_map_marker={MEMORY_MAP_MARKER}\nframe_allocator_status_marker={FRAME_ALLOCATOR_STATUS_MARKER}\nframe_allocator_marker={FRAME_ALLOCATOR_MARKER}\npage_table_status_marker={PAGE_TABLE_STATUS_MARKER}\npage_table_root_marker={PAGE_TABLE_ROOT_MARKER}\npage_table_checked_root_marker={PAGE_TABLE_CHECKED_ROOT_MARKER}\npage_table_checked_status_marker={PAGE_TABLE_CHECKED_STATUS_MARKER}\npage_table_kernel_candidate_marker={PAGE_TABLE_KERNEL_CANDIDATE_MARKER}\npage_table_user_candidate_marker={PAGE_TABLE_USER_CANDIDATE_MARKER}\npage_table_translate_offset_marker={PAGE_TABLE_TRANSLATE_OFFSET_MARKER}\npage_table_checked_translate_marker={PAGE_TABLE_CHECKED_TRANSLATE_MARKER}\npage_table_lookup_marker={PAGE_TABLE_LOOKUP_MARKER}\npage_table_presence_marker={PAGE_TABLE_PRESENCE_MARKER}\npage_table_protect_marker={PAGE_TABLE_PROTECT_MARKER}\npage_table_protect_range_marker={PAGE_TABLE_PROTECT_RANGE_MARKER}\npage_table_range_lookup_marker={PAGE_TABLE_RANGE_LOOKUP_MARKER}\npage_table_range_translate_marker={PAGE_TABLE_RANGE_TRANSLATE_MARKER}\npage_table_mapped_range_marker={PAGE_TABLE_MAPPED_RANGE_MARKER}\npage_table_unmapped_range_marker={PAGE_TABLE_UNMAPPED_RANGE_MARKER}\npage_table_kernel_range_marker={PAGE_TABLE_KERNEL_RANGE_MARKER}\npage_table_user_range_marker={PAGE_TABLE_USER_RANGE_MARKER}\npage_table_write_protected_range_marker={PAGE_TABLE_WRITE_PROTECTED_RANGE_MARKER}\npage_table_non_executable_range_marker={PAGE_TABLE_NON_EXECUTABLE_RANGE_MARKER}\npage_table_executable_range_marker={PAGE_TABLE_EXECUTABLE_RANGE_MARKER}\npage_table_normal_memory_range_marker={PAGE_TABLE_NORMAL_MEMORY_RANGE_MARKER}\npage_table_local_range_marker={PAGE_TABLE_LOCAL_RANGE_MARKER}\npage_table_kernel_space_range_marker={PAGE_TABLE_KERNEL_SPACE_RANGE_MARKER}\npage_table_user_space_range_marker={PAGE_TABLE_USER_SPACE_RANGE_MARKER}\npage_table_no_user_space_marker={PAGE_TABLE_NO_USER_SPACE_MARKER}\npage_table_no_executable_marker={PAGE_TABLE_NO_EXECUTABLE_MARKER}\npage_table_no_writable_marker={PAGE_TABLE_NO_WRITABLE_MARKER}\npage_table_no_device_marker={PAGE_TABLE_NO_DEVICE_MARKER}\npage_table_no_global_marker={PAGE_TABLE_NO_GLOBAL_MARKER}\npage_table_no_alias_marker={PAGE_TABLE_NO_ALIAS_MARKER}\npage_table_kernel_user_guard_marker={PAGE_TABLE_KERNEL_USER_GUARD_MARKER}\npage_table_kernel_only_marker={PAGE_TABLE_KERNEL_ONLY_MARKER}\npage_table_audit_marker={PAGE_TABLE_AUDIT_MARKER}\npage_table_visit_marker={PAGE_TABLE_VISIT_MARKER}\npage_table_flags_marker={PAGE_TABLE_FLAGS_MARKER}\npage_table_reclaim_marker={PAGE_TABLE_RECLAIM_MARKER}\npage_table_range_marker={PAGE_TABLE_RANGE_MARKER}\npage_table_flush_page_marker={PAGE_TABLE_FLUSH_PAGE_MARKER}\npage_table_marker={PAGE_TABLE_MARKER}\npaging_policy_model_status_marker={PAGING_POLICY_MODEL_STATUS_MARKER}\npaging_policy_model_section_layout_marker={PAGING_POLICY_MODEL_SECTION_LAYOUT_MARKER}\npaging_policy_model_text_rx_marker={PAGING_POLICY_MODEL_TEXT_RX_MARKER}\npaging_policy_model_rodata_read_only_marker={PAGING_POLICY_MODEL_RODATA_READ_ONLY_MARKER}\npaging_policy_model_data_rw_nx_marker={PAGING_POLICY_MODEL_DATA_RW_NX_MARKER}\npaging_policy_model_heap_reserved_marker={PAGING_POLICY_MODEL_HEAP_RESERVED_MARKER}\npaging_policy_model_guard_page_marker={PAGING_POLICY_MODEL_GUARD_PAGE_MARKER}\npaging_policy_model_null_page_marker={PAGING_POLICY_MODEL_NULL_PAGE_MARKER}\npaging_policy_model_hardware_image_marker={PAGING_POLICY_MODEL_HARDWARE_IMAGE_MARKER}\npaging_policy_model_hardware_arena_marker={PAGING_POLICY_MODEL_HARDWARE_ARENA_MARKER}\npaging_policy_model_hardware_root_marker={PAGING_POLICY_MODEL_HARDWARE_ROOT_MARKER}\npaging_policy_model_hardware_tables_marker={PAGING_POLICY_MODEL_HARDWARE_TABLES_MARKER}\npaging_policy_model_hardware_copied_marker={PAGING_POLICY_MODEL_HARDWARE_COPIED_MARKER}\npaging_policy_model_marker={PAGING_POLICY_MODEL_MARKER}\nbootinfo_marker={BOOTINFO_MARKER}\nserial_marker={SERIAL_MARKER}\ncpu_hardening_status_marker={CPU_HARDENING_STATUS_MARKER}\ncpu_hardening_marker={CPU_HARDENING_MARKER}\nentropy_policy_status_marker={ENTROPY_POLICY_STATUS_MARKER}\nentropy_policy_hardware_marker={ENTROPY_POLICY_HARDWARE_MARKER}\nentropy_policy_self_test_marker={ENTROPY_POLICY_SELF_TEST_MARKER}\nentropy_policy_fallback_marker={ENTROPY_POLICY_FALLBACK_MARKER}\nentropy_policy_generation_marker={ENTROPY_POLICY_GENERATION_MARKER}\nentropy_policy_random_token_marker={ENTROPY_POLICY_RANDOM_TOKEN_MARKER}\nentropy_policy_source_marker={ENTROPY_POLICY_SOURCE_MARKER}\nentropy_policy_marker={ENTROPY_POLICY_MARKER}\nheap_status_marker={HEAP_STATUS_MARKER}\nheap_slab_classes_marker={HEAP_SLAB_CLASSES_MARKER}\nheap_slab_reuse_marker={HEAP_SLAB_REUSE_MARKER}\nheap_page_run_marker={HEAP_PAGE_RUN_MARKER}\nheap_stress_marker={HEAP_STRESS_MARKER}\nheap_double_free_marker={HEAP_DOUBLE_FREE_MARKER}\nheap_invalid_free_marker={HEAP_INVALID_FREE_MARKER}\nheap_corrupt_free_list_marker={HEAP_CORRUPT_FREE_LIST_MARKER}\nheap_marker={HEAP_MARKER}\ncap_table_status_marker={CAP_TABLE_STATUS_MARKER}\ncap_table_marker={CAP_TABLE_MARKER}\nmemory_cap_status_marker={MEMORY_CAP_STATUS_MARKER}\nmemory_cap_marker={MEMORY_CAP_MARKER}\ncap_audit_status_marker={CAP_AUDIT_STATUS_MARKER}\ncap_audit_marker={CAP_AUDIT_MARKER}\nservice_queue_status_marker={SERVICE_QUEUE_STATUS_MARKER}\nservice_queue_marker={SERVICE_QUEUE_MARKER}\ntask_model_status_marker={TASK_MODEL_STATUS_MARKER}\ntask_model_marker={TASK_MODEL_MARKER}\ncooperative_sched_status_marker={COOPERATIVE_SCHED_STATUS_MARKER}\ncooperative_sched_marker={COOPERATIVE_SCHED_MARKER}\nscheduler_telemetry_status_marker={SCHEDULER_TELEMETRY_STATUS_MARKER}\nscheduler_telemetry_marker={SCHEDULER_TELEMETRY_MARKER}\ntelemetry_events_schema_marker={TELEMETRY_EVENTS_SCHEMA_MARKER}\ntelemetry_events_status_marker={TELEMETRY_EVENTS_STATUS_MARKER}\ntrace_event_boot_marker={TRACE_EVENT_BOOT_MARKER}\ntrace_event_capability_marker={TRACE_EVENT_CAPABILITY_MARKER}\ntrace_event_scheduler_marker={TRACE_EVENT_SCHEDULER_MARKER}\ntrace_event_task_redacted_marker={TRACE_EVENT_TASK_REDACTED_MARKER}\ntelemetry_events_marker={TELEMETRY_EVENTS_MARKER}\nai_policy_status_marker={AI_POLICY_STATUS_MARKER}\nai_policy_metadata_gate_marker={AI_POLICY_METADATA_GATE_MARKER}\nai_policy_heuristic_enabled_marker={AI_POLICY_HEURISTIC_ENABLED_MARKER}\nai_policy_heuristic_score_marker={AI_POLICY_HEURISTIC_SCORE_MARKER}\nai_policy_heuristic_core_marker={AI_POLICY_HEURISTIC_CORE_MARKER}\nai_policy_heuristic_disabled_marker={AI_POLICY_HEURISTIC_DISABLED_MARKER}\nai_policy_marker={AI_POLICY_MARKER}\nconcurrency_status_marker={CONCURRENCY_STATUS_MARKER}\nconcurrency_irq_marker={CONCURRENCY_IRQ_MARKER}\nconcurrency_nested_irq_marker={CONCURRENCY_NESTED_IRQ_MARKER}\nconcurrency_lock_marker={CONCURRENCY_LOCK_MARKER}\nconcurrency_irq_lock_marker={CONCURRENCY_IRQ_LOCK_MARKER}\nconcurrency_order_marker={CONCURRENCY_ORDER_MARKER}\nconcurrency_marker={CONCURRENCY_MARKER}\namp_core_status_marker={AMP_CORE_STATUS_MARKER}\namp_core_bootstrap_role_marker={AMP_CORE_BOOTSTRAP_ROLE_MARKER}\namp_core_capabilities_marker={AMP_CORE_CAPABILITIES_MARKER}\namp_core_registry_marker={AMP_CORE_REGISTRY_MARKER}\namp_core_telemetry_marker={AMP_CORE_TELEMETRY_MARKER}\namp_core_barrier_marker={AMP_CORE_BARRIER_MARKER}\namp_core_marker={AMP_CORE_MARKER}\nkernel_stack_pages_marker={KERNEL_STACK_PAGES_MARKER}\nkernel_stack_guard_status_marker={KERNEL_STACK_GUARD_STATUS_MARKER}\nkernel_stack_guard_marker={KERNEL_STACK_GUARD_MARKER}\nkernel_cr3_active_marker={KERNEL_CR3_ACTIVE_MARKER}\nkernel_cr3_marker={KERNEL_CR3_MARKER}\npanic_diagnostic_marker={PANIC_DIAGNOSTIC_MARKER}\npanic_registers_marker={PANIC_REGISTERS_MARKER}\npanic_marker={PANIC_MARKER}\ntimer_setup_marker={TIMER_SETUP_MARKER}\ntimer_tick_1_marker={TIMER_TICK_1_MARKER}\ntimer_tick_2_marker={TIMER_TICK_2_MARKER}\ntimer_tick_3_marker={TIMER_TICK_3_MARKER}\ntimer_delayed_log_marker={TIMER_DELAYED_LOG_MARKER}\nsleep_marker={SLEEP_MARKER}\ntimer_marker={TIMER_MARKER}\nrustc_version={}\ncargo_version={}\nlimine_version={}\nlimine_min_version={}\nxorriso_version={}\nqemu_version={}\n",
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
