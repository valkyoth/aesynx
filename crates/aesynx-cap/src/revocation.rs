use aesynx_abi::ObjectId;

use crate::{CapPerms, Capability};

pub trait RevocationEpochStore {
    fn increment_epoch(&mut self, object_id: ObjectId) -> Result<u64, RevocationError>;

    fn revoke_object(
        &mut self,
        authority: Capability,
        object_id: ObjectId,
    ) -> Result<u64, RevocationError> {
        ensure_revoke_authority(authority, object_id)?;
        self.increment_epoch(object_id)
    }
}

pub fn ensure_revoke_authority(
    authority: Capability,
    object_id: ObjectId,
) -> Result<(), RevocationError> {
    if authority.object_id != object_id {
        return Err(RevocationError::WrongObject);
    }

    if !authority.perms.contains(CapPerms::REVOKE) {
        return Err(RevocationError::MissingRevokePermission);
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RevocationError {
    MissingRevokePermission,
    WrongObject,
    StoreUnavailable,
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
            self.epoch += 1;
            Ok(self.epoch)
        }
    }

    fn cap(object_id: ObjectId, perms: CapPerms) -> Capability {
        Capability {
            object_id,
            base: None,
            len: None,
            perms,
            owner: PrincipalId::new(1),
            generation: 1,
            revocation_epoch: 1,
            kind: CapKind::Object,
        }
    }

    #[test]
    fn revoke_requires_revoke_permission_on_target_object() {
        let object_id = ObjectId::new(7);

        assert_eq!(
            ensure_revoke_authority(cap(object_id, CapPerms::READ), object_id),
            Err(RevocationError::MissingRevokePermission)
        );
        assert_eq!(
            ensure_revoke_authority(cap(ObjectId::new(8), CapPerms::REVOKE), object_id),
            Err(RevocationError::WrongObject)
        );
        assert_eq!(
            ensure_revoke_authority(cap(object_id, CapPerms::REVOKE), object_id),
            Ok(())
        );
    }

    #[test]
    fn revoke_object_checks_authority_before_incrementing_epoch() {
        let object_id = ObjectId::new(7);
        let mut store = TestEpochStore::default();

        assert_eq!(
            store.revoke_object(cap(object_id, CapPerms::READ), object_id),
            Err(RevocationError::MissingRevokePermission)
        );
        assert_eq!(store.epoch, 0);
        assert_eq!(
            store.revoke_object(cap(object_id, CapPerms::REVOKE), object_id),
            Ok(1)
        );
    }
}
