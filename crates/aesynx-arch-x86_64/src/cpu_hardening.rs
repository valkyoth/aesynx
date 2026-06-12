const CPUID_LEAF_7: u32 = 7;
const CPUID_LEAF_EXTENDED_MAX: u32 = 0x8000_0000;
const CPUID_LEAF_EXTENDED_FEATURES: u32 = 0x8000_0001;
const CPUID_EXT_FEATURE_EDX_NX: u32 = 1 << 20;
const CPUID_LEAF_7_EBX_SMEP: u32 = 1 << 7;
const CPUID_LEAF_7_EBX_SMAP: u32 = 1 << 20;
const CPUID_LEAF_7_ECX_UMIP: u32 = 1 << 2;

const MSR_EFER: u32 = 0xc000_0080;
const EFER_NXE: u64 = 1 << 11;
const CR0_WP: u64 = 1 << 16;
const CR4_UMIP: u64 = 1 << 11;
const CR4_SMEP: u64 = 1 << 20;
const CR4_SMAP: u64 = 1 << 21;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CpuHardeningCapabilities {
    pub nx: bool,
    pub smep: bool,
    pub smap: bool,
    pub umip: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CpuHardeningPlan {
    pub enable_nx: bool,
    pub enable_wp: bool,
    pub enable_smep: bool,
    pub enable_smap: bool,
    pub enable_umip: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CpuHardeningStatus {
    pub nx_enabled: bool,
    pub wp_enabled: bool,
    pub smep_enabled: bool,
    pub smap_enabled: bool,
    pub umip_enabled: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CpuHardeningError {
    NxUnavailable,
}

pub fn init() -> Result<CpuHardeningStatus, CpuHardeningError> {
    let plan = CpuHardeningPlan::required(detect_capabilities())?;
    // SAFETY: The plan is derived from CPUID feature bits and requires NX
    // support before EFER.NXE is written. Optional CR4 features are enabled
    // only when CPUID reports support. This function is called during the
    // terminal single-core boot smoke after Aesynx owns CR3.
    unsafe { apply_plan(plan) };
    Ok(CpuHardeningStatus::from_plan(plan))
}

pub fn detect_capabilities() -> CpuHardeningCapabilities {
    let extended_max = cpuid_eax(CPUID_LEAF_EXTENDED_MAX, 0);
    let nx = extended_max >= CPUID_LEAF_EXTENDED_FEATURES
        && cpuid_edx(CPUID_LEAF_EXTENDED_FEATURES, 0) & CPUID_EXT_FEATURE_EDX_NX != 0;
    let leaf_7_supported = cpuid_eax(0, 0) >= CPUID_LEAF_7;
    let smep = leaf_7_supported && cpuid_ebx(CPUID_LEAF_7, 0) & CPUID_LEAF_7_EBX_SMEP != 0;
    let smap = leaf_7_supported && cpuid_ebx(CPUID_LEAF_7, 0) & CPUID_LEAF_7_EBX_SMAP != 0;
    let umip = leaf_7_supported && cpuid_ecx(CPUID_LEAF_7, 0) & CPUID_LEAF_7_ECX_UMIP != 0;

    CpuHardeningCapabilities {
        nx,
        smep,
        smap,
        umip,
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
        })
    }
}

impl CpuHardeningStatus {
    const fn from_plan(plan: CpuHardeningPlan) -> Self {
        Self {
            nx_enabled: plan.enable_nx,
            wp_enabled: plan.enable_wp,
            smep_enabled: plan.enable_smep,
            smap_enabled: plan.enable_smap,
            umip_enabled: plan.enable_umip,
        }
    }
}

unsafe fn apply_plan(plan: CpuHardeningPlan) {
    let mut efer = read_msr(MSR_EFER);
    if plan.enable_nx {
        efer |= EFER_NXE;
    }
    // SAFETY: `MSR_EFER` is the architectural EFER MSR and the plan enables
    // only the NXE bit after CPUID reported NX support.
    unsafe {
        write_msr(MSR_EFER, efer);
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
}

fn read_msr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    // SAFETY: `rdmsr` reads the selected architectural MSR into EDX:EAX and
    // does not dereference Rust pointers. Callers provide admitted MSR values.
    unsafe {
        core::arch::asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") low,
            out("edx") high,
            options(nomem, nostack, preserves_flags)
        );
    }
    (u64::from(high) << 32) | u64::from(low)
}

unsafe fn write_msr(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    // SAFETY: The caller guarantees that `msr` and `value` preserve
    // architectural reserved-bit requirements for the selected MSR.
    unsafe {
        core::arch::asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") low,
            in("edx") high,
            options(nomem, nostack, preserves_flags)
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

#[cfg(target_arch = "x86_64")]
fn cpuid_eax(leaf: u32, subleaf: u32) -> u32 {
    core::arch::x86_64::__cpuid_count(leaf, subleaf).eax
}

#[cfg(not(target_arch = "x86_64"))]
const fn cpuid_eax(_leaf: u32, _subleaf: u32) -> u32 {
    0
}

#[cfg(target_arch = "x86_64")]
fn cpuid_ebx(leaf: u32, subleaf: u32) -> u32 {
    core::arch::x86_64::__cpuid_count(leaf, subleaf).ebx
}

#[cfg(not(target_arch = "x86_64"))]
const fn cpuid_ebx(_leaf: u32, _subleaf: u32) -> u32 {
    0
}

#[cfg(target_arch = "x86_64")]
fn cpuid_ecx(leaf: u32, subleaf: u32) -> u32 {
    core::arch::x86_64::__cpuid_count(leaf, subleaf).ecx
}

#[cfg(not(target_arch = "x86_64"))]
const fn cpuid_ecx(_leaf: u32, _subleaf: u32) -> u32 {
    0
}

#[cfg(target_arch = "x86_64")]
fn cpuid_edx(leaf: u32, subleaf: u32) -> u32 {
    core::arch::x86_64::__cpuid_count(leaf, subleaf).edx
}

#[cfg(not(target_arch = "x86_64"))]
const fn cpuid_edx(_leaf: u32, _subleaf: u32) -> u32 {
    0
}

#[cfg(test)]
mod tests {
    use super::{
        CpuHardeningCapabilities, CpuHardeningError, CpuHardeningPlan, CpuHardeningStatus,
    };

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
    fn hardening_status_matches_plan_without_register_values() {
        let plan = CpuHardeningPlan {
            enable_nx: true,
            enable_wp: true,
            enable_smep: false,
            enable_smap: true,
            enable_umip: false,
        };

        assert_eq!(
            CpuHardeningStatus::from_plan(plan),
            CpuHardeningStatus {
                nx_enabled: true,
                wp_enabled: true,
                smep_enabled: false,
                smap_enabled: true,
                umip_enabled: false,
            }
        );
    }
}
