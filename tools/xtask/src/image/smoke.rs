pub const BOOTINFO_FAIL_MARKER: &str = "[TEST] bootinfo=fail";
pub const BOOTINFO_MARKER: &str = "[TEST] bootinfo=ok";
pub const BOOT_DIAGNOSTIC_MARKER: &str = "[kernel][INFO] bootinfo normalized";
pub const CPU_SETUP_MARKER: &str = "[TEST] gdt=ok";
pub const EXCEPTION_SETUP_MARKER: &str = "[TEST] idt=ok";
pub const EXCEPTION_MARKER: &str = "[TEST] exception=ok";
pub const PAGE_FAULT_MARKER: &str = "[TEST] pagefault=ok";
pub const PANIC_DIAGNOSTIC_MARKER: &str = "[kernel][FATAL] panic handler entered";
pub const PANIC_MARKER: &str = "[TEST] panic=ok";
pub const PANIC_REGISTERS_MARKER: &str = "panic registers=";
pub const SERIAL_MARKER: &str = "[TEST] boot=ok";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SmokeKind {
    Boot,
    Panic,
    Exception,
}

impl SmokeKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Boot => "boot",
            Self::Panic => "panic",
            Self::Exception => "exception",
        }
    }

    pub fn markers(self) -> &'static str {
        match self {
            Self::Boot => {
                "[TEST] gdt=ok, [TEST] idt=ok, [TEST] exception=ok, [kernel][INFO] bootinfo normalized, [TEST] bootinfo=ok, [TEST] boot=ok"
            }
            Self::Panic => {
                "[TEST] gdt=ok, [TEST] idt=ok, [TEST] exception=ok, [kernel][FATAL] panic handler entered, panic registers=, [TEST] panic=ok"
            }
            Self::Exception => {
                "[TEST] gdt=ok, [TEST] idt=ok, [TEST] exception=ok, [TEST] pagefault=ok"
            }
        }
    }

    pub fn feature(self) -> Option<&'static str> {
        match self {
            Self::Boot => None,
            Self::Panic => Some("panic-smoke"),
            Self::Exception => Some("exception-smoke"),
        }
    }
}

pub fn parse_qemu_args(args: &[String]) -> Result<SmokeKind, &'static str> {
    match args {
        [] => Ok(SmokeKind::Boot),
        [flag] if flag == "--panic-smoke" => Ok(SmokeKind::Panic),
        [flag] if flag == "--exception-smoke" => Ok(SmokeKind::Exception),
        _ => Err("qemu accepts no arguments except --panic-smoke or --exception-smoke"),
    }
}
