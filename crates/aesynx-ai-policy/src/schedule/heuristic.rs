use aesynx_abi::CoreId;

use crate::{Confidence, DecisionReason, PolicyDecision, PolicyEngine, PolicyError};

use super::{FIXED_POINT_SCALE, ScheduleAdvice, ScheduleFeatures};

pub const DEFAULT_HEURISTIC_THRESHOLD: u32 = 5_000;

const IDLE_RATIO_WEIGHT: u32 = 20;
const RUN_QUEUE_RELIEF_WEIGHT: u32 = 15;
const IPC_RELIEF_WEIGHT: u32 = 15;
const OBJECT_LOCALITY_WEIGHT: u32 = 15;
const CACHE_RELIEF_WEIGHT: u32 = 20;
const PRIORITY_WEIGHT: u32 = 10;
const MIGRATION_RELIEF_WEIGHT: u32 = 5;
const HEURISTIC_WEIGHT_TOTAL: u32 = IDLE_RATIO_WEIGHT
    + RUN_QUEUE_RELIEF_WEIGHT
    + IPC_RELIEF_WEIGHT
    + OBJECT_LOCALITY_WEIGHT
    + CACHE_RELIEF_WEIGHT
    + PRIORITY_WEIGHT
    + MIGRATION_RELIEF_WEIGHT;
const _: () = assert!(HEURISTIC_WEIGHT_TOTAL == 100);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeuristicScheduleScore(u16);

impl HeuristicScheduleScore {
    pub const ZERO: Self = Self(0);

    pub const fn new(value: u32) -> Result<Self, PolicyError> {
        if value > FIXED_POINT_SCALE {
            return Err(PolicyError::FeatureOutOfRange);
        }

        Ok(Self(value as u16))
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }

    #[must_use]
    pub const fn confidence(self) -> Confidence {
        match Confidence::new(self.0) {
            Ok(confidence) => confidence,
            Err(_error) => Confidence::ZERO,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SchedulerPolicyConfig {
    heuristic_enabled: bool,
    fallback_core: CoreId,
    heuristic_core: CoreId,
    minimum_score: HeuristicScheduleScore,
}

impl SchedulerPolicyConfig {
    pub const fn new(
        heuristic_enabled: bool,
        fallback_core: CoreId,
        heuristic_core: CoreId,
        minimum_score: HeuristicScheduleScore,
    ) -> Self {
        Self {
            heuristic_enabled,
            fallback_core,
            heuristic_core,
            minimum_score,
        }
    }

    pub const fn local_round_robin(core: CoreId) -> Self {
        Self {
            heuristic_enabled: false,
            fallback_core: core,
            heuristic_core: core,
            minimum_score: HeuristicScheduleScore::ZERO,
        }
    }

    pub const fn heuristic(fallback_core: CoreId, heuristic_core: CoreId) -> Self {
        Self {
            heuristic_enabled: true,
            fallback_core,
            heuristic_core,
            minimum_score: match HeuristicScheduleScore::new(DEFAULT_HEURISTIC_THRESHOLD) {
                Ok(score) => score,
                Err(_error) => HeuristicScheduleScore::ZERO,
            },
        }
    }

    #[must_use]
    pub const fn heuristic_enabled(self) -> bool {
        self.heuristic_enabled
    }

    #[must_use]
    pub const fn fallback_core(self) -> CoreId {
        self.fallback_core
    }

    #[must_use]
    pub const fn heuristic_core(self) -> CoreId {
        self.heuristic_core
    }

    #[must_use]
    pub const fn minimum_score(self) -> HeuristicScheduleScore {
        self.minimum_score
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SchedulerPolicyDecisionRecord {
    heuristic_enabled: bool,
    score: Option<HeuristicScheduleScore>,
    selected_core: CoreId,
    fallback_core: CoreId,
    fallback_used: bool,
    reason: DecisionReason,
}

impl SchedulerPolicyDecisionRecord {
    #[must_use]
    pub const fn heuristic_enabled(self) -> bool {
        self.heuristic_enabled
    }

    #[must_use]
    pub const fn score(self) -> Option<HeuristicScheduleScore> {
        self.score
    }

    #[must_use]
    pub const fn selected_core(self) -> CoreId {
        self.selected_core
    }

    #[must_use]
    pub const fn fallback_core(self) -> CoreId {
        self.fallback_core
    }

    #[must_use]
    pub const fn fallback_used(self) -> bool {
        self.fallback_used
    }

    #[must_use]
    pub const fn reason(self) -> DecisionReason {
        self.reason
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeuristicSchedulerPolicy {
    config: SchedulerPolicyConfig,
}

impl HeuristicSchedulerPolicy {
    #[must_use]
    pub const fn new(config: SchedulerPolicyConfig) -> Self {
        Self { config }
    }

    #[must_use]
    pub const fn config(self) -> SchedulerPolicyConfig {
        self.config
    }

    #[must_use]
    pub fn evaluate_with_record(
        &self,
        input: ScheduleFeatures,
    ) -> (
        PolicyDecision<ScheduleAdvice>,
        SchedulerPolicyDecisionRecord,
    ) {
        if !self.config.heuristic_enabled {
            let decision = PolicyDecision::deterministic_fallback(self.fallback(input));
            return (decision, self.record(None, &decision));
        }

        let score = match score_features(input) {
            Ok(score) => score,
            Err(_error) => {
                let fallback = self.fallback(input);
                let decision = PolicyDecision::new(
                    fallback,
                    None,
                    Confidence::ZERO,
                    true,
                    DecisionReason::SafetyRejected,
                );
                return (decision, self.record(None, &decision));
            }
        };

        if score.get() < self.config.minimum_score.get() {
            let decision = PolicyDecision::deterministic_fallback(self.fallback(input));
            return (decision, self.record(Some(score), &decision));
        }

        let advice = ScheduleAdvice::new(
            self.config.heuristic_core,
            score.confidence(),
            DecisionReason::Heuristic,
        );
        let decision = PolicyDecision::new(
            advice,
            None,
            score.confidence(),
            false,
            DecisionReason::Heuristic,
        );
        (decision, self.record(Some(score), &decision))
    }

    fn record(
        &self,
        score: Option<HeuristicScheduleScore>,
        decision: &PolicyDecision<ScheduleAdvice>,
    ) -> SchedulerPolicyDecisionRecord {
        SchedulerPolicyDecisionRecord {
            heuristic_enabled: self.config.heuristic_enabled,
            score,
            selected_core: decision.output().target_core(),
            fallback_core: self.config.fallback_core,
            fallback_used: decision.fallback_used(),
            reason: decision.reason(),
        }
    }
}

impl PolicyEngine for HeuristicSchedulerPolicy {
    type Input = ScheduleFeatures;
    type Output = ScheduleAdvice;

    fn evaluate(&self, input: Self::Input) -> PolicyDecision<Self::Output> {
        self.evaluate_with_record(input).0
    }

    fn fallback(&self, _input: Self::Input) -> Self::Output {
        ScheduleAdvice::new(
            self.config.fallback_core,
            Confidence::ZERO,
            DecisionReason::DeterministicFallback,
        )
    }
}

pub fn score_features(features: ScheduleFeatures) -> Result<HeuristicScheduleScore, PolicyError> {
    let features = features.validate()?;
    let load_relief = FIXED_POINT_SCALE - features.run_queue_len;
    let ipc_pressure = max_u32(features.ipc_depth, features.queue_pressure);
    let ipc_relief = FIXED_POINT_SCALE - ipc_pressure;
    let cache_relief = FIXED_POINT_SCALE - features.cache_miss_rate;
    let migration_relief = FIXED_POINT_SCALE - features.migration_cost;
    let priority_score = (features.priority as u32 * FIXED_POINT_SCALE) / u8::MAX as u32;
    let weighted = features
        .idle_ratio
        .saturating_mul(IDLE_RATIO_WEIGHT)
        .saturating_add(load_relief.saturating_mul(RUN_QUEUE_RELIEF_WEIGHT))
        .saturating_add(ipc_relief.saturating_mul(IPC_RELIEF_WEIGHT))
        .saturating_add(
            features
                .object_locality_score
                .saturating_mul(OBJECT_LOCALITY_WEIGHT),
        )
        .saturating_add(cache_relief.saturating_mul(CACHE_RELIEF_WEIGHT))
        .saturating_add(priority_score.saturating_mul(PRIORITY_WEIGHT))
        .saturating_add(migration_relief.saturating_mul(MIGRATION_RELIEF_WEIGHT));
    HeuristicScheduleScore::new(weighted / HEURISTIC_WEIGHT_TOTAL)
}

const fn max_u32(left: u32, right: u32) -> u32 {
    if left >= right { left } else { right }
}
