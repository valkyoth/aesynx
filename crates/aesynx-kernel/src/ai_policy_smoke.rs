use aesynx_abi::{CoreId, ModelId};
use aesynx_ai_policy::{
    Confidence, DecisionReason, Hash256, MODEL_MANIFEST_SCHEMA_VERSION, ModelKind,
    ModelObjectManifest, ModelSafetyLimits, PolicyDecision, PolicyDomain, PolicyEngine,
    PolicyError, ScheduleAdvice, ScheduleFeatures, Signature64,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AiPolicySmokeStatus {
    pub schema_version: u16,
    pub accepted_manifest: bool,
    pub rejected_manifest: bool,
    pub fallback_used: bool,
    pub fallback_confidence: u16,
    pub fallback_core: u32,
    pub safety_gate_ok: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiPolicySmokeError {
    Hash(PolicyError),
    Signature(PolicyError),
    SafeManifestRejected(PolicyError),
    UnsafeManifestAccepted,
    FallbackFailed,
}

struct LocalFallbackScheduler;

impl PolicyEngine for LocalFallbackScheduler {
    type Input = ScheduleFeatures;
    type Output = ScheduleAdvice;

    fn evaluate(&self, input: Self::Input) -> PolicyDecision<Self::Output> {
        PolicyDecision::deterministic_fallback(self.fallback(input))
    }

    fn fallback(&self, _input: Self::Input) -> Self::Output {
        ScheduleAdvice::new(
            CoreId::new(0),
            Confidence::ZERO,
            DecisionReason::DeterministicFallback,
        )
    }
}

pub fn run() -> Result<AiPolicySmokeStatus, AiPolicySmokeError> {
    let safe_manifest = scheduler_manifest()?;
    let accepted_manifest = safe_manifest
        .validate_for_domain(PolicyDomain::Scheduler)
        .map_err(AiPolicySmokeError::SafeManifestRejected)?
        .manifest();

    let mut unsafe_manifest = safe_manifest;
    unsafe_manifest.safety_limits.fallback_required = false;
    let rejected_manifest = matches!(
        unsafe_manifest.validate_for_domain(PolicyDomain::Scheduler),
        Err(PolicyError::FallbackRequired)
    );
    if !rejected_manifest {
        return Err(AiPolicySmokeError::UnsafeManifestAccepted);
    }

    let policy = LocalFallbackScheduler;
    let decision = policy.evaluate(ScheduleFeatures::default());
    let fallback_ok = decision.fallback_used()
        && decision.model().is_none()
        && decision.confidence().get() == 0
        && decision.output().target_core() == CoreId::new(0)
        && policy.explain(&decision) == DecisionReason::DeterministicFallback;
    if !fallback_ok {
        return Err(AiPolicySmokeError::FallbackFailed);
    }

    Ok(AiPolicySmokeStatus {
        schema_version: accepted_manifest.schema_version,
        accepted_manifest: true,
        rejected_manifest,
        fallback_used: decision.fallback_used(),
        fallback_confidence: decision.confidence().get(),
        fallback_core: decision.output().target_core().get(),
        safety_gate_ok: rejected_manifest && fallback_ok,
    })
}

fn scheduler_manifest() -> Result<ModelObjectManifest, AiPolicySmokeError> {
    Ok(ModelObjectManifest {
        id: ModelId::new(1),
        schema_version: MODEL_MANIFEST_SCHEMA_VERSION,
        kind: ModelKind::FixedPointHeuristic,
        domain: PolicyDomain::Scheduler,
        input_schema_hash: hash(1)?,
        output_schema_hash: hash(2)?,
        weights_hash: hash(3)?,
        signature: signature()?,
        safety_limits: ModelSafetyLimits::scheduler_default(),
    })
}

fn hash(byte: u8) -> Result<Hash256, AiPolicySmokeError> {
    Hash256::new([byte; 32]).map_err(AiPolicySmokeError::Hash)
}

fn signature() -> Result<Signature64, AiPolicySmokeError> {
    Signature64::new([7; 64]).map_err(AiPolicySmokeError::Signature)
}

#[cfg(test)]
mod tests {
    use super::run;

    #[test]
    fn ai_policy_smoke_accepts_rejects_and_falls_back() {
        let status = match run() {
            Ok(status) => status,
            Err(error) => return assert_eq!(format!("{error:?}"), ""),
        };

        assert_eq!(status.schema_version, 1);
        assert!(status.accepted_manifest);
        assert!(status.rejected_manifest);
        assert!(status.fallback_used);
        assert_eq!(status.fallback_confidence, 0);
        assert_eq!(status.fallback_core, 0);
        assert!(status.safety_gate_ok);
    }
}
