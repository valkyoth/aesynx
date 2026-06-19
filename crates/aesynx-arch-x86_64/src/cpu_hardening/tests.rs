use super::{
    AdmittedMsr, CR0_WP, CR4_SMAP, CR4_SMEP, CR4_UMIP, CpuHardeningCapabilities, CpuHardeningError,
    CpuHardeningPlan, CpuHardeningStatus, EFER_NXE, MSR_EFER, MSR_IA32_SPEC_CTRL, SPEC_CTRL_IBRS,
    SPEC_CTRL_SSBD, SPEC_CTRL_STIBP, selected_boot_plan, verify_applied,
};

#[test]
fn admitted_msr_set_is_explicit() {
    assert_eq!(AdmittedMsr::Efer.index(), MSR_EFER);
    assert_eq!(AdmittedMsr::SpecCtrl.index(), MSR_IA32_SPEC_CTRL);
}

const fn base_capabilities() -> CpuHardeningCapabilities {
    CpuHardeningCapabilities {
        nx: true,
        smep: true,
        smap: true,
        umip: true,
        ibrs_ibpb: true,
        stibp: true,
        ssbd: true,
        arch_capabilities: true,
    }
}

#[test]
fn hardening_policy_requires_nx() {
    let capabilities = CpuHardeningCapabilities {
        nx: false,
        ..base_capabilities()
    };

    assert_eq!(
        CpuHardeningPlan::required(capabilities),
        Err(CpuHardeningError::NxUnavailable)
    );
}

#[test]
fn hardening_policy_enables_required_and_supported_bits() {
    let capabilities = CpuHardeningCapabilities {
        nx: true,
        smep: true,
        smap: false,
        umip: true,
        ibrs_ibpb: true,
        stibp: false,
        ssbd: true,
        arch_capabilities: true,
    };
    let plan = CpuHardeningPlan::required(capabilities);

    assert_eq!(
        plan,
        Ok(CpuHardeningPlan {
            enable_nx: true,
            enable_wp: true,
            enable_smep: true,
            enable_smap: false,
            enable_umip: true,
            enable_ibrs: true,
            enable_stibp: false,
            enable_ssbd: true,
            arch_capabilities_supported: true,
        })
    );
}

#[test]
fn strict_hardening_policy_rejects_missing_optional_bits() {
    let no_smep = CpuHardeningCapabilities {
        nx: true,
        smep: false,
        ..base_capabilities()
    };
    let no_smap = CpuHardeningCapabilities {
        nx: true,
        smap: false,
        ..base_capabilities()
    };
    let no_umip = CpuHardeningCapabilities {
        nx: true,
        umip: false,
        ..base_capabilities()
    };

    assert_eq!(
        CpuHardeningPlan::strict_required(no_smep),
        Err(CpuHardeningError::SmepUnavailable)
    );
    assert_eq!(
        CpuHardeningPlan::strict_required(no_smap),
        Err(CpuHardeningError::SmapUnavailable)
    );
    assert_eq!(
        CpuHardeningPlan::strict_required(no_umip),
        Err(CpuHardeningError::UmipUnavailable)
    );
}

#[test]
fn strict_hardening_policy_rejects_missing_speculative_controls() {
    let no_ibrs = CpuHardeningCapabilities {
        ibrs_ibpb: false,
        ..base_capabilities()
    };
    let no_stibp = CpuHardeningCapabilities {
        stibp: false,
        ..base_capabilities()
    };
    let no_ssbd = CpuHardeningCapabilities {
        ssbd: false,
        ..base_capabilities()
    };
    let no_arch_capabilities = CpuHardeningCapabilities {
        arch_capabilities: false,
        ..base_capabilities()
    };

    assert_eq!(
        CpuHardeningPlan::strict_required(no_ibrs),
        Err(CpuHardeningError::IbrsIbpbUnavailable)
    );
    assert_eq!(
        CpuHardeningPlan::strict_required(no_stibp),
        Err(CpuHardeningError::StibpUnavailable)
    );
    assert_eq!(
        CpuHardeningPlan::strict_required(no_ssbd),
        Err(CpuHardeningError::SsbdUnavailable)
    );
    assert_eq!(
        CpuHardeningPlan::strict_required(no_arch_capabilities),
        Err(CpuHardeningError::ArchCapabilitiesUnavailable)
    );
}

#[test]
fn strict_hardening_policy_requires_all_bits() {
    let capabilities = CpuHardeningCapabilities {
        ..base_capabilities()
    };

    assert_eq!(
        CpuHardeningPlan::strict_required(capabilities),
        Ok(CpuHardeningPlan {
            enable_nx: true,
            enable_wp: true,
            enable_smep: true,
            enable_smap: true,
            enable_umip: true,
            enable_ibrs: true,
            enable_stibp: true,
            enable_ssbd: true,
            arch_capabilities_supported: true,
        })
    );
}

#[cfg(not(feature = "strict-cpu-hardening"))]
#[test]
fn default_boot_plan_allows_missing_optional_bits_for_qemu() {
    let capabilities = CpuHardeningCapabilities {
        nx: true,
        smep: false,
        smap: false,
        umip: false,
        ibrs_ibpb: false,
        stibp: false,
        ssbd: false,
        arch_capabilities: false,
    };

    assert_eq!(
        selected_boot_plan(capabilities),
        Ok(CpuHardeningPlan {
            enable_nx: true,
            enable_wp: true,
            enable_smep: false,
            enable_smap: false,
            enable_umip: false,
            enable_ibrs: false,
            enable_stibp: false,
            enable_ssbd: false,
            arch_capabilities_supported: false,
        })
    );
}

#[cfg(feature = "strict-cpu-hardening")]
#[test]
fn strict_boot_plan_rejects_missing_optional_bits() {
    let capabilities = CpuHardeningCapabilities {
        nx: true,
        smep: false,
        smap: true,
        umip: true,
        ibrs_ibpb: true,
        stibp: true,
        ssbd: true,
        arch_capabilities: true,
    };

    assert_eq!(
        selected_boot_plan(capabilities),
        Err(CpuHardeningError::SmepUnavailable)
    );
}

#[test]
fn hardening_status_reports_read_back_register_bits() {
    assert_eq!(
        CpuHardeningStatus::from_registers(
            EFER_NXE,
            CR0_WP,
            CR4_SMAP,
            SPEC_CTRL_IBRS | SPEC_CTRL_SSBD,
            CpuHardeningCapabilities {
                ibrs_ibpb: true,
                ssbd: true,
                arch_capabilities: true,
                ..base_capabilities()
            },
        ),
        CpuHardeningStatus {
            nx_enabled: true,
            wp_enabled: true,
            smep_enabled: false,
            smap_enabled: true,
            umip_enabled: false,
            ibrs_enabled: true,
            ibpb_supported: true,
            stibp_enabled: false,
            ssbd_enabled: true,
            arch_capabilities_supported: true,
        }
    );
}

#[test]
fn hardening_readback_verification_requires_requested_bits() {
    let plan = CpuHardeningPlan {
        enable_nx: true,
        enable_wp: true,
        enable_smep: true,
        enable_smap: false,
        enable_umip: true,
        enable_ibrs: true,
        enable_stibp: false,
        enable_ssbd: true,
        arch_capabilities_supported: true,
    };
    let missing_smep = CpuHardeningStatus::from_registers(
        EFER_NXE,
        CR0_WP,
        CR4_UMIP,
        SPEC_CTRL_IBRS | SPEC_CTRL_SSBD,
        base_capabilities(),
    );
    let missing_ibrs = CpuHardeningStatus::from_registers(
        EFER_NXE,
        CR0_WP,
        CR4_SMEP | CR4_UMIP,
        SPEC_CTRL_SSBD,
        base_capabilities(),
    );
    let applied = CpuHardeningStatus::from_registers(
        EFER_NXE,
        CR0_WP,
        CR4_SMEP | CR4_UMIP,
        SPEC_CTRL_IBRS | SPEC_CTRL_SSBD,
        base_capabilities(),
    );

    assert_eq!(
        verify_applied(plan, missing_smep),
        Err(CpuHardeningError::HardeningWriteDidNotStick)
    );
    assert_eq!(
        verify_applied(plan, missing_ibrs),
        Err(CpuHardeningError::HardeningWriteDidNotStick)
    );
    assert_eq!(verify_applied(plan, applied), Ok(()));
}

#[test]
fn hardening_readback_allows_unrequested_extra_bits() {
    let plan = CpuHardeningPlan {
        enable_nx: true,
        enable_wp: true,
        enable_smep: false,
        enable_smap: false,
        enable_umip: false,
        enable_ibrs: false,
        enable_stibp: false,
        enable_ssbd: false,
        arch_capabilities_supported: false,
    };
    let status = CpuHardeningStatus::from_registers(
        EFER_NXE,
        CR0_WP,
        CR4_SMEP | CR4_SMAP | CR4_UMIP,
        SPEC_CTRL_IBRS | SPEC_CTRL_STIBP | SPEC_CTRL_SSBD,
        base_capabilities(),
    );

    assert_eq!(verify_applied(plan, status), Ok(()));
}
