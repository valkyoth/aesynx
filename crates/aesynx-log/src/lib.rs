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

pub const MAX_LOG_MESSAGE_LEN: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LogMessage<'a> {
    value: &'a str,
}

impl<'a> LogMessage<'a> {
    pub const fn new(value: &'a str) -> Result<Self, LogError> {
        if value.len() > MAX_LOG_MESSAGE_LEN {
            return Err(LogError::MessageTooLong);
        }

        if contains_record_separator(value) {
            return Err(LogError::RecordSeparatorNotAllowed);
        }

        Ok(Self { value })
    }

    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogError {
    MessageTooLong,
    RecordSeparatorNotAllowed,
}

pub trait LogSink {
    fn write_str(&self, level: LogLevel, component: &'static str, message: LogMessage<'_>);
}

const fn contains_record_separator(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'\n' || bytes[index] == b'\r' {
            return true;
        }
        index += 1;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::{LogError, LogMessage};

    #[test]
    fn log_message_rejects_record_separator() {
        assert_eq!(
            LogMessage::new("valid\nforged"),
            Err(LogError::RecordSeparatorNotAllowed)
        );
    }

    #[test]
    fn log_message_accepts_single_record() {
        assert_eq!(
            LogMessage::new("valid").map(LogMessage::as_str),
            Ok("valid")
        );
    }
}
