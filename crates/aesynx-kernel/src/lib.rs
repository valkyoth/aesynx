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
    if let Ok(message) = LogMessage::new(BOOT_BANNER) {
        log.write_str(LogLevel::Info, "kernel", message);
    }

    if let Ok(message) = LogMessage::new(message) {
        log.write_str(LogLevel::Info, "kernel", message);
    }
}
