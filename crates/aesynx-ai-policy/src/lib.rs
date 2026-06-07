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
    pub output: T,
    pub model: Option<ModelId>,
    pub confidence: u16,
    pub fallback_used: bool,
    pub reason: DecisionReason,
}

impl<T> PolicyDecision<T> {
    #[must_use]
    pub const fn confidence_is_valid(&self) -> bool {
        self.confidence <= MAX_CONFIDENCE
    }
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
    pub target_core: CoreId,
    pub confidence: u16,
    pub reason: DecisionReason,
}

impl ScheduleAdvice {
    #[must_use]
    pub const fn confidence_is_valid(self) -> bool {
        self.confidence <= MAX_CONFIDENCE
    }
}
