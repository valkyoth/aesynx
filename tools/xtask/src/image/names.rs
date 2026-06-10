use super::smoke::SmokeKind;

const STAGING_DIR_NAME: &str = "aesynx-v0.14.0-iso";
const IMAGE_NAME: &str = "aesynx-v0.14.0.iso";
const MANIFEST_NAME: &str = "aesynx-v0.14.0.manifest";
const SERIAL_LOG_NAME: &str = "aesynx-v0.14.0.serial.log";
const PANIC_STAGING_DIR_NAME: &str = "aesynx-v0.14.0-panic-iso";
const PANIC_IMAGE_NAME: &str = "aesynx-v0.14.0-panic.iso";
const PANIC_MANIFEST_NAME: &str = "aesynx-v0.14.0-panic.manifest";
const PANIC_SERIAL_LOG_NAME: &str = "aesynx-v0.14.0-panic.serial.log";
const EXCEPTION_STAGING_DIR_NAME: &str = "aesynx-v0.14.0-exception-iso";
const EXCEPTION_IMAGE_NAME: &str = "aesynx-v0.14.0-exception.iso";
const EXCEPTION_MANIFEST_NAME: &str = "aesynx-v0.14.0-exception.manifest";
const EXCEPTION_SERIAL_LOG_NAME: &str = "aesynx-v0.14.0-exception.serial.log";
const TIMER_STAGING_DIR_NAME: &str = "aesynx-v0.14.0-timer-iso";
const TIMER_IMAGE_NAME: &str = "aesynx-v0.14.0-timer.iso";
const TIMER_MANIFEST_NAME: &str = "aesynx-v0.14.0-timer.manifest";
const TIMER_SERIAL_LOG_NAME: &str = "aesynx-v0.14.0-timer.serial.log";

pub struct ImageNames {
    pub image: &'static str,
    pub manifest: &'static str,
    pub serial_log: &'static str,
    pub staging_dir: &'static str,
}

pub fn image_names(smoke: SmokeKind) -> ImageNames {
    match smoke {
        SmokeKind::Boot => ImageNames {
            image: IMAGE_NAME,
            manifest: MANIFEST_NAME,
            serial_log: SERIAL_LOG_NAME,
            staging_dir: STAGING_DIR_NAME,
        },
        SmokeKind::Panic => ImageNames {
            image: PANIC_IMAGE_NAME,
            manifest: PANIC_MANIFEST_NAME,
            serial_log: PANIC_SERIAL_LOG_NAME,
            staging_dir: PANIC_STAGING_DIR_NAME,
        },
        SmokeKind::Exception => ImageNames {
            image: EXCEPTION_IMAGE_NAME,
            manifest: EXCEPTION_MANIFEST_NAME,
            serial_log: EXCEPTION_SERIAL_LOG_NAME,
            staging_dir: EXCEPTION_STAGING_DIR_NAME,
        },
        SmokeKind::Timer => ImageNames {
            image: TIMER_IMAGE_NAME,
            manifest: TIMER_MANIFEST_NAME,
            serial_log: TIMER_SERIAL_LOG_NAME,
            staging_dir: TIMER_STAGING_DIR_NAME,
        },
    }
}
