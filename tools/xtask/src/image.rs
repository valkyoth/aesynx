mod host_tools;
mod names;
mod smoke;

use crate::kernel_flags::apply_kernel_rustflags;
use crate::workspace;
use host_tools::{HostToolVersions, MIN_LIMINE_VERSION_TEXT, validate_host_tools};
use names::image_names;
use smoke::{
    BOOT_DIAGNOSTIC_MARKER, BOOTINFO_FAIL_MARKER, BOOTINFO_MARKER, CPU_SETUP_MARKER,
    EXCEPTION_MARKER, EXCEPTION_SETUP_MARKER, FAULT_ADDRESS_MARKER, FAULT_ADDRESS_PRESENT_MARKER,
    FAULT_CR3_MARKER, FAULT_ERROR_DECODE_MARKER, FAULT_INTERRUPTS_MARKER, FAULT_RFLAGS_MARKER,
    FRAME_ALLOCATOR_FAIL_MARKER, FRAME_ALLOCATOR_MARKER, FRAME_ALLOCATOR_STATUS_MARKER,
    IRQ_SETUP_MARKER, MEMORY_MAP_FAIL_MARKER, MEMORY_MAP_MARKER, MEMORY_RESERVED_MARKER,
    MEMORY_TOTAL_MARKER, MEMORY_USABLE_MARKER, PAGE_FAULT_MARKER, PAGE_TABLE_AUDIT_MARKER,
    PAGE_TABLE_FAIL_MARKER, PAGE_TABLE_FLAGS_MARKER, PAGE_TABLE_LOOKUP_MARKER,
    PAGE_TABLE_MAPPED_RANGE_MARKER, PAGE_TABLE_MARKER, PAGE_TABLE_PRESENCE_MARKER,
    PAGE_TABLE_PROTECT_MARKER, PAGE_TABLE_PROTECT_RANGE_MARKER, PAGE_TABLE_RANGE_LOOKUP_MARKER,
    PAGE_TABLE_RANGE_MARKER, PAGE_TABLE_RECLAIM_MARKER, PAGE_TABLE_STATUS_MARKER,
    PAGE_TABLE_UNMAPPED_RANGE_MARKER, PAGE_TABLE_VISIT_MARKER, PANIC_DIAGNOSTIC_MARKER,
    PANIC_MARKER, PANIC_REGISTERS_MARKER, SERIAL_MARKER, SLEEP_MARKER, SmokeKind,
    TIMER_DELAYED_LOG_MARKER, TIMER_MARKER, TIMER_SETUP_MARKER, TIMER_TICK_1_MARKER,
    TIMER_TICK_2_MARKER, TIMER_TICK_3_MARKER, parse_qemu_args,
};

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::thread;
use std::time::{Duration, Instant};

const BOOT_CONFIG: &str = "boot/qemu/limine.conf";
const BUILD_DIR: &str = "build/qemu";
const KERNEL_TARGET: &str = "x86_64-unknown-none";
const KERNEL_PACKAGE: &str = "aesynx-kernel";
const KERNEL_BINARY: &str = "aesynx-kernel";
const KERNEL_PROFILE: &str = "release";
const QEMU_TIMEOUT: Duration = Duration::from_secs(5);

const BOOT_CONFIG_MARKERS: &[&str] = &[
    "serial: yes",
    "/Aesynx",
    "protocol: limine",
    "path: boot():/boot/aesynx-kernel",
    "kaslr: yes",
];

pub fn build(args: &[String]) -> ExitCode {
    if !args.is_empty() {
        eprintln!("xtask: image accepts no arguments");
        return ExitCode::from(2);
    }

    let root = match workspace::root() {
        Ok(root) => root,
        Err(error) => {
            eprintln!("xtask: {error}");
            return ExitCode::FAILURE;
        }
    };

    match build_image(&root, SmokeKind::Boot) {
        Ok(paths) => {
            println!("xtask: wrote QEMU Limine image: {}", paths.image.display());
            println!("xtask: wrote image manifest: {}", paths.manifest.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("xtask: {error}");
            ExitCode::FAILURE
        }
    }
}

pub fn qemu(args: &[String]) -> ExitCode {
    let smoke = match parse_qemu_args(args) {
        Ok(smoke) => smoke,
        Err(error) => {
            eprintln!("xtask: {error}");
            return ExitCode::from(2);
        }
    };

    let root = match workspace::root() {
        Ok(root) => root,
        Err(error) => {
            eprintln!("xtask: {error}");
            return ExitCode::FAILURE;
        }
    };

    let paths = match build_image(&root, smoke) {
        Ok(paths) => paths,
        Err(error) => {
            eprintln!("xtask: {error}");
            return ExitCode::FAILURE;
        }
    };

    match run_qemu(&paths, smoke) {
        Ok(()) => {
            println!(
                "xtask: QEMU {} smoke saw serial markers: {}",
                smoke.name(),
                smoke.markers()
            );
            println!("xtask: serial log: {}", paths.serial_log.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("xtask: {error}");
            ExitCode::FAILURE
        }
    }
}

struct ImagePaths {
    image: PathBuf,
    manifest: PathBuf,
    serial_log: PathBuf,
    staging_dir: PathBuf,
    kernel_elf: PathBuf,
}

fn build_image(root: &Path, smoke: SmokeKind) -> Result<ImagePaths, String> {
    validate_boot_config(root)?;
    let host_tools = validate_host_tools()?;

    let output_dir = root.join(BUILD_DIR);
    fs::create_dir_all(&output_dir)
        .map_err(|error| format!("failed to create {}: {error}", output_dir.display()))?;

    let names = image_names(smoke);
    let image = output_dir.join(names.image);
    let manifest = output_dir.join(names.manifest);
    let serial_log = output_dir.join(names.serial_log);
    let staging_dir = output_dir.join(names.staging_dir);
    let kernel_elf = build_kernel_elf(root, smoke)?;

    prepare_staging(root, &staging_dir, &kernel_elf)?;
    create_limine_iso(&staging_dir, &image)?;
    install_limine_bios(&image)?;
    write_manifest(&manifest, &image, &kernel_elf, &host_tools, smoke)?;

    Ok(ImagePaths {
        image,
        manifest,
        serial_log,
        staging_dir,
        kernel_elf,
    })
}

fn validate_boot_config(root: &Path) -> Result<(), String> {
    let path = root.join(BOOT_CONFIG);
    let contents = fs::read_to_string(&path)
        .map_err(|error| format!("required boot config unavailable: {BOOT_CONFIG}: {error}"))?;

    for marker in BOOT_CONFIG_MARKERS {
        if !contents.contains(marker) {
            return Err(format!(
                "{BOOT_CONFIG} is missing expected marker: {marker}"
            ));
        }
    }

    Ok(())
}

fn build_kernel_elf(root: &Path, smoke: SmokeKind) -> Result<PathBuf, String> {
    let mut command = Command::new("cargo");
    command.args([
        "build",
        "--target",
        KERNEL_TARGET,
        "-p",
        KERNEL_PACKAGE,
        "--bin",
        KERNEL_BINARY,
        "--release",
    ]);
    if let Some(feature) = smoke.feature() {
        command.args(["--features", feature]);
    }
    command.current_dir(root);
    apply_kernel_rustflags(&mut command, root);
    run_status(
        &mut command,
        "cargo build --target x86_64-unknown-none -p aesynx-kernel --bin aesynx-kernel --release",
    )?;

    let kernel = root
        .join("target")
        .join(KERNEL_TARGET)
        .join(KERNEL_PROFILE)
        .join(KERNEL_BINARY);
    if !kernel.is_file() {
        return Err(format!("kernel ELF was not produced: {}", kernel.display()));
    }

    Ok(kernel)
}

fn prepare_staging(root: &Path, staging_dir: &Path, kernel_elf: &Path) -> Result<(), String> {
    if staging_dir.exists() {
        fs::remove_dir_all(staging_dir)
            .map_err(|error| format!("failed to clear {}: {error}", staging_dir.display()))?;
    }

    let boot_dir = staging_dir.join("boot");
    let limine_dir = boot_dir.join("limine");
    let efi_dir = staging_dir.join("EFI").join("BOOT");
    fs::create_dir_all(&limine_dir)
        .map_err(|error| format!("failed to create {}: {error}", limine_dir.display()))?;
    fs::create_dir_all(&efi_dir)
        .map_err(|error| format!("failed to create {}: {error}", efi_dir.display()))?;

    copy_file(
        &root.join(BOOT_CONFIG),
        &limine_dir.join("limine.conf"),
        "Limine config",
    )?;
    copy_file(kernel_elf, &boot_dir.join(KERNEL_BINARY), "kernel ELF")?;

    let limine_data = limine_data_dir()?;
    for file in [
        "limine-bios.sys",
        "limine-bios-cd.bin",
        "limine-uefi-cd.bin",
    ] {
        copy_file(
            &limine_data.join(file),
            &limine_dir.join(file),
            "Limine bootloader asset",
        )?;
    }
    copy_file(
        &limine_data.join("BOOTX64.EFI"),
        &efi_dir.join("BOOTX64.EFI"),
        "Limine UEFI loader",
    )?;

    Ok(())
}

fn limine_data_dir() -> Result<PathBuf, String> {
    let output = Command::new("limine")
        .arg("--print-datadir")
        .output()
        .map_err(|error| format!("failed to query Limine data dir: {error}"))?;
    if !output.status.success() {
        return Err(command_error("limine --print-datadir", output));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|error| format!("Limine data dir was not valid UTF-8: {error}"))?;
    let path = stdout.trim();
    if path.is_empty() {
        return Err(String::from("Limine data dir was empty"));
    }

    Ok(PathBuf::from(path))
}

fn create_limine_iso(staging_dir: &Path, image: &Path) -> Result<(), String> {
    let _ = fs::remove_file(image);

    let mut command = Command::new("xorriso");
    command.args([
        "-as",
        "mkisofs",
        "-R",
        "-r",
        "-J",
        "-b",
        "boot/limine/limine-bios-cd.bin",
        "-no-emul-boot",
        "-boot-load-size",
        "4",
        "-boot-info-table",
        "-hfsplus",
        "-apm-block-size",
        "2048",
        "--efi-boot",
        "boot/limine/limine-uefi-cd.bin",
        "-efi-boot-part",
        "--efi-boot-image",
        "--protective-msdos-label",
    ]);
    command.arg(staging_dir);
    command.arg("-o");
    command.arg(image);
    run_status(&mut command, "xorriso Limine ISO creation")
}

fn install_limine_bios(image: &Path) -> Result<(), String> {
    let mut command = Command::new("limine");
    command.arg("bios-install");
    command.arg(image);
    run_status(&mut command, "limine bios-install")
}

fn write_manifest(
    manifest: &Path,
    image: &Path,
    kernel_elf: &Path,
    host_tools: &HostToolVersions,
    smoke: SmokeKind,
) -> Result<(), String> {
    let manifest_contents = format!(
        "name=Aesynx v0.15.0 page table mapper\nsmoke={}\nimage={}\nformat=iso\nbootloader=limine\nkernel={}\nkernel_target={KERNEL_TARGET}\nkernel_profile={KERNEL_PROFILE}\ncpu_setup_marker={CPU_SETUP_MARKER}\nexception_setup_marker={EXCEPTION_SETUP_MARKER}\nirq_setup_marker={IRQ_SETUP_MARKER}\nexception_marker={EXCEPTION_MARKER}\npage_fault_marker={PAGE_FAULT_MARKER}\nfault_address_present_marker={FAULT_ADDRESS_PRESENT_MARKER}\nfault_address_marker={FAULT_ADDRESS_MARKER}\nfault_cr3_marker={FAULT_CR3_MARKER}\nfault_rflags_marker={FAULT_RFLAGS_MARKER}\nfault_interrupts_marker={FAULT_INTERRUPTS_MARKER}\nfault_error_decode_marker={FAULT_ERROR_DECODE_MARKER}\nmemory_total_marker={MEMORY_TOTAL_MARKER}\nmemory_usable_marker={MEMORY_USABLE_MARKER}\nmemory_reserved_marker={MEMORY_RESERVED_MARKER}\nmemory_map_marker={MEMORY_MAP_MARKER}\nframe_allocator_status_marker={FRAME_ALLOCATOR_STATUS_MARKER}\nframe_allocator_marker={FRAME_ALLOCATOR_MARKER}\npage_table_status_marker={PAGE_TABLE_STATUS_MARKER}\npage_table_lookup_marker={PAGE_TABLE_LOOKUP_MARKER}\npage_table_presence_marker={PAGE_TABLE_PRESENCE_MARKER}\npage_table_protect_marker={PAGE_TABLE_PROTECT_MARKER}\npage_table_protect_range_marker={PAGE_TABLE_PROTECT_RANGE_MARKER}\npage_table_range_lookup_marker={PAGE_TABLE_RANGE_LOOKUP_MARKER}\npage_table_mapped_range_marker={PAGE_TABLE_MAPPED_RANGE_MARKER}\npage_table_unmapped_range_marker={PAGE_TABLE_UNMAPPED_RANGE_MARKER}\npage_table_audit_marker={PAGE_TABLE_AUDIT_MARKER}\npage_table_visit_marker={PAGE_TABLE_VISIT_MARKER}\npage_table_flags_marker={PAGE_TABLE_FLAGS_MARKER}\npage_table_reclaim_marker={PAGE_TABLE_RECLAIM_MARKER}\npage_table_range_marker={PAGE_TABLE_RANGE_MARKER}\npage_table_marker={PAGE_TABLE_MARKER}\nbootinfo_marker={BOOTINFO_MARKER}\nserial_marker={SERIAL_MARKER}\npanic_marker={PANIC_MARKER}\ntimer_setup_marker={TIMER_SETUP_MARKER}\ntimer_tick_1_marker={TIMER_TICK_1_MARKER}\ntimer_tick_2_marker={TIMER_TICK_2_MARKER}\ntimer_tick_3_marker={TIMER_TICK_3_MARKER}\ntimer_delayed_log_marker={TIMER_DELAYED_LOG_MARKER}\nsleep_marker={SLEEP_MARKER}\ntimer_marker={TIMER_MARKER}\nrustc_version={}\ncargo_version={}\nlimine_version={}\nlimine_min_version={}\nxorriso_version={}\nqemu_version={}\n",
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

fn run_qemu(paths: &ImagePaths, smoke: SmokeKind) -> Result<(), String> {
    let _ = fs::remove_file(&paths.serial_log);

    let serial_arg = format!("file:{}", paths.serial_log.display());
    let mut child = Command::new("qemu-system-x86_64")
        .args(["-machine", "q35", "-m", "128M", "-cdrom"])
        .arg(&paths.image)
        .args([
            "-boot",
            "d",
            "-serial",
            &serial_arg,
            "-display",
            "none",
            "-no-reboot",
            "-no-shutdown",
        ])
        .spawn()
        .map_err(|error| format!("failed to start qemu-system-x86_64: {error}"))?;

    let started = Instant::now();
    let mut marker_seen = false;

    while started.elapsed() < QEMU_TIMEOUT {
        if serial_log_contains_marker(&paths.serial_log, smoke) {
            marker_seen = true;
            break;
        }

        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("failed to poll QEMU: {error}"))?
        {
            return Err(format!("QEMU exited before serial marker: {status}"));
        }

        thread::sleep(Duration::from_millis(50));
    }

    if !marker_seen {
        marker_seen = serial_log_contains_marker(&paths.serial_log, smoke);
    }

    let _ = child.kill();
    let _status = child
        .wait()
        .map_err(|error| format!("failed waiting for QEMU shutdown: {error}"))?;

    if marker_seen {
        Ok(())
    } else {
        Err(format!(
            "QEMU {} smoke did not see serial markers {} within {} seconds; image={} kernel={} staging={}",
            smoke.name(),
            smoke.markers(),
            QEMU_TIMEOUT.as_secs(),
            paths.image.display(),
            paths.kernel_elf.display(),
            paths.staging_dir.display()
        ))
    }
}

fn serial_log_contains_marker(path: &Path, smoke: SmokeKind) -> bool {
    fs::read_to_string(path).is_ok_and(|contents| match smoke {
        SmokeKind::Boot => {
            !contents.contains(BOOTINFO_FAIL_MARKER)
                && !contents.contains(MEMORY_MAP_FAIL_MARKER)
                && !contents.contains(FRAME_ALLOCATOR_FAIL_MARKER)
                && !contents.contains(PAGE_TABLE_FAIL_MARKER)
                && contents.contains(CPU_SETUP_MARKER)
                && contents.contains(EXCEPTION_SETUP_MARKER)
                && contents.contains(IRQ_SETUP_MARKER)
                && contents.contains(EXCEPTION_MARKER)
                && contents.contains(BOOT_DIAGNOSTIC_MARKER)
                && contents.contains(MEMORY_TOTAL_MARKER)
                && contents.contains(MEMORY_USABLE_MARKER)
                && contents.contains(MEMORY_RESERVED_MARKER)
                && contents.contains(MEMORY_MAP_MARKER)
                && contents.contains(FRAME_ALLOCATOR_STATUS_MARKER)
                && contents.contains(FRAME_ALLOCATOR_MARKER)
                && contents.contains(PAGE_TABLE_STATUS_MARKER)
                && contents.contains(PAGE_TABLE_LOOKUP_MARKER)
                && contents.contains(PAGE_TABLE_PROTECT_MARKER)
                && contents.contains(PAGE_TABLE_PROTECT_RANGE_MARKER)
                && contents.contains(PAGE_TABLE_RANGE_LOOKUP_MARKER)
                && contents.contains(PAGE_TABLE_UNMAPPED_RANGE_MARKER)
                && contents.contains(PAGE_TABLE_AUDIT_MARKER)
                && contents.contains(PAGE_TABLE_VISIT_MARKER)
                && contents.contains(PAGE_TABLE_FLAGS_MARKER)
                && contents.contains(PAGE_TABLE_RECLAIM_MARKER)
                && contents.contains(PAGE_TABLE_RANGE_MARKER)
                && contents.contains(PAGE_TABLE_MARKER)
                && contents.contains(BOOTINFO_MARKER)
                && contents.contains(SERIAL_MARKER)
        }
        SmokeKind::Panic => {
            contents.contains(CPU_SETUP_MARKER)
                && contents.contains(EXCEPTION_SETUP_MARKER)
                && contents.contains(IRQ_SETUP_MARKER)
                && contents.contains(EXCEPTION_MARKER)
                && contents.contains(PANIC_DIAGNOSTIC_MARKER)
                && contents.contains(PANIC_MARKER)
                && contents.contains(PANIC_REGISTERS_MARKER)
        }
        SmokeKind::Exception => {
            contents.contains(CPU_SETUP_MARKER)
                && contents.contains(EXCEPTION_SETUP_MARKER)
                && contents.contains(IRQ_SETUP_MARKER)
                && contents.contains(EXCEPTION_MARKER)
                && contents.contains(FAULT_ADDRESS_PRESENT_MARKER)
                && contents.contains(FAULT_ADDRESS_MARKER)
                && contents.contains(FAULT_CR3_MARKER)
                && contents.contains(FAULT_RFLAGS_MARKER)
                && contents.contains(FAULT_INTERRUPTS_MARKER)
                && contents.contains(FAULT_ERROR_DECODE_MARKER)
                && contents.contains(PAGE_FAULT_MARKER)
        }
        SmokeKind::Timer => {
            contents.contains(CPU_SETUP_MARKER)
                && contents.contains(EXCEPTION_SETUP_MARKER)
                && contents.contains(IRQ_SETUP_MARKER)
                && contents.contains(EXCEPTION_MARKER)
                && contents.contains(TIMER_SETUP_MARKER)
                && contents.contains(TIMER_TICK_1_MARKER)
                && contents.contains(TIMER_TICK_2_MARKER)
                && contents.contains(TIMER_TICK_3_MARKER)
                && contents.contains(TIMER_DELAYED_LOG_MARKER)
                && contents.contains(SLEEP_MARKER)
                && contents.contains(TIMER_MARKER)
        }
    })
}

fn copy_file(from: &Path, to: &Path, description: &str) -> Result<(), String> {
    fs::copy(from, to).map_err(|error| {
        format!(
            "failed to copy {description} from {} to {}: {error}",
            from.display(),
            to.display()
        )
    })?;
    Ok(())
}

fn run_status(command: &mut Command, description: &str) -> Result<(), String> {
    let output = command
        .output()
        .map_err(|error| format!("failed to run {description}: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(command_error(description, output))
    }
}

fn command_error(description: &str, output: std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    format!(
        "{description} failed with status {}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        output.status
    )
}

#[cfg(test)]
mod tests;
