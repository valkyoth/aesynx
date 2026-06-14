use aesynx_abi::ModelId;

use crate::{Confidence, ModelSafetyLimits};

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
    /// Constructs a raw policy decision.
    ///
    /// # Security
    ///
    /// This constructor does not enforce a model manifest's confidence ceiling.
    /// Model-backed decisions should use [`Self::from_model`] so
    /// `ModelSafetyLimits::max_confidence` is applied at the trust boundary.
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

    /// Constructs a model-backed policy decision with manifest confidence
    /// enforcement.
    ///
    /// If `confidence` exceeds `limits.max_confidence`, this returns the
    /// provided deterministic `fallback`, clears the model identity, reports
    /// zero confidence, and marks the reason as [`DecisionReason::SafetyRejected`].
    #[must_use]
    pub fn from_model(
        output: T,
        fallback: T,
        model: ModelId,
        confidence: Confidence,
        limits: &ModelSafetyLimits,
        reason: DecisionReason,
    ) -> Self {
        if confidence.get() > limits.max_confidence.get() {
            return Self {
                output: fallback,
                model: None,
                confidence: Confidence::ZERO,
                fallback_used: true,
                reason: DecisionReason::SafetyRejected,
            };
        }

        Self {
            output,
            model: Some(model),
            confidence,
            fallback_used: false,
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
