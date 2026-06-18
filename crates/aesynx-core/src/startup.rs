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

    pub fn observe_arrival(
        &self,
        arrived_core: CoreId,
        hardware_id: CpuHardwareId,
    ) -> Result<CoreStartupArrival, CoreError> {
        let core_mismatch = arrived_core != self.target_core;
        let hardware_mismatch = hardware_id != self.hardware_id;
        if core_mismatch | hardware_mismatch {
            return Err(CoreError::StartupEvidenceMismatch);
        }

        Ok(CoreStartupArrival::new(
            arrived_core,
            hardware_id,
            self.coordinator_core,
            self.startup_epoch,
        ))
    }
}

impl Drop for CoreStartupTicket {
    fn drop(&mut self) {
        self.target_core = CoreId::new(0);
        self.hardware_id = CpuHardwareId::new(0);
        self.coordinator_core = CoreId::new(0);
        self.startup_epoch = 0;
        core::hint::black_box((
            self.target_core,
            self.hardware_id,
            self.coordinator_core,
            self.startup_epoch,
        ));
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
    pub(crate) const fn new(
        arrived_core: CoreId,
        hardware_id: CpuHardwareId,
        coordinator_core: CoreId,
        startup_epoch: u64,
    ) -> Self {
        Self {
            arrived_core,
            hardware_id,
            coordinator_core,
            startup_epoch,
        }
    }

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
    pub(crate) const fn startup_epoch(&self) -> u64 {
        self.startup_epoch
    }
}

impl Drop for CoreStartupArrival {
    fn drop(&mut self) {
        self.arrived_core = CoreId::new(0);
        self.hardware_id = CpuHardwareId::new(0);
        self.coordinator_core = CoreId::new(0);
        self.startup_epoch = 0;
        core::hint::black_box((
            self.arrived_core,
            self.hardware_id,
            self.coordinator_core,
            self.startup_epoch,
        ));
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
