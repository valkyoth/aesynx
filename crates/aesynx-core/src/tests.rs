use aesynx_abi::{CoreId, ROOT_CORE};

use crate::{
    BootBarrier, CoreCapabilitySet, CoreError, CoreIsa, CoreLocal, CorePerformanceClass,
    CoreRegistry, CoreRole, CoreState,
};

fn qemu_bootstrap_caps() -> CoreCapabilitySet {
    CoreCapabilitySet::new(CoreIsa::X86_64, CorePerformanceClass::Control)
        .with_local_timer(true)
        .with_ipi(true)
        .with_directed_irq(true)
        .with_shared_memory_atomics(true)
}

#[test]
fn core_roles_expose_amp_authority_classes() {
    assert_eq!(CoreRole::Bootstrap.label(), "bootstrap");
    assert!(CoreRole::Bootstrap.can_schedule_tasks());
    assert!(CoreRole::Bootstrap.can_own_driver_irq());
    assert!(CoreRole::Scheduler.can_schedule_tasks());
    assert!(!CoreRole::Scheduler.can_own_driver_irq());
    assert!(CoreRole::DriverService.can_own_driver_irq());
    assert!(!CoreRole::Idle.can_schedule_tasks());
}

#[test]
fn core_capabilities_record_heterogeneous_metadata() {
    let caps = CoreCapabilitySet::new(CoreIsa::Aarch64, CorePerformanceClass::Efficiency)
        .with_local_timer(true);

    assert_eq!(caps.isa(), CoreIsa::Aarch64);
    assert_eq!(caps.isa().label(), "aarch64");
    assert_eq!(caps.performance_class(), CorePerformanceClass::Efficiency);
    assert_eq!(caps.performance_class().label(), "efficiency");
    assert!(caps.has_local_timer());
    assert!(!caps.supports_ipi());
}

#[test]
fn core_local_tracks_role_state_and_telemetry() {
    let mut local = CoreLocal::new(
        ROOT_CORE,
        CoreRole::Idle,
        qemu_bootstrap_caps(),
        CoreState::Booting,
    );

    assert!(local.is_live());
    assert!(local.assign_role(CoreRole::Bootstrap).is_ok());
    local.set_state(CoreState::Online);
    assert!(local.telemetry_mut().record_local_event().is_ok());

    assert_eq!(local.id(), ROOT_CORE);
    assert_eq!(local.role(), CoreRole::Bootstrap);
    assert_eq!(local.state(), CoreState::Online);
    assert_eq!(local.telemetry().role_assignments(), 1);
    assert_eq!(local.telemetry().local_events(), 1);
}

#[test]
fn core_registry_is_owner_scoped_and_rejects_duplicates() {
    let local = CoreLocal::new(
        ROOT_CORE,
        CoreRole::Bootstrap,
        qemu_bootstrap_caps(),
        CoreState::Online,
    );
    let mut registry = match CoreRegistry::<2>::new(ROOT_CORE) {
        Ok(registry) => registry,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };

    assert!(registry.insert(local).is_ok());
    assert_eq!(registry.insert(local).err(), Some(CoreError::DuplicateCore));
    assert_eq!(registry.live_count(), 1);
    assert_eq!(registry.status().owner_core(), ROOT_CORE);
    assert_eq!(registry.status().len(), 1);
    assert_eq!(registry.status().capacity(), 2);
    assert_eq!(registry.status().epoch(), 1);
    assert!(
        registry
            .require_role(ROOT_CORE, CoreRole::Bootstrap)
            .is_ok()
    );
    assert_eq!(
        registry
            .require_role(ROOT_CORE, CoreRole::DriverService)
            .err(),
        Some(CoreError::RoleMismatch)
    );
    assert_eq!(
        registry.require_role(CoreId::new(9), CoreRole::Idle).err(),
        Some(CoreError::UnknownCore)
    );
}

#[test]
fn core_registry_insert_rejects_epoch_overflow_without_mutation() {
    let local = CoreLocal::new(
        ROOT_CORE,
        CoreRole::Bootstrap,
        qemu_bootstrap_caps(),
        CoreState::Online,
    );
    let mut registry = match CoreRegistry::<2>::new(ROOT_CORE)
        .map(|registry| registry.with_epoch_for_test(u64::MAX))
    {
        Ok(registry) => registry,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };

    assert_eq!(
        registry.insert(local).err(),
        Some(CoreError::TelemetryOverflow)
    );
    assert_eq!(registry.status().len(), 0);
    assert_eq!(registry.status().epoch(), u64::MAX);
    assert!(!registry.contains(ROOT_CORE));
    assert_eq!(registry.live_count(), 0);
}

#[test]
fn boot_barrier_is_validate_then_commit() {
    let mut barrier = match BootBarrier::<2>::new(ROOT_CORE) {
        Ok(barrier) => barrier,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };

    assert_eq!(
        barrier.arrive(ROOT_CORE).err(),
        Some(CoreError::BarrierNotSealed)
    );
    assert!(barrier.add_participant(ROOT_CORE).is_ok());
    assert_eq!(
        barrier.add_participant(ROOT_CORE).err(),
        Some(CoreError::DuplicateCore)
    );
    assert_eq!(barrier.status().participants(), 1);
    assert_eq!(barrier.status().arrivals(), 0);
    assert!(barrier.seal().is_ok());
    assert_eq!(
        barrier.add_participant(CoreId::new(1)).err(),
        Some(CoreError::BarrierSealed)
    );
    assert_eq!(
        barrier.arrive(CoreId::new(1)).err(),
        Some(CoreError::UnknownCore)
    );
    assert!(barrier.arrive(ROOT_CORE).is_ok());
    assert_eq!(
        barrier.arrive(ROOT_CORE).err(),
        Some(CoreError::AlreadyArrived)
    );

    let status = barrier.status();
    assert!(status.sealed());
    assert!(status.all_arrived());
    assert_eq!(status.owner_core(), ROOT_CORE);
}
