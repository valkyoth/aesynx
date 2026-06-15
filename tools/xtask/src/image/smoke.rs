use std::fs;
use std::path::Path;

pub const BOOTINFO_FAIL_MARKER: &str = "[TEST] bootinfo=fail";
pub const BOOTINFO_MARKER: &str = "[TEST] bootinfo=ok";
pub const BOOT_DIAGNOSTIC_MARKER: &str = "[kernel][INFO] bootinfo normalized";
pub const AI_POLICY_FAIL_MARKER: &str = "[TEST] ai-policy=fail";
pub const AI_POLICY_HEURISTIC_DISABLED_MARKER: &str = "heuristic_disabled_fallback_ok=true";
pub const AI_POLICY_HEURISTIC_ENABLED_MARKER: &str = "heuristic_enabled=true";
pub const AI_POLICY_HEURISTIC_SCORE_MARKER: &str = "heuristic_score=";
pub const AI_POLICY_MARKER: &str = "[TEST] ai-policy=ok";
pub const AI_POLICY_METADATA_GATE_MARKER: &str = "manifest_metadata_gate_ok=true";
pub const AI_POLICY_STATUS_MARKER: &str = "ai-policy schema=1";
pub const CAP_AUDIT_FAIL_MARKER: &str = "[TEST] cap-audit=fail";
pub const CAP_AUDIT_MARKER: &str = "[TEST] cap-audit=ok";
pub const CAP_AUDIT_STATUS_MARKER: &str = "cap-audit events=";
pub const CAP_TABLE_FAIL_MARKER: &str = "[TEST] cap=fail";
pub const CAP_TABLE_MARKER: &str = "[TEST] cap=ok";
pub const CAP_TABLE_STATUS_MARKER: &str = "cap-table capacity=";
pub const COOPERATIVE_SCHED_FAIL_MARKER: &str = "[TEST] cooperative-sched=fail";
pub const COOPERATIVE_SCHED_MARKER: &str = "[TEST] cooperative-sched=ok";
pub const COOPERATIVE_SCHED_STATUS_MARKER: &str = "cooperative-sched task_a=";
pub const SCHEDULER_TELEMETRY_FAIL_MARKER: &str = "[TEST] scheduler-telemetry=fail";
pub const SCHEDULER_TELEMETRY_MARKER: &str = "[TEST] scheduler-telemetry=ok";
pub const SCHEDULER_TELEMETRY_STATUS_MARKER: &str = "scheduler-telemetry decisions=";
pub const TELEMETRY_EVENTS_FAIL_MARKER: &str = "[TEST] telemetry-events=fail";
pub const TELEMETRY_EVENTS_MARKER: &str = "[TEST] telemetry-events=ok";
pub const TELEMETRY_EVENTS_SCHEMA_MARKER: &str = "telemetry-events schema=1";
pub const TELEMETRY_EVENTS_STATUS_MARKER: &str = "telemetry-events schema=1 events=";
pub const TRACE_EVENT_BOOT_MARKER: &str = "trace-event schema=1 event=boot-phase";
pub const TRACE_EVENT_CAPABILITY_MARKER: &str = "trace-event schema=1 event=capability-fault";
pub const TRACE_EVENT_SCHEDULER_MARKER: &str = "trace-event schema=1 event=scheduler-decision";
pub const TRACE_EVENT_TASK_REDACTED_MARKER: &str = "selected_task=<redacted>";
pub const CPU_SETUP_MARKER: &str = "[TEST] gdt=ok";
pub const CPU_HARDENING_FAIL_MARKER: &str = "[TEST] cpu-hardening=fail";
pub const CPU_HARDENING_MARKER: &str = "[TEST] cpu-hardening=ok";
pub const CPU_HARDENING_STATUS_MARKER: &str = "cpu-hardening nx=";
pub const ENTROPY_POLICY_FAIL_MARKER: &str = "[TEST] entropy-policy=fail";
pub const ENTROPY_POLICY_FALLBACK_MARKER: &str = "fallback_used=";
pub const ENTROPY_POLICY_GENERATION_MARKER: &str = "generation_counter_ok=true";
pub const ENTROPY_POLICY_HARDWARE_MARKER: &str = "hardware_present=";
pub const ENTROPY_POLICY_SELF_TEST_MARKER: &str = "hardware_self_test=false";
pub const ENTROPY_POLICY_MARKER: &str = "[TEST] entropy-policy=ok";
pub const ENTROPY_POLICY_RANDOM_TOKEN_MARKER: &str = "random_tokens_available=";
pub const ENTROPY_POLICY_SOURCE_MARKER: &str = "source=";
pub const ENTROPY_POLICY_STATUS_MARKER: &str = "entropy-policy rdrand=";
pub const EXCEPTION_SETUP_MARKER: &str = "[TEST] idt=ok";
pub const EXCEPTION_MARKER: &str = "[TEST] exception=ok";
pub const FRAME_ALLOCATOR_FAIL_MARKER: &str = "[TEST] frame-allocator=fail";
pub const FRAME_ALLOCATOR_MARKER: &str = "[TEST] frame-allocator=ok";
pub const FRAME_ALLOCATOR_STATUS_MARKER: &str = "frame-allocator total_frames=";
pub const HEAP_FAIL_MARKER: &str = "[TEST] heap=fail";
pub const HEAP_MARKER: &str = "[TEST] heap=ok";
pub const HEAP_STATUS_MARKER: &str = "heap bytes=";
pub const HEAP_SLAB_CLASSES_MARKER: &str = "slab_classes=";
pub const HEAP_SLAB_REUSE_MARKER: &str = "slab_reuse_ok=true";
pub const HEAP_PAGE_RUN_MARKER: &str = "page_run_ok=true";
pub const HEAP_STRESS_MARKER: &str = "stress_ok=true";
pub const HEAP_DOUBLE_FREE_MARKER: &str = "double_free_detected=true";
pub const HEAP_INVALID_FREE_MARKER: &str = "invalid_free_detected=true";
pub const HEAP_CORRUPT_FREE_LIST_MARKER: &str = "corrupt_free_list_detected=false";
pub const IRQ_SETUP_MARKER: &str = "[TEST] irq=ok";
pub const KERNEL_CR3_ACTIVE_MARKER: &str = "kernel-cr3 active=true";
pub const KERNEL_CR3_FAIL_MARKER: &str = "[TEST] kernel-cr3=fail";
pub const KERNEL_CR3_MARKER: &str = "[TEST] kernel-cr3=ok";
pub const KERNEL_STACK_GUARD_MARKER: &str = "[TEST] kernel-stack-guard=ok";
pub const KERNEL_STACK_GUARD_STATUS_MARKER: &str = "kernel_stack_guard_ok=true";
pub const KERNEL_STACK_PAGES_MARKER: &str = "kernel_stack_pages=";
pub const MEMORY_MAP_FAIL_MARKER: &str = "[TEST] memory-map=fail";
pub const MEMORY_MAP_MARKER: &str = "[TEST] memory-map=ok";
pub const MEMORY_CAP_FAIL_MARKER: &str = "[TEST] memory-cap=fail";
pub const MEMORY_CAP_MARKER: &str = "[TEST] memory-cap=ok";
pub const MEMORY_CAP_STATUS_MARKER: &str = "memory-cap map_allowed=";
pub const MEMORY_RESERVED_MARKER: &str = "memory reserved_bytes=";
pub const MEMORY_TOTAL_MARKER: &str = "memory total_bytes=";
pub const MEMORY_USABLE_MARKER: &str = "memory usable_bytes=";
pub const PAGE_TABLE_NO_DEVICE_MARKER: &str = "no_device_ok=true";
pub const PAGE_TABLE_NO_EXECUTABLE_MARKER: &str = "no_executable_ok=true";
pub const PAGE_TABLE_NO_GLOBAL_MARKER: &str = "no_global_ok=true";
pub const PAGE_TABLE_NO_ALIAS_MARKER: &str = "no_alias_ok=true";
pub const PAGE_TABLE_NO_USER_SPACE_MARKER: &str = "no_user_space_ok=true";
pub const PAGE_TABLE_NO_WRITABLE_MARKER: &str = "no_writable_ok=true";
pub const PAGE_TABLE_EXECUTABLE_RANGE_MARKER: &str = "executable_range_ok=true";
pub const PAGE_TABLE_NORMAL_MEMORY_RANGE_MARKER: &str = "normal_memory_range_ok=true";
pub const PAGE_TABLE_LOCAL_RANGE_MARKER: &str = "local_range_ok=true";
pub const PAGE_TABLE_KERNEL_SPACE_RANGE_MARKER: &str = "kernel_space_range_ok=true";
pub const PAGE_TABLE_USER_SPACE_RANGE_MARKER: &str = "user_space_range_ok=true";
pub const PAGE_TABLE_NON_EXECUTABLE_RANGE_MARKER: &str = "non_executable_range_ok=true";
pub const FAULT_ADDRESS_MARKER: &str = "cr2_offset=0x";
pub const FAULT_ADDRESS_PRESENT_MARKER: &str = "cr2_present=";
pub const FAULT_CR3_MARKER: &str = "cr3_offset=0x";
pub const FAULT_ERROR_DECODE_MARKER: &str = "present=";
pub const FAULT_INTERRUPTS_MARKER: &str = "interrupts_enabled=";
pub const FAULT_RFLAGS_MARKER: &str = "rflags=0x";
pub const PAGE_FAULT_MARKER: &str = "[TEST] pagefault=ok";
pub const PAGE_TABLE_AUDIT_MARKER: &str = "audit_ok=true";
pub const PAGE_TABLE_CHECKED_TRANSLATE_MARKER: &str = "checked_translate_ok=true";
pub const PAGE_TABLE_CHECKED_STATUS_MARKER: &str = "checked_status_ok=true";
pub const PAGE_TABLE_FAIL_MARKER: &str = "[TEST] page-table=fail";
pub const PAGE_TABLE_FLUSH_PAGE_MARKER: &str = "flush_page=true";
pub const PAGE_TABLE_FLAGS_MARKER: &str = "flags_ok=true";
pub const PAGE_TABLE_KERNEL_CANDIDATE_MARKER: &str = "kernel_candidate_ok=true";
pub const PAGE_TABLE_KERNEL_USER_GUARD_MARKER: &str = "kernel_user_guard_ok=true";
pub const PAGE_TABLE_KERNEL_ONLY_MARKER: &str = "kernel_only_ok=true";
pub const PAGE_TABLE_USER_CANDIDATE_MARKER: &str = "user_candidate_ok=true";
pub const PAGE_TABLE_KERNEL_RANGE_MARKER: &str = "kernel_range_ok=true";
pub const PAGE_TABLE_USER_RANGE_MARKER: &str = "user_range_ok=true";
pub const PAGE_TABLE_LOOKUP_MARKER: &str = "mapping_lookup_ok=true";
pub const PAGE_TABLE_MARKER: &str = "[TEST] page-table=ok";
pub const PAGE_TABLE_MAPPED_RANGE_MARKER: &str = "mapped_range_ok=true";
pub const PAGE_TABLE_PRESENCE_MARKER: &str = "presence_ok=true";
pub const PAGE_TABLE_PROTECT_MARKER: &str = "protect_ok=true";
pub const PAGE_TABLE_PROTECT_RANGE_MARKER: &str = "protect_range_ok=true";
pub const PAGE_TABLE_RANGE_LOOKUP_MARKER: &str = "range_lookup_ok=true";
pub const PAGE_TABLE_RANGE_TRANSLATE_MARKER: &str = "range_translate_ok=true";
pub const PAGE_TABLE_RANGE_MARKER: &str = "range_ok=true";
pub const PAGE_TABLE_RECLAIM_MARKER: &str = "reclaim_ok=true";
pub const PAGE_TABLE_ROOT_MARKER: &str = "root_ok=true";
pub const PAGE_TABLE_CHECKED_ROOT_MARKER: &str = "checked_root_ok=true";
pub const PAGE_TABLE_STATUS_MARKER: &str = "page-table total_tables=";
pub const PAGE_TABLE_TRANSLATE_OFFSET_MARKER: &str = "translate_offset_ok=true";
pub const PAGE_TABLE_UNMAPPED_RANGE_MARKER: &str = "unmapped_range_ok=true";
pub const PAGE_TABLE_VISIT_MARKER: &str = "visit_ok=true";
pub const PAGE_TABLE_WRITE_PROTECTED_RANGE_MARKER: &str = "write_protected_range_ok=true";
pub const PAGING_POLICY_MODEL_FAIL_MARKER: &str = "[TEST] paging-policy-model=fail";
pub const PAGING_POLICY_MODEL_DATA_RW_NX_MARKER: &str = "data_rw_nx_ok=true";
pub const PAGING_POLICY_MODEL_GUARD_PAGE_MARKER: &str = "guard_page_ok=true";
pub const PAGING_POLICY_MODEL_HARDWARE_ARENA_MARKER: &str = "hardware_arena_frames=";
pub const PAGING_POLICY_MODEL_HARDWARE_COPIED_MARKER: &str = "hardware_copied=true";
pub const PAGING_POLICY_MODEL_HARDWARE_IMAGE_MARKER: &str = "hardware_image_ok=true";
pub const PAGING_POLICY_MODEL_HARDWARE_ROOT_MARKER: &str = "hardware_root_allocated=true";
pub const PAGING_POLICY_MODEL_HARDWARE_TABLES_MARKER: &str = "hardware_tables_copied=";
pub const PAGING_POLICY_MODEL_HEAP_RESERVED_MARKER: &str = "heap_reserved_ok=true";
pub const PAGING_POLICY_MODEL_MARKER: &str = "[TEST] paging-policy-model=ok";
pub const PAGING_POLICY_MODEL_NULL_PAGE_MARKER: &str = "null_page_ok=true";
pub const PAGING_POLICY_MODEL_RODATA_READ_ONLY_MARKER: &str = "rodata_read_only_ok=true";
pub const PAGING_POLICY_MODEL_SECTION_LAYOUT_MARKER: &str = "section_layout_ok=true";
pub const PAGING_POLICY_MODEL_STATUS_MARKER: &str = "paging-policy-model mapped_pages=";
pub const PAGING_POLICY_MODEL_TEXT_RX_MARKER: &str = "text_rx_ok=true";
pub const PANIC_DIAGNOSTIC_MARKER: &str = "[kernel][FATAL] panic handler entered";
pub const PANIC_MARKER: &str = "[TEST] panic=ok";
pub const PANIC_REGISTERS_MARKER: &str = "panic registers=";
pub const SERIAL_MARKER: &str = "[TEST] boot=ok";
pub const SERVICE_QUEUE_FAIL_MARKER: &str = "[TEST] service-queue=fail";
pub const SERVICE_QUEUE_MARKER: &str = "[TEST] service-queue=ok";
pub const SERVICE_QUEUE_STATUS_MARKER: &str = "service-queue log_submitted=";
pub const TASK_MODEL_FAIL_MARKER: &str = "[TEST] task-model=fail";
pub const TASK_MODEL_MARKER: &str = "[TEST] task-model=ok";
pub const TASK_MODEL_STATUS_MARKER: &str = "task-model created=";
pub const SLEEP_MARKER: &str = "[TEST] sleep=ok";
pub const TIMER_DELAYED_LOG_MARKER: &str = "timer delayed-log";
pub const TIMER_MARKER: &str = "[TEST] timer=ok";
pub const TIMER_SETUP_MARKER: &str = "timer setup=pit";
pub const TIMER_TICK_1_MARKER: &str = "timer tick 1";
pub const TIMER_TICK_2_MARKER: &str = "timer tick 2";
pub const TIMER_TICK_3_MARKER: &str = "timer tick 3";

const BOOT_REQUIRED_MARKERS: &[&str] = &[
    CPU_SETUP_MARKER,
    EXCEPTION_SETUP_MARKER,
    IRQ_SETUP_MARKER,
    EXCEPTION_MARKER,
    BOOT_DIAGNOSTIC_MARKER,
    MEMORY_TOTAL_MARKER,
    MEMORY_USABLE_MARKER,
    MEMORY_RESERVED_MARKER,
    MEMORY_MAP_MARKER,
    FRAME_ALLOCATOR_STATUS_MARKER,
    FRAME_ALLOCATOR_MARKER,
    PAGE_TABLE_STATUS_MARKER,
    PAGE_TABLE_ROOT_MARKER,
    PAGE_TABLE_CHECKED_ROOT_MARKER,
    PAGE_TABLE_CHECKED_STATUS_MARKER,
    PAGE_TABLE_KERNEL_CANDIDATE_MARKER,
    PAGE_TABLE_USER_CANDIDATE_MARKER,
    PAGE_TABLE_TRANSLATE_OFFSET_MARKER,
    PAGE_TABLE_CHECKED_TRANSLATE_MARKER,
    PAGE_TABLE_LOOKUP_MARKER,
    PAGE_TABLE_PRESENCE_MARKER,
    PAGE_TABLE_PROTECT_MARKER,
    PAGE_TABLE_PROTECT_RANGE_MARKER,
    PAGE_TABLE_RANGE_LOOKUP_MARKER,
    PAGE_TABLE_RANGE_TRANSLATE_MARKER,
    PAGE_TABLE_MAPPED_RANGE_MARKER,
    PAGE_TABLE_UNMAPPED_RANGE_MARKER,
    PAGE_TABLE_KERNEL_RANGE_MARKER,
    PAGE_TABLE_USER_RANGE_MARKER,
    PAGE_TABLE_WRITE_PROTECTED_RANGE_MARKER,
    PAGE_TABLE_NON_EXECUTABLE_RANGE_MARKER,
    PAGE_TABLE_EXECUTABLE_RANGE_MARKER,
    PAGE_TABLE_NORMAL_MEMORY_RANGE_MARKER,
    PAGE_TABLE_LOCAL_RANGE_MARKER,
    PAGE_TABLE_KERNEL_SPACE_RANGE_MARKER,
    PAGE_TABLE_USER_SPACE_RANGE_MARKER,
    PAGE_TABLE_NO_USER_SPACE_MARKER,
    PAGE_TABLE_NO_EXECUTABLE_MARKER,
    PAGE_TABLE_NO_WRITABLE_MARKER,
    PAGE_TABLE_NO_DEVICE_MARKER,
    PAGE_TABLE_NO_GLOBAL_MARKER,
    PAGE_TABLE_NO_ALIAS_MARKER,
    PAGE_TABLE_KERNEL_USER_GUARD_MARKER,
    PAGE_TABLE_KERNEL_ONLY_MARKER,
    PAGE_TABLE_AUDIT_MARKER,
    PAGE_TABLE_VISIT_MARKER,
    PAGE_TABLE_FLAGS_MARKER,
    PAGE_TABLE_RECLAIM_MARKER,
    PAGE_TABLE_RANGE_MARKER,
    PAGE_TABLE_FLUSH_PAGE_MARKER,
    PAGE_TABLE_MARKER,
    PAGING_POLICY_MODEL_STATUS_MARKER,
    PAGING_POLICY_MODEL_SECTION_LAYOUT_MARKER,
    PAGING_POLICY_MODEL_TEXT_RX_MARKER,
    PAGING_POLICY_MODEL_RODATA_READ_ONLY_MARKER,
    PAGING_POLICY_MODEL_DATA_RW_NX_MARKER,
    PAGING_POLICY_MODEL_HEAP_RESERVED_MARKER,
    PAGING_POLICY_MODEL_GUARD_PAGE_MARKER,
    PAGING_POLICY_MODEL_NULL_PAGE_MARKER,
    PAGING_POLICY_MODEL_HARDWARE_IMAGE_MARKER,
    PAGING_POLICY_MODEL_HARDWARE_ARENA_MARKER,
    PAGING_POLICY_MODEL_HARDWARE_ROOT_MARKER,
    PAGING_POLICY_MODEL_HARDWARE_TABLES_MARKER,
    PAGING_POLICY_MODEL_HARDWARE_COPIED_MARKER,
    KERNEL_STACK_PAGES_MARKER,
    KERNEL_STACK_GUARD_STATUS_MARKER,
    KERNEL_STACK_GUARD_MARKER,
    PAGING_POLICY_MODEL_MARKER,
    BOOTINFO_MARKER,
    SERIAL_MARKER,
    CPU_HARDENING_STATUS_MARKER,
    CPU_HARDENING_MARKER,
    ENTROPY_POLICY_STATUS_MARKER,
    ENTROPY_POLICY_SELF_TEST_MARKER,
    ENTROPY_POLICY_HARDWARE_MARKER,
    ENTROPY_POLICY_FALLBACK_MARKER,
    ENTROPY_POLICY_GENERATION_MARKER,
    ENTROPY_POLICY_RANDOM_TOKEN_MARKER,
    ENTROPY_POLICY_SOURCE_MARKER,
    ENTROPY_POLICY_MARKER,
    HEAP_STATUS_MARKER,
    HEAP_SLAB_CLASSES_MARKER,
    HEAP_SLAB_REUSE_MARKER,
    HEAP_PAGE_RUN_MARKER,
    HEAP_STRESS_MARKER,
    HEAP_DOUBLE_FREE_MARKER,
    HEAP_INVALID_FREE_MARKER,
    HEAP_CORRUPT_FREE_LIST_MARKER,
    HEAP_MARKER,
    CAP_TABLE_STATUS_MARKER,
    CAP_TABLE_MARKER,
    MEMORY_CAP_STATUS_MARKER,
    MEMORY_CAP_MARKER,
    CAP_AUDIT_STATUS_MARKER,
    CAP_AUDIT_MARKER,
    SERVICE_QUEUE_STATUS_MARKER,
    SERVICE_QUEUE_MARKER,
    TASK_MODEL_STATUS_MARKER,
    TASK_MODEL_MARKER,
    COOPERATIVE_SCHED_STATUS_MARKER,
    COOPERATIVE_SCHED_MARKER,
    SCHEDULER_TELEMETRY_STATUS_MARKER,
    SCHEDULER_TELEMETRY_MARKER,
    TELEMETRY_EVENTS_SCHEMA_MARKER,
    TELEMETRY_EVENTS_STATUS_MARKER,
    TRACE_EVENT_BOOT_MARKER,
    TRACE_EVENT_CAPABILITY_MARKER,
    TRACE_EVENT_SCHEDULER_MARKER,
    TRACE_EVENT_TASK_REDACTED_MARKER,
    TELEMETRY_EVENTS_MARKER,
    AI_POLICY_STATUS_MARKER,
    AI_POLICY_METADATA_GATE_MARKER,
    AI_POLICY_HEURISTIC_ENABLED_MARKER,
    AI_POLICY_HEURISTIC_SCORE_MARKER,
    AI_POLICY_HEURISTIC_DISABLED_MARKER,
    AI_POLICY_MARKER,
    KERNEL_CR3_ACTIVE_MARKER,
    KERNEL_CR3_MARKER,
];

const BOOT_FORBIDDEN_MARKERS: &[&str] = &[
    BOOTINFO_FAIL_MARKER,
    MEMORY_MAP_FAIL_MARKER,
    FRAME_ALLOCATOR_FAIL_MARKER,
    PAGE_TABLE_FAIL_MARKER,
    PAGING_POLICY_MODEL_FAIL_MARKER,
    CPU_HARDENING_FAIL_MARKER,
    ENTROPY_POLICY_FAIL_MARKER,
    HEAP_FAIL_MARKER,
    CAP_TABLE_FAIL_MARKER,
    MEMORY_CAP_FAIL_MARKER,
    CAP_AUDIT_FAIL_MARKER,
    SERVICE_QUEUE_FAIL_MARKER,
    TASK_MODEL_FAIL_MARKER,
    COOPERATIVE_SCHED_FAIL_MARKER,
    SCHEDULER_TELEMETRY_FAIL_MARKER,
    TELEMETRY_EVENTS_FAIL_MARKER,
    AI_POLICY_FAIL_MARKER,
    KERNEL_CR3_FAIL_MARKER,
];

const PANIC_REQUIRED_MARKERS: &[&str] = &[
    CPU_SETUP_MARKER,
    EXCEPTION_SETUP_MARKER,
    IRQ_SETUP_MARKER,
    EXCEPTION_MARKER,
    PANIC_DIAGNOSTIC_MARKER,
    PANIC_MARKER,
    PANIC_REGISTERS_MARKER,
];

const EXCEPTION_REQUIRED_MARKERS: &[&str] = &[
    CPU_SETUP_MARKER,
    EXCEPTION_SETUP_MARKER,
    IRQ_SETUP_MARKER,
    EXCEPTION_MARKER,
    FAULT_ADDRESS_PRESENT_MARKER,
    FAULT_ADDRESS_MARKER,
    FAULT_CR3_MARKER,
    FAULT_RFLAGS_MARKER,
    FAULT_INTERRUPTS_MARKER,
    FAULT_ERROR_DECODE_MARKER,
    PAGE_FAULT_MARKER,
];

const TIMER_REQUIRED_MARKERS: &[&str] = &[
    CPU_SETUP_MARKER,
    EXCEPTION_SETUP_MARKER,
    IRQ_SETUP_MARKER,
    EXCEPTION_MARKER,
    TIMER_SETUP_MARKER,
    TIMER_TICK_1_MARKER,
    TIMER_TICK_2_MARKER,
    TIMER_TICK_3_MARKER,
    TIMER_DELAYED_LOG_MARKER,
    SLEEP_MARKER,
    TIMER_MARKER,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SmokeKind {
    Boot,
    Panic,
    Exception,
    Timer,
}

impl SmokeKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Boot => "boot",
            Self::Panic => "panic",
            Self::Exception => "exception",
            Self::Timer => "timer",
        }
    }

    pub fn markers(self) -> &'static str {
        match self {
            Self::Boot => {
                "[TEST] gdt=ok, [TEST] idt=ok, [TEST] irq=ok, [TEST] exception=ok, [kernel][INFO] bootinfo normalized, memory total_bytes=, memory usable_bytes=, memory reserved_bytes=, [TEST] memory-map=ok, frame-allocator total_frames=, [TEST] frame-allocator=ok, page-table total_tables=, root_ok=true, checked_root_ok=true, checked_status_ok=true, kernel_candidate_ok=true, user_candidate_ok=true, translate_offset_ok=true, checked_translate_ok=true, mapping_lookup_ok=true, presence_ok=true, protect_ok=true, protect_range_ok=true, range_lookup_ok=true, range_translate_ok=true, mapped_range_ok=true, unmapped_range_ok=true, kernel_range_ok=true, user_range_ok=true, write_protected_range_ok=true, non_executable_range_ok=true, executable_range_ok=true, normal_memory_range_ok=true, local_range_ok=true, kernel_space_range_ok=true, user_space_range_ok=true, no_user_space_ok=true, no_executable_ok=true, no_writable_ok=true, no_device_ok=true, no_global_ok=true, no_alias_ok=true, kernel_user_guard_ok=true, kernel_only_ok=true, audit_ok=true, visit_ok=true, flags_ok=true, reclaim_ok=true, range_ok=true, flush_page=true, [TEST] page-table=ok, paging-policy-model mapped_pages=, section_layout_ok=true, text_rx_ok=true, rodata_read_only_ok=true, data_rw_nx_ok=true, heap_reserved_ok=true, guard_page_ok=true, null_page_ok=true, hardware_image_ok=true, hardware_arena_frames=, hardware_root_allocated=true, hardware_tables_copied=, hardware_copied=true, kernel_stack_pages=, kernel_stack_guard_ok=true, [TEST] kernel-stack-guard=ok, [TEST] paging-policy-model=ok, [TEST] bootinfo=ok, [TEST] boot=ok, cpu-hardening nx=, [TEST] cpu-hardening=ok, entropy-policy rdrand=, hardware_self_test=false, hardware_present=, fallback_used=, generation_counter_ok=true, random_tokens_available=, source=, [TEST] entropy-policy=ok, heap bytes=, slab_classes=, slab_reuse_ok=true, page_run_ok=true, stress_ok=true, double_free_detected=true, invalid_free_detected=true, corrupt_free_list_detected=false, [TEST] heap=ok, cap-table capacity=, [TEST] cap=ok, memory-cap map_allowed=, [TEST] memory-cap=ok, cap-audit events=, [TEST] cap-audit=ok, service-queue log_submitted=, [TEST] service-queue=ok, task-model created=, [TEST] task-model=ok, cooperative-sched task_a=, [TEST] cooperative-sched=ok, scheduler-telemetry decisions=, [TEST] scheduler-telemetry=ok, telemetry-events schema=1 events=, trace-event schema=1 event=boot-phase, trace-event schema=1 event=capability-fault, trace-event schema=1 event=scheduler-decision, selected_task=<redacted>, [TEST] telemetry-events=ok, ai-policy schema=1, manifest_metadata_gate_ok=true, heuristic_enabled=true, heuristic_score=, heuristic_disabled_fallback_ok=true, [TEST] ai-policy=ok, kernel-cr3 active=true, [TEST] kernel-cr3=ok"
            }
            Self::Panic => {
                "[TEST] gdt=ok, [TEST] idt=ok, [TEST] irq=ok, [TEST] exception=ok, [kernel][FATAL] panic handler entered, panic registers=, [TEST] panic=ok"
            }
            Self::Exception => {
                "[TEST] gdt=ok, [TEST] idt=ok, [TEST] irq=ok, [TEST] exception=ok, cr2_present=, cr2_offset=0x, cr3_offset=0x, rflags=0x, interrupts_enabled=, present=, [TEST] pagefault=ok"
            }
            Self::Timer => {
                "[TEST] gdt=ok, [TEST] idt=ok, [TEST] irq=ok, [TEST] exception=ok, timer setup=pit, timer tick 1, timer tick 2, timer delayed-log, [TEST] sleep=ok, timer tick 3, [TEST] timer=ok"
            }
        }
    }

    pub(crate) fn required_markers(self) -> &'static [&'static str] {
        match self {
            Self::Boot => BOOT_REQUIRED_MARKERS,
            Self::Panic => PANIC_REQUIRED_MARKERS,
            Self::Exception => EXCEPTION_REQUIRED_MARKERS,
            Self::Timer => TIMER_REQUIRED_MARKERS,
        }
    }

    pub(crate) fn forbidden_markers(self) -> &'static [&'static str] {
        match self {
            Self::Boot => BOOT_FORBIDDEN_MARKERS,
            Self::Panic | Self::Exception | Self::Timer => &[],
        }
    }

    pub fn feature(self) -> Option<&'static str> {
        match self {
            Self::Boot => None,
            Self::Panic => Some("panic-smoke"),
            Self::Exception => Some("exception-smoke"),
            Self::Timer => Some("timer-smoke"),
        }
    }
}

pub fn serial_log_contains_marker(path: &Path, smoke: SmokeKind) -> bool {
    fs::read_to_string(path).is_ok_and(|contents| serial_log_contents_match(&contents, smoke))
}

pub(crate) fn serial_log_contents_match(contents: &str, smoke: SmokeKind) -> bool {
    contains_all(contents, smoke.required_markers())
        && contains_none(contents, smoke.forbidden_markers())
}

fn contains_all(contents: &str, markers: &[&str]) -> bool {
    markers
        .iter()
        .all(|marker| contains_marker(contents, marker))
}

fn contains_none(contents: &str, markers: &[&str]) -> bool {
    markers
        .iter()
        .all(|marker| !contains_marker(contents, marker))
}

fn contains_marker(contents: &str, marker: &str) -> bool {
    if !marker.contains('=') {
        return contents.contains(marker);
    }

    let mut offset = 0usize;
    while let Some(relative_start) = contents[offset..].find(marker) {
        let start = offset + relative_start;
        let end = start + marker.len();
        if is_marker_boundary(contents[..start].chars().next_back())
            && (is_value_prefix_marker(marker)
                || is_marker_boundary(contents[end..].chars().next()))
        {
            return true;
        }
        offset = end;
    }
    false
}

fn is_value_prefix_marker(marker: &str) -> bool {
    marker.ends_with('=') || marker.ends_with("=0x")
}

fn is_marker_boundary(character: Option<char>) -> bool {
    match character {
        None => true,
        Some(character) => character.is_ascii_whitespace() || character == ',',
    }
}

pub fn parse_qemu_args(args: &[String]) -> Result<SmokeKind, &'static str> {
    match args {
        [] => Ok(SmokeKind::Boot),
        [flag] if flag == "--panic-smoke" => Ok(SmokeKind::Panic),
        [flag] if flag == "--exception-smoke" => Ok(SmokeKind::Exception),
        [flag] if flag == "--timer-smoke" => Ok(SmokeKind::Timer),
        _ => Err(
            "qemu accepts no arguments except --panic-smoke, --exception-smoke, or --timer-smoke",
        ),
    }
}
