#![no_std]
#![deny(unsafe_code)]

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

pub trait LogSink {
    fn write_str(&self, level: LogLevel, message: &str);
}
