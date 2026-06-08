use core::sync::atomic::{AtomicU8, Ordering};

use aesynx_abi::CoreId;

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

#[cfg(test)]
mod tests {
    use super::{BootPhase, EARLY_BOOT_CORE, current_boot_phase, panic_snapshot, set_boot_phase};

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
}
