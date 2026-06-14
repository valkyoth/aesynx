#![no_std]
#![forbid(unsafe_code)]

pub const SHELL_NAME: &str = "aesh";
pub const PROMPT: &str = "aesh> ";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Builtin {
    Help,
    Version,
    Echo,
    Caps,
    Objects,
    Ps,
    Log,
    Reboot,
}
