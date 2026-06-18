use aesynx_abi::{CoreId, CpuHardwareId, ROOT_CORE};
use aesynx_core::{
    BootBarrier, CoreCapabilitySet, CoreError, CoreHardwareState, CoreIsa, CorePerformanceClass,
    CoreRole, CoreTopology, QEMU_MULTICORE_TOPOLOGY_CORES,
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
    pub startup_evidence_ok: bool,
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
    let mut topology = CoreTopology::<{ QEMU_MULTICORE_TOPOLOGY_CORES }>::new(ROOT_CORE)?;

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

    let root_ticket = topology.stage_startup_ticket(ROOT_CORE, ROOT_CORE)?;
    let scheduler_ticket = topology.stage_startup_ticket(ROOT_CORE, CoreId::new(1))?;
    let driver_ticket = topology.stage_startup_ticket(ROOT_CORE, CoreId::new(2))?;
    let idle_ticket = topology.stage_startup_ticket(ROOT_CORE, CoreId::new(3))?;

    topology.assign_role(ROOT_CORE, ROOT_CORE, CoreRole::Bootstrap)?;
    topology.assign_role(ROOT_CORE, CoreId::new(1), CoreRole::Scheduler)?;
    topology.assign_role(ROOT_CORE, CoreId::new(2), CoreRole::DriverService)?;
    topology.assign_role(ROOT_CORE, CoreId::new(3), CoreRole::Idle)?;

    let root_arrival = root_ticket.observe_arrival(ROOT_CORE, CpuHardwareId::new(0))?;
    let scheduler_arrival =
        scheduler_ticket.observe_arrival(CoreId::new(1), CpuHardwareId::new(1))?;
    let driver_arrival = driver_ticket.observe_arrival(CoreId::new(2), CpuHardwareId::new(2))?;
    let idle_arrival = idle_ticket.observe_arrival(CoreId::new(3), CpuHardwareId::new(3))?;
    let startup_evidence_ok = root_arrival.arrived_core() == ROOT_CORE
        && scheduler_arrival.arrived_core() == CoreId::new(1)
        && driver_arrival.arrived_core() == CoreId::new(2)
        && idle_arrival.arrived_core() == CoreId::new(3);

    topology.mark_hardware_online(ROOT_CORE, root_arrival)?;
    topology.mark_hardware_online(ROOT_CORE, scheduler_arrival)?;
    topology.mark_hardware_online(ROOT_CORE, driver_arrival)?;
    topology.mark_hardware_online(ROOT_CORE, idle_arrival)?;

    let mut barrier = BootBarrier::<4>::new(ROOT_CORE)?;
    for core in [ROOT_CORE, CoreId::new(1), CoreId::new(2), CoreId::new(3)] {
        barrier.add_participant(ROOT_CORE, core)?;
    }
    barrier.seal(ROOT_CORE)?;
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
        startup_evidence_ok,
        barrier_ok: barrier.status().all_arrived(),
    })
}

fn insert_qemu_core(
    topology: &mut CoreTopology<{ QEMU_MULTICORE_TOPOLOGY_CORES }>,
    core: CoreId,
    hardware_id: CpuHardwareId,
    performance_class: CorePerformanceClass,
) -> Result<(), CoreError> {
    topology.insert_discovered(
        ROOT_CORE,
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
        assert!(status.startup_evidence_ok);
        assert!(status.barrier_ok);
    }
}
