#![no_std]
#![deny(unsafe_code)]

use aesynx_abi::{CoreId, ModelId};

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionReason {
    DeterministicFallback,
    Heuristic,
    ModelAdvice,
    SafetyRejected,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ScheduleFeatures {
    pub run_queue_len: i32,
    pub ipc_depth: i32,
    pub queue_pressure: i32,
    pub object_locality_score: i32,
    pub cache_miss_rate: i32,
    pub idle_ratio: i32,
    pub migration_cost: i32,
    pub priority: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScheduleAdvice {
    pub target_core: CoreId,
    pub confidence: u16,
    pub reason: DecisionReason,
}
