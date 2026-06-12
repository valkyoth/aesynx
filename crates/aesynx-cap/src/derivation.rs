use aesynx_abi::{PrincipalId, VirtAddr};

use crate::CapPerms;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeriveRequest {
    pub perms: CapPerms,
    pub owner: PrincipalId,
    pub base: Option<VirtAddr>,
    pub len: Option<u64>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeriveError {
    MissingDerivePermission,
    MissingGrantPermission,
    ParentRevoked,
    ParentStaleGeneration,
    AuditRejected,
    PermissionsEscalate,
    RangeEscalates,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapValidationError {
    Revoked,
    StaleGeneration,
}
