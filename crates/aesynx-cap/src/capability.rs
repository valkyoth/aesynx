use aesynx_abi::{ObjectId, PrincipalId, VirtAddr};

use crate::{CapKind, CapPerms, DeriveError, DeriveRequest};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Capability {
    pub object_id: ObjectId,
    pub base: Option<VirtAddr>,
    pub len: Option<u64>,
    pub perms: CapPerms,
    pub owner: PrincipalId,
    pub generation: u32,
    pub revocation_epoch: u64,
    pub kind: CapKind,
}

impl Capability {
    #[must_use]
    pub const fn allows(self, required: CapPerms) -> bool {
        self.perms.contains(required)
    }

    #[must_use]
    pub const fn matches_revocation_epoch(self, current_epoch: u64) -> bool {
        self.revocation_epoch == current_epoch
    }

    pub fn derive(self, request: DeriveRequest) -> Result<Self, DeriveError> {
        if !self.perms.contains(CapPerms::DERIVE) {
            return Err(DeriveError::MissingDerivePermission);
        }

        if !self.perms.contains(request.perms) {
            return Err(DeriveError::PermissionsEscalate);
        }

        if !range_is_subset(self.base, self.len, request.base, request.len) {
            return Err(DeriveError::RangeEscalates);
        }

        Ok(Self {
            object_id: self.object_id,
            base: request.base,
            len: request.len,
            perms: request.perms,
            owner: request.owner,
            generation: self.generation,
            revocation_epoch: self.revocation_epoch,
            kind: self.kind,
        })
    }
}

fn range_is_subset(
    parent_base: Option<VirtAddr>,
    parent_len: Option<u64>,
    child_base: Option<VirtAddr>,
    child_len: Option<u64>,
) -> bool {
    match (parent_base, parent_len, child_base, child_len) {
        (None, None, None, None) => true,
        (None, None, Some(_), Some(_)) => true,
        (Some(_), Some(_), None, None) => false,
        (Some(parent_base), Some(parent_len), Some(child_base), Some(child_len)) => {
            bounded_range_contains(parent_base.get(), parent_len, child_base.get(), child_len)
        }
        _ => false,
    }
}

fn bounded_range_contains(
    parent_base: u64,
    parent_len: u64,
    child_base: u64,
    child_len: u64,
) -> bool {
    let Some(parent_end) = parent_base.checked_add(parent_len) else {
        return false;
    };
    let Some(child_end) = child_base.checked_add(child_len) else {
        return false;
    };

    child_base >= parent_base && child_end <= parent_end
}

#[cfg(test)]
mod tests {
    use aesynx_abi::{ObjectId, PrincipalId, VirtAddr};

    use crate::{CapKind, CapPerms, Capability, DeriveError, DeriveRequest};

    fn parent_cap(perms: CapPerms) -> Capability {
        Capability {
            object_id: ObjectId::new(7),
            base: Some(VirtAddr::new(100)),
            len: Some(50),
            perms,
            owner: PrincipalId::new(1),
            generation: 3,
            revocation_epoch: 9,
            kind: CapKind::Memory,
        }
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

        let expected = Capability {
            object_id: parent.object_id,
            base: Some(VirtAddr::new(120)),
            len: Some(10),
            perms: CapPerms::READ,
            owner: PrincipalId::new(2),
            generation: parent.generation,
            revocation_epoch: parent.revocation_epoch,
            kind: parent.kind,
        };

        assert_eq!(parent.derive(request), Ok(expected));
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
            parent.derive(request),
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
            parent.derive(request),
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

        assert_eq!(parent.derive(request), Err(DeriveError::RangeEscalates));
    }
}
