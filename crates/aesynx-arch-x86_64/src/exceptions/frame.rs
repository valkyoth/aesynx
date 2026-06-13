use core::mem::{offset_of, size_of};

use crate::RFLAGS_PUBLIC_MASK;

use super::PAGE_OFFSET_MASK;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ExceptionFrame {
    pub(super) vector: u8,
    pub(super) error_code: u64,
    pub(super) instruction_pointer: u64,
    pub(super) code_segment: u64,
    pub(super) rflags: u64,
}

impl ExceptionFrame {
    pub(super) fn from_raw(raw: *const RawExceptionFrame) -> Option<Self> {
        if raw.is_null() || raw.align_offset(core::mem::align_of::<RawExceptionFrame>()) != 0 {
            return None;
        }

        // SAFETY: The assembly stubs pass a pointer to the active exception
        // stack frame. Only value fields needed for bounded diagnostics are
        // copied, and no reference escapes this function.
        let raw = unsafe { raw.read() };
        let vector = u8::try_from(raw.vector).ok()?;
        Some(Self {
            vector,
            error_code: raw.error_code,
            instruction_pointer: raw.instruction_pointer,
            code_segment: raw.code_segment,
            rflags: raw.rflags,
        })
    }

    pub(super) const fn public_rflags(self) -> u64 {
        self.rflags & RFLAGS_PUBLIC_MASK
    }

    pub(super) const fn instruction_pointer_present(self) -> bool {
        self.instruction_pointer != 0
    }

    pub(super) const fn instruction_pointer_offset(self) -> u16 {
        (self.instruction_pointer & PAGE_OFFSET_MASK) as u16
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PageFaultErrorCode(u64);

impl PageFaultErrorCode {
    pub(super) const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub(super) const fn present(self) -> bool {
        self.0 & (1 << 0) != 0
    }

    pub(super) const fn write(self) -> bool {
        self.0 & (1 << 1) != 0
    }

    pub(super) const fn user(self) -> bool {
        self.0 & (1 << 2) != 0
    }

    pub(super) const fn reserved_bit(self) -> bool {
        self.0 & (1 << 3) != 0
    }

    pub(super) const fn instruction_fetch(self) -> bool {
        self.0 & (1 << 4) != 0
    }

    pub(super) const fn protection_key(self) -> bool {
        self.0 & (1 << 5) != 0
    }

    pub(super) const fn shadow_stack(self) -> bool {
        self.0 & (1 << 6) != 0
    }

    pub(super) const fn sgx(self) -> bool {
        self.0 & (1 << 15) != 0
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct RawExceptionFrame {
    pub(super) vector: u64,
    pub(super) error_code: u64,
    pub(super) instruction_pointer: u64,
    pub(super) code_segment: u64,
    pub(super) rflags: u64,
}

const _: () = assert!(
    size_of::<RawExceptionFrame>() == 40,
    "RawExceptionFrame size must match exception assembly stubs"
);
const _: () = assert!(
    offset_of!(RawExceptionFrame, vector) == 0,
    "RawExceptionFrame.vector offset must match exception assembly stubs"
);
const _: () = assert!(
    offset_of!(RawExceptionFrame, error_code) == 8,
    "RawExceptionFrame.error_code offset must match exception assembly stubs"
);
const _: () = assert!(
    offset_of!(RawExceptionFrame, instruction_pointer) == 16,
    "RawExceptionFrame.instruction_pointer offset must match exception assembly stubs"
);
const _: () = assert!(
    offset_of!(RawExceptionFrame, code_segment) == 24,
    "RawExceptionFrame.code_segment offset must match exception assembly stubs"
);
const _: () = assert!(
    offset_of!(RawExceptionFrame, rflags) == 32,
    "RawExceptionFrame.rflags offset must match exception assembly stubs"
);
