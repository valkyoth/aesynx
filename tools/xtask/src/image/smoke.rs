pub const BOOTINFO_FAIL_MARKER: &str = "[TEST] bootinfo=fail";
pub const BOOTINFO_MARKER: &str = "[TEST] bootinfo=ok";
pub const BOOT_DIAGNOSTIC_MARKER: &str = "[kernel][INFO] bootinfo normalized";
pub const CPU_SETUP_MARKER: &str = "[TEST] gdt=ok";
pub const EXCEPTION_SETUP_MARKER: &str = "[TEST] idt=ok";
pub const EXCEPTION_MARKER: &str = "[TEST] exception=ok";
pub const FRAME_ALLOCATOR_FAIL_MARKER: &str = "[TEST] frame-allocator=fail";
pub const FRAME_ALLOCATOR_MARKER: &str = "[TEST] frame-allocator=ok";
pub const FRAME_ALLOCATOR_STATUS_MARKER: &str = "frame-allocator total_frames=";
pub const IRQ_SETUP_MARKER: &str = "[TEST] irq=ok";
pub const MEMORY_MAP_FAIL_MARKER: &str = "[TEST] memory-map=fail";
pub const MEMORY_MAP_MARKER: &str = "[TEST] memory-map=ok";
pub const MEMORY_RESERVED_MARKER: &str = "memory reserved_bytes=";
pub const MEMORY_TOTAL_MARKER: &str = "memory total_bytes=";
pub const MEMORY_USABLE_MARKER: &str = "memory usable_bytes=";
pub const FAULT_ADDRESS_MARKER: &str = "cr2_offset=0x";
pub const FAULT_ADDRESS_PRESENT_MARKER: &str = "cr2_present=";
pub const FAULT_CR3_MARKER: &str = "cr3_offset=0x";
pub const FAULT_ERROR_DECODE_MARKER: &str = "present=";
pub const FAULT_INTERRUPTS_MARKER: &str = "interrupts_enabled=";
pub const FAULT_RFLAGS_MARKER: &str = "rflags=0x";
pub const PAGE_FAULT_MARKER: &str = "[TEST] pagefault=ok";
pub const PAGE_TABLE_AUDIT_MARKER: &str = "audit_ok=true";
pub const PAGE_TABLE_FAIL_MARKER: &str = "[TEST] page-table=fail";
pub const PAGE_TABLE_FLAGS_MARKER: &str = "flags_ok=true";
pub const PAGE_TABLE_LOOKUP_MARKER: &str = "mapping_lookup_ok=true";
pub const PAGE_TABLE_MARKER: &str = "[TEST] page-table=ok";
pub const PAGE_TABLE_PROTECT_MARKER: &str = "protect_ok=true";
pub const PAGE_TABLE_PROTECT_RANGE_MARKER: &str = "protect_range_ok=true";
pub const PAGE_TABLE_RANGE_LOOKUP_MARKER: &str = "range_lookup_ok=true";
pub const PAGE_TABLE_RANGE_MARKER: &str = "range_ok=true";
pub const PAGE_TABLE_RECLAIM_MARKER: &str = "reclaim_ok=true";
pub const PAGE_TABLE_STATUS_MARKER: &str = "page-table total_tables=";
pub const PAGE_TABLE_UNMAPPED_RANGE_MARKER: &str = "unmapped_range_ok=true";
pub const PAGE_TABLE_VISIT_MARKER: &str = "visit_ok=true";
pub const PANIC_DIAGNOSTIC_MARKER: &str = "[kernel][FATAL] panic handler entered";
pub const PANIC_MARKER: &str = "[TEST] panic=ok";
pub const PANIC_REGISTERS_MARKER: &str = "panic registers=";
pub const SERIAL_MARKER: &str = "[TEST] boot=ok";
pub const SLEEP_MARKER: &str = "[TEST] sleep=ok";
pub const TIMER_DELAYED_LOG_MARKER: &str = "timer delayed-log";
pub const TIMER_MARKER: &str = "[TEST] timer=ok";
pub const TIMER_SETUP_MARKER: &str = "timer setup=pit";
pub const TIMER_TICK_1_MARKER: &str = "timer tick 1";
pub const TIMER_TICK_2_MARKER: &str = "timer tick 2";
pub const TIMER_TICK_3_MARKER: &str = "timer tick 3";

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
                "[TEST] gdt=ok, [TEST] idt=ok, [TEST] irq=ok, [TEST] exception=ok, [kernel][INFO] bootinfo normalized, memory total_bytes=, memory usable_bytes=, memory reserved_bytes=, [TEST] memory-map=ok, frame-allocator total_frames=, [TEST] frame-allocator=ok, page-table total_tables=, mapping_lookup_ok=true, protect_ok=true, protect_range_ok=true, range_lookup_ok=true, unmapped_range_ok=true, audit_ok=true, visit_ok=true, flags_ok=true, reclaim_ok=true, range_ok=true, [TEST] page-table=ok, [TEST] bootinfo=ok, [TEST] boot=ok"
            }
            Self::Panic => {
                "[TEST] gdt=ok, [TEST] idt=ok, [TEST] irq=ok, [TEST] exception=ok, [kernel][FATAL] panic handler entered, panic registers=, [TEST] panic=ok"
            }
            Self::Exception => {
                "[TEST] gdt=ok, [TEST] idt=ok, [TEST] irq=ok, [TEST] exception=ok, cr2_present=, cr2_offset=0x, cr3_offset=0x, rflags=0x, interrupts_enabled=, present=, [TEST] pagefault=ok"
            }
            Self::Timer => {
                "[TEST] gdt=ok, [TEST] idt=ok, [TEST] irq=ok, [TEST] exception=ok, timer tick 1, timer tick 2, timer delayed-log, [TEST] sleep=ok, timer tick 3, [TEST] timer=ok"
            }
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
