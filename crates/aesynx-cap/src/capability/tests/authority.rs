use super::*;

#[test]
fn derive_reduces_authority_for_same_owner() {
    let parent = parent_cap(
        CapPerms::READ
            .union(CapPerms::WRITE)
            .union(CapPerms::DERIVE),
    );
    let request = bounded_request(CapPerms::READ, parent.owner(), VirtAddr::new(120), 10);

    let expected = Capability::new_for_test(TestCapabilitySpec {
        object_id: parent.object_id(),
        base: Some(VirtAddr::new(120)),
        len: Some(10),
        perms: CapPerms::READ,
        owner: parent.owner(),
        generation: parent.generation(),
        revocation_epoch: parent.revocation_epoch(),
        kind: parent.kind(),
    });

    assert_eq!(audited_derive(parent, request), Ok(expected));
}

#[test]
fn derive_cross_owner_requires_grant_permission() {
    let parent = parent_cap(CapPerms::READ.union(CapPerms::DERIVE));
    let request = bounded_request(CapPerms::READ, PrincipalId::new(2), VirtAddr::new(120), 10);

    assert_eq!(
        audited_derive(parent, request),
        Err(DeriveError::MissingGrantPermission)
    );
}

#[test]
fn derive_cross_owner_succeeds_with_grant_permission() {
    let parent = parent_cap(
        CapPerms::READ
            .union(CapPerms::DERIVE)
            .union(CapPerms::GRANT),
    );
    let request = bounded_request(CapPerms::READ, PrincipalId::new(2), VirtAddr::new(120), 10);

    assert_eq!(
        audited_derive(parent, request).map(|cap| cap.owner()),
        Ok(PrincipalId::new(2))
    );
}

#[test]
fn derive_cross_owner_strips_grant_permission_from_child() {
    let parent = parent_cap(
        CapPerms::READ
            .union(CapPerms::DERIVE)
            .union(CapPerms::GRANT),
    );
    let request = bounded_request(
        CapPerms::READ.union(CapPerms::GRANT),
        PrincipalId::new(2),
        VirtAddr::new(120),
        10,
    );

    assert_eq!(
        audited_derive(parent, request).map(|cap| cap.perms()),
        Ok(CapPerms::READ)
    );
}

#[test]
fn derive_cross_owner_strips_revoke_and_admin_permissions_from_child() {
    let parent = parent_cap(
        CapPerms::READ
            .union(CapPerms::DERIVE)
            .union(CapPerms::GRANT)
            .union(CapPerms::REVOKE)
            .union(CapPerms::ADMIN),
    );
    let request = bounded_request(
        CapPerms::READ
            .union(CapPerms::GRANT)
            .union(CapPerms::REVOKE)
            .union(CapPerms::ADMIN),
        PrincipalId::new(2),
        VirtAddr::new(120),
        10,
    );

    assert_eq!(
        audited_derive(parent, request).map(|cap| cap.perms()),
        Ok(CapPerms::READ)
    );
}

#[test]
fn derive_same_owner_may_retain_grant_permission() {
    let parent = parent_cap(
        CapPerms::READ
            .union(CapPerms::DERIVE)
            .union(CapPerms::GRANT),
    );
    let request = bounded_request(
        CapPerms::READ.union(CapPerms::GRANT),
        parent.owner(),
        VirtAddr::new(120),
        10,
    );

    assert_eq!(
        audited_derive(parent, request).map(|cap| cap.perms()),
        Ok(CapPerms::READ.union(CapPerms::GRANT))
    );
}

#[test]
fn derive_with_audit_records_chain_of_custody() {
    let parent = parent_cap(
        CapPerms::READ
            .union(CapPerms::DERIVE)
            .union(CapPerms::GRANT),
    );
    let request = bounded_request(CapPerms::READ, PrincipalId::new(2), VirtAddr::new(120), 10);
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
fn derive_live_rejects_stale_parent_before_audit() {
    let parent = parent_cap(
        CapPerms::READ
            .union(CapPerms::DERIVE)
            .union(CapPerms::GRANT),
    );
    let request = bounded_request(CapPerms::READ, PrincipalId::new(2), VirtAddr::new(120), 10);
    let mut audit = TestAudit::default();

    assert_eq!(
        parent.derive_live_with_audit(request, 2, 9, &mut audit),
        Err(DeriveError::ParentStaleGeneration)
    );
    assert_eq!(audit.last_event, None);
}

#[test]
fn derive_live_rejects_revoked_parent_before_audit() {
    let parent = parent_cap(
        CapPerms::READ
            .union(CapPerms::DERIVE)
            .union(CapPerms::GRANT),
    );
    let request = bounded_request(CapPerms::READ, PrincipalId::new(2), VirtAddr::new(120), 10);
    let mut audit = TestAudit::default();

    assert_eq!(
        parent.derive_live_with_audit(request, 3, 8, &mut audit),
        Err(DeriveError::ParentRevoked)
    );
    assert_eq!(audit.last_event, None);
}

#[test]
fn grant_live_rejects_revoked_parent_before_audit() {
    let parent = parent_cap(CapPerms::READ.union(CapPerms::GRANT));
    let mut audit = TestAudit::default();

    assert_eq!(
        parent.grant_live_with_audit(PrincipalId::new(2), 3, 8, &mut audit),
        Err(DeriveError::ParentRevoked)
    );
    assert_eq!(audit.last_event, None);
}

#[test]
fn derive_requires_derive_permission() {
    let parent = parent_cap(CapPerms::READ);
    let request = bounded_request(CapPerms::READ, PrincipalId::new(2), VirtAddr::new(120), 10);

    assert_eq!(
        audited_derive(parent, request),
        Err(DeriveError::MissingDerivePermission)
    );
}

#[test]
fn derive_rejects_permission_escalation() {
    let parent = parent_cap(CapPerms::READ.union(CapPerms::DERIVE));
    let request = bounded_request(
        CapPerms::READ.union(CapPerms::WRITE),
        parent.owner(),
        VirtAddr::new(120),
        10,
    );

    assert_eq!(
        audited_derive(parent, request),
        Err(DeriveError::PermissionsEscalate)
    );
}

#[test]
fn derive_rejects_range_expansion() {
    let parent = parent_cap(CapPerms::READ.union(CapPerms::DERIVE));
    let request = bounded_request(CapPerms::READ, parent.owner(), VirtAddr::new(120), 40);

    assert_eq!(
        audited_derive(parent, request),
        Err(DeriveError::RangeEscalates)
    );
}

#[test]
fn object_bounded_range_rejects_zero_length_before_derivation() {
    assert_eq!(
        ObjectBoundedRange::new_within_extent(VirtAddr::new(120), 0, VirtAddr::new(0), u64::MAX),
        Err(DeriveError::RangeEscalates)
    );
}

#[test]
fn object_bounded_range_rejects_overflow_before_derivation() {
    assert_eq!(
        ObjectBoundedRange::new_within_extent(
            VirtAddr::new(u64::MAX - 3),
            8,
            VirtAddr::new(0),
            u64::MAX
        ),
        Err(DeriveError::RangeEscalates)
    );
}

#[test]
fn unbounded_parent_derives_only_prevalidated_object_bounded_child_range() {
    let parent = unbounded_parent_cap(CapPerms::READ.union(CapPerms::DERIVE));
    let request = bounded_request(CapPerms::READ, parent.owner(), VirtAddr::new(120), 10);

    assert_eq!(
        audited_derive(parent, request).map(|cap| (cap.base(), cap.range_len())),
        Ok((Some(VirtAddr::new(120)), Some(10)))
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
fn grant_cross_owner_strips_revoke_and_admin_permissions_from_child() {
    let parent = parent_cap(
        CapPerms::READ
            .union(CapPerms::GRANT)
            .union(CapPerms::REVOKE)
            .union(CapPerms::ADMIN),
    );

    assert_eq!(
        audited_grant(parent, PrincipalId::new(2)).map(|cap| cap.perms()),
        Ok(CapPerms::READ)
    );
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
