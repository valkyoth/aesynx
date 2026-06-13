use aesynx_abi::{ObjectId, PrincipalId, VirtAddr};

use crate::{
    CapAuditError, CapAuditLog, CapKind, CapPerms, Capability, DeriveRequest, MemoryAccess,
    MemoryCapError, MemoryMapRequest,
};

use super::super::capability::TestCapabilitySpec;

struct NoopAudit;

impl CapAuditLog for NoopAudit {
    fn record(&mut self, _event: crate::CapAuditEvent) -> Result<(), CapAuditError> {
        Ok(())
    }
}

fn memory_cap(perms: CapPerms) -> Capability {
    Capability::new_for_test(TestCapabilitySpec {
        object_id: ObjectId::new(9),
        base: Some(VirtAddr::new(0x1000)),
        len: Some(0x3000),
        perms,
        owner: PrincipalId::new(1),
        generation: 1,
        revocation_epoch: 0,
        kind: CapKind::Memory,
    })
}

fn object_cap(perms: CapPerms) -> Capability {
    Capability::new_for_test(TestCapabilitySpec {
        object_id: ObjectId::new(9),
        base: Some(VirtAddr::new(0x1000)),
        len: Some(0x3000),
        perms,
        owner: PrincipalId::new(1),
        generation: 1,
        revocation_epoch: 0,
        kind: CapKind::Object,
    })
}

#[test]
fn memory_mapping_requires_memory_capability_kind() {
    let cap = object_cap(CapPerms::MAP.union(CapPerms::READ));
    let request = MemoryMapRequest::new(VirtAddr::new(0x1000), 0x1000, MemoryAccess::ReadOnly);

    assert_eq!(
        request.and_then(|request| cap.authorize_memory_map(request)),
        Err(MemoryCapError::WrongCapabilityKind)
    );
}

#[test]
fn memory_mapping_requires_read_permission() {
    let cap = memory_cap(CapPerms::MAP);
    let request = MemoryMapRequest::new(VirtAddr::new(0x1000), 0x1000, MemoryAccess::ReadOnly);

    assert_eq!(
        request.and_then(|request| cap.authorize_memory_map(request)),
        Err(MemoryCapError::MissingReadPermission)
    );
}

#[test]
fn memory_mapping_requires_write_permission_for_writable_maps() {
    let cap = memory_cap(CapPerms::MAP.union(CapPerms::READ));
    let request = MemoryMapRequest::new(VirtAddr::new(0x1000), 0x1000, MemoryAccess::ReadWrite);

    assert_eq!(
        request.and_then(|request| cap.authorize_memory_map(request)),
        Err(MemoryCapError::MissingWritePermission)
    );
}

#[test]
fn memory_mapping_requires_map_permission() {
    let cap = memory_cap(CapPerms::READ);
    let request = MemoryMapRequest::new(VirtAddr::new(0x1000), 0x1000, MemoryAccess::ReadOnly);

    assert_eq!(
        request.and_then(|request| cap.authorize_memory_map(request)),
        Err(MemoryCapError::MissingMapPermission)
    );
}

#[test]
fn memory_mapping_rejects_ranges_outside_capability() {
    let cap = memory_cap(CapPerms::MAP.union(CapPerms::READ));
    let request = MemoryMapRequest::new(VirtAddr::new(0x4000), 0x1000, MemoryAccess::ReadOnly);

    assert_eq!(
        request.and_then(|request| cap.authorize_memory_map(request)),
        Err(MemoryCapError::RangeEscapesCapability)
    );
}

#[test]
fn derived_memory_subrange_cannot_escape_parent_range() {
    let parent = memory_cap(
        CapPerms::MAP
            .union(CapPerms::READ)
            .union(CapPerms::WRITE)
            .union(CapPerms::DERIVE),
    );
    let mut audit = NoopAudit;
    let child = parent.derive_with_audit(
        DeriveRequest {
            perms: CapPerms::MAP.union(CapPerms::READ),
            owner: PrincipalId::new(1),
            base: Some(VirtAddr::new(0x2000)),
            len: Some(0x1000),
        },
        &mut audit,
    );

    assert!(child.is_ok());
    if let Ok(child) = child {
        let allowed = MemoryMapRequest::new(VirtAddr::new(0x2000), 0x1000, MemoryAccess::ReadOnly);
        let escaped = MemoryMapRequest::new(VirtAddr::new(0x3000), 0x1000, MemoryAccess::ReadOnly);

        assert_eq!(
            allowed.and_then(|request| child.authorize_memory_map(request)),
            Ok(())
        );
        assert_eq!(
            escaped.and_then(|request| child.authorize_memory_map(request)),
            Err(MemoryCapError::RangeEscapesCapability)
        );
        assert_eq!(
            MemoryMapRequest::new(VirtAddr::new(0x2000), 0x1000, MemoryAccess::ReadWrite)
                .and_then(|request| child.authorize_memory_map(request)),
            Err(MemoryCapError::MissingWritePermission)
        );
    }
}

#[test]
fn memory_map_request_rejects_empty_or_overflowing_ranges() {
    assert_eq!(
        MemoryMapRequest::new(VirtAddr::new(0x1000), 0, MemoryAccess::ReadOnly),
        Err(MemoryCapError::InvalidRange)
    );
    assert_eq!(
        MemoryMapRequest::new(VirtAddr::new(u64::MAX), 1, MemoryAccess::ReadOnly),
        Err(MemoryCapError::InvalidRange)
    );
}
