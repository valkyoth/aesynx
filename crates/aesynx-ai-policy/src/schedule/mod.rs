use aesynx_abi::CoreId;

use crate::{Confidence, DecisionReason, PolicyError};

mod heuristic;

pub use heuristic::{
    DEFAULT_HEURISTIC_THRESHOLD, HeuristicScheduleScore, HeuristicSchedulerPolicy,
    SchedulerPolicyConfig, SchedulerPolicyDecisionRecord,
};

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
    /// Discrete scheduler priority. This is a compact rank, not a
    /// `FIXED_POINT_SCALE` ratio.
    pub priority: u8,
}

impl ScheduleFeatures {
    pub const fn validate(self) -> Result<Self, PolicyError> {
        if self.run_queue_len > FIXED_POINT_SCALE
            || self.ipc_depth > FIXED_POINT_SCALE
            || self.queue_pressure > FIXED_POINT_SCALE
            || self.object_locality_score > FIXED_POINT_SCALE
            || self.cache_miss_rate > FIXED_POINT_SCALE
            || self.idle_ratio > FIXED_POINT_SCALE
            || self.migration_cost > FIXED_POINT_SCALE
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

pub fn score_features(features: ScheduleFeatures) -> Result<HeuristicScheduleScore, PolicyError> {
    heuristic::score_features(features)
}

#[cfg(test)]
mod tests;
