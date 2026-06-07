mod build_kernel;
mod process;
mod workspace;

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args();
    let _program = args.next();

    let Some(command) = args.next() else {
        print_help();
        return ExitCode::SUCCESS;
    };
    let rest: Vec<String> = args.collect();

    match command.as_str() {
        "build-kernel" => build_kernel::run(&rest),
        "check" => process::run_script("scripts/checks.sh"),
        "image" => not_ready("image", "v0.3.0"),
        "qemu" => not_ready("qemu", "v0.3.0"),
        "release-ready" => release_ready(&rest),
        "status" => {
            print_status();
            ExitCode::SUCCESS
        }
        "help" => {
            print_help();
            ExitCode::SUCCESS
        }
        _ => {
            print_help();
            ExitCode::from(2)
        }
    }
}

fn release_ready(args: &[String]) -> ExitCode {
    match args {
        [tag] => process::run_script_with_arg("scripts/validate-release-readiness.sh", tag),
        _ => {
            eprintln!("xtask: release-ready requires one tag, for example v0.2.0");
            ExitCode::from(2)
        }
    }
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
    println!("  build-kernel                         validate kernel build skeleton");
    println!("  build-kernel --custom-target-probe   try nightly build-std custom target probe");
    println!("  check                                run local repository checks");
    println!("  image                                create boot image once v0.3.0 lands");
    println!("  qemu                                 run QEMU once v0.3.0 lands");
    println!("  release-ready TAG                    validate release pentest gate for TAG");
    println!("  status                               print workspace status");
    println!("  help                                 print this help");
}
