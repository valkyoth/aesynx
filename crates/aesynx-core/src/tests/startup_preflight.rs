use aesynx_abi::{CoreId, CpuHardwareId, ROOT_CORE, VirtAddr};

use crate::{
    ApDescriptorTableReadiness, ApStartupPreflight, CoreError, CorePerformanceClass, CoreRole,
    CoreTopology,
};

use super::{qemu_bootstrap_caps, qemu_worker_caps};

#[test]
fn ap_startup_preflight_blocks_execution_until_per_core_descriptors_exist() {
    let mut topology = match staged_two_core_topology() {
        Ok(topology) => topology,
        Err(error) => return assert_eq!(Some(error), None),
    };
    assert!(
        topology
            .assign_role(ROOT_CORE, ROOT_CORE, CoreRole::Bootstrap)
            .is_ok()
    );
    assert!(
        topology
            .assign_role(ROOT_CORE, CoreId::new(1), CoreRole::Scheduler)
            .is_ok()
    );

    let root = match topology.get(ROOT_CORE) {
        Some(entry) => entry,
        None => return assert_eq!(Some(CoreError::UnknownCore), None),
    };
    let scheduler = match topology.get(CoreId::new(1)) {
        Some(entry) => entry,
        None => return assert_eq!(Some(CoreError::UnknownCore), None),
    };
    let mut preflight = match ApStartupPreflight::<2>::new(ROOT_CORE) {
        Ok(preflight) => preflight,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };

    assert!(
        preflight
            .add_staged_core(
                ROOT_CORE,
                root,
                VirtAddr::new(0xffff_ffff_9000_0000),
                0x8000,
                ApDescriptorTableReadiness::PerCoreReady,
                10_000,
            )
            .is_ok()
    );
    assert!(
        preflight
            .add_staged_core(
                ROOT_CORE,
                scheduler,
                VirtAddr::new(0xffff_ffff_9001_0000),
                0x8000,
                ApDescriptorTableReadiness::SharedBootstrapOnly,
                10_000,
            )
            .is_ok()
    );

    let status = preflight.status();
    assert_eq!(status.owner_core(), ROOT_CORE);
    assert_eq!(status.planned(), 2);
    assert_eq!(status.stack_ready(), 2);
    assert_eq!(status.watchdog_ready(), 2);
    assert_eq!(status.descriptor_ready(), 1);
    assert!(!status.execution_allowed());
}

#[test]
fn ap_startup_preflight_accepts_only_staged_booting_entries() {
    let mut topology = match CoreTopology::<1>::new(ROOT_CORE) {
        Ok(topology) => topology,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };
    assert!(
        topology
            .insert_discovered(
                ROOT_CORE,
                ROOT_CORE,
                CpuHardwareId::new(0),
                qemu_bootstrap_caps(),
            )
            .is_ok()
    );
    let entry = match topology.get(ROOT_CORE) {
        Some(entry) => entry,
        None => return assert_eq!(Some(CoreError::UnknownCore), None),
    };
    let mut preflight = match ApStartupPreflight::<1>::new(ROOT_CORE) {
        Ok(preflight) => preflight,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };

    assert_eq!(
        preflight
            .add_staged_core(
                ROOT_CORE,
                entry,
                VirtAddr::new(0xffff_ffff_9000_0000),
                0x8000,
                ApDescriptorTableReadiness::PerCoreReady,
                10_000,
            )
            .err(),
        Some(CoreError::InvalidStateTransition)
    );
}

#[test]
fn ap_startup_preflight_rejects_overlapping_stacks_without_mutation() {
    let topology = match staged_two_core_topology() {
        Ok(topology) => topology,
        Err(error) => return assert_eq!(Some(error), None),
    };
    let root = match topology.get(ROOT_CORE) {
        Some(entry) => entry,
        None => return assert_eq!(Some(CoreError::UnknownCore), None),
    };
    let scheduler = match topology.get(CoreId::new(1)) {
        Some(entry) => entry,
        None => return assert_eq!(Some(CoreError::UnknownCore), None),
    };
    let mut preflight = match ApStartupPreflight::<2>::new(ROOT_CORE) {
        Ok(preflight) => preflight,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };

    assert!(
        preflight
            .add_staged_core(
                ROOT_CORE,
                root,
                VirtAddr::new(0xffff_ffff_9000_0000),
                0x8000,
                ApDescriptorTableReadiness::PerCoreReady,
                10_000,
            )
            .is_ok()
    );
    let before = preflight.status();
    assert_eq!(
        preflight
            .add_staged_core(
                ROOT_CORE,
                scheduler,
                VirtAddr::new(0xffff_ffff_9000_4000),
                0x8000,
                ApDescriptorTableReadiness::PerCoreReady,
                10_000,
            )
            .err(),
        Some(CoreError::DuplicateStartupStack)
    );
    assert_eq!(preflight.status(), before);
    assert!(preflight.resource(CoreId::new(1)).is_none());
}

#[test]
fn ap_startup_preflight_rejects_missing_watchdog() {
    let topology = match staged_two_core_topology() {
        Ok(topology) => topology,
        Err(error) => return assert_eq!(Some(error), None),
    };
    let root = match topology.get(ROOT_CORE) {
        Some(entry) => entry,
        None => return assert_eq!(Some(CoreError::UnknownCore), None),
    };
    let mut preflight = match ApStartupPreflight::<1>::new(ROOT_CORE) {
        Ok(preflight) => preflight,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };

    assert_eq!(
        preflight
            .add_staged_core(
                ROOT_CORE,
                root,
                VirtAddr::new(0xffff_ffff_9000_0000),
                0x8000,
                ApDescriptorTableReadiness::PerCoreReady,
                0,
            )
            .err(),
        Some(CoreError::MissingStartupWatchdog)
    );
}

fn staged_two_core_topology() -> Result<CoreTopology<2>, CoreError> {
    let mut topology = CoreTopology::<2>::new(ROOT_CORE)?;
    topology.insert_discovered(
        ROOT_CORE,
        ROOT_CORE,
        CpuHardwareId::new(0),
        qemu_bootstrap_caps(),
    )?;
    topology.insert_discovered(
        ROOT_CORE,
        CoreId::new(1),
        CpuHardwareId::new(1),
        qemu_worker_caps(CorePerformanceClass::Performance),
    )?;
    let _root_ticket = topology.stage_startup_ticket(ROOT_CORE, ROOT_CORE)?;
    let _scheduler_ticket = topology.stage_startup_ticket(ROOT_CORE, CoreId::new(1))?;
    Ok(topology)
}
