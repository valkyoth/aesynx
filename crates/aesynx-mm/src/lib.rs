#![no_std]
#![deny(unsafe_code)]

use aesynx_abi::PhysFrame;

mod frame_allocator;

pub use frame_allocator::{
    AllocatedFrames, BitmapFrameAllocator, FRAME_SIZE, FrameAllocatorError, FrameAllocatorStatus,
    FrameRegionKind, FrameState,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AddressSpace {
    root: PhysFrame,
}

impl AddressSpace {
    #[must_use]
    pub const fn new(root: PhysFrame) -> Self {
        Self { root }
    }

    #[must_use]
    pub const fn root(self) -> PhysFrame {
        self.root
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GenericPageFlags {
    pub access: PageAccess,
    pub privilege: PagePrivilege,
    global: bool,
    pub device_memory: bool,
    pub cacheable: bool,
}

impl GenericPageFlags {
    #[must_use]
    pub const fn kernel(access: PageAccess) -> Self {
        Self {
            access,
            privilege: PagePrivilege::Kernel,
            global: false,
            device_memory: false,
            cacheable: true,
        }
    }

    #[must_use]
    pub const fn user(access: PageAccess) -> Self {
        Self {
            access,
            privilege: PagePrivilege::User,
            global: false,
            device_memory: false,
            cacheable: true,
        }
    }

    #[must_use]
    pub const fn device(mut self) -> Self {
        self.device_memory = true;
        self.cacheable = false;
        self
    }

    pub const fn with_global(mut self) -> Result<Self, MmError> {
        if matches!(self.privilege, PagePrivilege::User) {
            return Err(MmError::GlobalUserMappingNotAllowed);
        }

        self.global = true;
        Ok(self)
    }

    #[must_use]
    pub const fn is_global(self) -> bool {
        self.global
    }
}

impl Default for GenericPageFlags {
    fn default() -> Self {
        Self::kernel(PageAccess::ReadOnly)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PageAccess {
    #[default]
    ReadOnly,
    ReadWrite,
    ReadExecute,
}

impl PageAccess {
    #[must_use]
    pub const fn readable(self) -> bool {
        matches!(self, Self::ReadOnly | Self::ReadWrite | Self::ReadExecute)
    }

    #[must_use]
    pub const fn writable(self) -> bool {
        matches!(self, Self::ReadWrite)
    }

    #[must_use]
    pub const fn executable(self) -> bool {
        matches!(self, Self::ReadExecute)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PagePrivilege {
    #[default]
    Kernel,
    User,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MmError {
    GlobalUserMappingNotAllowed,
}

#[cfg(test)]
mod tests {
    use super::{GenericPageFlags, PageAccess, PagePrivilege};

    #[test]
    fn page_access_cannot_be_write_and_execute() {
        assert!(PageAccess::ReadOnly.readable());
        assert!(PageAccess::ReadWrite.writable());
        assert!(!PageAccess::ReadWrite.executable());
        assert!(!PageAccess::ReadExecute.writable());
        assert!(PageAccess::ReadExecute.executable());
    }

    #[test]
    fn user_mapping_is_explicit() {
        let flags = GenericPageFlags::user(PageAccess::ReadOnly);

        assert_eq!(flags.privilege, PagePrivilege::User);
        assert_eq!(flags.access, PageAccess::ReadOnly);
    }

    #[test]
    fn only_kernel_mappings_can_be_global() {
        assert_eq!(
            GenericPageFlags::user(PageAccess::ReadOnly).with_global(),
            Err(super::MmError::GlobalUserMappingNotAllowed)
        );
        assert_eq!(
            GenericPageFlags::kernel(PageAccess::ReadOnly)
                .with_global()
                .map(GenericPageFlags::is_global),
            Ok(true)
        );
    }
}
