#![no_std]
#![deny(unsafe_code)]

use aesynx_boot::BootInfo;
use aesynx_log::{LogLevel, LogMessage, LogSink};

pub const BOOT_BANNER: &str = "Aesynx: booting";

pub fn describe_boot(info: &BootInfo<'_>, log: &impl LogSink) {
    let message = match info.arch {
        aesynx_boot::ArchKind::X86_64 => "arch=x86_64",
        aesynx_boot::ArchKind::Aarch64 => "arch=aarch64",
        aesynx_boot::ArchKind::Unknown => "arch=unknown",
    };
    write_boot_log(log, BOOT_BANNER);
    write_boot_log(log, message);
}

fn write_boot_log(log: &impl LogSink, message: &'static str) {
    let message = LogMessage::new(message).unwrap_or(LogMessage::REJECTED);
    log.write_str(LogLevel::Info, "kernel", message);
}
