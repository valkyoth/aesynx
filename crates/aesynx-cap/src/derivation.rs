use aesynx_abi::{PrincipalId, VirtAddr};

use crate::CapPerms;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeriveRequest {
    perms: CapPerms,
    owner: PrincipalId,
    range: DeriveRange,
}

impl DeriveRequest {
    #[must_use]
    pub const fn whole_object(perms: CapPerms, owner: PrincipalId) -> Self {
        Self {
            perms,
            owner,
            range: DeriveRange::WholeObject,
        }
    }

    #[must_use]
    pub const fn bounded(perms: CapPerms, owner: PrincipalId, range: ObjectBoundedRange) -> Self {
        Self {
            perms,
            owner,
            range: DeriveRange::Bounded(range),
        }
    }

    #[must_use]
    pub const fn perms(self) -> CapPerms {
        self.perms
    }

    #[must_use]
    pub const fn owner(self) -> PrincipalId {
        self.owner
    }

    #[must_use]
    pub const fn base(self) -> Option<VirtAddr> {
        match self.range {
            DeriveRange::WholeObject => None,
            DeriveRange::Bounded(range) => Some(range.base),
        }
    }

    #[must_use]
    pub const fn range_len(self) -> Option<u64> {
        match self.range {
            DeriveRange::WholeObject => None,
            DeriveRange::Bounded(range) => Some(range.len),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DeriveRange {
    WholeObject,
    Bounded(ObjectBoundedRange),
}

/// A range already checked against the real backing extent of an object.
///
/// Bounded capability derivation requires this type instead of a raw
/// `(base, len)` pair so callers cannot accidentally derive arbitrary bounded
/// children from an unscoped whole-object parent without first naming and
/// checking the object's actual extent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectBoundedRange {
    base: VirtAddr,
    len: u64,
}

impl ObjectBoundedRange {
    pub const fn new_within_extent(
        base: VirtAddr,
        len: u64,
        extent_base: VirtAddr,
        extent_len: u64,
    ) -> Result<Self, DeriveError> {
        if len == 0 {
            return Err(DeriveError::RangeEscalates);
        }
        if !bounded_range_contains(extent_base.get(), extent_len, base.get(), len) {
            return Err(DeriveError::RangeEscalates);
        }

        Ok(Self { base, len })
    }

    #[must_use]
    pub const fn base(self) -> VirtAddr {
        self.base
    }

    #[must_use]
    pub const fn byte_len(self) -> u64 {
        self.len
    }

    #[cfg(test)]
    pub(crate) const fn new_for_test(base: VirtAddr, len: u64) -> Self {
        Self { base, len }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeriveError {
    MissingDerivePermission,
    MissingGrantPermission,
    ParentRevoked,
    ParentStaleGeneration,
    AuditRejected,
    PermissionsEscalate,
    RangeEscalates,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapValidationError {
    Revoked,
    StaleGeneration,
}

const fn bounded_range_contains(
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
    use super::{DeriveError, ObjectBoundedRange};
    use aesynx_abi::VirtAddr;

    #[test]
    fn object_bounded_range_rejects_zero_overflow_and_extent_escape() {
        assert_eq!(
            ObjectBoundedRange::new_within_extent(
                VirtAddr::new(0x1000),
                0,
                VirtAddr::new(0x1000),
                0x2000
            ),
            Err(DeriveError::RangeEscalates)
        );
        assert_eq!(
            ObjectBoundedRange::new_within_extent(
                VirtAddr::new(u64::MAX - 3),
                8,
                VirtAddr::new(0),
                u64::MAX
            ),
            Err(DeriveError::RangeEscalates)
        );
        assert_eq!(
            ObjectBoundedRange::new_within_extent(
                VirtAddr::new(0x3000),
                0x1000,
                VirtAddr::new(0x1000),
                0x2000
            ),
            Err(DeriveError::RangeEscalates)
        );
    }

    #[test]
    fn object_bounded_range_accepts_range_inside_extent() {
        let range = ObjectBoundedRange::new_within_extent(
            VirtAddr::new(0x2000),
            0x1000,
            VirtAddr::new(0x1000),
            0x3000,
        );

        assert_eq!(
            range.map(ObjectBoundedRange::base),
            Ok(VirtAddr::new(0x2000))
        );
        assert_eq!(range.map(ObjectBoundedRange::byte_len), Ok(0x1000));
    }
}
