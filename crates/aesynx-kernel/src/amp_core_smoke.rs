use aesynx_abi::ROOT_CORE;
use aesynx_core::{
    BootBarrier, CoreCapabilitySet, CoreError, CoreIsa, CoreLocal, CorePerformanceClass,
    CoreRegistry, CoreRole, CoreState,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AmpCoreSmokeStatus {
    pub bootstrap_role_ok: bool,
    pub capabilities_ok: bool,
    pub registry_ok: bool,
    pub telemetry_ok: bool,
    pub barrier_ok: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AmpCoreSmokeError {
    Core(CoreError),
}

impl From<CoreError> for AmpCoreSmokeError {
    fn from(error: CoreError) -> Self {
        Self::Core(error)
    }
}

pub fn run() -> Result<AmpCoreSmokeStatus, AmpCoreSmokeError> {
    let capabilities = CoreCapabilitySet::new(CoreIsa::X86_64, CorePerformanceClass::Control)
        .with_local_timer(true)
        .with_ipi(true)
        .with_directed_irq(true)
        .with_shared_memory_atomics(true);
    let mut local = CoreLocal::new(ROOT_CORE, CoreRole::Idle, capabilities, CoreState::Booting);
    local.assign_role(CoreRole::Bootstrap)?;
    local.mark_online()?;
    local.telemetry_mut().record_boot_barrier_arrival()?;
    local.telemetry_mut().record_local_event()?;

    let mut registry = CoreRegistry::<4>::new(ROOT_CORE)?;
    registry.insert(ROOT_CORE, local)?;
    let registry_status = registry.status();

    let mut barrier = BootBarrier::<4>::new(ROOT_CORE)?;
    barrier.add_participant(ROOT_CORE, ROOT_CORE)?;
    barrier.seal(ROOT_CORE)?;
    barrier.arrive(ROOT_CORE)?;
    let barrier_status = barrier.status();

    let registered = registry.require_role(ROOT_CORE, CoreRole::Bootstrap)?;
    Ok(AmpCoreSmokeStatus {
        bootstrap_role_ok: registered.role() == CoreRole::Bootstrap
            && registered.state() == CoreState::Online,
        capabilities_ok: registered.capabilities().isa() == CoreIsa::X86_64
            && registered.capabilities().performance_class() == CorePerformanceClass::Control
            && registered.capabilities().has_local_timer()
            && registered.capabilities().supports_ipi()
            && registered.capabilities().supports_directed_irq()
            && registered.capabilities().supports_shared_memory_atomics(),
        registry_ok: registry_status.owner_core() == ROOT_CORE
            && registry_status.len() == 1
            && registry_status.capacity() == 4
            && registry_status.epoch() == 1
            && registry.live_count() == 1,
        telemetry_ok: registered.telemetry().role_assignments() == 1
            && registered.telemetry().boot_barrier_arrivals() == 1
            && registered.telemetry().local_events() == 1,
        barrier_ok: barrier_status.owner_core() == ROOT_CORE
            && barrier_status.participants() == 1
            && barrier_status.arrivals() == 1
            && barrier_status.all_arrived(),
    })
}

#[cfg(test)]
mod tests {
    use super::run;

    #[test]
    fn amp_core_smoke_records_bootstrap_core_role() {
        let status = match run() {
            Ok(status) => status,
            Err(error) => return assert_eq!(Some(error), None),
        };

        assert!(status.bootstrap_role_ok);
        assert!(status.capabilities_ok);
        assert!(status.registry_ok);
        assert!(status.telemetry_ok);
        assert!(status.barrier_ok);
    }
}
