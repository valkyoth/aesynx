mod host_tools;
mod manifest;
mod names;
mod smoke;

use crate::kernel_flags::apply_kernel_rustflags;
use crate::workspace;
use aesynx_core::QEMU_MULTICORE_TOPOLOGY_CORES;
use host_tools::validate_host_tools;
use manifest::write_manifest;
use names::image_names;
use smoke::{SmokeKind, parse_qemu_args, serial_log_contains_marker};

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
pub(super) const QEMU_SMP_CPUS: usize = QEMU_MULTICORE_TOPOLOGY_CORES;
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

    match run_smoke(&root, smoke) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("xtask: {error}");
            ExitCode::FAILURE
        }
    }
}

pub fn qemu_suite(args: &[String]) -> ExitCode {
    if !args.is_empty() {
        eprintln!("xtask: qemu-suite accepts no arguments");
        return ExitCode::from(2);
    }

    let root = match workspace::root() {
        Ok(root) => root,
        Err(error) => {
            eprintln!("xtask: {error}");
            return ExitCode::FAILURE;
        }
    };

    for smoke in [
        SmokeKind::Boot,
        SmokeKind::Panic,
        SmokeKind::Exception,
        SmokeKind::Timer,
    ] {
        if let Err(error) = run_smoke(&root, smoke) {
            eprintln!("xtask: {error}");
            return ExitCode::FAILURE;
        }
    }

    println!("xtask: QEMU smoke suite passed");
    ExitCode::SUCCESS
}

fn run_smoke(root: &Path, smoke: SmokeKind) -> Result<(), String> {
    let paths = match build_image(root, smoke) {
        Ok(paths) => paths,
        Err(error) => {
            return Err(error);
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
            Ok(())
        }
        Err(error) => Err(error),
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

fn run_qemu(paths: &ImagePaths, smoke: SmokeKind) -> Result<(), String> {
    let _ = fs::remove_file(&paths.serial_log);

    let serial_arg = format!("file:{}", paths.serial_log.display());
    let smp_arg = QEMU_SMP_CPUS.to_string();
    let mut child = Command::new("qemu-system-x86_64")
        .args(["-machine", "q35", "-m", "128M", "-smp", &smp_arg, "-cdrom"])
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
mod marker_tests;
#[cfg(test)]
mod tests;
