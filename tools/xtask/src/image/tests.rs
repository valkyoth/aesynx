use super::smoke::{
    BOOT_DIAGNOSTIC_MARKER, BOOTINFO_FAIL_MARKER, BOOTINFO_MARKER, CPU_SETUP_MARKER,
    EXCEPTION_MARKER, EXCEPTION_SETUP_MARKER, FAULT_ADDRESS_MARKER, FAULT_ADDRESS_PRESENT_MARKER,
    FAULT_CR3_MARKER, FAULT_ERROR_DECODE_MARKER, FAULT_INTERRUPTS_MARKER, FAULT_RFLAGS_MARKER,
    FRAME_ALLOCATOR_FAIL_MARKER, FRAME_ALLOCATOR_MARKER, FRAME_ALLOCATOR_STATUS_MARKER,
    IRQ_SETUP_MARKER, MEMORY_MAP_FAIL_MARKER, MEMORY_MAP_MARKER, MEMORY_RESERVED_MARKER,
    MEMORY_TOTAL_MARKER, MEMORY_USABLE_MARKER, PAGE_FAULT_MARKER, PAGE_TABLE_AUDIT_MARKER,
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
    PAGE_TABLE_WRITE_PROTECTED_RANGE_MARKER, PAGING_POLICY_FAIL_MARKER, PAGING_POLICY_MARKER,
    PAGING_POLICY_STATUS_MARKER, PANIC_DIAGNOSTIC_MARKER, PANIC_MARKER, PANIC_REGISTERS_MARKER,
    SERIAL_MARKER, SLEEP_MARKER, SmokeKind, TIMER_DELAYED_LOG_MARKER, TIMER_MARKER,
    TIMER_SETUP_MARKER, TIMER_TICK_1_MARKER, TIMER_TICK_2_MARKER, TIMER_TICK_3_MARKER,
    parse_qemu_args, serial_log_contents_match,
};
use super::{BOOT_CONFIG_MARKERS, KERNEL_PROFILE, KERNEL_TARGET};
use crate::image::host_tools::HostToolVersions;
use crate::image::manifest::write_manifest;
use std::fs;
use std::path::PathBuf;

#[test]
fn qemu_markers_track_v0_16_contracts() {
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
    assert_eq!(PAGING_POLICY_FAIL_MARKER, "[TEST] paging-policy=fail");
    assert_eq!(PAGING_POLICY_MARKER, "[TEST] paging-policy=ok");
    assert_eq!(PAGING_POLICY_STATUS_MARKER, "paging-policy mapped_pages=");
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
fn boot_smoke_requires_full_v0_16_marker_set() {
    assert_smoke_contract_requires_each_marker(SmokeKind::Boot);

    let valid = SmokeKind::Boot.markers();
    let missing_root_only = valid.replacen("root_ok=true, ", "", 1);
    assert!(!serial_log_contents_match(
        &missing_root_only,
        SmokeKind::Boot
    ));
}

#[test]
fn diagnostic_smokes_require_each_declared_marker() {
    assert_smoke_contract_requires_each_marker(SmokeKind::Panic);
    assert_smoke_contract_requires_each_marker(SmokeKind::Exception);
    assert_smoke_contract_requires_each_marker(SmokeKind::Timer);
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

#[test]
fn image_manifest_records_required_smoke_markers() -> Result<(), String> {
    let manifest = temp_manifest_path("required-smoke-markers");
    let host_tools = HostToolVersions {
        rustc: String::from("rustc test"),
        cargo: String::from("cargo test"),
        limine: String::from("limine test"),
        xorriso: String::from("xorriso test"),
        qemu: String::from("qemu test"),
    };

    write_manifest(
        &manifest,
        &PathBuf::from("/tmp/aesynx.iso"),
        &PathBuf::from("/tmp/aesynx-kernel"),
        &host_tools,
        SmokeKind::Panic,
    )?;

    let contents = fs::read_to_string(&manifest)
        .map_err(|error| format!("failed to read manifest test output: {error}"))?;
    let _ = fs::remove_file(&manifest);

    assert!(contents.contains("smoke=panic\n"));
    for smoke in [
        SmokeKind::Boot,
        SmokeKind::Panic,
        SmokeKind::Exception,
        SmokeKind::Timer,
    ] {
        for marker in smoke.required_markers() {
            assert!(
                contents.contains(marker),
                "manifest does not record required {} smoke marker: {marker}",
                smoke.name()
            );
        }

        for marker in smoke.forbidden_markers() {
            assert!(
                !contents.contains(marker),
                "manifest records forbidden {} smoke marker: {marker}",
                smoke.name()
            );
        }
    }
    Ok(())
}

fn assert_smoke_contract_requires_each_marker(smoke: SmokeKind) {
    let valid = smoke.markers();
    assert!(
        serial_log_contents_match(valid, smoke),
        "{} smoke marker string is not accepted by its validator",
        smoke.name()
    );

    for marker in valid.split(", ") {
        let missing = valid.replace(marker, "");
        assert!(
            !serial_log_contents_match(&missing, smoke),
            "{} smoke accepted output without marker {marker}",
            smoke.name()
        );
    }
}

fn temp_manifest_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("aesynx-{name}-{}.manifest", std::process::id()))
}
