use std::env;
use std::fs;
use std::process::ExitCode;

use trace_decode::decode_serial_trace;

fn main() -> ExitCode {
    let mut args = env::args();
    let _program = args.next();
    let Some(path) = args.next() else {
        print_help();
        return ExitCode::from(2);
    };

    if path == "--help" || path == "-h" {
        print_help();
        return ExitCode::SUCCESS;
    }

    if args.next().is_some() {
        eprintln!("trace-decode: expected exactly one serial log path");
        return ExitCode::from(2);
    }

    let input = match fs::read_to_string(&path) {
        Ok(input) => input,
        Err(error) => {
            eprintln!("trace-decode: failed to read {path}: {error}");
            return ExitCode::FAILURE;
        }
    };

    let export = match decode_serial_trace(&input) {
        Ok(export) => export,
        Err(error) => {
            eprintln!("trace-decode: failed to decode trace: {error:?}");
            return ExitCode::FAILURE;
        }
    };

    for line in export.into_lines() {
        println!("{line}");
    }

    ExitCode::SUCCESS
}

fn print_help() {
    println!("usage: trace-decode <serial-log>");
}
