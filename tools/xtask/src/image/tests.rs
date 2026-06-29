use super::names::image_names;
use super::smoke::{SmokeKind, parse_qemu_args, serial_log_contents_match};
use super::{BOOT_CONFIG_MARKERS, KERNEL_PROFILE, KERNEL_TARGET, QEMU_SMP_CPUS};
use crate::image::host_tools::HostToolVersions;
use crate::image::manifest::write_manifest;
use std::fs;
use std::path::PathBuf;

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
fn boot_smoke_requires_full_current_marker_set() {
    assert_smoke_contract_requires_each_marker(SmokeKind::Boot);

    let valid = SmokeKind::Boot.markers();
    let missing_root_only = valid.replacen("root_ok=true, ", "", 1);
    assert!(!serial_log_contents_match(
        &missing_root_only,
        SmokeKind::Boot
    ));
    let missing_kernel_cr3 = valid.replacen("[TEST] kernel-cr3=ok", "", 1);
    assert!(!serial_log_contents_match(
        &missing_kernel_cr3,
        SmokeKind::Boot
    ));
    let failed_kernel_cr3 = format!("{valid}, [TEST] kernel-cr3=fail");
    assert!(!serial_log_contents_match(
        &failed_kernel_cr3,
        SmokeKind::Boot
    ));
    let missing_cpu_hardening = valid.replacen("[TEST] cpu-hardening=ok", "", 1);
    assert!(!serial_log_contents_match(
        &missing_cpu_hardening,
        SmokeKind::Boot
    ));
    let failed_cpu_hardening = format!("{valid}, [TEST] cpu-hardening=fail");
    assert!(!serial_log_contents_match(
        &failed_cpu_hardening,
        SmokeKind::Boot
    ));
    let missing_entropy = valid.replacen("[TEST] entropy-policy=ok", "", 1);
    assert!(!serial_log_contents_match(
        &missing_entropy,
        SmokeKind::Boot
    ));
    let failed_entropy = format!("{valid}, [TEST] entropy-policy=fail");
    assert!(!serial_log_contents_match(&failed_entropy, SmokeKind::Boot));
    let missing_entropy_generation = valid.replacen("generation_counter_ok=true", "", 1);
    assert!(!serial_log_contents_match(
        &missing_entropy_generation,
        SmokeKind::Boot
    ));
    let missing_entropy_self_test = valid.replacen("hardware_self_test=false", "", 1);
    assert!(!serial_log_contents_match(
        &missing_entropy_self_test,
        SmokeKind::Boot
    ));
    let missing_heap = valid.replacen("[TEST] heap=ok", "", 1);
    assert!(!serial_log_contents_match(&missing_heap, SmokeKind::Boot));
    let failed_heap = format!("{valid}, [TEST] heap=fail");
    assert!(!serial_log_contents_match(&failed_heap, SmokeKind::Boot));
    let missing_heap_double_free = valid.replacen("double_free_detected=true", "", 1);
    assert!(!serial_log_contents_match(
        &missing_heap_double_free,
        SmokeKind::Boot
    ));
    let missing_heap_invalid_free = valid.replacen("invalid_free_detected=true", "", 1);
    assert!(!serial_log_contents_match(
        &missing_heap_invalid_free,
        SmokeKind::Boot
    ));
    let missing_heap_accounting_overflow =
        valid.replacen("accounting_overflow_detected=false", "", 1);
    assert!(!serial_log_contents_match(
        &missing_heap_accounting_overflow,
        SmokeKind::Boot
    ));
    let missing_heap_corrupt_free_list = valid.replacen("corrupt_free_list_detected=false", "", 1);
    assert!(!serial_log_contents_match(
        &missing_heap_corrupt_free_list,
        SmokeKind::Boot
    ));
    let missing_stack_guard = valid.replacen("[TEST] kernel-stack-guard=ok", "", 1);
    assert!(!serial_log_contents_match(
        &missing_stack_guard,
        SmokeKind::Boot
    ));
    let missing_ap_dispatch_authority = valid.replacen("ap_dispatch_token_blocked_ok=true", "", 1);
    assert!(!serial_log_contents_match(
        &missing_ap_dispatch_authority,
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
fn qemu_smoke_runs_four_virtual_cpus() {
    assert_eq!(QEMU_SMP_CPUS, aesynx_core::QEMU_MULTICORE_TOPOLOGY_CORES);
    assert_eq!(QEMU_SMP_CPUS, 4);
}

#[test]
fn image_artifact_names_track_current_candidate_version() {
    let boot = image_names(SmokeKind::Boot);
    assert_eq!(boot.image, "aesynx-v0.36.0.iso");
    assert_eq!(boot.manifest, "aesynx-v0.36.0.manifest");
    assert_eq!(boot.serial_log, "aesynx-v0.36.0.serial.log");
    assert_eq!(boot.staging_dir, "aesynx-v0.36.0-iso");

    let panic = image_names(SmokeKind::Panic);
    assert_eq!(panic.image, "aesynx-v0.36.0-panic.iso");
    assert_eq!(panic.manifest, "aesynx-v0.36.0-panic.manifest");
    assert_eq!(panic.serial_log, "aesynx-v0.36.0-panic.serial.log");
    assert_eq!(panic.staging_dir, "aesynx-v0.36.0-panic-iso");

    let exception = image_names(SmokeKind::Exception);
    assert_eq!(exception.image, "aesynx-v0.36.0-exception.iso");
    assert_eq!(exception.manifest, "aesynx-v0.36.0-exception.manifest");
    assert_eq!(exception.serial_log, "aesynx-v0.36.0-exception.serial.log");
    assert_eq!(exception.staging_dir, "aesynx-v0.36.0-exception-iso");

    let timer = image_names(SmokeKind::Timer);
    assert_eq!(timer.image, "aesynx-v0.36.0-timer.iso");
    assert_eq!(timer.manifest, "aesynx-v0.36.0-timer.manifest");
    assert_eq!(timer.serial_log, "aesynx-v0.36.0-timer.serial.log");
    assert_eq!(timer.staging_dir, "aesynx-v0.36.0-timer-iso");
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

    assert!(contents.contains("name=Aesynx v0.36.0 core-to-core ping/pong candidate\n"));
    assert!(contents.contains("multicore_topology_state_table_marker=state_table_ok=true\n"));
    assert!(contents.contains("multicore_topology_ap_preflight_marker=ap_preflight_ok=true\n"));
    assert!(
        contents.contains(
            "multicore_topology_ap_execution_blocked_marker=ap_execution_blocked_ok=true\n"
        )
    );
    assert!(contents.contains(
        "multicore_topology_ap_dispatch_token_blocked_marker=ap_dispatch_token_blocked_ok=true\n"
    ));
    assert!(contents.contains("ipc_pingpong_status_marker=ipc-pingpong ping_seq=\n"));
    assert!(contents.contains("ipc_pingpong_backpressure_marker=backpressure_ok=true\n"));
    assert!(contents.contains("ipc_pingpong_release_acquire_marker=release_acquire_ok=true\n"));
    assert!(contents.contains("ipc_pingpong_pairwise_marker=pairwise_route_ok=true\n"));
    assert!(contents.contains("ipc_pingpong_marker=[TEST] ipc-pingpong=ok\n"));
    assert!(contents.contains("cpu_hardening_ibpb_attempted_marker=ibpb_attempted=\n"));
    assert!(contents.contains("smoke=panic\n"));
    assert!(contents.contains("qemu_smp_cpus=4\n"));
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

#[test]
fn image_manifest_rejects_newline_path_fields() {
    let manifest = temp_manifest_path("newline-path");
    let host_tools = test_host_tools();

    let result = write_manifest(
        &manifest,
        &PathBuf::from("/tmp/aesynx.iso\nmulticore_topology_startup_evidence_marker=true"),
        &PathBuf::from("/tmp/aesynx-kernel"),
        &host_tools,
        SmokeKind::Boot,
    );
    let _ = fs::remove_file(&manifest);

    assert!(result.is_err());

    let result = write_manifest(
        &manifest,
        &PathBuf::from("/tmp/aesynx.iso"),
        &PathBuf::from("/tmp/aesynx-kernel\rkernel_cr3_marker=true"),
        &host_tools,
        SmokeKind::Boot,
    );
    let _ = fs::remove_file(&manifest);

    assert!(result.is_err());
}

#[test]
fn image_manifest_rejects_control_and_separator_fields() {
    let manifest = temp_manifest_path("control-path");

    for image in [
        "/tmp/aesynx.iso\0truncated",
        "/tmp/aesynx.iso\x0bvertical-tab",
        "/tmp/aesynx.iso\x0cform-feed",
        "/tmp/aesynx.iso\u{2028}unicode-line-separator",
        "/tmp/aesynx.iso\u{2029}unicode-paragraph-separator",
        "/tmp/aesynx.iso=ambiguous",
    ] {
        let result = write_manifest(
            &manifest,
            &PathBuf::from(image),
            &PathBuf::from("/tmp/aesynx-kernel"),
            &test_host_tools(),
            SmokeKind::Boot,
        );
        let _ = fs::remove_file(&manifest);

        assert!(result.is_err());
    }

    let host_tools = HostToolVersions {
        rustc: String::from("rustc test\0truncated"),
        cargo: String::from("cargo test"),
        limine: String::from("limine test"),
        xorriso: String::from("xorriso test"),
        qemu: String::from("qemu test"),
    };
    let result = write_manifest(
        &manifest,
        &PathBuf::from("/tmp/aesynx.iso"),
        &PathBuf::from("/tmp/aesynx-kernel"),
        &host_tools,
        SmokeKind::Boot,
    );
    let _ = fs::remove_file(&manifest);

    assert!(result.is_err());
}

fn test_host_tools() -> HostToolVersions {
    HostToolVersions {
        rustc: String::from("rustc test"),
        cargo: String::from("cargo test"),
        limine: String::from("limine test"),
        xorriso: String::from("xorriso test"),
        qemu: String::from("qemu test"),
    }
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
