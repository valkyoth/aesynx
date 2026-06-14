use aesynx_abi::{CapId, ObjectId, PhysAddr, PrincipalId, VirtAddr};
use aesynx_cap::{
    CapAuditAction, CapAuditError, CapAuditEvent, CapAuditLog, CapKind, CapPerms, CapTableError,
    Capability, CapabilityTable, DeriveRequest, LiveAuthorityError, LiveAuthorityState,
    LiveAuthorityView, MemoryAccess, MemoryCapError, MemoryMapRequest, ObjectBoundedRange,
    RootCapabilitySpec,
};
use aesynx_mm::{FRAME_SIZE, GenericPageFlags, PageAccess, PageMapping, PageTableError};
use aesynx_telemetry::{CapFaultKind, CoreTelemetry, TelemetryError};

const ROOT_OWNER: PrincipalId = PrincipalId::new(1);
const CHILD_OWNER: PrincipalId = PrincipalId::new(2);
const GRANT_OWNER: PrincipalId = PrincipalId::new(3);
const OBJECT: ObjectId = ObjectId::new(0x0a20);
const MEMORY_VIRT: VirtAddr = VirtAddr::new(0x1000);
const MEMORY_PHYS: PhysAddr = PhysAddr::new(0x0030_0000);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapabilitySmokeStatus {
    pub capacity: usize,
    pub occupied_before_revoke: usize,
    pub occupied_after_revoke: usize,
    pub root_read_ok: bool,
    pub child_read_ok: bool,
    pub grant_read_ok: bool,
    pub grant_regrant_denied: bool,
    pub child_write_denied: bool,
    pub memory_map_allowed: bool,
    pub memory_mapping_descriptor_ok: bool,
    pub memory_read_denied: bool,
    pub memory_write_denied: bool,
    pub memory_range_escape_denied: bool,
    pub stale_root_denied: bool,
    pub stale_child_denied: bool,
    pub audit_events: usize,
    pub mint_audit_seen: bool,
    pub derive_audit_seen: bool,
    pub grant_audit_seen: bool,
    pub revoke_audit_seen: bool,
    pub revoke_audit_slots: u32,
    pub cap_fault_events: u64,
    pub revoked_slots: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilitySmokeError {
    AuditRejected,
    Memory(MemoryCapError),
    Mapping(PageTableError),
    Table(CapTableError),
    Telemetry(TelemetryError),
    UnexpectedAuthorityState,
}

pub fn run() -> Result<CapabilitySmokeStatus, CapabilitySmokeError> {
    let mut table = CapabilityTable::<8>::new();
    let mut audit = SmokeAudit::new();
    let live = SmokeLiveAuthority;
    let telemetry = CoreTelemetry::default();
    let root = insert_root(&mut table, &mut audit)?;
    let root_read_ok = table.check(root, CapPerms::READ).is_ok();
    let child = derive_child(&mut table, root, &live, &mut audit)?;
    let child_read_ok = table.check(child, CapPerms::READ).is_ok();
    let grant = table.grant_with_audit(root, GRANT_OWNER, &live, &mut audit)?;
    let grant_read_ok = table.check(grant, CapPerms::READ).is_ok();
    let grant_regrant_denied =
        table.check(grant, CapPerms::GRANT).map(|_| ()) == Err(CapTableError::MissingPermission);
    if grant_regrant_denied {
        telemetry.record_cap_fault(CapFaultKind::MissingPermission)?;
    }
    let child_write_denied =
        table.check(child, CapPerms::WRITE).map(|_| ()) == Err(CapTableError::MissingPermission);
    if child_write_denied {
        telemetry.record_cap_fault(CapFaultKind::MissingPermission)?;
    }
    let readless_root = insert_readless_root(&mut table, &mut audit)?;
    let child_cap = table.get(child)?;
    let memory_map_allowed = child_cap
        .authorize_memory_map(MemoryMapRequest::new(
            MEMORY_VIRT,
            FRAME_SIZE,
            MemoryAccess::ReadOnly,
        )?)
        .is_ok();
    let memory_mapping_descriptor_ok = checked_mapping_with_memory_cap(
        child_cap,
        MEMORY_VIRT,
        MEMORY_PHYS,
        GenericPageFlags::kernel(PageAccess::ReadOnly),
    )
    .is_ok();
    let readless_cap = table.get(readless_root)?;
    let memory_read_denied = readless_cap.authorize_memory_map(MemoryMapRequest::new(
        MEMORY_VIRT,
        FRAME_SIZE,
        MemoryAccess::ReadOnly,
    )?) == Err(MemoryCapError::MissingReadPermission);
    let memory_write_denied = child_cap.authorize_memory_map(MemoryMapRequest::new(
        MEMORY_VIRT,
        FRAME_SIZE,
        MemoryAccess::ReadWrite,
    )?) == Err(MemoryCapError::MissingWritePermission);
    let memory_range_escape_denied = child_cap.authorize_memory_map(MemoryMapRequest::new(
        VirtAddr::new(MEMORY_VIRT.get() + FRAME_SIZE),
        FRAME_SIZE,
        MemoryAccess::ReadOnly,
    )?) == Err(MemoryCapError::RangeEscapesCapability);
    let occupied_before_revoke = table.occupied_slots();
    let revoked_slots = table.revoke_with_audit(root, child, &live, &mut audit)?;
    let stale_root_denied =
        table.check(root, CapPerms::READ).map(|_| ()) == Err(CapTableError::StaleId);
    if stale_root_denied {
        telemetry.record_cap_fault(CapFaultKind::StaleId)?;
    }
    let stale_child_denied =
        table.check(child, CapPerms::READ).map(|_| ()) == Err(CapTableError::StaleId);
    if stale_child_denied {
        telemetry.record_cap_fault(CapFaultKind::StaleId)?;
    }
    let occupied_after_revoke = table.occupied_slots();
    let mint_audit_seen = audit.seen(CapAuditAction::Mint);
    let derive_audit_seen = audit.seen(CapAuditAction::Derive);
    let grant_audit_seen = audit.seen(CapAuditAction::Grant);
    let revoke_audit_seen = audit.seen(CapAuditAction::Revoke);
    let revoke_audit_slots = audit
        .last_for_action(CapAuditAction::Revoke)
        .map_or(0, |event| event.affected_slots);
    let cap_fault_events = telemetry.snapshot().cap_faults;

    if !(root_read_ok
        && child_read_ok
        && grant_read_ok
        && grant_regrant_denied
        && child_write_denied
        && memory_map_allowed
        && memory_mapping_descriptor_ok
        && memory_read_denied
        && memory_write_denied
        && memory_range_escape_denied
        && stale_root_denied
        && stale_child_denied
        && occupied_before_revoke == 4
        && occupied_after_revoke == 1
        && audit.len() == 5
        && mint_audit_seen
        && derive_audit_seen
        && grant_audit_seen
        && revoke_audit_seen
        && revoke_audit_slots == 3
        && cap_fault_events == 4
        && revoked_slots == 3)
    {
        return Err(CapabilitySmokeError::UnexpectedAuthorityState);
    }

    Ok(CapabilitySmokeStatus {
        capacity: table.capacity(),
        occupied_before_revoke,
        occupied_after_revoke,
        root_read_ok,
        child_read_ok,
        grant_read_ok,
        grant_regrant_denied,
        child_write_denied,
        memory_map_allowed,
        memory_mapping_descriptor_ok,
        memory_read_denied,
        memory_write_denied,
        memory_range_escape_denied,
        stale_root_denied,
        stale_child_denied,
        audit_events: audit.len(),
        mint_audit_seen,
        derive_audit_seen,
        grant_audit_seen,
        revoke_audit_seen,
        revoke_audit_slots,
        cap_fault_events,
        revoked_slots,
    })
}

fn insert_root(
    table: &mut CapabilityTable<8>,
    audit: &mut SmokeAudit,
) -> Result<CapId, CapTableError> {
    table.insert_root_with_audit(
        RootCapabilitySpec {
            object_id: OBJECT,
            kind: CapKind::Memory,
            owner: ROOT_OWNER,
            perms: CapPerms::READ
                .union(CapPerms::WRITE)
                .union(CapPerms::MAP)
                .union(CapPerms::DERIVE)
                .union(CapPerms::GRANT)
                .union(CapPerms::REVOKE),
            object_generation: 1,
            revocation_epoch: 0,
        },
        audit,
    )
}

fn insert_readless_root(
    table: &mut CapabilityTable<8>,
    audit: &mut SmokeAudit,
) -> Result<CapId, CapTableError> {
    table.insert_root_with_audit(
        RootCapabilitySpec {
            object_id: ObjectId::new(0x0a21),
            kind: CapKind::Memory,
            owner: ROOT_OWNER,
            perms: CapPerms::MAP,
            object_generation: 1,
            revocation_epoch: 0,
        },
        audit,
    )
}

fn derive_child(
    table: &mut CapabilityTable<8>,
    root: CapId,
    live: &SmokeLiveAuthority,
    audit: &mut SmokeAudit,
) -> Result<CapId, CapTableError> {
    let range = ObjectBoundedRange::new_within_extent(
        VirtAddr::new(0x1000),
        0x1000,
        VirtAddr::new(0),
        u64::MAX,
    )?;
    table.derive_with_audit(
        root,
        DeriveRequest::bounded(CapPerms::READ.union(CapPerms::MAP), CHILD_OWNER, range),
        live,
        audit,
    )
}

struct SmokeLiveAuthority;

impl LiveAuthorityView for SmokeLiveAuthority {
    fn live_authority(
        &self,
        object_id: ObjectId,
    ) -> Result<LiveAuthorityState, LiveAuthorityError> {
        if object_id == OBJECT || object_id == ObjectId::new(0x0a21) {
            Ok(LiveAuthorityState::new(1, 0))
        } else {
            Err(LiveAuthorityError::ObjectNotFound)
        }
    }
}

fn checked_mapping_with_memory_cap(
    cap: &Capability,
    virt: VirtAddr,
    phys: PhysAddr,
    flags: GenericPageFlags,
) -> Result<(), CapabilitySmokeError> {
    let request = MemoryMapRequest::new(virt, FRAME_SIZE, memory_access_for_flags(flags))?;
    cap.authorize_memory_map(request)?;
    PageMapping::new_checked(phys, flags)?;
    Ok(())
}

const fn memory_access_for_flags(flags: GenericPageFlags) -> MemoryAccess {
    match flags.access() {
        PageAccess::ReadOnly => MemoryAccess::ReadOnly,
        PageAccess::ReadWrite => MemoryAccess::ReadWrite,
        PageAccess::ReadExecute => MemoryAccess::ReadExecute,
    }
}

struct SmokeAudit {
    events: [Option<CapAuditEvent>; 5],
    len: usize,
}

impl SmokeAudit {
    const fn new() -> Self {
        Self {
            events: [None, None, None, None, None],
            len: 0,
        }
    }

    const fn len(&self) -> usize {
        self.len
    }
}

impl CapAuditLog for SmokeAudit {
    fn record(&mut self, event: CapAuditEvent) -> Result<(), CapAuditError> {
        if self.len >= self.events.len() {
            return Err(CapAuditError::Rejected);
        }

        self.events[self.len] = Some(event);
        self.len += 1;
        Ok(())
    }
}

impl SmokeAudit {
    fn seen(&self, action: CapAuditAction) -> bool {
        let mut index = 0usize;
        while index < self.len {
            if self.events[index].is_some_and(|event| event.action == action) {
                return true;
            }
            index += 1;
        }
        false
    }

    fn last_for_action(&self, action: CapAuditAction) -> Option<CapAuditEvent> {
        let mut index = self.len;
        while index > 0 {
            index -= 1;
            if self.events[index].is_some_and(|event| event.action == action) {
                return self.events[index];
            }
        }
        None
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

impl From<MemoryCapError> for CapabilitySmokeError {
    fn from(error: MemoryCapError) -> Self {
        Self::Memory(error)
    }
}

impl From<PageTableError> for CapabilitySmokeError {
    fn from(error: PageTableError) -> Self {
        Self::Mapping(error)
    }
}

impl From<TelemetryError> for CapabilitySmokeError {
    fn from(error: TelemetryError) -> Self {
        Self::Telemetry(error)
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
            assert_eq!(status.occupied_before_revoke, 4);
            assert_eq!(status.occupied_after_revoke, 1);
            assert!(status.root_read_ok);
            assert!(status.child_read_ok);
            assert!(status.grant_read_ok);
            assert!(status.grant_regrant_denied);
            assert!(status.child_write_denied);
            assert!(status.memory_map_allowed);
            assert!(status.memory_mapping_descriptor_ok);
            assert!(status.memory_read_denied);
            assert!(status.memory_write_denied);
            assert!(status.memory_range_escape_denied);
            assert!(status.stale_root_denied);
            assert!(status.stale_child_denied);
            assert_eq!(status.audit_events, 5);
            assert!(status.mint_audit_seen);
            assert!(status.derive_audit_seen);
            assert!(status.grant_audit_seen);
            assert!(status.revoke_audit_seen);
            assert_eq!(status.revoke_audit_slots, 3);
            assert_eq!(status.cap_fault_events, 4);
            assert_eq!(status.revoked_slots, 3);
        }
    }
}
