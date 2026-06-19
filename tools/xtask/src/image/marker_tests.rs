use super::smoke::{
    AI_POLICY_FAIL_MARKER, AI_POLICY_HEURISTIC_CORE_MARKER, AI_POLICY_HEURISTIC_DISABLED_MARKER,
    AI_POLICY_HEURISTIC_ENABLED_MARKER, AI_POLICY_HEURISTIC_SCORE_MARKER, AI_POLICY_MARKER,
    AI_POLICY_METADATA_GATE_MARKER, AI_POLICY_STATUS_MARKER, AMP_CORE_BARRIER_MARKER,
    AMP_CORE_BOOTSTRAP_ROLE_MARKER, AMP_CORE_CAPABILITIES_MARKER, AMP_CORE_FAIL_MARKER,
    AMP_CORE_MARKER, AMP_CORE_REGISTRY_MARKER, AMP_CORE_STATUS_MARKER, AMP_CORE_TELEMETRY_MARKER,
    BOOT_DIAGNOSTIC_MARKER, BOOTINFO_FAIL_MARKER, BOOTINFO_MARKER, CAP_AUDIT_FAIL_MARKER,
    CAP_AUDIT_MARKER, CAP_AUDIT_STATUS_MARKER, CAP_TABLE_FAIL_MARKER, CAP_TABLE_MARKER,
    CAP_TABLE_STATUS_MARKER, CONCURRENCY_FAIL_MARKER, CONCURRENCY_IRQ_LOCK_MARKER,
    CONCURRENCY_IRQ_MARKER, CONCURRENCY_LOCK_MARKER, CONCURRENCY_MARKER,
    CONCURRENCY_NESTED_IRQ_MARKER, CONCURRENCY_ORDER_MARKER, CONCURRENCY_STATUS_MARKER,
    COOPERATIVE_SCHED_FAIL_MARKER, COOPERATIVE_SCHED_MARKER, COOPERATIVE_SCHED_STATUS_MARKER,
    CPU_HARDENING_ARCH_CAPABILITIES_MARKER, CPU_HARDENING_FAIL_MARKER, CPU_HARDENING_IBPB_MARKER,
    CPU_HARDENING_IBRS_MARKER, CPU_HARDENING_MARKER, CPU_HARDENING_SSBD_MARKER,
    CPU_HARDENING_STATUS_MARKER, CPU_HARDENING_STIBP_MARKER, CPU_SETUP_MARKER,
    ENTROPY_POLICY_DRBG_SELF_TEST_MARKER, ENTROPY_POLICY_FAIL_MARKER,
    ENTROPY_POLICY_FALLBACK_MARKER, ENTROPY_POLICY_GENERATION_MARKER,
    ENTROPY_POLICY_HARDWARE_MARKER, ENTROPY_POLICY_MARKER, ENTROPY_POLICY_RANDOM_TOKEN_MARKER,
    ENTROPY_POLICY_SELF_TEST_MARKER, ENTROPY_POLICY_SOURCE_MARKER, ENTROPY_POLICY_STATUS_MARKER,
    EXCEPTION_MARKER, EXCEPTION_SETUP_MARKER, FAULT_ADDRESS_MARKER, FAULT_ADDRESS_PRESENT_MARKER,
    FAULT_CR3_MARKER, FAULT_ERROR_DECODE_MARKER, FAULT_INTERRUPTS_MARKER, FAULT_RFLAGS_MARKER,
    FRAME_ALLOCATOR_FAIL_MARKER, FRAME_ALLOCATOR_MARKER, FRAME_ALLOCATOR_STATUS_MARKER,
    HEAP_ACCOUNTING_OVERFLOW_MARKER, HEAP_CORRUPT_FREE_LIST_MARKER, HEAP_DOUBLE_FREE_MARKER,
    HEAP_FAIL_MARKER, HEAP_INVALID_FREE_MARKER, HEAP_MARKER, HEAP_PAGE_RUN_MARKER,
    HEAP_SLAB_CLASSES_MARKER, HEAP_SLAB_REUSE_MARKER, HEAP_STATUS_MARKER, HEAP_STRESS_MARKER,
    IRQ_SETUP_MARKER, KERNEL_CR3_ACTIVE_MARKER, KERNEL_CR3_FAIL_MARKER, KERNEL_CR3_MARKER,
    KERNEL_STACK_GUARD_MARKER, KERNEL_STACK_GUARD_STATUS_MARKER, KERNEL_STACK_PAGES_MARKER,
    MEMORY_CAP_FAIL_MARKER, MEMORY_CAP_MARKER, MEMORY_CAP_STATUS_MARKER, MEMORY_MAP_FAIL_MARKER,
    MEMORY_MAP_MARKER, MEMORY_RESERVED_MARKER, MEMORY_TOTAL_MARKER, MEMORY_USABLE_MARKER,
    MULTICORE_TOPOLOGY_AP_EXECUTION_BLOCKED_MARKER, MULTICORE_TOPOLOGY_AP_PREFLIGHT_MARKER,
    MULTICORE_TOPOLOGY_BARRIER_MARKER, MULTICORE_TOPOLOGY_BOOTSTRAP_MARKER,
    MULTICORE_TOPOLOGY_DRIVER_SERVICE_MARKER, MULTICORE_TOPOLOGY_FAIL_MARKER,
    MULTICORE_TOPOLOGY_HARDWARE_ONLINE_MARKER, MULTICORE_TOPOLOGY_IDLE_MARKER,
    MULTICORE_TOPOLOGY_MARKER, MULTICORE_TOPOLOGY_QEMU_SMP_MARKER,
    MULTICORE_TOPOLOGY_ROLE_ASSIGNMENT_MARKER, MULTICORE_TOPOLOGY_SCHEDULER_MARKER,
    MULTICORE_TOPOLOGY_STARTUP_EVIDENCE_MARKER, MULTICORE_TOPOLOGY_STATE_TABLE_MARKER,
    MULTICORE_TOPOLOGY_STATUS_MARKER, PAGE_FAULT_MARKER, PAGE_TABLE_AUDIT_MARKER,
    PAGE_TABLE_CHECKED_ROOT_MARKER, PAGE_TABLE_CHECKED_STATUS_MARKER,
    PAGE_TABLE_CHECKED_TRANSLATE_MARKER, PAGE_TABLE_EXECUTABLE_RANGE_MARKER,
    PAGE_TABLE_FAIL_MARKER, PAGE_TABLE_FLAGS_MARKER, PAGE_TABLE_FLUSH_PAGE_MARKER,
    PAGE_TABLE_KERNEL_CANDIDATE_MARKER, PAGE_TABLE_KERNEL_ONLY_MARKER,
    PAGE_TABLE_KERNEL_RANGE_MARKER, PAGE_TABLE_KERNEL_SPACE_RANGE_MARKER,
    PAGE_TABLE_KERNEL_USER_GUARD_MARKER, PAGE_TABLE_LOCAL_RANGE_MARKER, PAGE_TABLE_LOOKUP_MARKER,
    PAGE_TABLE_MAPPED_RANGE_MARKER, PAGE_TABLE_MARKER, PAGE_TABLE_NO_ALIAS_MARKER,
    PAGE_TABLE_NO_DEVICE_MARKER, PAGE_TABLE_NO_EXECUTABLE_MARKER, PAGE_TABLE_NO_GLOBAL_MARKER,
    PAGE_TABLE_NO_USER_SPACE_MARKER, PAGE_TABLE_NO_WRITABLE_MARKER,
    PAGE_TABLE_NON_EXECUTABLE_RANGE_MARKER, PAGE_TABLE_NORMAL_MEMORY_RANGE_MARKER,
    PAGE_TABLE_PRESENCE_MARKER, PAGE_TABLE_PROTECT_MARKER, PAGE_TABLE_PROTECT_RANGE_MARKER,
    PAGE_TABLE_RANGE_LOOKUP_MARKER, PAGE_TABLE_RANGE_MARKER, PAGE_TABLE_RANGE_TRANSLATE_MARKER,
    PAGE_TABLE_RECLAIM_MARKER, PAGE_TABLE_ROOT_MARKER, PAGE_TABLE_STATUS_MARKER,
    PAGE_TABLE_TRANSLATE_OFFSET_MARKER, PAGE_TABLE_UNMAPPED_RANGE_MARKER,
    PAGE_TABLE_USER_CANDIDATE_MARKER, PAGE_TABLE_USER_RANGE_MARKER,
    PAGE_TABLE_USER_SPACE_RANGE_MARKER, PAGE_TABLE_VISIT_MARKER,
    PAGE_TABLE_WRITE_PROTECTED_RANGE_MARKER, PAGING_POLICY_MODEL_DATA_RW_NX_MARKER,
    PAGING_POLICY_MODEL_FAIL_MARKER, PAGING_POLICY_MODEL_GUARD_PAGE_MARKER,
    PAGING_POLICY_MODEL_HARDWARE_ARENA_MARKER, PAGING_POLICY_MODEL_HARDWARE_COPIED_MARKER,
    PAGING_POLICY_MODEL_HARDWARE_IMAGE_MARKER, PAGING_POLICY_MODEL_HARDWARE_ROOT_MARKER,
    PAGING_POLICY_MODEL_HARDWARE_TABLES_MARKER, PAGING_POLICY_MODEL_HEAP_RESERVED_MARKER,
    PAGING_POLICY_MODEL_MARKER, PAGING_POLICY_MODEL_NULL_PAGE_MARKER,
    PAGING_POLICY_MODEL_RODATA_READ_ONLY_MARKER, PAGING_POLICY_MODEL_SECTION_LAYOUT_MARKER,
    PAGING_POLICY_MODEL_STATUS_MARKER, PAGING_POLICY_MODEL_TEXT_RX_MARKER, PANIC_DIAGNOSTIC_MARKER,
    PANIC_MARKER, PANIC_REGISTERS_MARKER, SCHEDULER_TELEMETRY_FAIL_MARKER,
    SCHEDULER_TELEMETRY_MARKER, SCHEDULER_TELEMETRY_STATUS_MARKER, SERIAL_MARKER,
    SERVICE_QUEUE_FAIL_MARKER, SERVICE_QUEUE_MARKER, SERVICE_QUEUE_STATUS_MARKER, SLEEP_MARKER,
    TASK_MODEL_FAIL_MARKER, TASK_MODEL_MARKER, TASK_MODEL_STATUS_MARKER,
    TELEMETRY_EVENTS_FAIL_MARKER, TELEMETRY_EVENTS_MARKER, TELEMETRY_EVENTS_SCHEMA_MARKER,
    TELEMETRY_EVENTS_STATUS_MARKER, TIMER_DELAYED_LOG_MARKER, TIMER_MARKER, TIMER_SETUP_MARKER,
    TIMER_TICK_1_MARKER, TIMER_TICK_2_MARKER, TIMER_TICK_3_MARKER, TRACE_EVENT_BOOT_MARKER,
    TRACE_EVENT_CAPABILITY_MARKER, TRACE_EVENT_SCHEDULER_MARKER, TRACE_EVENT_TASK_REDACTED_MARKER,
};

#[test]
fn qemu_markers_track_current_contracts() {
    assert_eq!(BOOTINFO_FAIL_MARKER, "[TEST] bootinfo=fail");
    assert_eq!(BOOTINFO_MARKER, "[TEST] bootinfo=ok");
    assert_eq!(BOOT_DIAGNOSTIC_MARKER, "[kernel][INFO] bootinfo normalized");
    assert_eq!(AI_POLICY_FAIL_MARKER, "[TEST] ai-policy=fail");
    assert_eq!(AI_POLICY_STATUS_MARKER, "ai-policy schema=1");
    assert_eq!(
        AI_POLICY_METADATA_GATE_MARKER,
        "manifest_metadata_gate_ok=true"
    );
    assert_eq!(AI_POLICY_HEURISTIC_ENABLED_MARKER, "heuristic_enabled=true");
    assert_eq!(
        AI_POLICY_HEURISTIC_SCORE_MARKER,
        "heuristic_score=<redacted>"
    );
    assert_eq!(AI_POLICY_HEURISTIC_CORE_MARKER, "heuristic_core=<redacted>");
    assert_eq!(
        AI_POLICY_HEURISTIC_DISABLED_MARKER,
        "heuristic_disabled_fallback_ok=true"
    );
    assert_eq!(AI_POLICY_MARKER, "[TEST] ai-policy=ok");
    assert_eq!(AMP_CORE_FAIL_MARKER, "[TEST] amp-core=fail");
    assert_eq!(AMP_CORE_STATUS_MARKER, "amp-core bootstrap_role_ok=true");
    assert_eq!(AMP_CORE_BOOTSTRAP_ROLE_MARKER, "bootstrap_role_ok=true");
    assert_eq!(AMP_CORE_CAPABILITIES_MARKER, "capabilities_ok=true");
    assert_eq!(AMP_CORE_REGISTRY_MARKER, "registry_ok=true");
    assert_eq!(AMP_CORE_TELEMETRY_MARKER, "telemetry_ok=true");
    assert_eq!(AMP_CORE_BARRIER_MARKER, "barrier_ok=true");
    assert_eq!(AMP_CORE_MARKER, "[TEST] amp-core=ok");
    assert_eq!(
        MULTICORE_TOPOLOGY_STATUS_MARKER,
        "multicore-topology qemu_smp_cores_ok=true"
    );
    assert_eq!(MULTICORE_TOPOLOGY_QEMU_SMP_MARKER, "qemu_smp_cores_ok=true");
    assert_eq!(
        MULTICORE_TOPOLOGY_HARDWARE_ONLINE_MARKER,
        "hardware_online_ok=true"
    );
    assert_eq!(
        MULTICORE_TOPOLOGY_ROLE_ASSIGNMENT_MARKER,
        "role_assignment_ok=true"
    );
    assert_eq!(MULTICORE_TOPOLOGY_STATE_TABLE_MARKER, "state_table_ok=true");
    assert_eq!(MULTICORE_TOPOLOGY_BOOTSTRAP_MARKER, "bootstrap_ok=true");
    assert_eq!(MULTICORE_TOPOLOGY_SCHEDULER_MARKER, "scheduler_ok=true");
    assert_eq!(
        MULTICORE_TOPOLOGY_DRIVER_SERVICE_MARKER,
        "driver_service_ok=true"
    );
    assert_eq!(MULTICORE_TOPOLOGY_IDLE_MARKER, "idle_ok=true");
    assert_eq!(
        MULTICORE_TOPOLOGY_STARTUP_EVIDENCE_MARKER,
        "startup_evidence_ok=true"
    );
    assert_eq!(
        MULTICORE_TOPOLOGY_AP_PREFLIGHT_MARKER,
        "ap_preflight_ok=true"
    );
    assert_eq!(
        MULTICORE_TOPOLOGY_AP_EXECUTION_BLOCKED_MARKER,
        "ap_execution_blocked_ok=true"
    );
    assert_eq!(
        MULTICORE_TOPOLOGY_BARRIER_MARKER,
        "multicore_barrier_ok=true"
    );
    assert_eq!(
        MULTICORE_TOPOLOGY_FAIL_MARKER,
        "[TEST] multicore-topology=fail"
    );
    assert_eq!(MULTICORE_TOPOLOGY_MARKER, "[TEST] multicore-topology=ok");
    assert_eq!(CAP_AUDIT_FAIL_MARKER, "[TEST] cap-audit=fail");
    assert_eq!(CAP_AUDIT_MARKER, "[TEST] cap-audit=ok");
    assert_eq!(CAP_AUDIT_STATUS_MARKER, "cap-audit events=");
    assert_eq!(CAP_TABLE_FAIL_MARKER, "[TEST] cap=fail");
    assert_eq!(CAP_TABLE_MARKER, "[TEST] cap=ok");
    assert_eq!(CAP_TABLE_STATUS_MARKER, "cap-table capacity=");
    assert_eq!(
        COOPERATIVE_SCHED_FAIL_MARKER,
        "[TEST] cooperative-sched=fail"
    );
    assert_eq!(COOPERATIVE_SCHED_MARKER, "[TEST] cooperative-sched=ok");
    assert_eq!(COOPERATIVE_SCHED_STATUS_MARKER, "cooperative-sched task_a=");
    assert_eq!(CONCURRENCY_FAIL_MARKER, "[TEST] concurrency=fail");
    assert_eq!(CONCURRENCY_STATUS_MARKER, "concurrency irq_guard_ok=true");
    assert_eq!(CONCURRENCY_IRQ_MARKER, "irq_guard_ok=true");
    assert_eq!(CONCURRENCY_NESTED_IRQ_MARKER, "nested_irq_guard_ok=true");
    assert_eq!(CONCURRENCY_LOCK_MARKER, "early_lock_ok=true");
    assert_eq!(CONCURRENCY_IRQ_LOCK_MARKER, "irq_lock_ok=true");
    assert_eq!(CONCURRENCY_ORDER_MARKER, "lock_order_ok=true");
    assert_eq!(CONCURRENCY_MARKER, "[TEST] concurrency=ok");
    assert_eq!(CPU_SETUP_MARKER, "[TEST] gdt=ok");
    assert_eq!(CPU_HARDENING_FAIL_MARKER, "[TEST] cpu-hardening=fail");
    assert_eq!(CPU_HARDENING_MARKER, "[TEST] cpu-hardening=ok");
    assert_eq!(CPU_HARDENING_STATUS_MARKER, "cpu-hardening nx=");
    assert_eq!(CPU_HARDENING_IBRS_MARKER, "ibrs=");
    assert_eq!(CPU_HARDENING_IBPB_MARKER, "ibpb_supported=");
    assert_eq!(CPU_HARDENING_STIBP_MARKER, "stibp=");
    assert_eq!(CPU_HARDENING_SSBD_MARKER, "ssbd=");
    assert_eq!(CPU_HARDENING_ARCH_CAPABILITIES_MARKER, "arch_capabilities=");
    assert_eq!(ENTROPY_POLICY_FAIL_MARKER, "[TEST] entropy-policy=fail");
    assert_eq!(ENTROPY_POLICY_STATUS_MARKER, "entropy-policy rdrand=");
    assert_eq!(ENTROPY_POLICY_HARDWARE_MARKER, "hardware_present=");
    assert_eq!(ENTROPY_POLICY_SELF_TEST_MARKER, "hardware_self_test=false");
    assert_eq!(ENTROPY_POLICY_DRBG_SELF_TEST_MARKER, "drbg_self_test=false");
    assert_eq!(ENTROPY_POLICY_FALLBACK_MARKER, "fallback_used=");
    assert_eq!(
        ENTROPY_POLICY_GENERATION_MARKER,
        "generation_counter_ok=true"
    );
    assert_eq!(
        ENTROPY_POLICY_RANDOM_TOKEN_MARKER,
        "random_tokens_available="
    );
    assert_eq!(ENTROPY_POLICY_SOURCE_MARKER, "source=");
    assert_eq!(ENTROPY_POLICY_MARKER, "[TEST] entropy-policy=ok");
    assert_eq!(EXCEPTION_SETUP_MARKER, "[TEST] idt=ok");
    assert_eq!(EXCEPTION_MARKER, "[TEST] exception=ok");
    assert_eq!(FRAME_ALLOCATOR_FAIL_MARKER, "[TEST] frame-allocator=fail");
    assert_eq!(FRAME_ALLOCATOR_MARKER, "[TEST] frame-allocator=ok");
    assert_eq!(
        FRAME_ALLOCATOR_STATUS_MARKER,
        "frame-allocator total_frames="
    );
    assert_eq!(HEAP_FAIL_MARKER, "[TEST] heap=fail");
    assert_eq!(HEAP_MARKER, "[TEST] heap=ok");
    assert_eq!(HEAP_STATUS_MARKER, "heap bytes=");
    assert_eq!(HEAP_SLAB_CLASSES_MARKER, "slab_classes=");
    assert_eq!(HEAP_SLAB_REUSE_MARKER, "slab_reuse_ok=true");
    assert_eq!(HEAP_PAGE_RUN_MARKER, "page_run_ok=true");
    assert_eq!(HEAP_STRESS_MARKER, "stress_ok=true");
    assert_eq!(HEAP_DOUBLE_FREE_MARKER, "double_free_detected=true");
    assert_eq!(HEAP_INVALID_FREE_MARKER, "invalid_free_detected=true");
    assert_eq!(
        HEAP_ACCOUNTING_OVERFLOW_MARKER,
        "accounting_overflow_detected=false"
    );
    assert_eq!(
        HEAP_CORRUPT_FREE_LIST_MARKER,
        "corrupt_free_list_detected=false"
    );
    assert_eq!(
        SCHEDULER_TELEMETRY_FAIL_MARKER,
        "[TEST] scheduler-telemetry=fail"
    );
    assert_eq!(SCHEDULER_TELEMETRY_MARKER, "[TEST] scheduler-telemetry=ok");
    assert_eq!(
        SCHEDULER_TELEMETRY_STATUS_MARKER,
        "scheduler-telemetry decisions="
    );
    assert_eq!(TELEMETRY_EVENTS_FAIL_MARKER, "[TEST] telemetry-events=fail");
    assert_eq!(TELEMETRY_EVENTS_MARKER, "[TEST] telemetry-events=ok");
    assert_eq!(TELEMETRY_EVENTS_SCHEMA_MARKER, "telemetry-events schema=1");
    assert_eq!(
        TELEMETRY_EVENTS_STATUS_MARKER,
        "telemetry-events schema=1 events="
    );
    assert_eq!(
        TRACE_EVENT_BOOT_MARKER,
        "trace-event schema=1 event=boot-phase"
    );
    assert_eq!(
        TRACE_EVENT_CAPABILITY_MARKER,
        "trace-event schema=1 event=capability-fault"
    );
    assert_eq!(
        TRACE_EVENT_SCHEDULER_MARKER,
        "trace-event schema=1 event=scheduler-decision"
    );
    assert_eq!(TRACE_EVENT_TASK_REDACTED_MARKER, "selected_task=<redacted>");
    assert_eq!(IRQ_SETUP_MARKER, "[TEST] irq=ok");
    assert_eq!(KERNEL_CR3_ACTIVE_MARKER, "kernel-cr3 active=true");
    assert_eq!(KERNEL_CR3_FAIL_MARKER, "[TEST] kernel-cr3=fail");
    assert_eq!(KERNEL_CR3_MARKER, "[TEST] kernel-cr3=ok");
    assert_eq!(KERNEL_STACK_GUARD_MARKER, "[TEST] kernel-stack-guard=ok");
    assert_eq!(
        KERNEL_STACK_GUARD_STATUS_MARKER,
        "kernel_stack_guard_ok=true"
    );
    assert_eq!(KERNEL_STACK_PAGES_MARKER, "kernel_stack_pages=");
    assert_eq!(MEMORY_MAP_FAIL_MARKER, "[TEST] memory-map=fail");
    assert_eq!(MEMORY_MAP_MARKER, "[TEST] memory-map=ok");
    assert_eq!(MEMORY_CAP_FAIL_MARKER, "[TEST] memory-cap=fail");
    assert_eq!(MEMORY_CAP_MARKER, "[TEST] memory-cap=ok");
    assert_eq!(MEMORY_CAP_STATUS_MARKER, "memory-cap map_allowed=");
    assert_eq!(MEMORY_RESERVED_MARKER, "memory reserved_bytes=");
    assert_eq!(MEMORY_TOTAL_MARKER, "memory total_bytes=");
    assert_eq!(MEMORY_USABLE_MARKER, "memory usable_bytes=");
    assert_eq!(FAULT_ADDRESS_MARKER, "cr2_offset=0x");
    assert_eq!(FAULT_ADDRESS_PRESENT_MARKER, "cr2_present=");
    assert_eq!(FAULT_CR3_MARKER, "cr3_offset=0x");
    assert_eq!(FAULT_ERROR_DECODE_MARKER, "present=");
    assert_eq!(FAULT_INTERRUPTS_MARKER, "interrupts_enabled=");
    assert_eq!(FAULT_RFLAGS_MARKER, "rflags=0x");
    assert_eq!(PAGE_FAULT_MARKER, "[TEST] pagefault=ok");
    assert_eq!(PAGE_TABLE_FAIL_MARKER, "[TEST] page-table=fail");
    assert_eq!(PAGE_TABLE_CHECKED_ROOT_MARKER, "checked_root_ok=true");
    assert_eq!(PAGE_TABLE_CHECKED_STATUS_MARKER, "checked_status_ok=true");
    assert_eq!(
        PAGE_TABLE_CHECKED_TRANSLATE_MARKER,
        "checked_translate_ok=true"
    );
    assert_eq!(
        PAGE_TABLE_TRANSLATE_OFFSET_MARKER,
        "translate_offset_ok=true"
    );
    assert_eq!(PAGE_TABLE_LOOKUP_MARKER, "mapping_lookup_ok=true");
    assert_eq!(PAGE_TABLE_MARKER, "[TEST] page-table=ok");
    assert_eq!(PAGE_TABLE_PRESENCE_MARKER, "presence_ok=true");
    assert_eq!(PAGE_TABLE_PROTECT_MARKER, "protect_ok=true");
    assert_eq!(PAGE_TABLE_PROTECT_RANGE_MARKER, "protect_range_ok=true");
    assert_eq!(PAGE_TABLE_RANGE_LOOKUP_MARKER, "range_lookup_ok=true");
    assert_eq!(PAGE_TABLE_RANGE_TRANSLATE_MARKER, "range_translate_ok=true");
    assert_eq!(PAGE_TABLE_MAPPED_RANGE_MARKER, "mapped_range_ok=true");
    assert_eq!(PAGE_TABLE_UNMAPPED_RANGE_MARKER, "unmapped_range_ok=true");
    assert_eq!(PAGE_TABLE_KERNEL_RANGE_MARKER, "kernel_range_ok=true");
    assert_eq!(PAGE_TABLE_USER_RANGE_MARKER, "user_range_ok=true");
    assert_eq!(
        PAGE_TABLE_WRITE_PROTECTED_RANGE_MARKER,
        "write_protected_range_ok=true"
    );
    assert_eq!(
        PAGE_TABLE_NON_EXECUTABLE_RANGE_MARKER,
        "non_executable_range_ok=true"
    );
    assert_eq!(
        PAGE_TABLE_EXECUTABLE_RANGE_MARKER,
        "executable_range_ok=true"
    );
    assert_eq!(
        PAGE_TABLE_NORMAL_MEMORY_RANGE_MARKER,
        "normal_memory_range_ok=true"
    );
    assert_eq!(PAGE_TABLE_LOCAL_RANGE_MARKER, "local_range_ok=true");
    assert_eq!(
        PAGE_TABLE_KERNEL_SPACE_RANGE_MARKER,
        "kernel_space_range_ok=true"
    );
    assert_eq!(
        PAGE_TABLE_USER_SPACE_RANGE_MARKER,
        "user_space_range_ok=true"
    );
    assert_eq!(PAGE_TABLE_NO_USER_SPACE_MARKER, "no_user_space_ok=true");
    assert_eq!(PAGE_TABLE_NO_EXECUTABLE_MARKER, "no_executable_ok=true");
    assert_eq!(PAGE_TABLE_NO_WRITABLE_MARKER, "no_writable_ok=true");
    assert_eq!(PAGE_TABLE_NO_DEVICE_MARKER, "no_device_ok=true");
    assert_eq!(PAGE_TABLE_NO_GLOBAL_MARKER, "no_global_ok=true");
    assert_eq!(PAGE_TABLE_NO_ALIAS_MARKER, "no_alias_ok=true");
    assert_eq!(
        PAGE_TABLE_KERNEL_CANDIDATE_MARKER,
        "kernel_candidate_ok=true"
    );
    assert_eq!(PAGE_TABLE_USER_CANDIDATE_MARKER, "user_candidate_ok=true");
    assert_eq!(
        PAGE_TABLE_KERNEL_USER_GUARD_MARKER,
        "kernel_user_guard_ok=true"
    );
    assert_eq!(PAGE_TABLE_KERNEL_ONLY_MARKER, "kernel_only_ok=true");
    assert_eq!(PAGE_TABLE_AUDIT_MARKER, "audit_ok=true");
    assert_eq!(PAGE_TABLE_VISIT_MARKER, "visit_ok=true");
    assert_eq!(PAGE_TABLE_FLAGS_MARKER, "flags_ok=true");
    assert_eq!(PAGE_TABLE_RANGE_MARKER, "range_ok=true");
    assert_eq!(PAGE_TABLE_RECLAIM_MARKER, "reclaim_ok=true");
    assert_eq!(PAGE_TABLE_FLUSH_PAGE_MARKER, "flush_page=true");
    assert_eq!(PAGE_TABLE_ROOT_MARKER, "root_ok=true");
    assert_eq!(PAGE_TABLE_STATUS_MARKER, "page-table total_tables=");
    assert_eq!(
        PAGING_POLICY_MODEL_FAIL_MARKER,
        "[TEST] paging-policy-model=fail"
    );
    assert_eq!(PAGING_POLICY_MODEL_MARKER, "[TEST] paging-policy-model=ok");
    assert_eq!(
        PAGING_POLICY_MODEL_STATUS_MARKER,
        "paging-policy-model mapped_pages="
    );
    assert_eq!(
        PAGING_POLICY_MODEL_SECTION_LAYOUT_MARKER,
        "section_layout_ok=true"
    );
    assert_eq!(PAGING_POLICY_MODEL_TEXT_RX_MARKER, "text_rx_ok=true");
    assert_eq!(
        PAGING_POLICY_MODEL_RODATA_READ_ONLY_MARKER,
        "rodata_read_only_ok=true"
    );
    assert_eq!(PAGING_POLICY_MODEL_DATA_RW_NX_MARKER, "data_rw_nx_ok=true");
    assert_eq!(
        PAGING_POLICY_MODEL_HEAP_RESERVED_MARKER,
        "heap_reserved_ok=true"
    );
    assert_eq!(PAGING_POLICY_MODEL_GUARD_PAGE_MARKER, "guard_page_ok=true");
    assert_eq!(PAGING_POLICY_MODEL_NULL_PAGE_MARKER, "null_page_ok=true");
    assert_eq!(
        PAGING_POLICY_MODEL_HARDWARE_IMAGE_MARKER,
        "hardware_image_ok=true"
    );
    assert_eq!(
        PAGING_POLICY_MODEL_HARDWARE_ARENA_MARKER,
        "hardware_arena_frames="
    );
    assert_eq!(
        PAGING_POLICY_MODEL_HARDWARE_ROOT_MARKER,
        "hardware_root_allocated=true"
    );
    assert_eq!(
        PAGING_POLICY_MODEL_HARDWARE_TABLES_MARKER,
        "hardware_tables_copied="
    );
    assert_eq!(
        PAGING_POLICY_MODEL_HARDWARE_COPIED_MARKER,
        "hardware_copied=true"
    );
    assert_eq!(
        PANIC_DIAGNOSTIC_MARKER,
        "[kernel][FATAL] panic handler entered"
    );
    assert_eq!(PANIC_MARKER, "[TEST] panic=ok");
    assert_eq!(PANIC_REGISTERS_MARKER, "panic registers=");
    assert_eq!(SERIAL_MARKER, "[TEST] boot=ok");
    assert_eq!(SERVICE_QUEUE_FAIL_MARKER, "[TEST] service-queue=fail");
    assert_eq!(SERVICE_QUEUE_MARKER, "[TEST] service-queue=ok");
    assert_eq!(SERVICE_QUEUE_STATUS_MARKER, "service-queue log_submitted=");
    assert_eq!(TASK_MODEL_FAIL_MARKER, "[TEST] task-model=fail");
    assert_eq!(TASK_MODEL_MARKER, "[TEST] task-model=ok");
    assert_eq!(TASK_MODEL_STATUS_MARKER, "task-model created=");
    assert_eq!(TIMER_SETUP_MARKER, "timer setup=pit");
    assert_eq!(TIMER_TICK_1_MARKER, "timer tick 1");
    assert_eq!(TIMER_TICK_2_MARKER, "timer tick 2");
    assert_eq!(TIMER_TICK_3_MARKER, "timer tick 3");
    assert_eq!(TIMER_DELAYED_LOG_MARKER, "timer delayed-log");
    assert_eq!(SLEEP_MARKER, "[TEST] sleep=ok");
    assert_eq!(TIMER_MARKER, "[TEST] timer=ok");
}
