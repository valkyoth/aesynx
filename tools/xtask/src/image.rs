mod host_tools;

use crate::workspace;
use host_tools::{HostToolVersions, MIN_LIMINE_VERSION_TEXT, validate_host_tools};

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::thread;
use std::time::{Duration, Instant};

const BOOT_CONFIG: &str = "boot/qemu/limine.conf";
const BUILD_DIR: &str = "build/qemu";
const STAGING_DIR_NAME: &str = "aesynx-v0.4.0-iso";
const IMAGE_NAME: &str = "aesynx-v0.4.0.iso";
const MANIFEST_NAME: &str = "aesynx-v0.4.0.manifest";
const SERIAL_LOG_NAME: &str = "aesynx-v0.4.0.serial.log";
const KERNEL_TARGET: &str = "x86_64-unknown-none";
const KERNEL_PACKAGE: &str = "aesynx-kernel";
const KERNEL_BINARY: &str = "aesynx-kernel";
const KERNEL_PROFILE: &str = "release";
const SERIAL_MARKER: &str = "[TEST] boot=ok";
const QEMU_TIMEOUT: Duration = Duration::from_secs(5);

const BOOT_CONFIG_MARKERS: &[&str] = &[
    "serial: yes",
    "/Aesynx",
    "protocol: limine",
    "path: boot():/boot/aesynx-kernel",
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

    match build_image(&root) {
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
    if !args.is_empty() {
        eprintln!("xtask: qemu accepts no arguments");
        return ExitCode::from(2);
    }

    let root = match workspace::root() {
        Ok(root) => root,
        Err(error) => {
            eprintln!("xtask: {error}");
            return ExitCode::FAILURE;
        }
    };

    let paths = match build_image(&root) {
        Ok(paths) => paths,
        Err(error) => {
            eprintln!("xtask: {error}");
            return ExitCode::FAILURE;
        }
    };

    match run_qemu(&paths) {
        Ok(()) => {
            println!("xtask: QEMU boot smoke saw serial marker: {SERIAL_MARKER}");
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

fn build_image(root: &Path) -> Result<ImagePaths, String> {
    validate_boot_config(root)?;
    let host_tools = validate_host_tools()?;

    let output_dir = root.join(BUILD_DIR);
    fs::create_dir_all(&output_dir)
        .map_err(|error| format!("failed to create {}: {error}", output_dir.display()))?;

    let image = output_dir.join(IMAGE_NAME);
    let manifest = output_dir.join(MANIFEST_NAME);
    let serial_log = output_dir.join(SERIAL_LOG_NAME);
    let staging_dir = output_dir.join(STAGING_DIR_NAME);
    let kernel_elf = build_kernel_elf(root)?;

    prepare_staging(root, &staging_dir, &kernel_elf)?;
    create_limine_iso(&staging_dir, &image)?;
    install_limine_bios(&image)?;
    write_manifest(&manifest, &image, &kernel_elf, &host_tools)?;

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

fn build_kernel_elf(root: &Path) -> Result<PathBuf, String> {
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
    command.current_dir(root);
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
) -> Result<(), String> {
    let manifest_contents = format!(
        "name=Aesynx v0.4.0 first serial boot\nimage={}\nformat=iso\nbootloader=limine\nkernel={}\nkernel_target={KERNEL_TARGET}\nkernel_profile={KERNEL_PROFILE}\nserial_marker={SERIAL_MARKER}\nrustc_version={}\ncargo_version={}\nlimine_version={}\nlimine_min_version={}\nxorriso_version={}\nqemu_version={}\n",
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

fn run_qemu(paths: &ImagePaths) -> Result<(), String> {
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
        if serial_log_contains_marker(&paths.serial_log) {
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
        marker_seen = serial_log_contains_marker(&paths.serial_log);
    }

    let _ = child.kill();
    let _status = child
        .wait()
        .map_err(|error| format!("failed waiting for QEMU shutdown: {error}"))?;

    if marker_seen {
        Ok(())
    } else {
        Err(format!(
            "QEMU boot smoke did not see serial marker {SERIAL_MARKER:?} within {} seconds; image={} kernel={} staging={}",
            QEMU_TIMEOUT.as_secs(),
            paths.image.display(),
            paths.kernel_elf.display(),
            paths.staging_dir.display()
        ))
    }
}

fn serial_log_contains_marker(path: &Path) -> bool {
    fs::read_to_string(path).is_ok_and(|contents| contents.contains(SERIAL_MARKER))
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
mod tests {
    use super::{BOOT_CONFIG_MARKERS, KERNEL_PROFILE, KERNEL_TARGET, SERIAL_MARKER};

    #[test]
    fn qemu_marker_tracks_v0_4_boot_contract() {
        assert_eq!(SERIAL_MARKER, "[TEST] boot=ok");
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
}
