use aesynx_abi::{CapId, ObjectId, PrincipalId, VirtAddr};
use core::fmt::{self, Write};

use crate::{
    CapAuditAction, CapAuditError, CapAuditEvent, CapAuditLog, CapIdError, CapKind, CapPerms,
    CapSlotIndex, CapValidationError, Capability, DeriveError, DeriveRequest,
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

fn unbounded_parent_cap(perms: CapPerms) -> Capability {
    Capability::new_for_test(TestCapabilitySpec {
        object_id: ObjectId::new(7),
        base: None,
        len: None,
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
    fn record(&mut self, event: CapAuditEvent) -> Result<(), CapAuditError> {
        self.last_event = Some(event);
        Ok(())
    }
}

struct DebugBuffer {
    bytes: [u8; 256],
    len: usize,
}

impl DebugBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; 256],
            len: 0,
        }
    }

    fn contains(&self, needle: &str) -> bool {
        let needle = needle.as_bytes();
        if needle.is_empty() || needle.len() > self.len {
            return false;
        }

        let mut start = 0usize;
        while start + needle.len() <= self.len {
            if &self.bytes[start..start + needle.len()] == needle {
                return true;
            }
            start += 1;
        }

        false
    }
}

impl Write for DebugBuffer {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let bytes = value.as_bytes();
        let Some(end) = self.len.checked_add(bytes.len()) else {
            return Err(fmt::Error);
        };
        if end > self.bytes.len() {
            return Err(fmt::Error);
        }

        self.bytes[self.len..end].copy_from_slice(bytes);
        self.len = end;
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
fn live_validation_rejects_stale_generation_and_epoch() {
    let parent = parent_cap(CapPerms::READ);

    assert_eq!(parent.validate_live(3, 9), Ok(()));
    assert_eq!(
        parent.validate_live(2, 9),
        Err(CapValidationError::StaleGeneration)
    );
    assert_eq!(parent.validate_live(3, 8), Err(CapValidationError::Revoked));
}

#[test]
fn capability_derives_cap_id_from_slot_and_generation() {
    let parent = parent_cap(CapPerms::READ);

    assert_eq!(
        parent.id_for_slot(CapSlotIndex::new(42)),
        Ok(CapId::new(0x0000_0003_0000_002a))
    );
}

#[test]
fn capability_id_rejects_zero_generation() {
    let cap = Capability::new_for_test(TestCapabilitySpec {
        object_id: ObjectId::new(7),
        base: None,
        len: None,
        perms: CapPerms::READ,
        owner: PrincipalId::new(1),
        generation: 0,
        revocation_epoch: 9,
        kind: CapKind::Object,
    });

    assert_eq!(
        cap.id_for_slot(CapSlotIndex::new(42)),
        Err(CapIdError::ZeroGeneration)
    );
}

#[test]
fn capability_debug_redacts_authority_identifiers() {
    let mut rendered = DebugBuffer::new();

    assert_eq!(
        write!(&mut rendered, "{:?}", parent_cap(CapPerms::READ)),
        Ok(())
    );
    assert!(rendered.contains("object_id: \"<redacted>\""));
    assert!(rendered.contains("owner: \"<redacted>\""));
    assert!(!rendered.contains("ObjectId"));
    assert!(!rendered.contains("PrincipalId"));
    assert!(!rendered.contains("VirtAddr"));
}

mod authority;
