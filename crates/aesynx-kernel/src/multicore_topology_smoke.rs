use aesynx_abi::{CoreId, CpuHardwareId, ROOT_CORE};
use aesynx_core::{
    BootBarrier, CoreCapabilitySet, CoreError, CoreHardwareState, CoreIsa, CorePerformanceClass,
    CoreRole, CoreTopology,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MulticoreTopologySmokeStatus {
    pub qemu_smp_cores_ok: bool,
    pub hardware_online_ok: bool,
    pub role_assignment_ok: bool,
    pub bootstrap_ok: bool,
    pub scheduler_ok: bool,
    pub driver_service_ok: bool,
    pub idle_ok: bool,
    pub barrier_ok: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MulticoreTopologySmokeError {
    Core(CoreError),
}

impl From<CoreError> for MulticoreTopologySmokeError {
    fn from(error: CoreError) -> Self {
        Self::Core(error)
    }
}

pub fn run() -> Result<MulticoreTopologySmokeStatus, MulticoreTopologySmokeError> {
    let mut topology = CoreTopology::<4>::new(ROOT_CORE)?;

    insert_qemu_core(
        &mut topology,
        ROOT_CORE,
        CpuHardwareId::new(0),
        CorePerformanceClass::Control,
    )?;
    insert_qemu_core(
        &mut topology,
        CoreId::new(1),
        CpuHardwareId::new(1),
        CorePerformanceClass::Performance,
    )?;
    insert_qemu_core(
        &mut topology,
        CoreId::new(2),
        CpuHardwareId::new(2),
        CorePerformanceClass::Control,
    )?;
    insert_qemu_core(
        &mut topology,
        CoreId::new(3),
        CpuHardwareId::new(3),
        CorePerformanceClass::Efficiency,
    )?;

    for core in [ROOT_CORE, CoreId::new(1), CoreId::new(2), CoreId::new(3)] {
        topology.stage_startup(core)?;
    }

    topology.assign_role(ROOT_CORE, CoreRole::Bootstrap)?;
    topology.assign_role(CoreId::new(1), CoreRole::Scheduler)?;
    topology.assign_role(CoreId::new(2), CoreRole::DriverService)?;
    topology.assign_role(CoreId::new(3), CoreRole::Idle)?;

    for core in [ROOT_CORE, CoreId::new(1), CoreId::new(2), CoreId::new(3)] {
        topology.mark_hardware_online(core)?;
    }

    let mut barrier = BootBarrier::<4>::new(ROOT_CORE)?;
    for core in [ROOT_CORE, CoreId::new(1), CoreId::new(2), CoreId::new(3)] {
        barrier.add_participant(core)?;
    }
    barrier.seal()?;
    for core in [ROOT_CORE, CoreId::new(1), CoreId::new(2), CoreId::new(3)] {
        barrier.arrive(core)?;
    }

    let status = topology.status();
    let root = topology.get(ROOT_CORE);
    let scheduler = topology.get(CoreId::new(1));
    let driver = topology.get(CoreId::new(2));
    let idle = topology.get(CoreId::new(3));

    Ok(MulticoreTopologySmokeStatus {
        qemu_smp_cores_ok: status.discovered() == 4 && status.capacity() == 4,
        hardware_online_ok: status.hardware_online() == 4
            && root.is_some_and(|entry| entry.hardware_state() == CoreHardwareState::Online)
            && scheduler.is_some_and(|entry| entry.hardware_state() == CoreHardwareState::Online)
            && driver.is_some_and(|entry| entry.hardware_state() == CoreHardwareState::Online)
            && idle.is_some_and(|entry| entry.hardware_state() == CoreHardwareState::Online),
        role_assignment_ok: status.assigned() == 4,
        bootstrap_ok: status.bootstrap_roles() == 1
            && root.is_some_and(|entry| entry.role() == CoreRole::Bootstrap),
        scheduler_ok: status.scheduler_roles() == 1
            && scheduler.is_some_and(|entry| entry.role() == CoreRole::Scheduler),
        driver_service_ok: status.driver_service_roles() == 1
            && driver.is_some_and(|entry| entry.role() == CoreRole::DriverService),
        idle_ok: status.idle_roles() == 1
            && idle.is_some_and(|entry| entry.role() == CoreRole::Idle),
        barrier_ok: barrier.status().all_arrived(),
    })
}

fn insert_qemu_core(
    topology: &mut CoreTopology<4>,
    core: CoreId,
    hardware_id: CpuHardwareId,
    performance_class: CorePerformanceClass,
) -> Result<(), CoreError> {
    topology.insert_discovered(
        core,
        hardware_id,
        CoreCapabilitySet::new(CoreIsa::X86_64, performance_class)
            .with_local_timer(true)
            .with_ipi(true)
            .with_directed_irq(true)
            .with_shared_memory_atomics(true),
    )
}

#[cfg(test)]
mod tests {
    use super::run;

    #[test]
    fn multicore_topology_smoke_models_four_qemu_cores() {
        let status = match run() {
            Ok(status) => status,
            Err(error) => return assert_eq!(Some(error), None),
        };

        assert!(status.qemu_smp_cores_ok);
        assert!(status.hardware_online_ok);
        assert!(status.role_assignment_ok);
        assert!(status.bootstrap_ok);
        assert!(status.scheduler_ok);
        assert!(status.driver_service_ok);
        assert!(status.idle_ok);
        assert!(status.barrier_ok);
    }
}
