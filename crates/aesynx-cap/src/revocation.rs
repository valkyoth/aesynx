use aesynx_abi::ObjectId;

use crate::{CapPerms, CapValidationError, Capability};

pub trait RevocationEpochStore {
    /// Increments and returns the object's revocation epoch.
    ///
    /// Implementations must return `Err(RevocationError::StoreUnavailable)`
    /// instead of wrapping if incrementing the epoch would overflow `u64::MAX`.
    /// Wrapped epochs can make revoked capabilities spuriously validate again.
    fn increment_epoch(&mut self, object_id: ObjectId) -> Result<u64, RevocationError>;

    fn revoke_object_live(
        &mut self,
        authority: &Capability,
        object_id: ObjectId,
        current_generation: u32,
        current_epoch: u64,
    ) -> Result<u64, RevocationError> {
        authority
            .validate_live(current_generation, current_epoch)
            .map_err(RevocationError::from)?;
        ensure_revoke_authority(authority, object_id)?;
        self.increment_epoch(object_id)
    }
}

pub fn ensure_revoke_authority(
    authority: &Capability,
    object_id: ObjectId,
) -> Result<(), RevocationError> {
    if authority.object_id() != object_id {
        return Err(RevocationError::WrongObject);
    }

    if !authority.perms().contains(CapPerms::REVOKE) {
        return Err(RevocationError::MissingRevokePermission);
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RevocationError {
    MissingRevokePermission,
    StaleAuthority,
    AuthorityRevoked,
    WrongObject,
    StoreUnavailable,
}

impl From<CapValidationError> for RevocationError {
    fn from(error: CapValidationError) -> Self {
        match error {
            CapValidationError::StaleGeneration => Self::StaleAuthority,
            CapValidationError::Revoked => Self::AuthorityRevoked,
        }
    }
}

#[cfg(test)]
mod tests {
    use aesynx_abi::{ObjectId, PrincipalId};

    use crate::{
        CapKind, CapPerms, Capability, RevocationEpochStore, RevocationError,
        ensure_revoke_authority,
    };

    #[derive(Default)]
    struct TestEpochStore {
        epoch: u64,
    }

    impl RevocationEpochStore for TestEpochStore {
        fn increment_epoch(&mut self, _object_id: ObjectId) -> Result<u64, RevocationError> {
            self.epoch = self
                .epoch
                .checked_add(1)
                .ok_or(RevocationError::StoreUnavailable)?;
            Ok(self.epoch)
        }
    }

    fn cap(object_id: ObjectId, perms: CapPerms) -> Capability {
        Capability::new_root(object_id, CapKind::Object, PrincipalId::new(1), perms, 1, 1)
    }

    #[test]
    fn revoke_requires_revoke_permission_on_target_object() {
        let object_id = ObjectId::new(7);

        assert_eq!(
            ensure_revoke_authority(&cap(object_id, CapPerms::READ), object_id),
            Err(RevocationError::MissingRevokePermission)
        );
        assert_eq!(
            ensure_revoke_authority(&cap(ObjectId::new(8), CapPerms::REVOKE), object_id),
            Err(RevocationError::WrongObject)
        );
        assert_eq!(
            ensure_revoke_authority(&cap(object_id, CapPerms::REVOKE), object_id),
            Ok(())
        );
    }

    #[test]
    fn revoke_object_checks_authority_before_incrementing_epoch() {
        let object_id = ObjectId::new(7);
        let mut store = TestEpochStore::default();

        assert_eq!(
            store.revoke_object_live(&cap(object_id, CapPerms::READ), object_id, 1, 1),
            Err(RevocationError::MissingRevokePermission)
        );
        assert_eq!(store.epoch, 0);
        assert_eq!(
            store.revoke_object_live(&cap(object_id, CapPerms::REVOKE), object_id, 1, 1),
            Ok(1)
        );
    }

    #[test]
    fn revoke_object_rejects_stale_or_revoked_authority_before_epoch_increment() {
        let object_id = ObjectId::new(7);
        let mut store = TestEpochStore::default();

        assert_eq!(
            store.revoke_object_live(&cap(object_id, CapPerms::REVOKE), object_id, 2, 1),
            Err(RevocationError::StaleAuthority)
        );
        assert_eq!(store.epoch, 0);
        assert_eq!(
            store.revoke_object_live(&cap(object_id, CapPerms::REVOKE), object_id, 1, 2),
            Err(RevocationError::AuthorityRevoked)
        );
        assert_eq!(store.epoch, 0);
    }

    #[test]
    fn revoke_epoch_increment_rejects_overflow() {
        let object_id = ObjectId::new(7);
        let mut store = TestEpochStore { epoch: u64::MAX };

        assert_eq!(
            store.revoke_object_live(&cap(object_id, CapPerms::REVOKE), object_id, 1, 1),
            Err(RevocationError::StoreUnavailable)
        );
        assert_eq!(store.epoch, u64::MAX);
    }
}
