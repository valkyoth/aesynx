#![no_std]
#![forbid(unsafe_code)]

mod confidence;
mod manifest;
mod policy;
mod schedule;

pub use confidence::{Confidence, MAX_CONFIDENCE};
pub use manifest::{
    Hash256, MODEL_MANIFEST_SCHEMA_VERSION, ModelKind, ModelObjectManifest, ModelSafetyLimits,
    PolicyDomain, RequiredTelemetryFields, Signature64, ValidatedModelManifest,
};
pub use policy::{DecisionReason, PolicyDecision, PolicyEngine, PolicyError};
pub use schedule::{FIXED_POINT_SCALE, ScheduleAdvice, ScheduleFeatures};
