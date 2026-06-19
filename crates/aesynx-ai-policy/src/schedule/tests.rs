use core::fmt::{self, Write};

use aesynx_abi::{CoreId, ModelId};

use super::{
    HeuristicScheduleScore, HeuristicSchedulerPolicy, ScheduleAdvice, ScheduleFeatures,
    SchedulerPolicyConfig,
};
use crate::{
    Confidence, DecisionReason, FIXED_POINT_SCALE, Hash256, MODEL_MANIFEST_SCHEMA_VERSION,
    ModelKind, ModelObjectManifest, ModelSafetyLimits, PolicyDecision, PolicyDomain, PolicyEngine,
    PolicyError, RequiredTelemetryFields, Signature64,
};

struct DenySchedulerPolicy;

struct DebugBuffer {
    bytes: [u8; 1024],
    len: usize,
}

impl DebugBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; 1024],
            len: 0,
        }
    }

    fn contains(&self, needle: &[u8]) -> bool {
        if needle.is_empty() || needle.len() > self.len {
            return false;
        }
        self.bytes[..self.len]
            .windows(needle.len())
            .any(|window| window == needle)
    }
}

impl Write for DebugBuffer {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let end = match self.len.checked_add(text.len()) {
            Some(end) if end <= self.bytes.len() => end,
            _ => return Err(fmt::Error),
        };
        self.bytes[self.len..end].copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}

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
fn model_decision_enforces_manifest_confidence_ceiling() {
    let confidence = match Confidence::new(1) {
        Ok(confidence) => confidence,
        Err(error) => return assert_eq!(error, PolicyError::ConfidenceOutOfRange),
    };
    let decision = PolicyDecision::from_model(
        7u8,
        3u8,
        ModelId::new(42),
        confidence,
        &ModelSafetyLimits::scheduler_default(),
        DecisionReason::ModelAdvice,
    );

    assert_eq!(*decision.output(), 3);
    assert_eq!(decision.model(), None);
    assert_eq!(decision.confidence(), Confidence::ZERO);
    assert!(decision.fallback_used());
    assert_eq!(decision.reason(), DecisionReason::SafetyRejected);
}

#[test]
fn model_decision_accepts_confidence_within_manifest_ceiling() {
    let confidence = match Confidence::new(7) {
        Ok(confidence) => confidence,
        Err(error) => return assert_eq!(error, PolicyError::ConfidenceOutOfRange),
    };
    let limits = ModelSafetyLimits::new(
        10_000,
        64 * 1024,
        confidence,
        true,
        RequiredTelemetryFields::scheduler_minimum(),
    );
    let decision = PolicyDecision::from_model(
        7u8,
        3u8,
        ModelId::new(42),
        confidence,
        &limits,
        DecisionReason::ModelAdvice,
    );

    assert_eq!(*decision.output(), 7);
    assert_eq!(decision.model(), Some(ModelId::new(42)));
    assert_eq!(decision.confidence(), confidence);
    assert!(!decision.fallback_used());
    assert_eq!(decision.reason(), DecisionReason::ModelAdvice);
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

    let invalid_raw_count = ScheduleFeatures {
        run_queue_len: FIXED_POINT_SCALE + 1,
        ..ScheduleFeatures::default()
    };
    assert_eq!(
        invalid_raw_count.validate(),
        Err(PolicyError::FeatureOutOfRange)
    );

    let invalid_migration_cost = ScheduleFeatures {
        migration_cost: FIXED_POINT_SCALE + 1,
        ..ScheduleFeatures::default()
    };
    assert_eq!(
        invalid_migration_cost.validate(),
        Err(PolicyError::FeatureOutOfRange)
    );
}

#[test]
fn heuristic_scheduler_scores_bounded_features() {
    let score = match super::score_features(ScheduleFeatures {
        run_queue_len: 1_000,
        ipc_depth: 1_000,
        queue_pressure: 2_000,
        object_locality_score: 9_000,
        idle_ratio: 8_000,
        migration_cost: 500,
        priority: 200,
        cache_miss_rate: 0,
    }) {
        Ok(score) => score,
        Err(error) => return assert_eq!(error, PolicyError::FeatureOutOfRange),
    };

    assert!(score.get() <= FIXED_POINT_SCALE as u16);
    assert!(score.get() >= 7_000);
}

#[test]
fn heuristic_scheduler_penalizes_cache_miss_pressure() {
    let base = ScheduleFeatures {
        run_queue_len: 1_000,
        ipc_depth: 1_000,
        queue_pressure: 1_000,
        object_locality_score: 9_000,
        idle_ratio: 9_000,
        migration_cost: 500,
        priority: 200,
        cache_miss_rate: 0,
    };
    let low_cache_miss_score = match super::score_features(base) {
        Ok(score) => score,
        Err(error) => return assert_eq!(error, PolicyError::FeatureOutOfRange),
    };
    let high_cache_miss_score = match super::score_features(ScheduleFeatures {
        cache_miss_rate: 9_000,
        ..base
    }) {
        Ok(score) => score,
        Err(error) => return assert_eq!(error, PolicyError::FeatureOutOfRange),
    };

    assert!(high_cache_miss_score.get() < low_cache_miss_score.get());
}

#[test]
fn heuristic_scheduler_can_be_disabled_for_round_robin_fallback() {
    let policy =
        HeuristicSchedulerPolicy::new(SchedulerPolicyConfig::local_round_robin(CoreId::new(0)));
    let (decision, record) = policy.evaluate_with_record(ScheduleFeatures {
        run_queue_len: 0,
        ipc_depth: 0,
        queue_pressure: 0,
        object_locality_score: FIXED_POINT_SCALE,
        idle_ratio: FIXED_POINT_SCALE,
        migration_cost: 0,
        priority: u8::MAX,
        cache_miss_rate: 0,
    });

    assert_eq!(decision.output().target_core(), CoreId::new(0));
    assert_eq!(decision.reason(), DecisionReason::DeterministicFallback);
    assert!(decision.fallback_used());
    assert!(!record.heuristic_enabled());
    assert_eq!(record.score(), None);
    assert_eq!(record.selected_core(), CoreId::new(0));
}

#[test]
fn heuristic_scheduler_records_heuristic_decisions() {
    let policy = HeuristicSchedulerPolicy::new(SchedulerPolicyConfig::heuristic(
        CoreId::new(0),
        CoreId::new(1),
    ));
    let (decision, record) = policy.evaluate_with_record(ScheduleFeatures {
        run_queue_len: 500,
        ipc_depth: 500,
        queue_pressure: 500,
        object_locality_score: 9_000,
        idle_ratio: 9_000,
        migration_cost: 100,
        priority: u8::MAX,
        cache_miss_rate: 0,
    });

    assert_eq!(decision.output().target_core(), CoreId::new(1));
    assert_eq!(decision.reason(), DecisionReason::Heuristic);
    assert!(!decision.fallback_used());
    assert!(record.heuristic_enabled());
    assert!(record.score().is_some());
    assert_eq!(record.selected_core(), CoreId::new(1));
    assert_eq!(record.fallback_core(), CoreId::new(0));
    assert_eq!(record.reason(), DecisionReason::Heuristic);
}

#[test]
fn heuristic_scheduler_fails_closed_on_invalid_features() {
    let policy = HeuristicSchedulerPolicy::new(SchedulerPolicyConfig::heuristic(
        CoreId::new(0),
        CoreId::new(1),
    ));
    let (decision, record) = policy.evaluate_with_record(ScheduleFeatures {
        run_queue_len: FIXED_POINT_SCALE + 1,
        ..ScheduleFeatures::default()
    });

    assert_eq!(decision.output().target_core(), CoreId::new(0));
    assert_eq!(decision.reason(), DecisionReason::SafetyRejected);
    assert!(decision.fallback_used());
    assert_eq!(record.score(), None);
    assert_eq!(record.reason(), DecisionReason::SafetyRejected);
}

#[test]
fn heuristic_score_rejects_out_of_range_values() {
    assert_eq!(
        HeuristicScheduleScore::new(FIXED_POINT_SCALE + 1),
        Err(PolicyError::FeatureOutOfRange)
    );
}

#[test]
fn model_manifest_debug_redacts_hashes_signature_and_identity() {
    let manifest = match scheduler_manifest() {
        Ok(manifest) => manifest,
        Err(error) => return assert_eq!(error, PolicyError::EmptyHash),
    };
    let mut rendered = DebugBuffer::new();
    assert!(core::write!(&mut rendered, "{manifest:?}").is_ok());

    assert!(rendered.contains(b"ModelObjectManifest"));
    assert!(rendered.contains(b"id: \"<redacted>\""));
    assert!(rendered.contains(b"Hash256(<redacted>)"));
    assert!(rendered.contains(b"Signature64(<redacted>)"));
    assert!(!rendered.contains(b"[1, 1"));
    assert!(!rendered.contains(b"[2, 2"));
    assert!(!rendered.contains(b"[3, 3"));
    assert!(!rendered.contains(b"[7, 7"));
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
        manifest.validate_for_domain(PolicyDomain::Capability).err(),
        Some(PolicyError::UnsupportedDomain)
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
        manifest.validate_for_domain(PolicyDomain::Scheduler).err(),
        Some(PolicyError::FallbackRequired)
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
        manifest.validate_for_domain(PolicyDomain::Scheduler).err(),
        Some(PolicyError::FeatureOutOfRange)
    );
}

#[test]
fn model_manifest_rejects_unbacked_model_kinds() {
    let mut manifest = match scheduler_manifest() {
        Ok(manifest) => manifest,
        Err(error) => return assert_eq!(error, PolicyError::EmptyHash),
    };

    manifest.kind = ModelKind::NeuralNetwork;
    assert_eq!(
        manifest.validate_for_domain(PolicyDomain::Scheduler).err(),
        Some(PolicyError::UnsupportedModelKind)
    );

    manifest.kind = ModelKind::WasmComponent;
    assert_eq!(
        manifest.validate_for_domain(PolicyDomain::Scheduler).err(),
        Some(PolicyError::UnsupportedModelKind)
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
