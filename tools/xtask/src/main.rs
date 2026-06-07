use std::env;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let mut args = env::args();
    let _program = args.next();
    match args.next().as_deref() {
        Some("check") => run_script("scripts/checks.sh"),
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

fn run_script(path: &str) -> ExitCode {
    match Command::new(path).status() {
        Ok(status) if status.success() => ExitCode::SUCCESS,
        Ok(status) => status
            .code()
            .and_then(|code| u8::try_from(code).ok())
            .map_or(ExitCode::FAILURE, ExitCode::from),
        Err(error) => {
            eprintln!("xtask: failed to run {path}: {error}");
            ExitCode::FAILURE
        }
    }
}

fn print_status() {
    println!("Aesynx workspace foundation is active.");
}

fn print_help() {
    println!("xtask commands:");
    println!("  check   run local repository checks");
    println!("  status  print workspace status");
    println!("  help    print this help");
}
