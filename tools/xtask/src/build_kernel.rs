use crate::process::run_command;
use crate::workspace;

use std::fs;
use std::path::Path;
use std::process::{Command, ExitCode};

const CARGO_CONFIG: &str = ".cargo/config.toml";
const KERNEL_TARGET: &str = "targets/x86_64-unknown-aesynx.json";
const KERNEL_LINKER: &str = "linker/kernel-x86_64.ld";

const CARGO_CONFIG_MARKERS: &[&str] = &[
    "linker = \"rust-lld\"",
    "link-arg=-Tlinker/kernel-x86_64.ld",
    "panic=abort",
];

const KERNEL_TARGET_MARKERS: &[&str] = &[
    "\"arch\": \"x86_64\"",
    "\"code-model\": \"kernel\"",
    "\"disable-redzone\": true",
    "\"linker\": \"rust-lld\"",
    "\"os\": \"aesynx\"",
    "\"panic-strategy\": \"abort\"",
    "\"relocation-model\": \"static\"",
    "\"target-pointer-width\": \"64\"",
];

const KERNEL_LINKER_MARKERS: &[&str] = &[
    "ENTRY(_start)",
    "KERNEL_VMA = 0xffffffff80000000;",
    ".text : ALIGN(4K)",
    ".rodata : ALIGN(4K)",
    ".data : ALIGN(4K)",
    ".bss : ALIGN(4K)",
    "/DISCARD/ :",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BuildKernelMode {
    StableValidation,
    CustomTargetProbe,
}

pub fn run(args: &[String]) -> ExitCode {
    let mode = match parse_args(args) {
        Ok(mode) => mode,
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

    match mode {
        BuildKernelMode::StableValidation => stable_validation(&root),
        BuildKernelMode::CustomTargetProbe => custom_target_probe(&root),
    }
}

fn parse_args(args: &[String]) -> Result<BuildKernelMode, &'static str> {
    match args {
        [] => Ok(BuildKernelMode::StableValidation),
        [flag] if flag == "--custom-target-probe" => Ok(BuildKernelMode::CustomTargetProbe),
        _ => Err("build-kernel accepts no arguments except --custom-target-probe"),
    }
}

fn stable_validation(root: &Path) -> ExitCode {
    if !validate_build_files(root) {
        return ExitCode::FAILURE;
    }

    let mut command = Command::new("cargo");
    command.args(["check", "-p", "aesynx-kernel"]);
    command.current_dir(root);
    let host_check = run_command(&mut command, "cargo check -p aesynx-kernel");
    if host_check != ExitCode::SUCCESS {
        return host_check;
    }

    println!("xtask: kernel host check passed");
    println!("xtask: custom target metadata validated at {KERNEL_TARGET}");
    println!("xtask: linker script validated at {KERNEL_LINKER}");
    println!("xtask: stable build skeleton is ready for v0.2.0");
    println!(
        "xtask: optional custom target probe is available with: cargo xtask build-kernel --custom-target-probe"
    );
    ExitCode::SUCCESS
}

fn custom_target_probe(root: &Path) -> ExitCode {
    if !validate_build_files(root) {
        return ExitCode::FAILURE;
    }

    eprintln!(
        "xtask: custom target probe uses nightly Cargo build-std and is not the stable release gate"
    );

    let mut command = Command::new("cargo");
    command.args([
        "+nightly",
        "build",
        "-Z",
        "build-std=core",
        "--target",
        KERNEL_TARGET,
        "-p",
        "aesynx-kernel",
    ]);
    command.current_dir(root);
    run_command(
        &mut command,
        "cargo +nightly build -Z build-std=core --target targets/x86_64-unknown-aesynx.json -p aesynx-kernel",
    )
}

fn validate_build_files(root: &Path) -> bool {
    let config_ok = required_file_contains(root, CARGO_CONFIG, CARGO_CONFIG_MARKERS);
    let target_ok = required_file_contains(root, KERNEL_TARGET, KERNEL_TARGET_MARKERS);
    let linker_ok = required_file_contains(root, KERNEL_LINKER, KERNEL_LINKER_MARKERS);

    config_ok && target_ok && linker_ok
}

fn required_file_contains(root: &Path, path: &str, markers: &[&str]) -> bool {
    let full_path = root.join(path);
    let contents = match fs::read_to_string(&full_path) {
        Ok(contents) => contents,
        Err(error) => {
            eprintln!("xtask: required file unavailable: {path}: {error}");
            return false;
        }
    };

    let mut valid = true;
    for marker in markers {
        if !contents.contains(marker) {
            eprintln!("xtask: {path} is missing expected marker: {marker}");
            valid = false;
        }
    }

    valid
}

#[cfg(test)]
mod tests {
    use super::{BuildKernelMode, parse_args};

    #[test]
    fn build_kernel_defaults_to_stable_validation() {
        let args = Vec::new();

        assert_eq!(parse_args(&args), Ok(BuildKernelMode::StableValidation));
    }

    #[test]
    fn build_kernel_accepts_custom_target_probe() {
        let args = vec![String::from("--custom-target-probe")];

        assert_eq!(parse_args(&args), Ok(BuildKernelMode::CustomTargetProbe));
    }

    #[test]
    fn build_kernel_rejects_unknown_args() {
        let args = vec![String::from("--unknown")];

        assert!(parse_args(&args).is_err());
    }
}
