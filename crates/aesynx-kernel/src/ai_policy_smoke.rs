use aesynx_abi::{CoreId, ModelId};
use aesynx_ai_policy::{
    DecisionReason, Hash256, HeuristicSchedulerPolicy, MODEL_MANIFEST_SCHEMA_VERSION, ModelKind,
    ModelObjectManifest, ModelSafetyLimits, PolicyDomain, PolicyEngine, PolicyError,
    ScheduleFeatures, SchedulerPolicyConfig, Signature64,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AiPolicySmokeStatus {
    pub schema_version: u16,
    pub accepted_manifest: bool,
    pub rejected_manifest: bool,
    pub fallback_used: bool,
    pub fallback_confidence: u16,
    pub fallback_core: u32,
    pub manifest_metadata_gate_ok: bool,
    pub heuristic_enabled: bool,
    pub heuristic_score_recorded: bool,
    pub heuristic_core_selected: bool,
    pub heuristic_disabled_fallback_ok: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiPolicySmokeError {
    Hash(PolicyError),
    Signature(PolicyError),
    SafeManifestRejected(PolicyError),
    UnsafeManifestAccepted,
    FallbackFailed,
    HeuristicFailed,
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

    let fallback_policy =
        HeuristicSchedulerPolicy::new(SchedulerPolicyConfig::local_round_robin(CoreId::new(0)));
    let decision = fallback_policy.evaluate(ScheduleFeatures::default());
    let fallback_ok = decision.fallback_used()
        && decision.model().is_none()
        && decision.confidence().get() == 0
        && decision.output().target_core() == CoreId::new(0)
        && fallback_policy.explain(&decision) == DecisionReason::DeterministicFallback;
    if !fallback_ok {
        return Err(AiPolicySmokeError::FallbackFailed);
    }

    let heuristic_policy = HeuristicSchedulerPolicy::new(SchedulerPolicyConfig::heuristic(
        CoreId::new(0),
        CoreId::new(1),
    ));
    let (heuristic_decision, heuristic_record) =
        heuristic_policy.evaluate_with_record(heuristic_features());
    let Some(heuristic_score) = heuristic_record.score() else {
        return Err(AiPolicySmokeError::HeuristicFailed);
    };
    let heuristic_ok = !heuristic_decision.fallback_used()
        && heuristic_decision.model().is_none()
        && heuristic_decision.output().target_core() == CoreId::new(1)
        && heuristic_decision.reason() == DecisionReason::Heuristic
        && heuristic_record.heuristic_enabled()
        && heuristic_score.get() >= 7_000
        && heuristic_record.selected_core() == CoreId::new(1)
        && heuristic_record.fallback_core() == CoreId::new(0);
    if !heuristic_ok {
        return Err(AiPolicySmokeError::HeuristicFailed);
    }

    Ok(AiPolicySmokeStatus {
        schema_version: accepted_manifest.schema_version,
        accepted_manifest: true,
        rejected_manifest,
        fallback_used: decision.fallback_used(),
        fallback_confidence: decision.confidence().get(),
        fallback_core: decision.output().target_core().get(),
        manifest_metadata_gate_ok: rejected_manifest && fallback_ok,
        heuristic_enabled: heuristic_record.heuristic_enabled(),
        heuristic_score_recorded: heuristic_record.score().is_some(),
        heuristic_core_selected: heuristic_decision.output().target_core() == CoreId::new(1),
        heuristic_disabled_fallback_ok: fallback_ok,
    })
}

const fn heuristic_features() -> ScheduleFeatures {
    ScheduleFeatures {
        run_queue_len: 500,
        ipc_depth: 500,
        queue_pressure: 500,
        object_locality_score: 9_000,
        cache_miss_rate: 0,
        idle_ratio: 9_000,
        migration_cost: 100,
        priority: u8::MAX,
    }
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
        assert!(status.manifest_metadata_gate_ok);
        assert!(status.heuristic_enabled);
        assert!(status.heuristic_score_recorded);
        assert!(status.heuristic_core_selected);
        assert!(status.heuristic_disabled_fallback_ok);
    }
}
