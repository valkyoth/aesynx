#![no_std]
#![deny(unsafe_code)]

use aesynx_abi::{CoreId, ModelId};

pub const MAX_CONFIDENCE: u16 = 10_000;

pub trait PolicyEngine {
    type Input;
    type Output;

    fn evaluate(&self, input: Self::Input) -> PolicyDecision<Self::Output>;
    fn fallback(&self, input: Self::Input) -> Self::Output;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PolicyDecision<T> {
    output: T,
    model: Option<ModelId>,
    confidence: Confidence,
    fallback_used: bool,
    reason: DecisionReason,
}

impl<T> PolicyDecision<T> {
    pub const fn new(
        output: T,
        model: Option<ModelId>,
        confidence: Confidence,
        fallback_used: bool,
        reason: DecisionReason,
    ) -> Self {
        Self {
            output,
            model,
            confidence,
            fallback_used,
            reason,
        }
    }

    #[must_use]
    pub const fn output(&self) -> &T {
        &self.output
    }

    #[must_use]
    pub const fn model(&self) -> Option<ModelId> {
        self.model
    }

    #[must_use]
    pub const fn confidence(&self) -> Confidence {
        self.confidence
    }

    #[must_use]
    pub const fn fallback_used(&self) -> bool {
        self.fallback_used
    }

    #[must_use]
    pub const fn reason(&self) -> DecisionReason {
        self.reason
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Confidence(u16);

impl Confidence {
    pub const fn new(value: u16) -> Result<Self, PolicyError> {
        if value > MAX_CONFIDENCE {
            return Err(PolicyError::ConfidenceOutOfRange);
        }

        Ok(Self(value))
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyError {
    ConfidenceOutOfRange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionReason {
    DeterministicFallback,
    Heuristic,
    ModelAdvice,
    SafetyRejected,
}

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
    use aesynx_abi::CoreId;

    use super::{
        Confidence, DecisionReason, MAX_CONFIDENCE, PolicyDecision, PolicyError, ScheduleAdvice,
    };

    #[test]
    fn confidence_rejects_out_of_range_values() {
        assert_eq!(
            Confidence::new(MAX_CONFIDENCE + 1),
            Err(PolicyError::ConfidenceOutOfRange)
        );
    }

    #[test]
    fn policy_decision_uses_bounded_confidence() {
        let confidence = match Confidence::new(MAX_CONFIDENCE) {
            Ok(confidence) => confidence,
            Err(error) => return assert_eq!(error, PolicyError::ConfidenceOutOfRange),
        };
        let decision = PolicyDecision::new(7u8, None, confidence, false, DecisionReason::Heuristic);

        assert_eq!(*decision.output(), 7);
        assert_eq!(decision.confidence().get(), MAX_CONFIDENCE);
        assert!(!decision.fallback_used());
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
}
