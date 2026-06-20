mod cpuid;

pub use cpuid::detect_capabilities;

const MSR_EFER: u32 = 0xc000_0080;
const MSR_IA32_SPEC_CTRL: u32 = 0x0000_0048;
const MSR_IA32_PRED_CMD: u32 = 0x0000_0049;
const EFER_NXE: u64 = 1 << 11;
const SPEC_CTRL_IBRS: u64 = 1 << 0;
const SPEC_CTRL_STIBP: u64 = 1 << 1;
const SPEC_CTRL_SSBD: u64 = 1 << 2;
const PRED_CMD_IBPB: u64 = 1 << 0;
const CR0_WP: u64 = 1 << 16;
const CR4_UMIP: u64 = 1 << 11;
const CR4_SMEP: u64 = 1 << 20;
const CR4_SMAP: u64 = 1 << 21;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AdmittedMsr {
    Efer,
    PredCmd,
    SpecCtrl,
}

impl AdmittedMsr {
    const fn index(self) -> u32 {
        match self {
            Self::Efer => MSR_EFER,
            Self::PredCmd => MSR_IA32_PRED_CMD,
            Self::SpecCtrl => MSR_IA32_SPEC_CTRL,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CpuHardeningCapabilities {
    pub nx: bool,
    pub smep: bool,
    pub smap: bool,
    pub umip: bool,
    pub ibrs: bool,
    pub ibpb: bool,
    pub stibp: bool,
    pub ssbd: bool,
    pub arch_capabilities: bool,
}

impl CpuHardeningCapabilities {
    const fn spec_ctrl_supported(self) -> bool {
        self.ibrs || self.stibp || self.ssbd
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CpuHardeningPlan {
    pub enable_nx: bool,
    pub enable_wp: bool,
    pub enable_smep: bool,
    pub enable_smap: bool,
    pub enable_umip: bool,
    pub enable_ibpb: bool,
    pub enable_ibrs: bool,
    pub enable_stibp: bool,
    pub enable_ssbd: bool,
    pub arch_capabilities_supported: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CpuHardeningStatus {
    pub nx_enabled: bool,
    pub wp_enabled: bool,
    pub smep_enabled: bool,
    pub smap_enabled: bool,
    pub umip_enabled: bool,
    pub ibrs_enabled: bool,
    pub ibpb_supported: bool,
    pub ibpb_attempted: bool,
    pub stibp_enabled: bool,
    pub ssbd_enabled: bool,
    pub arch_capabilities_supported: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CpuHardeningError {
    HardeningWriteDidNotStick,
    NxUnavailable,
    ArchCapabilitiesUnavailable,
    IbrsUnavailable,
    IbpbUnavailable,
    SmapUnavailable,
    SmepUnavailable,
    SsbdUnavailable,
    StibpUnavailable,
    UmipUnavailable,
}

pub fn init() -> Result<CpuHardeningStatus, CpuHardeningError> {
    let capabilities = detect_capabilities();
    let plan = selected_boot_plan(capabilities)?;
    // SAFETY: The plan is derived from CPUID feature bits and requires NX
    // support before EFER.NXE is written. Optional CR4 features are enabled
    // only when CPUID reports support. IA32_SPEC_CTRL bits are requested only
    // when CPUID reports their controls. IA32_PRED_CMD is write-only; the
    // boot-time IBPB write can be attempted and #GP-fail if unsupported, but
    // cannot be confirmed effective by readback. This function is called
    // during the terminal single-core boot smoke after Aesynx owns CR3.
    unsafe { apply_plan(plan) };
    let status = read_status(capabilities, plan);
    verify_applied(plan, status)?;
    Ok(status)
}

fn selected_boot_plan(
    capabilities: CpuHardeningCapabilities,
) -> Result<CpuHardeningPlan, CpuHardeningError> {
    if cfg!(feature = "strict-cpu-hardening") {
        CpuHardeningPlan::strict_required(capabilities)
    } else {
        CpuHardeningPlan::required(capabilities)
    }
}

impl CpuHardeningPlan {
    pub const fn required(
        capabilities: CpuHardeningCapabilities,
    ) -> Result<Self, CpuHardeningError> {
        if !capabilities.nx {
            return Err(CpuHardeningError::NxUnavailable);
        }

        Ok(Self {
            enable_nx: true,
            enable_wp: true,
            enable_smep: capabilities.smep,
            enable_smap: capabilities.smap,
            enable_umip: capabilities.umip,
            enable_ibpb: capabilities.ibpb,
            enable_ibrs: capabilities.ibrs,
            enable_stibp: capabilities.stibp,
            enable_ssbd: capabilities.ssbd,
            arch_capabilities_supported: capabilities.arch_capabilities,
        })
    }

    pub const fn strict_required(
        capabilities: CpuHardeningCapabilities,
    ) -> Result<Self, CpuHardeningError> {
        if !capabilities.nx {
            return Err(CpuHardeningError::NxUnavailable);
        }
        if !capabilities.smep {
            return Err(CpuHardeningError::SmepUnavailable);
        }
        if !capabilities.smap {
            return Err(CpuHardeningError::SmapUnavailable);
        }
        if !capabilities.umip {
            return Err(CpuHardeningError::UmipUnavailable);
        }
        if !capabilities.ibrs {
            return Err(CpuHardeningError::IbrsUnavailable);
        }
        if !capabilities.ibpb {
            return Err(CpuHardeningError::IbpbUnavailable);
        }
        if !capabilities.stibp {
            return Err(CpuHardeningError::StibpUnavailable);
        }
        if !capabilities.ssbd {
            return Err(CpuHardeningError::SsbdUnavailable);
        }
        if !capabilities.arch_capabilities {
            return Err(CpuHardeningError::ArchCapabilitiesUnavailable);
        }

        Ok(Self {
            enable_nx: true,
            enable_wp: true,
            enable_smep: true,
            enable_smap: true,
            enable_umip: true,
            enable_ibpb: true,
            enable_ibrs: true,
            enable_stibp: true,
            enable_ssbd: true,
            arch_capabilities_supported: true,
        })
    }
}

impl CpuHardeningStatus {
    const fn from_registers(
        efer: u64,
        cr0: u64,
        cr4: u64,
        spec_ctrl: u64,
        capabilities: CpuHardeningCapabilities,
        ibpb_attempted: bool,
    ) -> Self {
        Self {
            nx_enabled: efer & EFER_NXE != 0,
            wp_enabled: cr0 & CR0_WP != 0,
            smep_enabled: cr4 & CR4_SMEP != 0,
            smap_enabled: cr4 & CR4_SMAP != 0,
            umip_enabled: cr4 & CR4_UMIP != 0,
            ibrs_enabled: spec_ctrl & SPEC_CTRL_IBRS != 0,
            ibpb_supported: capabilities.ibpb,
            ibpb_attempted,
            stibp_enabled: spec_ctrl & SPEC_CTRL_STIBP != 0,
            ssbd_enabled: spec_ctrl & SPEC_CTRL_SSBD != 0,
            arch_capabilities_supported: capabilities.arch_capabilities,
        }
    }
}

fn read_status(
    capabilities: CpuHardeningCapabilities,
    plan: CpuHardeningPlan,
) -> CpuHardeningStatus {
    let spec_ctrl = if capabilities.spec_ctrl_supported() {
        read_msr(AdmittedMsr::SpecCtrl)
    } else {
        0
    };
    CpuHardeningStatus::from_registers(
        read_msr(AdmittedMsr::Efer),
        read_cr0(),
        read_cr4(),
        spec_ctrl,
        capabilities,
        plan.enable_ibpb,
    )
}

const fn verify_applied(
    plan: CpuHardeningPlan,
    status: CpuHardeningStatus,
) -> Result<(), CpuHardeningError> {
    if (plan.enable_nx && !status.nx_enabled)
        || (plan.enable_wp && !status.wp_enabled)
        || (plan.enable_smep && !status.smep_enabled)
        || (plan.enable_smap && !status.smap_enabled)
        || (plan.enable_umip && !status.umip_enabled)
        || (plan.enable_ibrs && !status.ibrs_enabled)
        || (plan.enable_stibp && !status.stibp_enabled)
        || (plan.enable_ssbd && !status.ssbd_enabled)
        || (plan.arch_capabilities_supported && !status.arch_capabilities_supported)
    {
        return Err(CpuHardeningError::HardeningWriteDidNotStick);
    }

    // IA32_PRED_CMD.IBPB is intentionally absent from register read-back
    // verification. The architectural MSR is write-only: a supported write is
    // a one-shot barrier, not sticky state. A failed unsupported write should
    // raise #GP before this function is reached, but non-conformant hardware or
    // a hypervisor that silently drops WRMSR cannot be detected here. Status
    // therefore separates `ibpb_supported` from the boot-time
    // `ibpb_attempted` evidence and makes no per-context-switch IBPB claim.
    Ok(())
}

unsafe fn apply_plan(plan: CpuHardeningPlan) {
    let mut efer = read_msr(AdmittedMsr::Efer);
    if plan.enable_nx {
        efer |= EFER_NXE;
    }
    // SAFETY: `AdmittedMsr::Efer` is the architectural EFER MSR and the plan
    // enables only the NXE bit after CPUID reported NX support.
    unsafe {
        write_msr(AdmittedMsr::Efer, efer);
    }

    let mut cr0 = read_cr0();
    if plan.enable_wp {
        cr0 |= CR0_WP;
    }
    // SAFETY: The new CR0 value preserves all existing bits and only forces WP
    // on, making supervisor writes respect read-only pages.
    unsafe {
        write_cr0(cr0);
    }

    let mut cr4 = read_cr4();
    if plan.enable_umip {
        cr4 |= CR4_UMIP;
    }
    if plan.enable_smep {
        cr4 |= CR4_SMEP;
    }
    if plan.enable_smap {
        cr4 |= CR4_SMAP;
    }
    // SAFETY: The new CR4 value preserves all existing bits and only enables
    // CPUID-gated hardening features.
    unsafe {
        write_cr4(cr4);
    }

    let mut spec_ctrl = 0u64;
    if plan.enable_ibrs || plan.enable_stibp || plan.enable_ssbd {
        spec_ctrl = read_msr(AdmittedMsr::SpecCtrl);
    }
    if plan.enable_ibrs {
        spec_ctrl |= SPEC_CTRL_IBRS;
    }
    if plan.enable_stibp {
        spec_ctrl |= SPEC_CTRL_STIBP;
    }
    if plan.enable_ssbd {
        spec_ctrl |= SPEC_CTRL_SSBD;
    }
    if plan.enable_ibrs || plan.enable_stibp || plan.enable_ssbd {
        // SAFETY: `AdmittedMsr::SpecCtrl` is the architectural IA32_SPEC_CTRL
        // MSR. The plan enables only CPUID-gated speculative-execution control
        // bits and preserves all existing bits read from the register.
        unsafe {
            write_msr(AdmittedMsr::SpecCtrl, spec_ctrl);
        }
    }
    if plan.enable_ibpb {
        // SAFETY: `AdmittedMsr::PredCmd` is the architectural IA32_PRED_CMD
        // MSR and CPUID reported Intel or AMD IBPB support. Writing bit 0
        // issues a one-shot indirect-branch predictor barrier.
        unsafe {
            write_msr(AdmittedMsr::PredCmd, PRED_CMD_IBPB);
        }
    }
}

fn read_msr(msr: AdmittedMsr) -> u64 {
    let low: u32;
    let high: u32;
    let index = msr.index();
    // SAFETY: `rdmsr` reads the selected architectural MSR into EDX:EAX and
    // does not dereference Rust pointers. The enum admits only reviewed MSRs.
    unsafe {
        core::arch::asm!(
            "rdmsr",
            in("ecx") index,
            out("eax") low,
            out("edx") high,
            options(nomem, nostack, preserves_flags)
        );
    }
    (u64::from(high) << 32) | u64::from(low)
}

unsafe fn write_msr(msr: AdmittedMsr, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    let index = msr.index();
    // SAFETY: The caller guarantees that `value` preserves architectural
    // reserved-bit requirements for the selected admitted MSR.
    unsafe {
        core::arch::asm!(
            "wrmsr",
            in("ecx") index,
            in("eax") low,
            in("edx") high,
            options(nostack, preserves_flags)
        );
    }
}

fn read_cr0() -> u64 {
    let value: u64;
    // SAFETY: This copies CR0 into a general-purpose register and does not
    // dereference Rust pointers.
    unsafe {
        core::arch::asm!("mov {value}, cr0", value = lateout(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

unsafe fn write_cr0(value: u64) {
    // SAFETY: The caller guarantees that `value` preserves required CR0 bits.
    unsafe {
        core::arch::asm!("mov cr0, {value}", value = in(reg) value, options(nostack, preserves_flags));
    }
}

fn read_cr4() -> u64 {
    let value: u64;
    // SAFETY: This copies CR4 into a general-purpose register and does not
    // dereference Rust pointers.
    unsafe {
        core::arch::asm!("mov {value}, cr4", value = lateout(reg) value, options(nomem, nostack, preserves_flags));
    }
    value
}

unsafe fn write_cr4(value: u64) {
    // SAFETY: The caller guarantees that `value` preserves required CR4 bits
    // and enables only CPUID-supported features.
    unsafe {
        core::arch::asm!("mov cr4, {value}", value = in(reg) value, options(nostack, preserves_flags));
    }
}

#[cfg(test)]
mod tests;
