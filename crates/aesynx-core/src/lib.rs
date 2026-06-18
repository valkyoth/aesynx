#![no_std]
#![forbid(unsafe_code)]

mod barrier;
mod capabilities;
mod local;
mod registry;
mod role;
mod startup;
mod startup_preflight;
mod topology;

pub use barrier::{BootBarrier, BootBarrierStatus};
pub use capabilities::{CoreCapabilitySet, CoreIsa, CorePerformanceClass};
pub use local::{CoreLocal, CoreLocalTelemetry, CoreState};
pub use registry::{CoreRegistry, CoreRegistryStatus};
pub use role::CoreRole;
pub use startup::{CoreStartupArrival, CoreStartupTicket};
pub use startup_preflight::{
    ApDescriptorTableReadiness, ApStartupPreflight, ApStartupPreflightStatus, ApStartupResource,
    MIN_AP_STACK_BYTES,
};
pub use topology::{
    CoreAssignmentState, CoreHardwareState, CoreTopology, CoreTopologyEntry, CoreTopologyStatus,
    QEMU_MULTICORE_TOPOLOGY_CORES,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreError {
    AlreadyArrived,
    BarrierNotSealed,
    BarrierSealed,
    CapacityZero,
    DuplicateCore,
    DuplicateHardwareId,
    DuplicateStartupStack,
    InvalidStateTransition,
    InvalidStartupStack,
    MissingStartupWatchdog,
    OwnerMismatch,
    RegistryFull,
    RoleMismatch,
    InvalidStartupEpoch,
    StartupEvidenceMismatch,
    TelemetryOverflow,
    UnknownCore,
}

#[cfg(test)]
mod tests;
