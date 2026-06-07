#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapPerms(u32);

impl CapPerms {
    pub const READ: Self = Self(1 << 0);
    pub const WRITE: Self = Self(1 << 1);
    pub const EXECUTE: Self = Self(1 << 2);
    pub const GRANT: Self = Self(1 << 3);
    pub const DERIVE: Self = Self(1 << 4);
    pub const MAP: Self = Self(1 << 5);
    pub const SEND: Self = Self(1 << 6);
    pub const REVOKE: Self = Self(1 << 7);
    pub const INTROSPECT: Self = Self(1 << 8);
    pub const ADMIN: Self = Self(1 << 9);

    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    #[must_use]
    pub const fn contains(self, required: Self) -> bool {
        self.0 & required.0 == required.0
    }

    #[must_use]
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[must_use]
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    #[must_use]
    pub const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::CapPerms;

    #[test]
    fn union_contains_both_permissions() {
        let perms = CapPerms::READ.union(CapPerms::WRITE);

        assert!(perms.contains(CapPerms::READ));
        assert!(perms.contains(CapPerms::WRITE));
        assert!(!perms.contains(CapPerms::EXECUTE));
    }

    #[test]
    fn empty_contains_no_permissions() {
        let perms = CapPerms::empty();

        assert_eq!(perms.bits(), 0);
        assert!(!perms.contains(CapPerms::READ));
    }

    #[test]
    fn intersection_keeps_only_shared_permissions() {
        let left = CapPerms::READ.union(CapPerms::WRITE);
        let right = CapPerms::WRITE.union(CapPerms::EXECUTE);

        assert_eq!(left.intersection(right), CapPerms::WRITE);
        assert!(left.intersects(right));
    }

    #[test]
    fn without_removes_permissions() {
        let perms = CapPerms::READ
            .union(CapPerms::WRITE)
            .without(CapPerms::WRITE);

        assert_eq!(perms, CapPerms::READ);
    }
}
