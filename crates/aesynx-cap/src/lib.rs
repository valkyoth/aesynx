#![no_std]
#![deny(unsafe_code)]

mod capability;
mod derivation;
mod id;
mod kind;
mod perms;
mod revocation;
mod table;

pub use capability::{CapAuditAction, CapAuditEvent, CapAuditLog, Capability};
pub use derivation::{CapValidationError, DeriveError, DeriveRequest};
pub use id::{CapGeneration, CapIdError, CapIdParts, CapSlotIndex};
pub use kind::CapKind;
pub use perms::{CapPerms, PermissionBitsError};
pub use revocation::{RevocationEpochStore, RevocationError, ensure_revoke_authority};
pub use table::{CapTableError, CapabilityTable};
