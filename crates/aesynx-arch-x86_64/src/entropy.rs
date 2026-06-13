use aesynx_entropy::EntropyCapabilities;

const CPUID_LEAF_1: u32 = 1;
const CPUID_LEAF_7: u32 = 7;
const CPUID_LEAF_1_ECX_RDRAND: u32 = 1 << 30;
const CPUID_LEAF_7_EBX_RDSEED: u32 = 1 << 18;

#[must_use]
pub fn detect_capabilities() -> EntropyCapabilities {
    let max_leaf = cpuid_eax(0, 0);
    let rdrand =
        max_leaf >= CPUID_LEAF_1 && cpuid_ecx(CPUID_LEAF_1, 0) & CPUID_LEAF_1_ECX_RDRAND != 0;
    let rdseed =
        max_leaf >= CPUID_LEAF_7 && cpuid_ebx(CPUID_LEAF_7, 0) & CPUID_LEAF_7_EBX_RDSEED != 0;

    EntropyCapabilities { rdrand, rdseed }
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

#[cfg(test)]
mod tests {
    use aesynx_entropy::{EntropyCapabilities, EntropyEvidence, EntropyPolicyStatus};

    #[test]
    fn entropy_capabilities_feed_safe_policy() {
        let status = EntropyPolicyStatus::classify(EntropyEvidence {
            capabilities: EntropyCapabilities {
                rdrand: true,
                rdseed: false,
            },
            generation_counter_ok: true,
            hardware_self_test_passed: true,
        });

        assert!(status.rdrand_supported);
        assert!(!status.rdseed_supported);
        assert!(status.hardware_entropy_present);
        assert!(!status.fallback_used);
    }
}
