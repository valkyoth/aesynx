use aesynx_abi::{CapId, ObjectId, PrincipalId, VirtAddr};
use alloc::format;

use crate::{
    CapAuditAction, CapAuditError, CapAuditEvent, CapAuditLog, CapIdError, CapKind, CapPerms,
    CapTableError, CapabilityTable, DeriveRequest, LiveAuthorityError, LiveAuthorityState,
    LiveAuthorityView, ObjectBoundedRange, RootCapabilitySpec,
};

#[derive(Default)]
struct TestAudit {
    events: [Option<CapAuditEvent>; 4],
    len: usize,
}

impl TestAudit {
    fn len(&self) -> usize {
        self.len
    }

    fn first_action(&self) -> Option<CapAuditAction> {
        self.events[0].map(|event| event.action)
    }

    fn last_action(&self) -> Option<CapAuditAction> {
        if self.len == 0 {
            None
        } else {
            self.events[self.len - 1].map(|event| event.action)
        }
    }

    fn last_event(&self) -> Option<CapAuditEvent> {
        if self.len == 0 {
            None
        } else {
            self.events[self.len - 1]
        }
    }
}

impl CapAuditLog for TestAudit {
    fn record(&mut self, event: CapAuditEvent) -> Result<(), CapAuditError> {
        if self.len >= self.events.len() {
            return Err(CapAuditError::Rejected);
        }

        self.events[self.len] = Some(event);
        self.len += 1;
        Ok(())
    }
}

struct TestLiveAuthority {
    object_id: ObjectId,
    generation: u32,
    revocation_epoch: u64,
}

impl TestLiveAuthority {
    const fn matching_root() -> Self {
        Self {
            object_id: ObjectId::new(42),
            generation: 1,
            revocation_epoch: 0,
        }
    }

    const fn for_object(object_id: ObjectId) -> Self {
        Self {
            object_id,
            generation: 1,
            revocation_epoch: 0,
        }
    }
}

impl LiveAuthorityView for TestLiveAuthority {
    fn live_authority(
        &self,
        object_id: ObjectId,
    ) -> Result<LiveAuthorityState, LiveAuthorityError> {
        if object_id != self.object_id {
            return Err(LiveAuthorityError::ObjectNotFound);
        }

        Ok(LiveAuthorityState::new(
            self.generation,
            self.revocation_epoch,
        ))
    }
}

#[test]
fn audit_event_debug_redacts_authority_identifiers() {
    let event = CapAuditEvent {
        action: CapAuditAction::Grant,
        object_id: ObjectId::new(0xfeed_cafe),
        source_owner: PrincipalId::new(0xdead_beef),
        target_owner: PrincipalId::new(0xabcd_1234),
        perms: CapPerms::READ.union(CapPerms::GRANT),
        generation: 0x1357_2468,
        revocation_epoch: 0x9988_7766,
        affected_slots: 1,
    };

    let debug = format!("{:?}", event);

    assert!(debug.contains("redacted"));
    assert!(debug.contains("Grant"));
    assert!(!debug.contains("ObjectId"));
    assert!(!debug.contains("PrincipalId"));
    assert!(!debug.contains("feed"));
    assert!(!debug.contains("dead"));
    assert!(!debug.contains("13572468"));
    assert!(!debug.contains("99887766"));
}

fn insert_root(table: &mut CapabilityTable<4>) -> Result<CapId, CapTableError> {
    table.insert_root(
        ObjectId::new(42),
        CapKind::Memory,
        PrincipalId::new(1),
        CapPerms::READ
            .union(CapPerms::WRITE)
            .union(CapPerms::DERIVE)
            .union(CapPerms::GRANT)
            .union(CapPerms::REVOKE),
        1,
        0,
    )
}

fn bounded_request(perms: CapPerms, owner: PrincipalId, base: VirtAddr, len: u64) -> DeriveRequest {
    DeriveRequest::bounded(perms, owner, ObjectBoundedRange::new_for_test(base, len))
}

#[test]
fn table_mints_root_into_new_slot_with_audit() {
    let mut table = CapabilityTable::<4>::new();
    let mut audit = TestAudit::default();
    let root = table.insert_root_with_audit(
        RootCapabilitySpec {
            object_id: ObjectId::new(42),
            kind: CapKind::Memory,
            owner: PrincipalId::new(1),
            perms: CapPerms::READ,
            object_generation: 1,
            revocation_epoch: 0,
        },
        &mut audit,
    );

    assert!(root.is_ok());
    if let Ok(root) = root {
        assert!(table.check(root, CapPerms::READ).is_ok());
        assert_eq!(audit.len(), 1);
        assert_eq!(audit.first_action(), Some(CapAuditAction::Mint));
        assert_eq!(
            audit.last_event().map(|event| event.affected_slots),
            Some(1)
        );
    }
}

#[test]
fn audited_root_mint_rejects_audit_failure_without_mutation() {
    let mut table = CapabilityTable::<4>::new();
    let mut audit = TestAudit {
        events: [None; 4],
        len: 4,
    };

    assert_eq!(
        table.insert_root_with_audit(
            RootCapabilitySpec {
                object_id: ObjectId::new(42),
                kind: CapKind::Memory,
                owner: PrincipalId::new(1),
                perms: CapPerms::READ,
                object_generation: 1,
                revocation_epoch: 0,
            },
            &mut audit,
        ),
        Err(CapTableError::AuditRejected)
    );
    assert_eq!(table.occupied_slots(), 0);
}

#[test]
fn table_grants_child_into_new_slot_with_audit() {
    let mut table = CapabilityTable::<4>::new();
    let root = insert_root(&mut table);
    let mut audit = TestAudit::default();
    let live = TestLiveAuthority::matching_root();

    if let Ok(root) = root {
        let child = table.grant_with_audit(root, PrincipalId::new(2), &live, &mut audit);

        assert!(child.is_ok());
        if let Ok(child) = child {
            assert!(table.check(child, CapPerms::READ).is_ok());
            assert_eq!(
                table.check(child, CapPerms::GRANT).map(|_| ()),
                Err(CapTableError::MissingPermission)
            );
            assert_eq!(audit.len(), 1);
            assert_eq!(audit.first_action(), Some(CapAuditAction::Grant));
            assert_eq!(table.occupied_slots(), 2);
        }
    }
}

#[test]
fn table_inserts_root_and_checks_permissions() {
    let mut table = CapabilityTable::<4>::new();
    let root = insert_root(&mut table);

    assert!(root.is_ok());
    if let Ok(root) = root {
        assert!(table.check(root, CapPerms::READ).is_ok());
        assert_eq!(
            table.check(root, CapPerms::EXECUTE).map(|_| ()),
            Err(CapTableError::MissingPermission)
        );
        assert_eq!(table.occupied_slots(), 1);
    }
}

#[test]
fn table_rejects_zero_object_generation_without_mutation() {
    let mut table = CapabilityTable::<4>::new();

    assert_eq!(
        table.insert_root(
            ObjectId::new(42),
            CapKind::Memory,
            PrincipalId::new(1),
            CapPerms::READ,
            0,
            0,
        ),
        Err(CapTableError::Id(CapIdError::ZeroGeneration))
    );
    assert_eq!(table.occupied_slots(), 0);
}

#[test]
fn table_derives_child_into_new_slot_with_audit() {
    let mut table = CapabilityTable::<4>::new();
    let root = insert_root(&mut table);
    let mut audit = TestAudit::default();
    let live = TestLiveAuthority::matching_root();

    if let Ok(root) = root {
        let child = table.derive_with_audit(
            root,
            bounded_request(
                CapPerms::READ,
                PrincipalId::new(2),
                VirtAddr::new(0x1000),
                0x1000,
            ),
            &live,
            &mut audit,
        );

        assert!(child.is_ok());
        if let Ok(child) = child {
            assert!(table.check(child, CapPerms::READ).is_ok());
            assert_eq!(
                table.check(child, CapPerms::WRITE).map(|_| ()),
                Err(CapTableError::MissingPermission)
            );
            assert_eq!(audit.len(), 1);
            assert_eq!(audit.first_action(), Some(CapAuditAction::Derive));
            assert_eq!(table.occupied_slots(), 2);
        }
    }
}

#[test]
fn table_rejects_stale_live_state_before_audit_or_mutation() {
    let mut table = CapabilityTable::<4>::new();
    let root = insert_root(&mut table);
    let mut audit = TestAudit::default();
    let stale_live = TestLiveAuthority {
        object_id: ObjectId::new(42),
        generation: 2,
        revocation_epoch: 0,
    };

    if let Ok(root) = root {
        assert_eq!(
            table.derive_with_audit(
                root,
                bounded_request(
                    CapPerms::READ,
                    PrincipalId::new(2),
                    VirtAddr::new(0x1000),
                    0x1000,
                ),
                &stale_live,
                &mut audit,
            ),
            Err(CapTableError::StaleId)
        );
        assert_eq!(
            table.grant_with_audit(root, PrincipalId::new(2), &stale_live, &mut audit),
            Err(CapTableError::StaleId)
        );
        assert_eq!(
            table.revoke_with_audit(root, root, &stale_live, &mut audit),
            Err(CapTableError::Revoke(
                crate::RevocationError::StaleAuthority
            ))
        );
        assert_eq!(audit.len(), 0);
        assert_eq!(table.occupied_slots(), 1);
    }
}

#[test]
fn table_rejects_stale_ids_after_revoke() {
    let mut table = CapabilityTable::<4>::new();
    let root = insert_root(&mut table);
    let mut audit = TestAudit::default();
    let live = TestLiveAuthority::matching_root();

    if let Ok(root) = root {
        let child = table.derive_with_audit(
            root,
            bounded_request(
                CapPerms::READ,
                PrincipalId::new(2),
                VirtAddr::new(0x1000),
                0x1000,
            ),
            &live,
            &mut audit,
        );

        if let Ok(child) = child {
            assert_eq!(
                table.revoke_with_audit(root, child, &live, &mut audit),
                Ok(2)
            );
            assert_eq!(
                table.check(root, CapPerms::READ).map(|_| ()),
                Err(CapTableError::StaleId)
            );
            assert_eq!(
                table.check(child, CapPerms::READ).map(|_| ()),
                Err(CapTableError::StaleId)
            );
            assert_eq!(table.occupied_slots(), 0);
            assert_eq!(audit.last_action(), Some(CapAuditAction::Revoke));
            assert_eq!(
                audit.last_event().map(|event| event.affected_slots),
                Some(2)
            );
        }
    }
}

#[test]
fn table_fails_closed_when_full() {
    let mut table = CapabilityTable::<1>::new();

    assert!(
        table
            .insert_root(
                ObjectId::new(1),
                CapKind::Object,
                PrincipalId::new(1),
                CapPerms::READ,
                1,
                0,
            )
            .is_ok()
    );
    assert_eq!(
        table.insert_root(
            ObjectId::new(2),
            CapKind::Object,
            PrincipalId::new(1),
            CapPerms::READ,
            1,
            0,
        ),
        Err(CapTableError::TableFull)
    );
}

#[test]
fn full_table_derive_does_not_emit_phantom_audit() {
    let mut table = CapabilityTable::<1>::new();
    let root = table.insert_root(
        ObjectId::new(1),
        CapKind::Memory,
        PrincipalId::new(1),
        CapPerms::READ.union(CapPerms::DERIVE),
        1,
        0,
    );
    let mut audit = TestAudit::default();
    let live = TestLiveAuthority::for_object(ObjectId::new(1));

    if let Ok(root) = root {
        assert_eq!(
            table.derive_with_audit(
                root,
                bounded_request(
                    CapPerms::READ,
                    PrincipalId::new(1),
                    VirtAddr::new(0x1000),
                    0x1000,
                ),
                &live,
                &mut audit,
            ),
            Err(CapTableError::TableFull)
        );
        assert_eq!(audit.len(), 0);
    }
}

#[test]
fn full_table_grant_does_not_emit_phantom_audit() {
    let mut table = CapabilityTable::<1>::new();
    let root = table.insert_root(
        ObjectId::new(1),
        CapKind::Object,
        PrincipalId::new(1),
        CapPerms::READ.union(CapPerms::GRANT),
        1,
        0,
    );
    let mut audit = TestAudit::default();
    let live = TestLiveAuthority::for_object(ObjectId::new(1));

    if let Ok(root) = root {
        assert_eq!(
            table.grant_with_audit(root, PrincipalId::new(2), &live, &mut audit),
            Err(CapTableError::TableFull)
        );
        assert_eq!(audit.len(), 0);
    }
}
