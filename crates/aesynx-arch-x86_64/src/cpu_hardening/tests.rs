use super::{
    AdmittedMsr, CR0_WP, CR4_SMAP, CR4_SMEP, CR4_UMIP, CpuHardeningCapabilities, CpuHardeningError,
    CpuHardeningPlan, CpuHardeningStatus, EFER_NXE, MSR_EFER, selected_boot_plan, verify_applied,
};

#[test]
fn admitted_msr_set_is_explicit() {
    assert_eq!(AdmittedMsr::Efer.index(), MSR_EFER);
}

#[test]
fn hardening_policy_requires_nx() {
    let capabilities = CpuHardeningCapabilities {
        nx: false,
        smep: true,
        smap: true,
        umip: true,
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
        })
    );
}

#[test]
fn strict_hardening_policy_rejects_missing_optional_bits() {
    let no_smep = CpuHardeningCapabilities {
        nx: true,
        smep: false,
        smap: true,
        umip: true,
    };
    let no_smap = CpuHardeningCapabilities {
        nx: true,
        smep: true,
        smap: false,
        umip: true,
    };
    let no_umip = CpuHardeningCapabilities {
        nx: true,
        smep: true,
        smap: true,
        umip: false,
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
fn strict_hardening_policy_requires_all_bits() {
    let capabilities = CpuHardeningCapabilities {
        nx: true,
        smep: true,
        smap: true,
        umip: true,
    };

    assert_eq!(
        CpuHardeningPlan::strict_required(capabilities),
        Ok(CpuHardeningPlan {
            enable_nx: true,
            enable_wp: true,
            enable_smep: true,
            enable_smap: true,
            enable_umip: true,
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
    };

    assert_eq!(
        selected_boot_plan(capabilities),
        Ok(CpuHardeningPlan {
            enable_nx: true,
            enable_wp: true,
            enable_smep: false,
            enable_smap: false,
            enable_umip: false,
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
    };

    assert_eq!(
        selected_boot_plan(capabilities),
        Err(CpuHardeningError::SmepUnavailable)
    );
}

#[test]
fn hardening_status_reports_read_back_register_bits() {
    assert_eq!(
        CpuHardeningStatus::from_registers(EFER_NXE, CR0_WP, CR4_SMAP),
        CpuHardeningStatus {
            nx_enabled: true,
            wp_enabled: true,
            smep_enabled: false,
            smap_enabled: true,
            umip_enabled: false,
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
    };
    let missing_smep = CpuHardeningStatus::from_registers(EFER_NXE, CR0_WP, CR4_UMIP);
    let applied = CpuHardeningStatus::from_registers(EFER_NXE, CR0_WP, CR4_SMEP | CR4_UMIP);

    assert_eq!(
        verify_applied(plan, missing_smep),
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
    };
    let status =
        CpuHardeningStatus::from_registers(EFER_NXE, CR0_WP, CR4_SMEP | CR4_SMAP | CR4_UMIP);

    assert_eq!(verify_applied(plan, status), Ok(()));
}
