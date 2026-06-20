use core::cell::Cell;
use core::fmt;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering, fence};

use aesynx_abi::{CoreId, CpuHardwareId};

use crate::CoreError;

pub struct CoreStartupTicket {
    target_core: AtomicU32,
    hardware_id: AtomicU64,
    coordinator_core: AtomicU32,
    startup_epoch: AtomicU64,
    _not_sync: PhantomData<Cell<()>>,
}

impl CoreStartupTicket {
    pub(crate) fn new(
        target_core: CoreId,
        hardware_id: CpuHardwareId,
        coordinator_core: CoreId,
        startup_epoch: u64,
    ) -> Self {
        Self {
            target_core: AtomicU32::new(target_core.get()),
            hardware_id: AtomicU64::new(hardware_id.get()),
            coordinator_core: AtomicU32::new(coordinator_core.get()),
            startup_epoch: AtomicU64::new(startup_epoch),
            _not_sync: PhantomData,
        }
    }

    #[must_use]
    pub fn target_core(&self) -> CoreId {
        CoreId::new(self.target_core.load(Ordering::Acquire))
    }

    #[must_use]
    pub fn hardware_id(&self) -> CpuHardwareId {
        CpuHardwareId::new(self.hardware_id.load(Ordering::Acquire))
    }

    #[must_use]
    pub fn coordinator_core(&self) -> CoreId {
        CoreId::new(self.coordinator_core.load(Ordering::Acquire))
    }

    pub fn observe_arrival(
        self,
        arrived_core: CoreId,
        hardware_id: CpuHardwareId,
    ) -> Result<CoreStartupArrival, CoreError> {
        let target_core = self.target_core();
        let target_hardware_id = self.hardware_id();
        let coordinator_core = self.coordinator_core();
        let startup_epoch = self.startup_epoch.load(Ordering::Acquire);
        // Startup evidence fields are CPU topology identifiers, not
        // cryptographic secrets. Constant-time comparison is not required
        // here; bitwise OR still evaluates both mismatch checks.
        let core_mismatch = arrived_core != target_core;
        let hardware_mismatch = hardware_id != target_hardware_id;
        if core_mismatch | hardware_mismatch {
            return Err(CoreError::StartupEvidenceMismatch);
        }

        Ok(CoreStartupArrival::new(
            arrived_core,
            hardware_id,
            coordinator_core,
            startup_epoch,
        ))
    }
}

impl Drop for CoreStartupTicket {
    fn drop(&mut self) {
        fence(Ordering::SeqCst);
        self.target_core.store(0, Ordering::Relaxed);
        self.hardware_id.store(0, Ordering::Relaxed);
        self.coordinator_core.store(0, Ordering::Relaxed);
        self.startup_epoch.store(0, Ordering::Relaxed);
        fence(Ordering::SeqCst);
    }
}

impl PartialEq for CoreStartupTicket {
    fn eq(&self, other: &Self) -> bool {
        self.target_core() == other.target_core()
            && self.hardware_id() == other.hardware_id()
            && self.coordinator_core() == other.coordinator_core()
            && self.startup_epoch.load(Ordering::Acquire)
                == other.startup_epoch.load(Ordering::Acquire)
    }
}

impl Eq for CoreStartupTicket {}

impl fmt::Debug for CoreStartupTicket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CoreStartupTicket")
            .field("target_core", &self.target_core())
            .field("hardware_id", &"<redacted>")
            .field("coordinator_core", &self.coordinator_core())
            .field("startup_epoch", &"<redacted>")
            .finish()
    }
}

pub struct CoreStartupArrival {
    arrived_core: AtomicU32,
    hardware_id: AtomicU64,
    coordinator_core: AtomicU32,
    startup_epoch: AtomicU64,
    _not_sync: PhantomData<Cell<()>>,
}

impl CoreStartupArrival {
    pub(crate) fn new(
        arrived_core: CoreId,
        hardware_id: CpuHardwareId,
        coordinator_core: CoreId,
        startup_epoch: u64,
    ) -> Self {
        Self {
            arrived_core: AtomicU32::new(arrived_core.get()),
            hardware_id: AtomicU64::new(hardware_id.get()),
            coordinator_core: AtomicU32::new(coordinator_core.get()),
            startup_epoch: AtomicU64::new(startup_epoch),
            _not_sync: PhantomData,
        }
    }

    #[must_use]
    pub fn arrived_core(&self) -> CoreId {
        CoreId::new(self.arrived_core.load(Ordering::Acquire))
    }

    #[must_use]
    pub fn hardware_id(&self) -> CpuHardwareId {
        CpuHardwareId::new(self.hardware_id.load(Ordering::Acquire))
    }

    #[must_use]
    pub fn coordinator_core(&self) -> CoreId {
        CoreId::new(self.coordinator_core.load(Ordering::Acquire))
    }

    #[must_use]
    pub(crate) fn startup_epoch(&self) -> u64 {
        self.startup_epoch.load(Ordering::Acquire)
    }
}

impl Drop for CoreStartupArrival {
    fn drop(&mut self) {
        fence(Ordering::SeqCst);
        self.arrived_core.store(0, Ordering::Relaxed);
        self.hardware_id.store(0, Ordering::Relaxed);
        self.coordinator_core.store(0, Ordering::Relaxed);
        self.startup_epoch.store(0, Ordering::Relaxed);
        fence(Ordering::SeqCst);
    }
}

impl PartialEq for CoreStartupArrival {
    fn eq(&self, other: &Self) -> bool {
        self.arrived_core() == other.arrived_core()
            && self.hardware_id() == other.hardware_id()
            && self.coordinator_core() == other.coordinator_core()
            && self.startup_epoch() == other.startup_epoch()
    }
}

impl Eq for CoreStartupArrival {}

impl fmt::Debug for CoreStartupArrival {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CoreStartupArrival")
            .field("arrived_core", &self.arrived_core())
            .field("hardware_id", &"<redacted>")
            .field("coordinator_core", &self.coordinator_core())
            .field("startup_epoch", &"<redacted>")
            .finish()
    }
}
