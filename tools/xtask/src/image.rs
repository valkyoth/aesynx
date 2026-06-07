use crate::workspace;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::thread;
use std::time::{Duration, Instant};

const BOOT_CONFIG: &str = "boot/qemu/stage0.toml";
const BUILD_DIR: &str = "build/qemu";
const IMAGE_NAME: &str = "aesynx-v0.3.0.raw";
const MANIFEST_NAME: &str = "aesynx-v0.3.0.manifest";
const SERIAL_LOG_NAME: &str = "aesynx-v0.3.0.serial.log";
const IMAGE_SIZE: usize = 1_474_560;
const SERIAL_MARKER: &str = "[TEST] bootloader=skeleton";
const QEMU_TIMEOUT: Duration = Duration::from_secs(3);

const BOOT_CONFIG_MARKERS: &[&str] = &[
    "format = \"raw-bios-stage0\"",
    "serial_marker = \"[TEST] bootloader=skeleton\"",
    "next_milestone = \"v0.4.0\"",
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
            println!(
                "xtask: wrote QEMU image skeleton: {}",
                paths.image.display()
            );
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
}

fn build_image(root: &Path) -> Result<ImagePaths, String> {
    validate_boot_config(root)?;

    let output_dir = root.join(BUILD_DIR);
    fs::create_dir_all(&output_dir)
        .map_err(|error| format!("failed to create {}: {error}", output_dir.display()))?;

    let image = output_dir.join(IMAGE_NAME);
    let manifest = output_dir.join(MANIFEST_NAME);
    let serial_log = output_dir.join(SERIAL_LOG_NAME);

    let mut bytes = vec![0u8; IMAGE_SIZE];
    let boot_sector = stage0_boot_sector();
    bytes[..boot_sector.len()].copy_from_slice(&boot_sector);
    fs::write(&image, bytes).map_err(|error| format!("failed to write image: {error}"))?;

    let manifest_contents = format!(
        "name=Aesynx v0.3.0 QEMU image skeleton\nimage={}\nformat=raw\nsize_bytes={IMAGE_SIZE}\nboot_sector=stage0-serial-probe\nserial_marker={SERIAL_MARKER}\nnext_kernel_boot_milestone=v0.4.0\n",
        image.display()
    );
    fs::write(&manifest, manifest_contents)
        .map_err(|error| format!("failed to write manifest: {error}"))?;

    Ok(ImagePaths {
        image,
        manifest,
        serial_log,
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

fn run_qemu(paths: &ImagePaths) -> Result<(), String> {
    let _ = fs::remove_file(&paths.serial_log);

    let drive_arg = format!("format=raw,file={}", paths.image.display());
    let serial_arg = format!("file:{}", paths.serial_log.display());
    let mut child = Command::new("qemu-system-x86_64")
        .args([
            "-machine",
            "q35",
            "-m",
            "128M",
            "-drive",
            &drive_arg,
            "-boot",
            "c",
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
            "QEMU boot smoke did not see serial marker {SERIAL_MARKER:?} within {} seconds",
            QEMU_TIMEOUT.as_secs()
        ))
    }
}

fn serial_log_contains_marker(path: &Path) -> bool {
    fs::read_to_string(path).is_ok_and(|contents| contents.contains(SERIAL_MARKER))
}

fn stage0_boot_sector() -> [u8; 512] {
    let mut bytes = Vec::new();

    bytes.extend_from_slice(&[0xfa]); // cli
    bytes.extend_from_slice(&[0x31, 0xc0]); // xor ax, ax
    bytes.extend_from_slice(&[0x8e, 0xd0]); // mov ss, ax
    bytes.extend_from_slice(&[0xbc, 0x00, 0x7c]); // mov sp, 0x7c00
    bytes.extend_from_slice(&[0x8e, 0xd8]); // mov ds, ax
    bytes.extend_from_slice(&[0xfb]); // sti

    let message_operand = bytes.len() + 1;
    bytes.extend_from_slice(&[0xbe, 0x00, 0x00]); // mov si, message

    let call_init = append_call_placeholder(&mut bytes);

    let message_loop = bytes.len();
    bytes.push(0xac); // lodsb
    bytes.extend_from_slice(&[0x84, 0xc0]); // test al, al
    let jump_halt_operand = bytes.len() + 1;
    bytes.extend_from_slice(&[0x74, 0x00]); // jz halt
    let call_write = append_call_placeholder(&mut bytes);
    append_short_jump(&mut bytes, message_loop);

    let halt = bytes.len();
    bytes.extend_from_slice(&[0xfa, 0xf4, 0xeb, 0xfd]); // cli; hlt; jmp $

    let serial_init = bytes.len();
    append_serial_init(&mut bytes);

    let serial_write = bytes.len();
    append_serial_write(&mut bytes);

    let message = bytes.len();
    bytes.extend_from_slice(
        b"Aesynx v0.3.0 boot image skeleton\r\n[TEST] bootloader=skeleton\r\n\0",
    );

    patch_u16(&mut bytes, message_operand, 0x7c00 + message as u16);
    patch_call(&mut bytes, call_init, serial_init);
    patch_call(&mut bytes, call_write, serial_write);
    patch_i8(&mut bytes, jump_halt_operand, halt, jump_halt_operand + 1);

    let mut sector = [0u8; 512];
    let payload_len = bytes.len().min(510);
    sector[..payload_len].copy_from_slice(&bytes[..payload_len]);
    sector[510] = 0x55;
    sector[511] = 0xaa;
    sector
}

fn append_call_placeholder(bytes: &mut Vec<u8>) -> usize {
    let opcode = bytes.len();
    bytes.extend_from_slice(&[0xe8, 0x00, 0x00]);
    opcode
}

fn append_short_jump(bytes: &mut Vec<u8>, target: usize) {
    let opcode = bytes.len();
    bytes.extend_from_slice(&[0xeb, 0x00]);
    patch_i8(bytes, opcode + 1, target, opcode + 2);
}

fn append_serial_init(bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&[0xba, 0xf9, 0x03]); // mov dx, 0x3f9
    bytes.extend_from_slice(&[0x30, 0xc0]); // xor al, al
    bytes.push(0xee); // out dx, al
    bytes.extend_from_slice(&[0xba, 0xfb, 0x03]); // mov dx, 0x3fb
    bytes.extend_from_slice(&[0xb0, 0x80]); // mov al, 0x80
    bytes.push(0xee);
    bytes.extend_from_slice(&[0xba, 0xf8, 0x03]); // mov dx, 0x3f8
    bytes.extend_from_slice(&[0xb0, 0x03]); // divisor low
    bytes.push(0xee);
    bytes.extend_from_slice(&[0xba, 0xf9, 0x03]); // mov dx, 0x3f9
    bytes.extend_from_slice(&[0x30, 0xc0]);
    bytes.push(0xee);
    bytes.extend_from_slice(&[0xba, 0xfb, 0x03]); // mov dx, 0x3fb
    bytes.extend_from_slice(&[0xb0, 0x03]); // 8N1
    bytes.push(0xee);
    bytes.extend_from_slice(&[0xba, 0xfa, 0x03]); // mov dx, 0x3fa
    bytes.extend_from_slice(&[0xb0, 0xc7]); // FIFO
    bytes.push(0xee);
    bytes.extend_from_slice(&[0xba, 0xfc, 0x03]); // mov dx, 0x3fc
    bytes.extend_from_slice(&[0xb0, 0x0b]); // DTR, RTS, OUT2
    bytes.push(0xee);
    bytes.push(0xc3); // ret
}

fn append_serial_write(bytes: &mut Vec<u8>) {
    bytes.push(0x50); // push ax
    let wait = bytes.len();
    bytes.extend_from_slice(&[0xba, 0xfd, 0x03]); // mov dx, 0x3fd
    bytes.push(0xec); // in al, dx
    bytes.extend_from_slice(&[0xa8, 0x20]); // test al, 0x20
    let retry_operand = bytes.len() + 1;
    bytes.extend_from_slice(&[0x74, 0x00]); // jz wait
    bytes.push(0x58); // pop ax
    bytes.extend_from_slice(&[0xba, 0xf8, 0x03]); // mov dx, 0x3f8
    bytes.push(0xee); // out dx, al
    bytes.push(0xc3); // ret
    patch_i8(bytes, retry_operand, wait, retry_operand + 1);
}

fn patch_call(bytes: &mut [u8], opcode: usize, target: usize) {
    let after_instruction = opcode + 3;
    let displacement = (target as isize - after_instruction as isize) as i16;
    let [lo, hi] = displacement.to_le_bytes();
    bytes[opcode + 1] = lo;
    bytes[opcode + 2] = hi;
}

fn patch_u16(bytes: &mut [u8], operand: usize, value: u16) {
    let [lo, hi] = value.to_le_bytes();
    bytes[operand] = lo;
    bytes[operand + 1] = hi;
}

fn patch_i8(bytes: &mut [u8], operand: usize, target: usize, after_instruction: usize) {
    let displacement = (target as isize - after_instruction as isize) as i8;
    bytes[operand] = displacement as u8;
}

#[cfg(test)]
mod tests {
    use super::{SERIAL_MARKER, stage0_boot_sector};

    #[test]
    fn stage0_boot_sector_has_bios_signature() {
        let sector = stage0_boot_sector();

        assert_eq!(sector.len(), 512);
        assert_eq!(sector[510], 0x55);
        assert_eq!(sector[511], 0xaa);
    }

    #[test]
    fn stage0_boot_sector_contains_serial_marker() {
        let sector = stage0_boot_sector();
        let marker = SERIAL_MARKER.as_bytes();

        assert!(sector.windows(marker.len()).any(|window| window == marker));
    }
}
