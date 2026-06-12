mod build_kernel;
mod image;
mod kernel_flags;
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
        "image" => image::build(&rest),
        "qemu" => image::qemu(&rest),
        "qemu-suite" => image::qemu_suite(&rest),
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

fn print_status() {
    println!("Aesynx workspace foundation is active.");
}

fn print_help() {
    println!("xtask commands:");
    println!("  build-kernel                         build and validate kernel boot path");
    println!("  build-kernel --custom-target-probe   try nightly build-std custom target probe");
    println!("  check                                run local repository checks");
    println!("  image                                create v0.16 Limine QEMU boot image");
    println!("  qemu                                 run v0.16 QEMU boot smoke");
    println!("  qemu --panic-smoke                   run v0.16 QEMU panic diagnostics smoke");
    println!("  qemu --exception-smoke               run v0.16 QEMU exception smoke");
    println!("  qemu --timer-smoke                   run v0.16 QEMU timer/sleep smoke");
    println!("  qemu-suite                           run all v0.16 QEMU smoke paths");
    println!("  release-ready TAG                    validate release pentest gate for TAG");
    println!("  status                               print workspace status");
    println!("  help                                 print this help");
}
