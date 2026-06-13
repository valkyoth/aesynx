use aesynx_abi::{CapId, ObjectId, PrincipalId, VirtAddr};

use crate::{
    CapAuditAction, CapAuditEvent, CapAuditLog, CapKind, CapPerms, CapTableError, CapabilityTable,
    DeriveError, DeriveRequest,
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
}

impl CapAuditLog for TestAudit {
    fn record(&mut self, event: CapAuditEvent) -> Result<(), DeriveError> {
        if self.len >= self.events.len() {
            return Err(DeriveError::AuditRejected);
        }

        self.events[self.len] = Some(event);
        self.len += 1;
        Ok(())
    }
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
fn table_derives_child_into_new_slot_with_audit() {
    let mut table = CapabilityTable::<4>::new();
    let root = insert_root(&mut table);
    let mut audit = TestAudit::default();

    if let Ok(root) = root {
        let child = table.derive_with_audit(
            root,
            DeriveRequest {
                perms: CapPerms::READ,
                owner: PrincipalId::new(2),
                base: Some(VirtAddr::new(0x1000)),
                len: Some(0x1000),
            },
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
fn table_rejects_stale_ids_after_revoke() {
    let mut table = CapabilityTable::<4>::new();
    let root = insert_root(&mut table);
    let mut audit = TestAudit::default();

    if let Ok(root) = root {
        let child = table.derive_with_audit(
            root,
            DeriveRequest {
                perms: CapPerms::READ,
                owner: PrincipalId::new(2),
                base: Some(VirtAddr::new(0x1000)),
                len: Some(0x1000),
            },
            &mut audit,
        );

        if let Ok(child) = child {
            assert_eq!(table.revoke(root, child), Ok(2));
            assert_eq!(
                table.check(root, CapPerms::READ).map(|_| ()),
                Err(CapTableError::StaleId)
            );
            assert_eq!(
                table.check(child, CapPerms::READ).map(|_| ()),
                Err(CapTableError::StaleId)
            );
            assert_eq!(table.occupied_slots(), 0);
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

    if let Ok(root) = root {
        assert_eq!(
            table.derive_with_audit(
                root,
                DeriveRequest {
                    perms: CapPerms::READ,
                    owner: PrincipalId::new(1),
                    base: Some(VirtAddr::new(0x1000)),
                    len: Some(0x1000),
                },
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

    if let Ok(root) = root {
        assert_eq!(
            table.grant_with_audit(root, PrincipalId::new(2), &mut audit),
            Err(CapTableError::TableFull)
        );
        assert_eq!(audit.len(), 0);
    }
}
