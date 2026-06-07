use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const KERNEL_TARGET: &str = "targets/x86_64-unknown-aesynx.json";
const KERNEL_LINKER: &str = "linker/kernel-x86_64.ld";

fn main() -> ExitCode {
    let mut args = env::args();
    let _program = args.next();
    match args.next().as_deref() {
        Some("build-kernel") => build_kernel(),
        Some("check") => run_script("scripts/checks.sh"),
        Some("image") => not_ready("image", "v0.3.0"),
        Some("qemu") => not_ready("qemu", "v0.3.0"),
        Some("release-ready") => match args.next() {
            Some(tag) => run_script_with_arg("scripts/validate-release-readiness.sh", &tag),
            None => {
                eprintln!("xtask: release-ready requires a tag, for example v0.1.0");
                ExitCode::from(2)
            }
        },
        Some("status") => {
            print_status();
            ExitCode::SUCCESS
        }
        Some("help") | None => {
            print_help();
            ExitCode::SUCCESS
        }
        Some(_) => {
            print_help();
            ExitCode::from(2)
        }
    }
}

fn build_kernel() -> ExitCode {
    let root = match workspace_root() {
        Ok(root) => root,
        Err(error) => {
            eprintln!("xtask: {error}");
            return ExitCode::FAILURE;
        }
    };

    if !required_path_exists(&root, KERNEL_TARGET) || !required_path_exists(&root, KERNEL_LINKER) {
        return ExitCode::FAILURE;
    }

    let mut command = Command::new("cargo");
    command.args(["check", "-p", "aesynx-kernel"]);
    command.current_dir(&root);
    let host_check = run_command(&mut command, "cargo check -p aesynx-kernel");
    if host_check != ExitCode::SUCCESS {
        return host_check;
    }

    println!("xtask: kernel host check passed");
    println!("xtask: custom target configured at {KERNEL_TARGET}");
    println!("xtask: custom-target compilation will require a build-std path in a later milestone");
    ExitCode::SUCCESS
}

fn run_script(path: &str) -> ExitCode {
    let root = match workspace_root() {
        Ok(root) => root,
        Err(error) => {
            eprintln!("xtask: {error}");
            return ExitCode::FAILURE;
        }
    };
    let script = root.join(path);
    let mut command = Command::new(script);
    command.current_dir(root);
    run_command(&mut command, path)
}

fn run_script_with_arg(path: &str, arg: &str) -> ExitCode {
    let root = match workspace_root() {
        Ok(root) => root,
        Err(error) => {
            eprintln!("xtask: {error}");
            return ExitCode::FAILURE;
        }
    };
    let script = root.join(path);
    let mut command = Command::new(script);
    command.arg(arg);
    command.current_dir(root);
    run_command(&mut command, path)
}

fn run_command(command: &mut Command, label: &str) -> ExitCode {
    match command.status() {
        Ok(status) if status.success() => ExitCode::SUCCESS,
        Ok(status) => status
            .code()
            .and_then(|code| u8::try_from(code).ok())
            .map_or(ExitCode::FAILURE, ExitCode::from),
        Err(error) => {
            eprintln!("xtask: failed to run {label}: {error}");
            ExitCode::FAILURE
        }
    }
}

fn required_path_exists(root: &Path, path: &str) -> bool {
    if root.join(path).exists() {
        return true;
    }

    eprintln!("xtask: required path missing: {path}");
    false
}

fn workspace_root() -> Result<PathBuf, &'static str> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let Some(tools_dir) = manifest_dir.parent() else {
        return Err("cannot resolve tools directory from CARGO_MANIFEST_DIR");
    };
    let Some(root) = tools_dir.parent() else {
        return Err("cannot resolve workspace root from tools directory");
    };

    Ok(root.to_path_buf())
}

fn not_ready(command: &str, milestone: &str) -> ExitCode {
    eprintln!("xtask: {command} pipeline is intentionally not implemented until {milestone}");
    ExitCode::from(3)
}

fn print_status() {
    println!("Aesynx workspace foundation is active.");
}

fn print_help() {
    println!("xtask commands:");
    println!("  build-kernel       validate kernel build skeleton");
    println!("  check              run local repository checks");
    println!("  image              create boot image once v0.3.0 lands");
    println!("  qemu               run QEMU once v0.3.0 lands");
    println!("  release-ready TAG  validate release pentest gate for TAG");
    println!("  status             print workspace status");
    println!("  help               print this help");
}
