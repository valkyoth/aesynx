use aesynx_abi::{ObjectId, PrincipalId};

use super::{TestAudit, TestLiveAuthority, insert_root};
use crate::{CapAuditAction, CapKind, CapPerms, CapTableError, CapabilityTable};

#[test]
fn table_grants_child_into_receiver_table_with_audit() {
    let mut sender = CapabilityTable::<4>::new();
    let mut receiver = CapabilityTable::<2>::new();
    let root = insert_root(&mut sender);
    let mut audit = TestAudit::default();
    let live = TestLiveAuthority::matching_root();

    if let Ok(root) = root {
        let receiver_cap = sender.grant_to_table_with_audit(
            root,
            &mut receiver,
            PrincipalId::new(2),
            &live,
            &mut audit,
        );

        assert!(receiver_cap.is_ok());
        if let Ok(receiver_cap) = receiver_cap {
            assert_eq!(sender.occupied_slots(), 1);
            assert_eq!(receiver.occupied_slots(), 1);
            assert!(receiver.check(receiver_cap, CapPerms::READ).is_ok());
            assert_eq!(
                receiver.check(receiver_cap, CapPerms::GRANT).map(|_| ()),
                Err(CapTableError::MissingPermission)
            );
            assert_eq!(audit.len(), 1);
            assert_eq!(audit.first_action(), Some(CapAuditAction::Grant));
        }
    }
}

#[test]
fn cross_table_grant_rejects_missing_grant_permission_without_receiver_mutation() {
    let mut sender = CapabilityTable::<4>::new();
    let mut receiver = CapabilityTable::<2>::new();
    let source = sender.insert_root(
        ObjectId::new(42),
        CapKind::Memory,
        PrincipalId::new(1),
        CapPerms::READ,
        1,
        0,
    );
    let mut audit = TestAudit::default();
    let live = TestLiveAuthority::matching_root();

    if let Ok(source) = source {
        assert_eq!(
            sender.grant_to_table_with_audit(
                source,
                &mut receiver,
                PrincipalId::new(2),
                &live,
                &mut audit,
            ),
            Err(CapTableError::MissingPermission)
        );
        assert_eq!(receiver.occupied_slots(), 0);
        assert_eq!(audit.len(), 0);
    }
}
