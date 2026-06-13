#![no_std]
#![deny(unsafe_code)]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntropyCapabilities {
    pub rdrand: bool,
    pub rdseed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntropySource {
    RdSeed,
    RdRand,
    DeterministicMonotonicFallback,
}

impl EntropySource {
    #[must_use]
    pub const fn quality(self) -> EntropyQuality {
        match self {
            Self::RdSeed => EntropyQuality::HardwareSeed,
            Self::RdRand => EntropyQuality::HardwareRandom,
            Self::DeterministicMonotonicFallback => EntropyQuality::DeterministicAntiConfusion,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntropyQuality {
    HardwareSeed,
    HardwareRandom,
    DeterministicAntiConfusion,
}

impl EntropyQuality {
    #[must_use]
    pub const fn attacker_unpredictable(self) -> bool {
        matches!(self, Self::HardwareSeed | Self::HardwareRandom)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntropyPolicyStatus {
    pub rdrand_supported: bool,
    pub rdseed_supported: bool,
    pub hardware_entropy_present: bool,
    pub fallback_used: bool,
    pub generation_counter_ok: bool,
    pub random_tokens_available: bool,
    pub primary_source: EntropySource,
}

impl EntropyPolicyStatus {
    #[must_use]
    pub const fn classify(capabilities: EntropyCapabilities) -> Self {
        let hardware_entropy_present = capabilities.rdrand || capabilities.rdseed;
        let primary_source = if capabilities.rdseed {
            EntropySource::RdSeed
        } else if capabilities.rdrand {
            EntropySource::RdRand
        } else {
            EntropySource::DeterministicMonotonicFallback
        };

        Self {
            rdrand_supported: capabilities.rdrand,
            rdseed_supported: capabilities.rdseed,
            hardware_entropy_present,
            fallback_used: !hardware_entropy_present,
            generation_counter_ok: true,
            random_tokens_available: primary_source.quality().attacker_unpredictable(),
            primary_source,
        }
    }

    pub const fn require_random_tokens(self) -> Result<RandomTokenPolicy, EntropyError> {
        if !self.random_tokens_available {
            return Err(EntropyError::RandomTokenRequiresHardwareEntropy);
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
    RandomTokenRequiresHardwareEntropy,
}

#[cfg(test)]
mod tests {
    use super::{
        EntropyCapabilities, EntropyError, EntropyPolicyStatus, EntropyQuality, EntropySource,
        GenerationCounter,
    };

    #[test]
    fn source_quality_distinguishes_random_from_anti_confusion() {
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
            EntropyPolicyStatus::classify(EntropyCapabilities {
                rdrand: true,
                rdseed: true,
            })
            .primary_source,
            EntropySource::RdSeed
        );
        assert_eq!(
            EntropyPolicyStatus::classify(EntropyCapabilities {
                rdrand: true,
                rdseed: false,
            })
            .primary_source,
            EntropySource::RdRand
        );
        assert_eq!(
            EntropyPolicyStatus::classify(EntropyCapabilities {
                rdrand: false,
                rdseed: false,
            })
            .primary_source,
            EntropySource::DeterministicMonotonicFallback
        );
    }

    #[test]
    fn deterministic_fallback_is_anti_confusion_only() {
        let status = EntropyPolicyStatus::classify(EntropyCapabilities {
            rdrand: false,
            rdseed: false,
        });

        assert!(status.fallback_used);
        assert!(status.generation_counter_ok);
        assert!(!status.random_tokens_available);
        assert_eq!(
            status.require_random_tokens(),
            Err(EntropyError::RandomTokenRequiresHardwareEntropy)
        );
    }

    #[test]
    fn random_tokens_require_hardware_entropy() {
        let status = EntropyPolicyStatus::classify(EntropyCapabilities {
            rdrand: true,
            rdseed: false,
        });

        assert!(!status.fallback_used);
        assert!(status.random_tokens_available);
        assert_eq!(
            status.require_random_tokens().map(|policy| policy.source()),
            Ok(EntropySource::RdRand)
        );
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
