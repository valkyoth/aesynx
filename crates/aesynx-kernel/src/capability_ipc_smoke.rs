use aesynx_abi::{CapId, CoreId, ObjectId, PrincipalId, ROOT_CORE};
use aesynx_cap::{
    CapAuditAction, CapAuditError, CapAuditEvent, CapAuditLog, CapKind, CapPerms, CapTableError,
    CapabilityTable, RevocationEpochStore, RevocationError, RootCapabilitySpec,
};
use aesynx_ipc::{
    CoreValidationError, FabricError, FabricMessage, LiveCoreSet, MessageHeader, MessageKind,
    MessagePayload, MessageRequest, PairwiseSpscQueue, ValidatedCoreId,
};
use aesynx_object::{KernelObject, ObjectCreate, ObjectRegistry, ObjectRegistryError};

const ROOT_OWNER: PrincipalId = PrincipalId::new(1);
const RECEIVER_OWNER: PrincipalId = PrincipalId::new(2);
const OBJECT: ObjectId = ObjectId::new(0x3700);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapabilityIpcSmokeStatus {
    pub grant_seq: u64,
    pub revoke_seq: u64,
    pub receiver_occupied: usize,
    pub grant_message_ok: bool,
    pub receiver_read_ok: bool,
    pub receiver_write_denied: bool,
    pub sender_missing_grant_denied: bool,
    pub revoke_message_ok: bool,
    pub registry_epoch_bumped: bool,
    pub receiver_revoked: bool,
    pub audit_events: usize,
    pub grant_audit_seen: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityIpcSmokeError {
    AuditRejected,
    Cap(CapTableError),
    Core(CoreValidationError),
    Fabric(FabricError),
    Object(ObjectRegistryError),
    Revoke(RevocationError),
    UnexpectedState,
}

pub fn run() -> Result<CapabilityIpcSmokeStatus, CapabilityIpcSmokeError> {
    let live = CapabilityIpcCoreSet;
    let root_core =
        ValidatedCoreId::new(ROOT_CORE, &live).map_err(CapabilityIpcSmokeError::Core)?;
    let peer_core =
        ValidatedCoreId::new(CoreId::new(1), &live).map_err(CapabilityIpcSmokeError::Core)?;
    let mut queue = PairwiseSpscQueue::<2>::new(root_core, peer_core)
        .map_err(CapabilityIpcSmokeError::Fabric)?;
    let mut registry = ObjectRegistry::<2>::new();
    let object = registry
        .create(ObjectCreate::memory(OBJECT, ROOT_CORE))
        .map_err(CapabilityIpcSmokeError::Object)?;
    let mut sender = CapabilityTable::<4>::new();
    let mut receiver = CapabilityTable::<2>::new();
    let mut audit = SmokeAudit::new();
    let root = insert_granting_root(&mut sender, object.generation(), &mut audit)?;
    let read_only = insert_read_only_root(&mut sender, object.generation(), &mut audit)?;

    let receiver_cap = sender.grant_to_table_with_audit(
        root,
        &mut receiver,
        RECEIVER_OWNER,
        &registry,
        &mut audit,
    )?;
    let grant_seq = send_message(
        &mut queue,
        root_core,
        peer_core,
        1,
        MessageKind::GrantCap,
        MessagePayload::Cap(receiver_cap),
    )?;
    let grant_message = queue
        .pop(peer_core.get())
        .map_err(CapabilityIpcSmokeError::Fabric)?
        .into_value();
    let grant_message_ok = grant_message.header().kind() == MessageKind::GrantCap
        && grant_message.header().seq() == grant_seq
        && grant_message.payload() == MessagePayload::Cap(receiver_cap);
    let receiver_read_ok = receiver.check(receiver_cap, CapPerms::READ).is_ok();
    let receiver_write_denied = receiver.check(receiver_cap, CapPerms::WRITE).map(|_| ())
        == Err(CapTableError::MissingPermission);
    let receiver_occupied = receiver.occupied_slots();

    let sender_missing_grant_denied = sender.grant_to_table_with_audit(
        read_only,
        &mut receiver,
        RECEIVER_OWNER,
        &registry,
        &mut audit,
    ) == Err(CapTableError::MissingPermission);

    let root_cap = sender.get(root)?;
    let revoked_epoch = registry
        .revoke_object_live(
            root_cap,
            object.object_id(),
            object.generation(),
            object.revocation_epoch(),
        )
        .map_err(CapabilityIpcSmokeError::Revoke)?;
    let revoke_seq = send_message(
        &mut queue,
        root_core,
        peer_core,
        2,
        MessageKind::RevokeCap,
        MessagePayload::Object(object.object_id()),
    )?;
    let revoke_message = queue
        .pop(peer_core.get())
        .map_err(CapabilityIpcSmokeError::Fabric)?
        .into_value();
    let revoke_message_ok = revoke_message.header().kind() == MessageKind::RevokeCap
        && revoke_message.header().seq() == revoke_seq
        && revoke_message.payload() == MessagePayload::Object(object.object_id());
    let registry_epoch_bumped = revoked_epoch == object.revocation_epoch() + 1
        && registry
            .get(object.object_id())
            .map_err(CapabilityIpcSmokeError::Object)?
            .revocation_epoch()
            == revoked_epoch;
    let receiver_revoked = receiver
        .get(receiver_cap)
        .map_err(CapabilityIpcSmokeError::Cap)
        .and_then(|cap| {
            registry
                .resolve_capability(cap, CapPerms::READ)
                .map(|_| ())
                .map_err(CapabilityIpcSmokeError::Object)
        })
        == Err(CapabilityIpcSmokeError::Object(
            ObjectRegistryError::Revoked,
        ));
    let grant_audit_seen = audit.seen(CapAuditAction::Grant);

    if !(grant_message_ok
        && receiver_read_ok
        && receiver_write_denied
        && receiver_occupied == 1
        && sender_missing_grant_denied
        && revoke_message_ok
        && registry_epoch_bumped
        && receiver_revoked
        && grant_audit_seen
        && audit.len() == 3)
    {
        return Err(CapabilityIpcSmokeError::UnexpectedState);
    }

    Ok(CapabilityIpcSmokeStatus {
        grant_seq,
        revoke_seq,
        receiver_occupied,
        grant_message_ok,
        receiver_read_ok,
        receiver_write_denied,
        sender_missing_grant_denied,
        revoke_message_ok,
        registry_epoch_bumped,
        receiver_revoked,
        audit_events: audit.len(),
        grant_audit_seen,
    })
}

fn insert_granting_root(
    table: &mut CapabilityTable<4>,
    object_generation: u32,
    audit: &mut SmokeAudit,
) -> Result<CapId, CapTableError> {
    table.insert_root_with_audit(
        RootCapabilitySpec {
            object_id: OBJECT,
            kind: CapKind::Memory,
            owner: ROOT_OWNER,
            perms: CapPerms::READ
                .union(CapPerms::GRANT)
                .union(CapPerms::REVOKE),
            object_generation,
            revocation_epoch: 0,
        },
        audit,
    )
}

fn insert_read_only_root(
    table: &mut CapabilityTable<4>,
    object_generation: u32,
    audit: &mut SmokeAudit,
) -> Result<CapId, CapTableError> {
    table.insert_root_with_audit(
        RootCapabilitySpec {
            object_id: OBJECT,
            kind: CapKind::Memory,
            owner: ROOT_OWNER,
            perms: CapPerms::READ,
            object_generation,
            revocation_epoch: 0,
        },
        audit,
    )
}

fn send_message<const CAPACITY: usize>(
    queue: &mut PairwiseSpscQueue<CAPACITY>,
    src: ValidatedCoreId,
    dst: ValidatedCoreId,
    seq: u64,
    kind: MessageKind,
    payload: MessagePayload,
) -> Result<u64, CapabilityIpcSmokeError> {
    let request = MessageRequest {
        dst: dst.get(),
        kind,
        reply_to: None,
    };
    let message = FabricMessage::new(MessageHeader::stamp(request, src, seq, dst), payload);
    queue
        .push(src.get(), message)
        .map_err(CapabilityIpcSmokeError::Fabric)?;

    Ok(seq)
}

struct CapabilityIpcCoreSet;

impl LiveCoreSet for CapabilityIpcCoreSet {
    fn contains(&self, core: CoreId) -> bool {
        core == ROOT_CORE || core == CoreId::new(1)
    }
}

struct SmokeAudit {
    events: [Option<CapAuditEvent>; 3],
    len: usize,
}

impl SmokeAudit {
    const fn new() -> Self {
        Self {
            events: [None; 3],
            len: 0,
        }
    }

    const fn len(&self) -> usize {
        self.len
    }

    fn seen(&self, action: CapAuditAction) -> bool {
        self.events
            .iter()
            .flatten()
            .any(|event| event.action == action)
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

impl From<CapTableError> for CapabilityIpcSmokeError {
    fn from(error: CapTableError) -> Self {
        Self::Cap(error)
    }
}

impl From<CapAuditError> for CapabilityIpcSmokeError {
    fn from(_error: CapAuditError) -> Self {
        Self::AuditRejected
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn capability_ipc_smoke_grants_and_revokes_receiver_authority() {
        let status = super::run();

        assert_eq!(status.map(|value| value.grant_seq), Ok(1));
        assert_eq!(status.map(|value| value.revoke_seq), Ok(2));
        assert_eq!(status.map(|value| value.receiver_occupied), Ok(1));
        assert_eq!(status.map(|value| value.grant_message_ok), Ok(true));
        assert_eq!(status.map(|value| value.receiver_read_ok), Ok(true));
        assert_eq!(status.map(|value| value.receiver_write_denied), Ok(true));
        assert_eq!(
            status.map(|value| value.sender_missing_grant_denied),
            Ok(true)
        );
        assert_eq!(status.map(|value| value.revoke_message_ok), Ok(true));
        assert_eq!(status.map(|value| value.registry_epoch_bumped), Ok(true));
        assert_eq!(status.map(|value| value.receiver_revoked), Ok(true));
        assert_eq!(status.map(|value| value.audit_events), Ok(3));
        assert_eq!(status.map(|value| value.grant_audit_seen), Ok(true));
    }
}
