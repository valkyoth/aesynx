use aesynx_abi::VirtAddr;

use crate::{CapKind, CapPerms, Capability};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryAccess {
    ReadOnly,
    ReadWrite,
    ReadExecute,
}

impl MemoryAccess {
    #[must_use]
    pub const fn required_perms(self) -> CapPerms {
        match self {
            Self::ReadOnly => CapPerms::READ,
            Self::ReadWrite => CapPerms::READ.union(CapPerms::WRITE),
            Self::ReadExecute => CapPerms::READ.union(CapPerms::EXECUTE),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryMapRequest {
    base: VirtAddr,
    len: u64,
    access: MemoryAccess,
}

impl MemoryMapRequest {
    pub const fn new(
        base: VirtAddr,
        len: u64,
        access: MemoryAccess,
    ) -> Result<Self, MemoryCapError> {
        if len == 0 {
            return Err(MemoryCapError::InvalidRange);
        }
        if base.get().checked_add(len).is_none() {
            return Err(MemoryCapError::InvalidRange);
        }

        Ok(Self { base, len, access })
    }

    #[must_use]
    pub const fn base(self) -> VirtAddr {
        self.base
    }

    #[must_use]
    pub const fn byte_len(self) -> u64 {
        self.len
    }

    #[must_use]
    pub const fn access(self) -> MemoryAccess {
        self.access
    }

    #[must_use]
    pub const fn required_perms(self) -> CapPerms {
        CapPerms::MAP.union(self.access.required_perms())
    }
}

impl Capability {
    pub const fn authorize_memory_map(
        &self,
        request: MemoryMapRequest,
    ) -> Result<(), MemoryCapError> {
        if !matches!(self.kind(), CapKind::Memory) {
            return Err(MemoryCapError::WrongCapabilityKind);
        }
        if !range_is_subset(
            self.base(),
            self.range_len(),
            request.base(),
            request.byte_len(),
        ) {
            return Err(MemoryCapError::RangeEscapesCapability);
        }
        if !self.perms().contains(CapPerms::MAP) {
            return Err(MemoryCapError::MissingMapPermission);
        }
        if !self.perms().contains(CapPerms::READ) {
            return Err(MemoryCapError::MissingReadPermission);
        }
        if matches!(request.access(), MemoryAccess::ReadWrite)
            && !self.perms().contains(CapPerms::WRITE)
        {
            return Err(MemoryCapError::MissingWritePermission);
        }
        if matches!(request.access(), MemoryAccess::ReadExecute)
            && !self.perms().contains(CapPerms::EXECUTE)
        {
            return Err(MemoryCapError::MissingExecutePermission);
        }

        Ok(())
    }
}

const fn range_is_subset(
    parent_base: Option<VirtAddr>,
    parent_len: Option<u64>,
    child_base: VirtAddr,
    child_len: u64,
) -> bool {
    match (parent_base, parent_len) {
        (None, None) => child_base.get().checked_add(child_len).is_some(),
        (Some(parent_base), Some(parent_len)) => {
            bounded_range_contains(parent_base.get(), parent_len, child_base.get(), child_len)
        }
        _ => false,
    }
}

const fn bounded_range_contains(
    parent_base: u64,
    parent_len: u64,
    child_base: u64,
    child_len: u64,
) -> bool {
    let Some(parent_end) = parent_base.checked_add(parent_len) else {
        return false;
    };
    let Some(child_end) = child_base.checked_add(child_len) else {
        return false;
    };

    child_base >= parent_base && child_end <= parent_end
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryCapError {
    InvalidRange,
    MissingExecutePermission,
    MissingMapPermission,
    MissingReadPermission,
    MissingWritePermission,
    RangeEscapesCapability,
    WrongCapabilityKind,
}

#[cfg(test)]
mod tests;
