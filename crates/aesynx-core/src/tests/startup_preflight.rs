use aesynx_abi::{CoreId, CpuHardwareId, ROOT_CORE, VirtAddr};
use core::fmt::{self, Write as _};

use crate::{
    ApDescriptorTableReadiness, ApStackPlan, ApStackRegion, ApStartupPreflight, CoreError,
    CorePerformanceClass, CoreRole, CoreTopology,
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
                test_ap_stack_plan(VirtAddr::new(0xffff_ffff_9000_0000), 0x8000),
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
                test_ap_stack_plan(VirtAddr::new(0xffff_ffff_9001_0000), 0x8000),
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
    assert_eq!(
        preflight.into_dispatch_token(ROOT_CORE).err(),
        Some(CoreError::StartupPreflightBlocked)
    );
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
                test_ap_stack_plan(VirtAddr::new(0xffff_ffff_9000_0000), 0x8000),
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
                test_ap_stack_plan(VirtAddr::new(0xffff_ffff_9000_0000), 0x8000),
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
                test_ap_stack_plan(VirtAddr::new(0xffff_ffff_9000_4000), 0x8000),
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
                test_ap_stack_plan(VirtAddr::new(0xffff_ffff_9000_0000), 0x8000),
                ApDescriptorTableReadiness::PerCoreReady,
                0,
            )
            .err(),
        Some(CoreError::MissingStartupWatchdog)
    );
}

#[test]
fn ap_startup_preflight_rejects_stack_outside_ap_region() {
    let topology = match staged_two_core_topology() {
        Ok(topology) => topology,
        Err(error) => return assert_eq!(Some(error), None),
    };
    let root = match topology.get(ROOT_CORE) {
        Some(entry) => entry,
        None => return assert_eq!(Some(CoreError::UnknownCore), None),
    };
    let preflight = match ApStartupPreflight::<1>::new(ROOT_CORE) {
        Ok(preflight) => preflight,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };

    assert_eq!(
        ApStackPlan::new(VirtAddr::new(0), 0x8000, test_ap_stack_region()).err(),
        Some(CoreError::InvalidStartupStack)
    );
    assert!(preflight.resource(root.core()).is_none());
    assert_eq!(
        preflight.status().planned(),
        0,
        "failed stack validation must not mutate preflight resources"
    );
}

#[test]
fn ap_startup_preflight_dispatch_token_requires_owner_and_execution_ready() {
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
    let mut wrong_owner = match ApStartupPreflight::<2>::new(ROOT_CORE) {
        Ok(preflight) => preflight,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };

    assert!(
        wrong_owner
            .add_staged_core(
                ROOT_CORE,
                root,
                test_ap_stack_plan(VirtAddr::new(0xffff_ffff_9000_0000), 0x8000),
                ApDescriptorTableReadiness::PerCoreReady,
                10_000,
            )
            .is_ok()
    );
    assert!(
        wrong_owner
            .add_staged_core(
                ROOT_CORE,
                scheduler,
                test_ap_stack_plan(VirtAddr::new(0xffff_ffff_9001_0000), 0x8000),
                ApDescriptorTableReadiness::PerCoreReady,
                10_000,
            )
            .is_ok()
    );
    assert_eq!(
        wrong_owner.into_dispatch_token(CoreId::new(1)).err(),
        Some(CoreError::OwnerMismatch)
    );

    let mut ready = match ApStartupPreflight::<2>::new(ROOT_CORE) {
        Ok(preflight) => preflight,
        Err(error) => return assert_eq!(error, CoreError::CapacityZero),
    };
    assert!(
        ready
            .add_staged_core(
                ROOT_CORE,
                root,
                test_ap_stack_plan(VirtAddr::new(0xffff_ffff_9000_0000), 0x8000),
                ApDescriptorTableReadiness::PerCoreReady,
                10_000,
            )
            .is_ok()
    );
    assert!(
        ready
            .add_staged_core(
                ROOT_CORE,
                scheduler,
                test_ap_stack_plan(VirtAddr::new(0xffff_ffff_9001_0000), 0x8000),
                ApDescriptorTableReadiness::PerCoreReady,
                10_000,
            )
            .is_ok()
    );

    let dispatch_permit = match ready.into_dispatch_token(ROOT_CORE) {
        Ok(dispatch_permit) => dispatch_permit,
        Err(error) => return assert_eq!(Some(error), None),
    };
    assert_eq!(dispatch_permit.owner_core(), ROOT_CORE);
    assert_eq!(dispatch_permit.planned(), 2);
    assert_eq!(dispatch_permit.capacity(), 2);
    assert!(dispatch_permit.resource(ROOT_CORE).is_some());
    assert!(dispatch_permit.resource(CoreId::new(1)).is_some());
    let mut debug = DebugBuffer::new();
    assert!(write!(&mut debug, "{dispatch_permit:?}").is_ok());
    assert!(!debug.as_str().contains("9000"));
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

fn test_ap_stack_plan(base: VirtAddr, len: u64) -> ApStackPlan {
    ApStackPlan::test_only(base, len, test_ap_stack_region())
}

fn test_ap_stack_region() -> ApStackRegion {
    ApStackRegion::test_only(
        VirtAddr::new(0xffff_ffff_8000_0000),
        VirtAddr::new(0xffff_ffff_c000_0000),
    )
}

struct DebugBuffer {
    bytes: [u8; 256],
    len: usize,
}

impl DebugBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; 256],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
    }
}

impl fmt::Write for DebugBuffer {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let remaining = self.bytes.len().saturating_sub(self.len);
        if text.len() > remaining {
            return Err(fmt::Error);
        }
        let end = self.len + text.len();
        self.bytes[self.len..end].copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}
