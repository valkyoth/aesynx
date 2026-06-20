use super::CpuHardeningCapabilities;

pub(super) const CPUID_LEAF_7_EDX_IBRS_IBPB: u32 = 1 << 26;
pub(super) const CPUID_LEAF_7_EDX_ARCH_CAPABILITIES: u32 = 1 << 29;
pub(super) const CPUID_AMD_EXTENDED_EBX_IBRS: u32 = 1 << 9;
pub(super) const CPUID_AMD_EXTENDED_EBX_IBPB: u32 = 1 << 12;
pub(super) const CPUID_AMD_EXTENDED_EBX_STIBP: u32 = 1 << 15;
pub(super) const CPUID_AMD_EXTENDED_EBX_SSBD: u32 = 1 << 24;

const CPUID_LEAF_7: u32 = 7;
const CPUID_LEAF_EXTENDED_MAX: u32 = 0x8000_0000;
const CPUID_LEAF_EXTENDED_FEATURES: u32 = 0x8000_0001;
const CPUID_LEAF_AMD_EXTENDED_SECURITY: u32 = 0x8000_0008;
const CPUID_EXT_FEATURE_EDX_NX: u32 = 1 << 20;
const CPUID_LEAF_7_EBX_SMEP: u32 = 1 << 7;
const CPUID_LEAF_7_EBX_SMAP: u32 = 1 << 20;
const CPUID_LEAF_7_ECX_UMIP: u32 = 1 << 2;
const CPUID_LEAF_7_EDX_STIBP: u32 = 1 << 27;
const CPUID_LEAF_7_EDX_SSBD: u32 = 1 << 31;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct CpuidSnapshot {
    eax: u32,
    pub(super) ebx: u32,
    ecx: u32,
    pub(super) edx: u32,
}

impl CpuidSnapshot {
    pub(super) const ZERO: Self = Self {
        eax: 0,
        ebx: 0,
        ecx: 0,
        edx: 0,
    };

    #[cfg(test)]
    pub(super) const fn from_regs(eax: u32, ebx: u32, ecx: u32, edx: u32) -> Self {
        Self { eax, ebx, ecx, edx }
    }
}

pub fn detect_capabilities() -> CpuHardeningCapabilities {
    let extended_max = cpuid_eax(CPUID_LEAF_EXTENDED_MAX, 0);
    let nx = extended_max >= CPUID_LEAF_EXTENDED_FEATURES
        && cpuid_snapshot(CPUID_LEAF_EXTENDED_FEATURES, 0).edx & CPUID_EXT_FEATURE_EDX_NX != 0;
    let leaf_7_supported = cpuid_eax(0, 0) >= CPUID_LEAF_7;
    let leaf_7 = if leaf_7_supported {
        cpuid_snapshot(CPUID_LEAF_7, 0)
    } else {
        CpuidSnapshot::ZERO
    };
    let smep = leaf_7.ebx & CPUID_LEAF_7_EBX_SMEP != 0;
    let smap = leaf_7.ebx & CPUID_LEAF_7_EBX_SMAP != 0;
    let umip = leaf_7.ecx & CPUID_LEAF_7_ECX_UMIP != 0;
    let amd_extended = if extended_max >= CPUID_LEAF_AMD_EXTENDED_SECURITY {
        cpuid_snapshot(CPUID_LEAF_AMD_EXTENDED_SECURITY, 0)
    } else {
        CpuidSnapshot::ZERO
    };

    capabilities_from_cpuid(nx, smep, smap, umip, leaf_7, amd_extended)
}

pub(super) const fn capabilities_from_cpuid(
    nx: bool,
    smep: bool,
    smap: bool,
    umip: bool,
    leaf_7: CpuidSnapshot,
    amd_extended: CpuidSnapshot,
) -> CpuHardeningCapabilities {
    let intel_ibrs_ibpb = leaf_7.edx & CPUID_LEAF_7_EDX_IBRS_IBPB != 0;
    let amd_ibrs = amd_extended.ebx & CPUID_AMD_EXTENDED_EBX_IBRS != 0;
    let amd_ibpb = amd_extended.ebx & CPUID_AMD_EXTENDED_EBX_IBPB != 0;
    let stibp = leaf_7.edx & CPUID_LEAF_7_EDX_STIBP != 0
        || amd_extended.ebx & CPUID_AMD_EXTENDED_EBX_STIBP != 0;
    let ssbd = leaf_7.edx & CPUID_LEAF_7_EDX_SSBD != 0
        || amd_extended.ebx & CPUID_AMD_EXTENDED_EBX_SSBD != 0;
    let arch_capabilities = leaf_7.edx & CPUID_LEAF_7_EDX_ARCH_CAPABILITIES != 0;

    CpuHardeningCapabilities {
        nx,
        smep,
        smap,
        umip,
        ibrs: intel_ibrs_ibpb || amd_ibrs,
        ibpb: intel_ibrs_ibpb || amd_ibpb,
        stibp,
        ssbd,
        arch_capabilities,
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
fn cpuid_snapshot(leaf: u32, subleaf: u32) -> CpuidSnapshot {
    let result = core::arch::x86_64::__cpuid_count(leaf, subleaf);
    CpuidSnapshot {
        eax: result.eax,
        ebx: result.ebx,
        ecx: result.ecx,
        edx: result.edx,
    }
}

#[cfg(not(target_arch = "x86_64"))]
const fn cpuid_snapshot(_leaf: u32, _subleaf: u32) -> CpuidSnapshot {
    CpuidSnapshot::ZERO
}
