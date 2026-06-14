use aesynx_abi::CoreId;

use crate::{Confidence, DecisionReason, PolicyError};

pub const FIXED_POINT_SCALE: u32 = 10_000;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ScheduleFeatures {
    pub run_queue_len: u32,
    pub ipc_depth: u32,
    pub queue_pressure: u32,
    pub object_locality_score: u32,
    pub cache_miss_rate: u32,
    pub idle_ratio: u32,
    pub migration_cost: u32,
    pub priority: u8,
}

impl ScheduleFeatures {
    pub const fn validate(self) -> Result<Self, PolicyError> {
        if self.object_locality_score > FIXED_POINT_SCALE
            || self.cache_miss_rate > FIXED_POINT_SCALE
            || self.idle_ratio > FIXED_POINT_SCALE
        {
            return Err(PolicyError::FeatureOutOfRange);
        }
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScheduleAdvice {
    target_core: CoreId,
    confidence: Confidence,
    reason: DecisionReason,
}

impl ScheduleAdvice {
    #[must_use]
    pub const fn new(target_core: CoreId, confidence: Confidence, reason: DecisionReason) -> Self {
        Self {
            target_core,
            confidence,
            reason,
        }
    }

    #[must_use]
    pub const fn target_core(self) -> CoreId {
        self.target_core
    }

    #[must_use]
    pub const fn confidence(self) -> Confidence {
        self.confidence
    }

    #[must_use]
    pub const fn reason(self) -> DecisionReason {
        self.reason
    }
}

#[cfg(test)]
mod tests {
    use aesynx_abi::{CoreId, ModelId};

    use super::ScheduleFeatures;
    use crate::{
        Confidence, DecisionReason, FIXED_POINT_SCALE, Hash256, MODEL_MANIFEST_SCHEMA_VERSION,
        ModelKind, ModelObjectManifest, ModelSafetyLimits, PolicyDecision, PolicyDomain,
        PolicyEngine, PolicyError, RequiredTelemetryFields, ScheduleAdvice, Signature64,
    };

    struct DenySchedulerPolicy;

    impl PolicyEngine for DenySchedulerPolicy {
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

    fn hash(byte: u8) -> Result<Hash256, PolicyError> {
        Hash256::new([byte; 32])
    }

    fn signature() -> Result<Signature64, PolicyError> {
        Signature64::new([7; 64])
    }

    fn scheduler_manifest() -> Result<ModelObjectManifest, PolicyError> {
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

    #[test]
    fn confidence_rejects_out_of_range_values() {
        assert_eq!(
            Confidence::new(crate::MAX_CONFIDENCE + 1),
            Err(PolicyError::ConfidenceOutOfRange)
        );
    }

    #[test]
    fn policy_decision_uses_bounded_confidence() {
        let confidence = match Confidence::new(crate::MAX_CONFIDENCE) {
            Ok(confidence) => confidence,
            Err(error) => return assert_eq!(error, PolicyError::ConfidenceOutOfRange),
        };
        let decision = PolicyDecision::new(7u8, None, confidence, false, DecisionReason::Heuristic);

        assert_eq!(*decision.output(), 7);
        assert_eq!(decision.confidence().get(), crate::MAX_CONFIDENCE);
        assert!(!decision.fallback_used());
    }

    #[test]
    fn deterministic_fallback_decision_is_marked_fail_closed() {
        let decision = PolicyDecision::deterministic_fallback(3u8);

        assert_eq!(*decision.output(), 3);
        assert_eq!(decision.model(), None);
        assert_eq!(decision.confidence().get(), 0);
        assert!(decision.fallback_used());
        assert_eq!(decision.reason(), DecisionReason::DeterministicFallback);
    }

    #[test]
    fn schedule_advice_uses_bounded_confidence() {
        let confidence = match Confidence::new(1) {
            Ok(confidence) => confidence,
            Err(error) => return assert_eq!(error, PolicyError::ConfidenceOutOfRange),
        };
        let advice = ScheduleAdvice::new(CoreId::new(2), confidence, DecisionReason::ModelAdvice);

        assert_eq!(advice.target_core(), CoreId::new(2));
        assert_eq!(advice.confidence().get(), 1);
    }

    #[test]
    fn schedule_features_use_fixed_point_bounds() {
        assert!(ScheduleFeatures::default().validate().is_ok());

        let invalid = ScheduleFeatures {
            idle_ratio: FIXED_POINT_SCALE + 1,
            ..ScheduleFeatures::default()
        };
        assert_eq!(invalid.validate(), Err(PolicyError::FeatureOutOfRange));
    }

    #[test]
    fn model_manifest_accepts_scheduler_safe_manifest() {
        let manifest = match scheduler_manifest() {
            Ok(manifest) => manifest,
            Err(error) => return assert_eq!(error, PolicyError::EmptyHash),
        };
        let validated = match manifest.validate_for_domain(PolicyDomain::Scheduler) {
            Ok(validated) => validated,
            Err(error) => return assert_eq!(error, PolicyError::UnsupportedSchema),
        };

        assert_eq!(validated.manifest().id, ModelId::new(1));
    }

    #[test]
    fn model_manifest_rejects_wrong_domain() {
        let manifest = match scheduler_manifest() {
            Ok(manifest) => manifest,
            Err(error) => return assert_eq!(error, PolicyError::EmptyHash),
        };

        assert_eq!(
            manifest.validate_for_domain(PolicyDomain::Capability),
            Err(PolicyError::UnsupportedDomain)
        );
    }

    #[test]
    fn model_manifest_rejects_missing_fallback() {
        let mut manifest = match scheduler_manifest() {
            Ok(manifest) => manifest,
            Err(error) => return assert_eq!(error, PolicyError::EmptyHash),
        };
        manifest.safety_limits.fallback_required = false;

        assert_eq!(
            manifest.validate_for_domain(PolicyDomain::Scheduler),
            Err(PolicyError::FallbackRequired)
        );
    }

    #[test]
    fn model_manifest_rejects_missing_scheduler_features() {
        let mut manifest = match scheduler_manifest() {
            Ok(manifest) => manifest,
            Err(error) => return assert_eq!(error, PolicyError::EmptyHash),
        };
        manifest.safety_limits.required_telemetry =
            RequiredTelemetryFields::RUN_QUEUE_LEN.union(RequiredTelemetryFields::IPC_DEPTH);

        assert_eq!(
            manifest.validate_for_domain(PolicyDomain::Scheduler),
            Err(PolicyError::FeatureOutOfRange)
        );
    }

    #[test]
    fn deterministic_scheduler_fallback_always_works_without_model_identity() {
        let policy = DenySchedulerPolicy;
        let decision = policy.evaluate(ScheduleFeatures::default());

        assert_eq!(decision.output().target_core(), CoreId::new(0));
        assert_eq!(decision.model(), None);
        assert_eq!(decision.confidence().get(), 0);
        assert!(decision.fallback_used());
        assert_eq!(
            policy.explain(&decision),
            DecisionReason::DeterministicFallback
        );
    }
}
