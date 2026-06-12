use super::{
    LimineError, claim_bootinfo_normalization_once, limine_response_revision_compatible,
    reset_bootinfo_normalization_for_test,
};

#[test]
fn limine_response_revision_policy_accepts_forward_compatible_revisions() {
    assert!(limine_response_revision_compatible(0, 0));
    assert!(limine_response_revision_compatible(1, 0));
    assert!(!limine_response_revision_compatible(0, 1));
}

#[test]
fn bootinfo_normalization_claim_is_one_shot() {
    reset_bootinfo_normalization_for_test();

    assert_eq!(claim_bootinfo_normalization_once(), Ok(()));
    assert_eq!(
        claim_bootinfo_normalization_once(),
        Err(LimineError::AlreadyNormalized)
    );

    reset_bootinfo_normalization_for_test();
}
