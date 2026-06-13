use aesynx_abi::{CapId, ObjectId, PrincipalId, VirtAddr};
use aesynx_cap::{
    CapAuditEvent, CapAuditLog, CapKind, CapPerms, CapTableError, CapabilityTable, DeriveError,
    DeriveRequest,
};

const ROOT_OWNER: PrincipalId = PrincipalId::new(1);
const CHILD_OWNER: PrincipalId = PrincipalId::new(2);
const OBJECT: ObjectId = ObjectId::new(0x0a20);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapabilitySmokeStatus {
    pub capacity: usize,
    pub occupied_before_revoke: usize,
    pub occupied_after_revoke: usize,
    pub root_read_ok: bool,
    pub child_read_ok: bool,
    pub child_write_denied: bool,
    pub stale_root_denied: bool,
    pub stale_child_denied: bool,
    pub audit_events: usize,
    pub revoked_slots: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilitySmokeError {
    AuditRejected,
    Table(CapTableError),
    UnexpectedAuthorityState,
}

pub fn run() -> Result<CapabilitySmokeStatus, CapabilitySmokeError> {
    let mut table = CapabilityTable::<8>::new();
    let mut audit = SmokeAudit::new();
    let root = insert_root(&mut table)?;
    let root_read_ok = table.check(root, CapPerms::READ).is_ok();
    let child = derive_child(&mut table, root, &mut audit)?;
    let child_read_ok = table.check(child, CapPerms::READ).is_ok();
    let child_write_denied =
        table.check(child, CapPerms::WRITE).map(|_| ()) == Err(CapTableError::MissingPermission);
    let occupied_before_revoke = table.occupied_slots();
    let revoked_slots = table.revoke(root, child)?;
    let stale_root_denied =
        table.check(root, CapPerms::READ).map(|_| ()) == Err(CapTableError::StaleId);
    let stale_child_denied =
        table.check(child, CapPerms::READ).map(|_| ()) == Err(CapTableError::StaleId);
    let occupied_after_revoke = table.occupied_slots();

    if !(root_read_ok
        && child_read_ok
        && child_write_denied
        && stale_root_denied
        && stale_child_denied
        && occupied_before_revoke == 2
        && occupied_after_revoke == 0
        && audit.len() == 1
        && revoked_slots == 2)
    {
        return Err(CapabilitySmokeError::UnexpectedAuthorityState);
    }

    Ok(CapabilitySmokeStatus {
        capacity: table.capacity(),
        occupied_before_revoke,
        occupied_after_revoke,
        root_read_ok,
        child_read_ok,
        child_write_denied,
        stale_root_denied,
        stale_child_denied,
        audit_events: audit.len(),
        revoked_slots,
    })
}

fn insert_root(table: &mut CapabilityTable<8>) -> Result<CapId, CapTableError> {
    table.insert_root(
        OBJECT,
        CapKind::Memory,
        ROOT_OWNER,
        CapPerms::READ
            .union(CapPerms::WRITE)
            .union(CapPerms::DERIVE)
            .union(CapPerms::GRANT)
            .union(CapPerms::REVOKE),
        1,
        0,
    )
}

fn derive_child(
    table: &mut CapabilityTable<8>,
    root: CapId,
    audit: &mut SmokeAudit,
) -> Result<CapId, CapTableError> {
    table.derive_with_audit(
        root,
        DeriveRequest {
            perms: CapPerms::READ,
            owner: CHILD_OWNER,
            base: Some(VirtAddr::new(0x1000)),
            len: Some(0x1000),
        },
        audit,
    )
}

struct SmokeAudit {
    events: [Option<CapAuditEvent>; 2],
    len: usize,
}

impl SmokeAudit {
    const fn new() -> Self {
        Self {
            events: [None, None],
            len: 0,
        }
    }

    const fn len(&self) -> usize {
        self.len
    }
}

impl CapAuditLog for SmokeAudit {
    fn record(&mut self, event: CapAuditEvent) -> Result<(), DeriveError> {
        if self.len >= self.events.len() {
            return Err(DeriveError::AuditRejected);
        }

        self.events[self.len] = Some(event);
        self.len += 1;
        Ok(())
    }
}

impl From<CapTableError> for CapabilitySmokeError {
    fn from(error: CapTableError) -> Self {
        match error {
            CapTableError::AuditRejected => Self::AuditRejected,
            error => Self::Table(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::run;

    #[test]
    fn capability_smoke_exercises_table_lifecycle() {
        let result = run();

        assert!(result.is_ok());
        if let Ok(status) = result {
            assert_eq!(status.capacity, 8);
            assert_eq!(status.occupied_before_revoke, 2);
            assert_eq!(status.occupied_after_revoke, 0);
            assert!(status.root_read_ok);
            assert!(status.child_read_ok);
            assert!(status.child_write_denied);
            assert!(status.stale_root_denied);
            assert!(status.stale_child_denied);
            assert_eq!(status.audit_events, 1);
            assert_eq!(status.revoked_slots, 2);
        }
    }
}
