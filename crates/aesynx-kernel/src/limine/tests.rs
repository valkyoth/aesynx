use super::{
    LimineError, claim_bootinfo_normalization_once, limine_response_revision_compatible,
    reset_bootinfo_normalization_for_test, valid_handoff_virt,
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

#[test]
fn limine_payload_addresses_must_be_high_half_canonical() {
    const X86_64_KERNEL_VMA_MIN: u64 = 0xffff_8000_0000_0000;

    assert!(valid_handoff_virt(
        0xffff_8000_0000_1000,
        X86_64_KERNEL_VMA_MIN
    ));
    assert!(!valid_handoff_virt(0x1000, X86_64_KERNEL_VMA_MIN));
    assert!(!valid_handoff_virt(
        0x0000_8000_0000_0000,
        X86_64_KERNEL_VMA_MIN
    ));
    assert!(!valid_handoff_virt(
        0xffff_0000_0000_0000,
        X86_64_KERNEL_VMA_MIN
    ));
}
