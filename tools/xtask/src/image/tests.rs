use super::{
    BOOT_CONFIG_MARKERS, BOOT_DIAGNOSTIC_MARKER, BOOTINFO_FAIL_MARKER, BOOTINFO_MARKER,
    CPU_SETUP_MARKER, EXCEPTION_MARKER, EXCEPTION_SETUP_MARKER, FAULT_ADDRESS_MARKER,
    FAULT_ADDRESS_PRESENT_MARKER, FAULT_CR3_MARKER, FAULT_ERROR_DECODE_MARKER,
    FAULT_INTERRUPTS_MARKER, FAULT_RFLAGS_MARKER, FRAME_ALLOCATOR_FAIL_MARKER,
    FRAME_ALLOCATOR_MARKER, FRAME_ALLOCATOR_STATUS_MARKER, IRQ_SETUP_MARKER, KERNEL_PROFILE,
    KERNEL_TARGET, MEMORY_MAP_FAIL_MARKER, MEMORY_MAP_MARKER, MEMORY_RESERVED_MARKER,
    MEMORY_TOTAL_MARKER, MEMORY_USABLE_MARKER, PAGE_FAULT_MARKER, PAGE_TABLE_FAIL_MARKER,
    PAGE_TABLE_MARKER, PAGE_TABLE_STATUS_MARKER, PANIC_DIAGNOSTIC_MARKER, PANIC_MARKER,
    PANIC_REGISTERS_MARKER, SERIAL_MARKER, SLEEP_MARKER, SmokeKind, TIMER_DELAYED_LOG_MARKER,
    TIMER_MARKER, TIMER_SETUP_MARKER, TIMER_TICK_1_MARKER, TIMER_TICK_2_MARKER,
    TIMER_TICK_3_MARKER, parse_qemu_args,
};

#[test]
fn qemu_markers_track_v0_15_contracts() {
    assert_eq!(BOOTINFO_FAIL_MARKER, "[TEST] bootinfo=fail");
    assert_eq!(BOOTINFO_MARKER, "[TEST] bootinfo=ok");
    assert_eq!(BOOT_DIAGNOSTIC_MARKER, "[kernel][INFO] bootinfo normalized");
    assert_eq!(CPU_SETUP_MARKER, "[TEST] gdt=ok");
    assert_eq!(EXCEPTION_SETUP_MARKER, "[TEST] idt=ok");
    assert_eq!(EXCEPTION_MARKER, "[TEST] exception=ok");
    assert_eq!(FRAME_ALLOCATOR_FAIL_MARKER, "[TEST] frame-allocator=fail");
    assert_eq!(FRAME_ALLOCATOR_MARKER, "[TEST] frame-allocator=ok");
    assert_eq!(
        FRAME_ALLOCATOR_STATUS_MARKER,
        "frame-allocator total_frames="
    );
    assert_eq!(IRQ_SETUP_MARKER, "[TEST] irq=ok");
    assert_eq!(MEMORY_MAP_FAIL_MARKER, "[TEST] memory-map=fail");
    assert_eq!(MEMORY_MAP_MARKER, "[TEST] memory-map=ok");
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
    assert_eq!(PAGE_TABLE_MARKER, "[TEST] page-table=ok");
    assert_eq!(PAGE_TABLE_STATUS_MARKER, "page-table total_tables=");
    assert_eq!(
        PANIC_DIAGNOSTIC_MARKER,
        "[kernel][FATAL] panic handler entered"
    );
    assert_eq!(PANIC_MARKER, "[TEST] panic=ok");
    assert_eq!(PANIC_REGISTERS_MARKER, "panic registers=");
    assert_eq!(SERIAL_MARKER, "[TEST] boot=ok");
    assert_eq!(TIMER_SETUP_MARKER, "timer setup=pit");
    assert_eq!(TIMER_TICK_1_MARKER, "timer tick 1");
    assert_eq!(TIMER_TICK_2_MARKER, "timer tick 2");
    assert_eq!(TIMER_TICK_3_MARKER, "timer tick 3");
    assert_eq!(TIMER_DELAYED_LOG_MARKER, "timer delayed-log");
    assert_eq!(SLEEP_MARKER, "[TEST] sleep=ok");
    assert_eq!(TIMER_MARKER, "[TEST] timer=ok");
}

#[test]
fn qemu_args_select_smoke_kind() {
    assert_eq!(parse_qemu_args(&[]), Ok(SmokeKind::Boot));
    assert_eq!(
        parse_qemu_args(&[String::from("--panic-smoke")]),
        Ok(SmokeKind::Panic)
    );
    assert_eq!(
        parse_qemu_args(&[String::from("--exception-smoke")]),
        Ok(SmokeKind::Exception)
    );
    assert_eq!(
        parse_qemu_args(&[String::from("--timer-smoke")]),
        Ok(SmokeKind::Timer)
    );
    assert_eq!(
        parse_qemu_args(&[String::from("--unknown")]),
        Err("qemu accepts no arguments except --panic-smoke, --exception-smoke, or --timer-smoke")
    );
}

#[test]
fn kernel_target_is_stable_freestanding_target() {
    assert_eq!(KERNEL_TARGET, "x86_64-unknown-none");
}

#[test]
fn image_kernel_profile_is_release() {
    assert_eq!(KERNEL_PROFILE, "release");
}

#[test]
fn boot_config_markers_cover_limine_kernel_path() {
    assert!(
        BOOT_CONFIG_MARKERS
            .iter()
            .any(|marker| marker.contains("protocol: limine"))
    );
    assert!(
        BOOT_CONFIG_MARKERS
            .iter()
            .any(|marker| marker.contains("path: boot():/boot/aesynx-kernel"))
    );
}
