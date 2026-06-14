#![no_std]
#![forbid(unsafe_code)]

#[cfg(test)]
extern crate alloc;

mod capability;
mod derivation;
mod id;
mod kind;
mod memory;
mod perms;
mod revocation;
mod table;

pub use capability::{
    CapAuditAction, CapAuditError, CapAuditEvent, CapAuditLog, Capability, RedactedCapAuditEvent,
};
pub use derivation::{CapValidationError, DeriveError, DeriveRequest, ObjectBoundedRange};
pub use id::{CapGeneration, CapIdError, CapIdParts, CapSlotIndex};
pub use kind::CapKind;
pub use memory::{MemoryAccess, MemoryCapError, MemoryMapRequest};
pub use perms::{CapPerms, PermissionBitsError};
pub use revocation::{RevocationEpochStore, RevocationError, ensure_revoke_authority};
pub use table::{
    CapTableError, CapabilityTable, LiveAuthorityError, LiveAuthorityState, LiveAuthorityView,
    RootCapabilitySpec,
};
