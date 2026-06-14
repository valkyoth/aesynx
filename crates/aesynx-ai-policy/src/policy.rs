use aesynx_abi::ModelId;

use crate::Confidence;

/// Policy engines may use heuristics or model advice, but their fallback path
/// is part of the security contract.
///
/// `fallback` must be deterministic, side-effect-free, and fail closed for the
/// policy domain. For scheduling this means returning a conservative local
/// decision; for authority or admission policy this means denying or selecting
/// the least-privileged outcome.
pub trait PolicyEngine {
    type Input;
    type Output;

    fn evaluate(&self, input: Self::Input) -> PolicyDecision<Self::Output>;

    /// Returns the deterministic fail-closed output for this policy.
    fn fallback(&self, input: Self::Input) -> Self::Output;

    fn explain(&self, decision: &PolicyDecision<Self::Output>) -> DecisionReason {
        decision.reason()
    }
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
    pub const fn deterministic_fallback(output: T) -> Self {
        Self {
            output,
            model: None,
            confidence: Confidence::ZERO,
            fallback_used: true,
            reason: DecisionReason::DeterministicFallback,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionReason {
    DeterministicFallback,
    Heuristic,
    ModelAdvice,
    SafetyRejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyError {
    ConfidenceOutOfRange,
    EmptyHash,
    EmptyModel,
    EmptySignature,
    FeatureOutOfRange,
    FallbackRequired,
    ResourceLimitExceeded,
    UnsupportedDomain,
    UnsupportedModelKind,
    UnsupportedSchema,
}
