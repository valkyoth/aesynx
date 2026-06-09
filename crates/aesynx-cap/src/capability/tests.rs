use aesynx_abi::{ObjectId, PrincipalId, VirtAddr};

use crate::{
    CapAuditAction, CapAuditEvent, CapAuditLog, CapKind, CapPerms, CapValidationError, Capability,
    DeriveError, DeriveRequest,
};

use super::TestCapabilitySpec;

fn parent_cap(perms: CapPerms) -> Capability {
    Capability::new_for_test(TestCapabilitySpec {
        object_id: ObjectId::new(7),
        base: Some(VirtAddr::new(100)),
        len: Some(50),
        perms,
        owner: PrincipalId::new(1),
        generation: 3,
        revocation_epoch: 9,
        kind: CapKind::Memory,
    })
}

#[derive(Default)]
struct TestAudit {
    last_event: Option<CapAuditEvent>,
}

impl CapAuditLog for TestAudit {
    fn record(&mut self, event: CapAuditEvent) -> Result<(), DeriveError> {
        self.last_event = Some(event);
        Ok(())
    }
}

fn audited_derive(parent: Capability, request: DeriveRequest) -> Result<Capability, DeriveError> {
    let mut audit = TestAudit::default();

    parent.derive_with_audit(request, &mut audit)
}

fn audited_grant(parent: Capability, target_owner: PrincipalId) -> Result<Capability, DeriveError> {
    let mut audit = TestAudit::default();

    parent.grant_with_audit(target_owner, &mut audit)
}

#[test]
fn derive_reduces_authority_and_changes_owner() {
    let parent = parent_cap(
        CapPerms::READ
            .union(CapPerms::WRITE)
            .union(CapPerms::DERIVE),
    );
    let request = DeriveRequest {
        perms: CapPerms::READ,
        owner: PrincipalId::new(2),
        base: Some(VirtAddr::new(120)),
        len: Some(10),
    };

    let expected = Capability::new_for_test(TestCapabilitySpec {
        object_id: parent.object_id(),
        base: Some(VirtAddr::new(120)),
        len: Some(10),
        perms: CapPerms::READ,
        owner: PrincipalId::new(2),
        generation: parent.generation(),
        revocation_epoch: parent.revocation_epoch(),
        kind: parent.kind(),
    });

    assert_eq!(audited_derive(parent, request), Ok(expected));
}

#[test]
fn derive_with_audit_records_chain_of_custody() {
    let parent = parent_cap(CapPerms::READ.union(CapPerms::DERIVE));
    let request = DeriveRequest {
        perms: CapPerms::READ,
        owner: PrincipalId::new(2),
        base: Some(VirtAddr::new(120)),
        len: Some(10),
    };
    let mut audit = TestAudit::default();

    assert_eq!(
        parent
            .derive_with_audit(request, &mut audit)
            .map(|cap| cap.owner()),
        Ok(PrincipalId::new(2))
    );
    assert_eq!(
        audit.last_event.map(|event| event.action),
        Some(CapAuditAction::Derive)
    );
}

#[test]
fn derive_requires_derive_permission() {
    let parent = parent_cap(CapPerms::READ);
    let request = DeriveRequest {
        perms: CapPerms::READ,
        owner: PrincipalId::new(2),
        base: Some(VirtAddr::new(120)),
        len: Some(10),
    };

    assert_eq!(
        audited_derive(parent, request),
        Err(DeriveError::MissingDerivePermission)
    );
}

#[test]
fn derive_rejects_permission_escalation() {
    let parent = parent_cap(CapPerms::READ.union(CapPerms::DERIVE));
    let request = DeriveRequest {
        perms: CapPerms::READ.union(CapPerms::WRITE),
        owner: PrincipalId::new(2),
        base: Some(VirtAddr::new(120)),
        len: Some(10),
    };

    assert_eq!(
        audited_derive(parent, request),
        Err(DeriveError::PermissionsEscalate)
    );
}

#[test]
fn derive_rejects_range_expansion() {
    let parent = parent_cap(CapPerms::READ.union(CapPerms::DERIVE));
    let request = DeriveRequest {
        perms: CapPerms::READ,
        owner: PrincipalId::new(2),
        base: Some(VirtAddr::new(120)),
        len: Some(40),
    };

    assert_eq!(
        audited_derive(parent, request),
        Err(DeriveError::RangeEscalates)
    );
}

#[test]
fn derive_rejects_zero_length_range() {
    let parent = parent_cap(CapPerms::READ.union(CapPerms::DERIVE));
    let request = DeriveRequest {
        perms: CapPerms::READ,
        owner: PrincipalId::new(2),
        base: Some(VirtAddr::new(120)),
        len: Some(0),
    };

    assert_eq!(
        audited_derive(parent, request),
        Err(DeriveError::RangeEscalates)
    );
}

#[test]
fn grant_requires_grant_permission() {
    let parent = parent_cap(CapPerms::READ);

    assert_eq!(
        audited_grant(parent, PrincipalId::new(2)),
        Err(DeriveError::MissingGrantPermission)
    );
}

#[test]
fn grant_transfers_authority_without_regrant_by_default() {
    let parent = parent_cap(CapPerms::READ.union(CapPerms::GRANT));
    let expected = Capability::new_for_test(TestCapabilitySpec {
        object_id: parent.object_id(),
        base: parent.base(),
        len: parent.range_len(),
        perms: CapPerms::READ,
        owner: PrincipalId::new(2),
        generation: parent.generation(),
        revocation_epoch: parent.revocation_epoch(),
        kind: parent.kind(),
    });

    assert_eq!(audited_grant(parent, PrincipalId::new(2)), Ok(expected));
}

#[test]
fn grant_with_audit_records_chain_of_custody() {
    let parent = parent_cap(CapPerms::READ.union(CapPerms::GRANT));
    let mut audit = TestAudit::default();

    assert_eq!(
        parent
            .grant_with_audit(PrincipalId::new(2), &mut audit)
            .map(|cap| cap.owner()),
        Ok(PrincipalId::new(2))
    );
    assert_eq!(
        audit.last_event.map(|event| event.action),
        Some(CapAuditAction::Grant)
    );
}

#[test]
fn live_validation_rejects_stale_generation_and_epoch() {
    let parent = parent_cap(CapPerms::READ);

    assert_eq!(parent.validate_live(3, 9), Ok(()));
    assert_eq!(
        parent.validate_live(2, 9),
        Err(CapValidationError::StaleGeneration)
    );
    assert_eq!(parent.validate_live(3, 8), Err(CapValidationError::Revoked));
}
