#![no_std]
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntropyCapabilities {
    pub rdrand: bool,
    pub rdseed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntropyEvidence {
    pub capabilities: EntropyCapabilities,
    pub generation_counter_ok: bool,
    pub hardware_self_test_passed: bool,
    pub drbg_self_test_passed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntropySource {
    Drbg,
    RdSeed,
    RdRand,
    DeterministicMonotonicFallback,
}

impl EntropySource {
    #[must_use]
    pub const fn quality(self) -> EntropyQuality {
        match self {
            Self::Drbg => EntropyQuality::DrbgOutput,
            Self::RdSeed => EntropyQuality::HardwareSeed,
            Self::RdRand => EntropyQuality::HardwareRandom,
            Self::DeterministicMonotonicFallback => EntropyQuality::DeterministicAntiConfusion,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntropyQuality {
    DrbgOutput,
    HardwareSeed,
    HardwareRandom,
    DeterministicAntiConfusion,
}

impl EntropyQuality {
    #[must_use]
    pub const fn attacker_unpredictable(self) -> bool {
        matches!(
            self,
            Self::DrbgOutput | Self::HardwareSeed | Self::HardwareRandom
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntropyPolicyStatus {
    pub rdrand_supported: bool,
    pub rdseed_supported: bool,
    pub hardware_entropy_present: bool,
    pub hardware_self_test_passed: bool,
    pub drbg_self_test_passed: bool,
    pub fallback_used: bool,
    pub generation_counter_ok: bool,
    pub random_tokens_available: bool,
    pub primary_source: EntropySource,
}

impl EntropyPolicyStatus {
    #[must_use]
    pub const fn classify(evidence: EntropyEvidence) -> Self {
        let hardware_feature_present = evidence.capabilities.rdrand || evidence.capabilities.rdseed;
        let hardware_entropy_present =
            hardware_feature_present && evidence.hardware_self_test_passed;
        let drbg_ready = hardware_entropy_present && evidence.drbg_self_test_passed;
        let primary_source = if drbg_ready {
            EntropySource::Drbg
        } else if hardware_entropy_present && evidence.capabilities.rdseed {
            EntropySource::RdSeed
        } else if hardware_entropy_present && evidence.capabilities.rdrand {
            EntropySource::RdRand
        } else {
            EntropySource::DeterministicMonotonicFallback
        };

        Self {
            rdrand_supported: evidence.capabilities.rdrand,
            rdseed_supported: evidence.capabilities.rdseed,
            hardware_entropy_present,
            hardware_self_test_passed: evidence.hardware_self_test_passed,
            drbg_self_test_passed: evidence.drbg_self_test_passed,
            fallback_used: !hardware_entropy_present,
            generation_counter_ok: evidence.generation_counter_ok,
            random_tokens_available: evidence.generation_counter_ok
                && drbg_ready
                && primary_source.quality().attacker_unpredictable(),
            primary_source,
        }
    }

    pub const fn require_random_tokens(self) -> Result<RandomTokenPolicy, EntropyError> {
        if !self.random_tokens_available {
            return Err(EntropyError::RandomTokenRequiresDrbg);
        }

        Ok(RandomTokenPolicy {
            source: self.primary_source,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RandomTokenPolicy {
    source: EntropySource,
}

impl RandomTokenPolicy {
    #[must_use]
    pub const fn source(self) -> EntropySource {
        self.source
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Generation(u64);

impl Generation {
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GenerationCounter {
    next: u64,
}

impl GenerationCounter {
    #[must_use]
    pub const fn new(start: u64) -> Self {
        Self { next: start }
    }

    pub const fn next_generation(&mut self) -> Result<Generation, EntropyError> {
        let Some(next) = self.next.checked_add(1) else {
            return Err(EntropyError::GenerationCounterOverflow);
        };
        self.next = next;
        Ok(Generation(next))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntropyError {
    GenerationCounterOverflow,
    RandomTokenRequiresDrbg,
    RandomTokenRequiresHardwareEntropy,
}

#[cfg(test)]
mod tests {
    use super::{
        EntropyCapabilities, EntropyError, EntropyEvidence, EntropyPolicyStatus, EntropyQuality,
        EntropySource, GenerationCounter,
    };

    #[test]
    fn source_quality_distinguishes_random_from_anti_confusion() {
        assert_eq!(EntropySource::Drbg.quality(), EntropyQuality::DrbgOutput);
        assert_eq!(
            EntropySource::RdSeed.quality(),
            EntropyQuality::HardwareSeed
        );
        assert_eq!(
            EntropySource::RdRand.quality(),
            EntropyQuality::HardwareRandom
        );
        assert_eq!(
            EntropySource::DeterministicMonotonicFallback.quality(),
            EntropyQuality::DeterministicAntiConfusion
        );
        assert!(EntropySource::Drbg.quality().attacker_unpredictable());
        assert!(EntropySource::RdSeed.quality().attacker_unpredictable());
        assert!(
            !EntropySource::DeterministicMonotonicFallback
                .quality()
                .attacker_unpredictable()
        );
    }

    #[test]
    fn policy_prefers_rdseed_then_rdrand_then_fallback() {
        assert_eq!(
            EntropyPolicyStatus::classify(EntropyEvidence {
                capabilities: EntropyCapabilities {
                    rdrand: true,
                    rdseed: true,
                },
                generation_counter_ok: true,
                hardware_self_test_passed: true,
                drbg_self_test_passed: false,
            })
            .primary_source,
            EntropySource::RdSeed
        );
        assert_eq!(
            EntropyPolicyStatus::classify(EntropyEvidence {
                capabilities: EntropyCapabilities {
                    rdrand: true,
                    rdseed: false,
                },
                generation_counter_ok: true,
                hardware_self_test_passed: true,
                drbg_self_test_passed: false,
            })
            .primary_source,
            EntropySource::RdRand
        );
        assert_eq!(
            EntropyPolicyStatus::classify(EntropyEvidence {
                capabilities: EntropyCapabilities {
                    rdrand: false,
                    rdseed: false,
                },
                generation_counter_ok: true,
                hardware_self_test_passed: true,
                drbg_self_test_passed: false,
            })
            .primary_source,
            EntropySource::DeterministicMonotonicFallback
        );
    }

    #[test]
    fn deterministic_fallback_is_anti_confusion_only() {
        let status = EntropyPolicyStatus::classify(EntropyEvidence {
            capabilities: EntropyCapabilities {
                rdrand: false,
                rdseed: false,
            },
            generation_counter_ok: true,
            hardware_self_test_passed: false,
            drbg_self_test_passed: false,
        });

        assert!(status.fallback_used);
        assert!(status.generation_counter_ok);
        assert!(!status.random_tokens_available);
        assert_eq!(
            status.require_random_tokens(),
            Err(EntropyError::RandomTokenRequiresDrbg)
        );
    }

    #[test]
    fn random_tokens_require_drbg_seeded_from_hardware_entropy() {
        let status = EntropyPolicyStatus::classify(EntropyEvidence {
            capabilities: EntropyCapabilities {
                rdrand: true,
                rdseed: false,
            },
            generation_counter_ok: true,
            hardware_self_test_passed: true,
            drbg_self_test_passed: true,
        });

        assert!(!status.fallback_used);
        assert!(status.random_tokens_available);
        assert_eq!(
            status.require_random_tokens().map(|policy| policy.source()),
            Ok(EntropySource::Drbg)
        );
    }

    #[test]
    fn hardware_entropy_without_drbg_does_not_enable_random_tokens() {
        let status = EntropyPolicyStatus::classify(EntropyEvidence {
            capabilities: EntropyCapabilities {
                rdrand: true,
                rdseed: false,
            },
            generation_counter_ok: true,
            hardware_self_test_passed: true,
            drbg_self_test_passed: false,
        });

        assert!(!status.fallback_used);
        assert!(!status.random_tokens_available);
        assert_eq!(status.primary_source, EntropySource::RdRand);
        assert_eq!(
            status.require_random_tokens(),
            Err(EntropyError::RandomTokenRequiresDrbg)
        );
    }

    #[test]
    fn cpuid_without_runtime_self_test_does_not_enable_random_tokens() {
        let status = EntropyPolicyStatus::classify(EntropyEvidence {
            capabilities: EntropyCapabilities {
                rdrand: true,
                rdseed: true,
            },
            generation_counter_ok: true,
            hardware_self_test_passed: false,
            drbg_self_test_passed: false,
        });

        assert!(status.rdrand_supported);
        assert!(status.rdseed_supported);
        assert!(!status.hardware_entropy_present);
        assert!(status.fallback_used);
        assert!(!status.random_tokens_available);
        assert_eq!(
            status.primary_source,
            EntropySource::DeterministicMonotonicFallback
        );
    }

    #[test]
    fn failed_generation_check_is_reflected_in_status() {
        let status = EntropyPolicyStatus::classify(EntropyEvidence {
            capabilities: EntropyCapabilities {
                rdrand: true,
                rdseed: false,
            },
            generation_counter_ok: false,
            hardware_self_test_passed: true,
            drbg_self_test_passed: true,
        });

        assert!(!status.generation_counter_ok);
        assert!(!status.random_tokens_available);
    }

    #[test]
    fn generation_counter_fails_instead_of_wrapping() {
        let mut counter = GenerationCounter::new(u64::MAX - 1);

        assert_eq!(
            counter.next_generation().map(|generation| generation.get()),
            Ok(u64::MAX)
        );
        assert_eq!(
            counter.next_generation(),
            Err(EntropyError::GenerationCounterOverflow)
        );
    }
}
