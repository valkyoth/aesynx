#![no_std]
#![forbid(unsafe_code)]

mod barrier;
mod capabilities;
mod local;
mod registry;
mod role;

pub use barrier::{BootBarrier, BootBarrierStatus};
pub use capabilities::{CoreCapabilitySet, CoreIsa, CorePerformanceClass};
pub use local::{CoreLocal, CoreLocalTelemetry, CoreState};
pub use registry::{CoreRegistry, CoreRegistryStatus};
pub use role::CoreRole;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreError {
    AlreadyArrived,
    BarrierNotSealed,
    BarrierSealed,
    CapacityZero,
    DuplicateCore,
    RegistryFull,
    RoleMismatch,
    TelemetryOverflow,
    UnknownCore,
}

#[cfg(test)]
mod tests;
