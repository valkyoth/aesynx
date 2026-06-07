use crate::workspace;

use std::process::{Command, ExitCode};

pub fn run_script(path: &str) -> ExitCode {
    let root = match workspace::root() {
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

pub fn run_script_with_arg(path: &str, arg: &str) -> ExitCode {
    let root = match workspace::root() {
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

pub fn run_command(command: &mut Command, label: &str) -> ExitCode {
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
