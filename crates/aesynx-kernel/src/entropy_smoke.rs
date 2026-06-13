use aesynx_entropy::{EntropyEvidence, EntropyPolicyStatus, EntropySource, GenerationCounter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntropySmokeError {
    GenerationCounterWrapped,
    RandomTokenAcceptedFallback,
}

pub fn run() -> Result<EntropyPolicyStatus, EntropySmokeError> {
    let capabilities = aesynx_arch_x86_64::entropy::detect_capabilities();
    let generation_counter_ok = verify_generation_counter().is_ok();
    if !generation_counter_ok {
        return Err(EntropySmokeError::GenerationCounterWrapped);
    }

    // v0.18.1 deliberately does not execute RDRAND/RDSEED. CPUID feature bits
    // are visible in telemetry, but random-token policy remains disabled until
    // a future runtime self-test-backed read path exists.
    let status = EntropyPolicyStatus::classify(EntropyEvidence {
        capabilities,
        generation_counter_ok,
        hardware_self_test_passed: false,
    });
    verify_random_token_policy(status)?;

    Ok(status)
}

fn verify_generation_counter() -> Result<(), EntropySmokeError> {
    let mut counter = GenerationCounter::new(u64::MAX - 1);
    if counter.next_generation().is_err() {
        return Err(EntropySmokeError::GenerationCounterWrapped);
    }
    if counter.next_generation().is_ok() {
        return Err(EntropySmokeError::GenerationCounterWrapped);
    }
    Ok(())
}

fn verify_random_token_policy(status: EntropyPolicyStatus) -> Result<(), EntropySmokeError> {
    if status.primary_source == EntropySource::DeterministicMonotonicFallback
        && status.require_random_tokens().is_ok()
    {
        return Err(EntropySmokeError::RandomTokenAcceptedFallback);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use aesynx_entropy::{
        EntropyCapabilities, EntropyEvidence, EntropyPolicyStatus, EntropySource,
    };

    use super::{run, verify_random_token_policy};

    #[test]
    fn fallback_cannot_satisfy_random_token_policy() {
        let status = EntropyPolicyStatus::classify(EntropyEvidence {
            capabilities: EntropyCapabilities {
                rdrand: false,
                rdseed: false,
            },
            generation_counter_ok: true,
            hardware_self_test_passed: false,
        });

        assert_eq!(verify_random_token_policy(status), Ok(()));
        assert!(!status.random_tokens_available);
        assert_eq!(
            status.primary_source,
            EntropySource::DeterministicMonotonicFallback
        );
    }

    #[test]
    fn entropy_smoke_reports_redacted_status() {
        let result = run();
        assert!(result.is_ok());

        if let Ok(status) = result {
            assert!(status.generation_counter_ok);
            assert_eq!(
                status.hardware_entropy_present,
                (status.rdrand_supported || status.rdseed_supported)
                    && status.hardware_self_test_passed
            );
            assert!(!status.hardware_self_test_passed);
            assert!(!status.random_tokens_available);
        }
    }
}
