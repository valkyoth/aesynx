use core::fmt;
use core::sync::atomic::{AtomicU8, Ordering};

use aesynx_abi::CoreId;
use aesynx_log::{LogLevel, LogMessage};

static BOOT_PHASE: AtomicU8 = AtomicU8::new(BootPhase::Entry as u8);

pub const EARLY_BOOT_CORE: CoreId = CoreId::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum BootPhase {
    Entry = 0,
    BootloaderHandoff = 1,
    BootInfoNormalized = 2,
    Running = 3,
    PanicSmoke = 4,
    Panic = 5,
    Unknown = u8::MAX,
}

impl BootPhase {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Entry => "entry",
            Self::BootloaderHandoff => "bootloader-handoff",
            Self::BootInfoNormalized => "bootinfo-normalized",
            Self::Running => "running",
            Self::PanicSmoke => "panic-smoke",
            Self::Panic => "panic",
            Self::Unknown => "unknown",
        }
    }

    #[must_use]
    pub const fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Entry,
            1 => Self::BootloaderHandoff,
            2 => Self::BootInfoNormalized,
            3 => Self::Running,
            4 => Self::PanicSmoke,
            5 => Self::Panic,
            _unknown => Self::Unknown,
        }
    }
}

pub fn set_boot_phase(phase: BootPhase) {
    BOOT_PHASE.store(phase as u8, Ordering::Release);
}

#[must_use]
pub fn current_boot_phase() -> BootPhase {
    BootPhase::from_raw(BOOT_PHASE.load(Ordering::Acquire))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PanicSnapshot {
    pub core: CoreId,
    pub phase: BootPhase,
}

#[must_use]
pub fn panic_snapshot() -> PanicSnapshot {
    PanicSnapshot {
        core: EARLY_BOOT_CORE,
        phase: current_boot_phase(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiagnosticRecord<'a> {
    pub core: CoreId,
    pub phase: BootPhase,
    pub level: LogLevel,
    pub component: &'static str,
    pub message: LogMessage<'a>,
}

impl<'a> DiagnosticRecord<'a> {
    #[must_use]
    pub const fn new(
        core: CoreId,
        phase: BootPhase,
        level: LogLevel,
        component: &'static str,
        message: LogMessage<'a>,
    ) -> Self {
        Self {
            core,
            phase,
            level,
            component,
            message,
        }
    }

    #[must_use]
    pub fn current(level: LogLevel, component: &'static str, message: LogMessage<'a>) -> Self {
        Self::new(
            EARLY_BOOT_CORE,
            current_boot_phase(),
            level,
            component,
            message,
        )
    }

    pub fn write_to(self, output: &mut impl fmt::Write) -> fmt::Result {
        writeln!(
            output,
            "[core={}][phase={}][{}][{}] {}",
            self.core.get(),
            self.phase.label(),
            self.component,
            log_level_label(self.level),
            self.message.as_str()
        )
    }
}

#[must_use]
pub const fn log_level_label(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Trace => "TRACE",
        LogLevel::Debug => "DEBUG",
        LogLevel::Info => "INFO",
        LogLevel::Warn => "WARN",
        LogLevel::Error => "ERROR",
        LogLevel::Fatal => "FATAL",
    }
}

#[cfg(test)]
mod tests {
    use core::fmt::{self, Write};

    use aesynx_log::{LogLevel, LogMessage};

    use super::{
        BootPhase, DiagnosticRecord, EARLY_BOOT_CORE, current_boot_phase, log_level_label,
        panic_snapshot, set_boot_phase,
    };

    #[test]
    fn boot_phase_labels_are_stable() {
        assert_eq!(BootPhase::Entry.label(), "entry");
        assert_eq!(BootPhase::BootloaderHandoff.label(), "bootloader-handoff");
        assert_eq!(BootPhase::BootInfoNormalized.label(), "bootinfo-normalized");
        assert_eq!(BootPhase::Running.label(), "running");
        assert_eq!(BootPhase::PanicSmoke.label(), "panic-smoke");
        assert_eq!(BootPhase::Panic.label(), "panic");
        assert_eq!(BootPhase::Unknown.label(), "unknown");
    }

    #[test]
    fn invalid_boot_phase_bytes_fall_back_to_unknown() {
        assert_eq!(BootPhase::from_raw(99), BootPhase::Unknown);
    }

    #[test]
    fn boot_phase_tracking_is_visible_to_panic_snapshot() {
        set_boot_phase(BootPhase::PanicSmoke);

        assert_eq!(current_boot_phase(), BootPhase::PanicSmoke);
        assert_eq!(
            panic_snapshot(),
            super::PanicSnapshot {
                core: EARLY_BOOT_CORE,
                phase: BootPhase::PanicSmoke,
            }
        );

        set_boot_phase(BootPhase::Entry);
    }

    #[test]
    fn log_level_labels_are_stable() {
        assert_eq!(log_level_label(LogLevel::Trace), "TRACE");
        assert_eq!(log_level_label(LogLevel::Debug), "DEBUG");
        assert_eq!(log_level_label(LogLevel::Info), "INFO");
        assert_eq!(log_level_label(LogLevel::Warn), "WARN");
        assert_eq!(log_level_label(LogLevel::Error), "ERROR");
        assert_eq!(log_level_label(LogLevel::Fatal), "FATAL");
    }

    #[test]
    fn diagnostic_record_formats_with_core_phase_component_and_level() {
        let record = DiagnosticRecord::new(
            EARLY_BOOT_CORE,
            BootPhase::BootInfoNormalized,
            LogLevel::Info,
            "kernel",
            LogMessage::new("bootinfo normalized").unwrap_or(LogMessage::REJECTED),
        );
        let mut output = FixedBuf::default();

        assert_eq!(record.write_to(&mut output), Ok(()));
        assert_eq!(
            output.as_str(),
            "[core=0][phase=bootinfo-normalized][kernel][INFO] bootinfo normalized\n"
        );
    }

    struct FixedBuf {
        bytes: [u8; 128],
        len: usize,
    }

    impl Default for FixedBuf {
        fn default() -> Self {
            Self {
                bytes: [0; 128],
                len: 0,
            }
        }
    }

    impl FixedBuf {
        fn as_str(&self) -> &str {
            core::str::from_utf8(&self.bytes[..self.len]).unwrap_or_default()
        }
    }

    impl Write for FixedBuf {
        fn write_str(&mut self, value: &str) -> fmt::Result {
            if self.len + value.len() > self.bytes.len() {
                return Err(fmt::Error);
            }

            let end = self.len + value.len();
            self.bytes[self.len..end].copy_from_slice(value.as_bytes());
            self.len = end;
            Ok(())
        }
    }
}
