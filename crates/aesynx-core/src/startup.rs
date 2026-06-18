use core::fmt;

use aesynx_abi::{CoreId, CpuHardwareId};

use crate::CoreError;

#[derive(Eq, PartialEq)]
pub struct CoreStartupTicket {
    target_core: CoreId,
    hardware_id: CpuHardwareId,
    coordinator_core: CoreId,
    startup_epoch: u64,
}

impl CoreStartupTicket {
    pub(crate) const fn new(
        target_core: CoreId,
        hardware_id: CpuHardwareId,
        coordinator_core: CoreId,
        startup_epoch: u64,
    ) -> Self {
        Self {
            target_core,
            hardware_id,
            coordinator_core,
            startup_epoch,
        }
    }

    #[must_use]
    pub const fn target_core(&self) -> CoreId {
        self.target_core
    }

    #[must_use]
    pub const fn hardware_id(&self) -> CpuHardwareId {
        self.hardware_id
    }

    #[must_use]
    pub const fn coordinator_core(&self) -> CoreId {
        self.coordinator_core
    }

    #[must_use]
    pub const fn startup_epoch(&self) -> u64 {
        self.startup_epoch
    }

    pub fn observe_arrival(
        &self,
        arrived_core: CoreId,
        hardware_id: CpuHardwareId,
    ) -> Result<CoreStartupArrival, CoreError> {
        if arrived_core != self.target_core || hardware_id != self.hardware_id {
            return Err(CoreError::StartupEvidenceMismatch);
        }

        Ok(CoreStartupArrival {
            arrived_core,
            hardware_id,
            coordinator_core: self.coordinator_core,
            startup_epoch: self.startup_epoch,
        })
    }
}

impl fmt::Debug for CoreStartupTicket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CoreStartupTicket")
            .field("target_core", &self.target_core)
            .field("hardware_id", &"<redacted>")
            .field("coordinator_core", &self.coordinator_core)
            .field("startup_epoch", &"<redacted>")
            .finish()
    }
}

#[derive(Eq, PartialEq)]
pub struct CoreStartupArrival {
    arrived_core: CoreId,
    hardware_id: CpuHardwareId,
    coordinator_core: CoreId,
    startup_epoch: u64,
}

impl CoreStartupArrival {
    #[must_use]
    pub const fn arrived_core(&self) -> CoreId {
        self.arrived_core
    }

    #[must_use]
    pub const fn hardware_id(&self) -> CpuHardwareId {
        self.hardware_id
    }

    #[must_use]
    pub const fn coordinator_core(&self) -> CoreId {
        self.coordinator_core
    }

    #[must_use]
    pub const fn startup_epoch(&self) -> u64 {
        self.startup_epoch
    }
}

impl fmt::Debug for CoreStartupArrival {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CoreStartupArrival")
            .field("arrived_core", &self.arrived_core)
            .field("hardware_id", &"<redacted>")
            .field("coordinator_core", &self.coordinator_core)
            .field("startup_epoch", &"<redacted>")
            .finish()
    }
}
