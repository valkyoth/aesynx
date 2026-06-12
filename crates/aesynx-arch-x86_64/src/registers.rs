use core::fmt;

use aesynx_abi::PhysAddr;

use crate::RFLAGS_PUBLIC_MASK;

const PAGE_OFFSET_MASK: u64 = 0xfff;
const RFLAGS_INTERRUPT_ENABLE: u64 = 1 << 9;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct EarlyRegisterSnapshot {
    stack_pointer: u64,
    frame_pointer: u64,
    rflags: u64,
    cr3: u64,
}

impl EarlyRegisterSnapshot {
    #[must_use]
    pub fn capture() -> Self {
        let stack_pointer: u64;
        let frame_pointer: u64;
        let rflags: u64;
        let cr3: u64;

        // SAFETY: These instructions copy architectural register values into
        // general-purpose outputs. They do not dereference pointers or create
        // Rust references. `pushfq; pop` temporarily uses the current stack only
        // to read RFLAGS.
        unsafe {
            core::arch::asm!(
                "mov {stack_pointer}, rsp",
                "mov {frame_pointer}, rbp",
                "pushfq",
                "pop {rflags}",
                "mov {cr3}, cr3",
                stack_pointer = lateout(reg) stack_pointer,
                frame_pointer = lateout(reg) frame_pointer,
                rflags = lateout(reg) rflags,
                cr3 = lateout(reg) cr3,
                options(preserves_flags)
            );
        }

        Self {
            stack_pointer,
            frame_pointer,
            rflags,
            cr3,
        }
    }

    #[must_use]
    pub const fn stack_pointer_present(self) -> bool {
        self.stack_pointer != 0
    }

    #[must_use]
    pub const fn frame_pointer_present(self) -> bool {
        self.frame_pointer != 0
    }

    #[must_use]
    pub const fn stack_pointer_alignment(self) -> u16 {
        (self.stack_pointer & 0xf) as u16
    }

    #[must_use]
    pub const fn frame_pointer_alignment(self) -> u16 {
        (self.frame_pointer & 0xf) as u16
    }

    #[must_use]
    pub const fn public_rflags(self) -> u64 {
        self.rflags & RFLAGS_PUBLIC_MASK
    }

    #[must_use]
    pub const fn cr3_page_offset(self) -> u16 {
        (self.cr3 & PAGE_OFFSET_MASK) as u16
    }

    #[must_use]
    pub const fn cr3_page_matches(self, phys: PhysAddr) -> bool {
        (self.cr3 & !PAGE_OFFSET_MASK) == (phys.get() & !PAGE_OFFSET_MASK)
    }
}

impl fmt::Debug for EarlyRegisterSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EarlyRegisterSnapshot")
            .field("stack_pointer", &"redacted")
            .field("frame_pointer", &"redacted")
            .field("stack_pointer_alignment", &self.stack_pointer_alignment())
            .field("frame_pointer_alignment", &self.frame_pointer_alignment())
            .field("rflags", &self.public_rflags())
            .field("cr3", &"redacted")
            .field("cr3_page_offset", &self.cr3_page_offset())
            .finish()
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct FaultRegisterSnapshot {
    fault_address: u64,
    rflags: u64,
    cr3: u64,
}

impl FaultRegisterSnapshot {
    #[must_use]
    pub fn capture() -> Self {
        let fault_address = Self::capture_fault_address();
        Self::capture_with_fault_address(fault_address)
    }

    #[must_use]
    pub(crate) fn capture_fault_address() -> u64 {
        let fault_address: u64;

        // SAFETY: This copies CR2 into a general-purpose register. It does not
        // dereference pointers or create Rust references.
        unsafe {
            core::arch::asm!(
                "mov {fault_address}, cr2",
                fault_address = lateout(reg) fault_address,
                options(nomem, nostack, preserves_flags)
            );
        }

        fault_address
    }

    #[must_use]
    pub(crate) fn capture_with_fault_address(fault_address: u64) -> Self {
        let rflags: u64;
        let cr3: u64;

        // SAFETY: These instructions copy architectural register values into
        // general-purpose outputs. CR3 is copied only for a redacted low-bit
        // summary. `pushfq; pop` temporarily uses the current aligned exception
        // stack to read RFLAGS.
        unsafe {
            core::arch::asm!(
                "pushfq",
                "pop {rflags}",
                "mov {cr3}, cr3",
                rflags = lateout(reg) rflags,
                cr3 = lateout(reg) cr3,
                options(preserves_flags)
            );
        }

        Self {
            fault_address,
            rflags,
            cr3,
        }
    }

    #[must_use]
    pub const fn fault_address_present(self) -> bool {
        self.fault_address != 0
    }

    #[must_use]
    pub const fn fault_address_page_offset(self) -> u16 {
        (self.fault_address & PAGE_OFFSET_MASK) as u16
    }

    #[must_use]
    pub const fn public_rflags(self) -> u64 {
        self.rflags & RFLAGS_PUBLIC_MASK
    }

    #[must_use]
    pub const fn interrupts_enabled(self) -> bool {
        self.rflags & RFLAGS_INTERRUPT_ENABLE != 0
    }

    #[must_use]
    pub const fn cr3_page_offset(self) -> u16 {
        (self.cr3 & PAGE_OFFSET_MASK) as u16
    }

    #[must_use]
    pub const fn cr3_page_matches(self, phys: PhysAddr) -> bool {
        (self.cr3 & !PAGE_OFFSET_MASK) == (phys.get() & !PAGE_OFFSET_MASK)
    }
}

impl fmt::Debug for FaultRegisterSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FaultRegisterSnapshot")
            .field("fault_address", &"redacted")
            .field("fault_address_present", &self.fault_address_present())
            .field(
                "fault_address_page_offset",
                &self.fault_address_page_offset(),
            )
            .field("rflags", &self.public_rflags())
            .field("interrupts_enabled", &self.interrupts_enabled())
            .field("cr3", &"redacted")
            .field("cr3_page_offset", &self.cr3_page_offset())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use core::fmt::{self, Write};

    use aesynx_abi::PhysAddr;

    use super::{EarlyRegisterSnapshot, FaultRegisterSnapshot};

    #[test]
    fn register_snapshot_debug_redacts_address_values() {
        let snapshot = EarlyRegisterSnapshot {
            stack_pointer: 0xffff_ffff_8000_1008,
            frame_pointer: 0xffff_ffff_8000_2000,
            rflags: 0x246,
            cr3: 0x1234_5000,
        };
        let mut output = FixedBuf::default();

        assert_eq!(
            write!(&mut output, "{snapshot:?}").map(|()| output.contains("redacted")),
            Ok(true)
        );
        assert!(!output.contains("ffff"));
        assert!(!output.contains("12345"));
    }

    #[test]
    fn register_snapshot_exposes_only_redacted_summary() {
        let snapshot = EarlyRegisterSnapshot {
            stack_pointer: 0xffff_ffff_8000_1008,
            frame_pointer: 0xffff_ffff_8000_2000,
            rflags: 0xffff_ffff_0000_0ed7,
            cr3: 0x1234_5abc,
        };

        assert!(snapshot.stack_pointer_present());
        assert!(snapshot.frame_pointer_present());
        assert_eq!(snapshot.stack_pointer_alignment(), 8);
        assert_eq!(snapshot.frame_pointer_alignment(), 0);
        assert_eq!(snapshot.public_rflags(), 0x0cd5);
        assert_eq!(snapshot.cr3_page_offset(), 0xabc);
        assert!(snapshot.cr3_page_matches(PhysAddr::new(0x1234_5000)));
        assert!(snapshot.cr3_page_matches(PhysAddr::new(0x1234_5fff)));
        assert!(!snapshot.cr3_page_matches(PhysAddr::new(0x1234_6000)));
    }

    #[test]
    fn public_rflags_excludes_debug_interrupt_and_privilege_state() {
        const TF_IF_IOPL_AC: u64 = (1 << 8) | (1 << 9) | (3 << 12) | (1 << 18);
        let snapshot = EarlyRegisterSnapshot {
            stack_pointer: 0,
            frame_pointer: 0,
            rflags: TF_IF_IOPL_AC,
            cr3: 0,
        };

        assert_eq!(snapshot.public_rflags(), 0);
    }

    #[test]
    fn fault_snapshot_exposes_only_redacted_fault_address_summary() {
        let snapshot = FaultRegisterSnapshot {
            fault_address: 0xffff_ffff_8000_dead,
            rflags: 0xffff_ffff_0000_0ed7,
            cr3: 0x1234_5abc,
        };
        let mut output = FixedBuf::default();

        assert_eq!(
            write!(&mut output, "{snapshot:?}").map(|()| output.contains("redacted")),
            Ok(true)
        );
        assert!(snapshot.fault_address_present());
        assert_eq!(snapshot.fault_address_page_offset(), 0xead);
        assert_eq!(snapshot.public_rflags(), 0x0cd5);
        assert!(snapshot.interrupts_enabled());
        assert_eq!(snapshot.cr3_page_offset(), 0xabc);
        assert!(snapshot.cr3_page_matches(PhysAddr::new(0x1234_5000)));
        assert!(!snapshot.cr3_page_matches(PhysAddr::new(0x1234_4000)));
        assert!(!output.contains("ffff"));
        assert!(!output.contains("dead"));
        assert!(!output.contains("12345"));
    }

    struct FixedBuf {
        bytes: [u8; 256],
        len: usize,
    }

    impl Default for FixedBuf {
        fn default() -> Self {
            Self {
                bytes: [0; 256],
                len: 0,
            }
        }
    }

    impl FixedBuf {
        fn contains(&self, needle: &str) -> bool {
            self.as_str().contains(needle)
        }

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
