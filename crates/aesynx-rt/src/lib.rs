#![no_std]
#![forbid(unsafe_code)]

use core::fmt;

use aesynx_abi::CapId;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Env {
    console_in: CapId,
    console_out: CapId,
    process_service: CapId,
    object_root: CapId,
}

impl fmt::Debug for Env {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Env")
            .field("console_in", &"<redacted>")
            .field("console_out", &"<redacted>")
            .field("process_service", &"<redacted>")
            .field("object_root", &"<redacted>")
            .finish()
    }
}

impl Env {
    pub const fn new(
        console_in: CapId,
        console_out: CapId,
        process_service: CapId,
        object_root: CapId,
    ) -> Self {
        Self {
            console_in,
            console_out,
            process_service,
            object_root,
        }
    }

    pub const fn console_in(self) -> CapId {
        self.console_in
    }

    pub const fn console_out(self) -> CapId {
        self.console_out
    }

    pub const fn process_service(self) -> CapId {
        self.process_service
    }

    pub const fn object_root(self) -> CapId {
        self.object_root
    }
}

#[cfg(test)]
mod tests {
    use aesynx_abi::CapId;
    use core::fmt::{self, Write};

    use super::Env;

    #[test]
    fn env_capability_ids_are_read_only() {
        let env = Env::new(CapId::new(1), CapId::new(2), CapId::new(3), CapId::new(4));

        assert_eq!(env.console_in(), CapId::new(1));
        assert_eq!(env.console_out(), CapId::new(2));
        assert_eq!(env.process_service(), CapId::new(3));
        assert_eq!(env.object_root(), CapId::new(4));
    }

    #[test]
    fn env_debug_redacts_capability_ids() {
        let env = Env::new(
            CapId::new(0xfeed),
            CapId::new(0xbeef),
            CapId::new(0xcafe),
            CapId::new(0xaced),
        );
        let mut rendered = TestBuffer::new();
        assert_eq!(write!(&mut rendered, "{env:?}"), Ok(()));

        assert!(rendered.contains("<redacted>"));
        assert!(!rendered.contains("feed"));
        assert!(!rendered.contains("beef"));
        assert!(!rendered.contains("cafe"));
        assert!(!rendered.contains("aced"));
    }

    struct TestBuffer {
        bytes: [u8; 256],
        len: usize,
    }

    impl TestBuffer {
        const fn new() -> Self {
            Self {
                bytes: [0; 256],
                len: 0,
            }
        }

        fn contains(&self, needle: &str) -> bool {
            let needle = needle.as_bytes();
            if needle.is_empty() {
                return true;
            }
            if needle.len() > self.len {
                return false;
            }

            let mut start = 0usize;
            while start + needle.len() <= self.len {
                if &self.bytes[start..start + needle.len()] == needle {
                    return true;
                }
                start += 1;
            }

            false
        }
    }

    impl Write for TestBuffer {
        fn write_str(&mut self, value: &str) -> fmt::Result {
            let bytes = value.as_bytes();
            let Some(end) = self.len.checked_add(bytes.len()) else {
                return Err(fmt::Error);
            };
            if end > self.bytes.len() {
                return Err(fmt::Error);
            }

            self.bytes[self.len..end].copy_from_slice(bytes);
            self.len = end;
            Ok(())
        }
    }
}
