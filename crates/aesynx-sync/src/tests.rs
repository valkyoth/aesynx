use super::{
    ArchIrqDisableProof, EarlyLock, LocalInterruptMask, LockOrderTracker, LockRank, SyncError,
};

#[test]
fn early_lock_rejects_double_lock_and_release_allows_relock() {
    let lock = EarlyLock::new();
    let guard = match lock.try_lock() {
        Ok(guard) => guard,
        Err(error) => return assert_eq!(error, SyncError::AlreadyLocked),
    };

    assert!(lock.is_locked());
    assert_eq!(lock.try_lock().err(), Some(SyncError::AlreadyLocked));
    assert!(guard.release().is_ok());
    assert!(!lock.is_locked());
    assert!(lock.try_lock().is_ok());
}

#[test]
fn explicit_release_does_not_double_unlock_on_drop() {
    let lock = EarlyLock::new();
    let guard = match lock.try_lock() {
        Ok(guard) => guard,
        Err(error) => return assert_eq!(error, SyncError::AlreadyLocked),
    };

    assert!(guard.release().is_ok());
    assert!(!lock.is_locked());
    assert!(lock.try_lock().is_ok());
}

#[test]
fn nested_interrupt_guards_restore_only_outer_enabled_state() {
    let interrupts = LocalInterruptMask::new_enabled();
    {
        let outer = match interrupts.mask() {
            Ok(guard) => guard,
            Err(error) => return assert_eq!(error, SyncError::LockDepthOverflow),
        };
        assert!(outer.snapshot().interrupts_were_enabled());
        assert!(!interrupts.interrupts_enabled());
        {
            let inner = match interrupts.mask() {
                Ok(guard) => guard,
                Err(error) => return assert_eq!(error, SyncError::LockDepthOverflow),
            };
            assert!(!inner.snapshot().interrupts_were_enabled());
            assert!(!interrupts.interrupts_enabled());
        }
        assert!(!interrupts.interrupts_enabled());
    }
    assert!(interrupts.interrupts_enabled());
    assert_eq!(interrupts.depth(), 0);
}

#[test]
fn initially_disabled_interrupts_remain_disabled_after_guard() {
    let interrupts = LocalInterruptMask::new_disabled();
    {
        let guard = match interrupts.mask() {
            Ok(guard) => guard,
            Err(error) => return assert_eq!(error, SyncError::LockDepthOverflow),
        };
        assert!(!guard.snapshot().interrupts_were_enabled());
        assert!(!interrupts.interrupts_enabled());
    }
    assert!(!interrupts.interrupts_enabled());
}

#[test]
fn irq_lock_masks_interrupts_while_held_and_restores_after_release() {
    let lock = EarlyLock::new();
    let interrupts = LocalInterruptMask::new_enabled();
    let proof = ArchIrqDisableProof::model_only_for_single_core_smoke();
    assert!(proof.is_model_only());
    let guard = match lock.try_lock_irq(&interrupts, proof) {
        Ok(guard) => guard,
        Err(error) => return assert_eq!(error, SyncError::AlreadyLocked),
    };

    assert!(lock.is_locked());
    assert!(!interrupts.interrupts_enabled());
    assert!(guard.release().is_ok());
    assert!(!lock.is_locked());
    assert!(interrupts.interrupts_enabled());
}

#[test]
fn lock_order_rejects_inversion_and_restores_after_drop() {
    let tracker = LockOrderTracker::new();
    {
        let descriptor = match tracker.try_enter(LockRank::DescriptorTables) {
            Ok(guard) => guard,
            Err(error) => return assert_eq!(error, SyncError::LockOrderViolation),
        };
        assert_eq!(tracker.depth(), 1);
        assert_eq!(tracker.current_rank(), Some(LockRank::DescriptorTables));
        assert_eq!(
            tracker.try_enter(LockRank::InterruptController).err(),
            Some(SyncError::LockOrderViolation)
        );
        let heap = match tracker.try_enter(LockRank::KernelHeap) {
            Ok(guard) => guard,
            Err(error) => return assert_eq!(error, SyncError::LockOrderViolation),
        };
        assert_eq!(tracker.depth(), 2);
        drop(heap);
        assert_eq!(tracker.current_rank(), Some(LockRank::DescriptorTables));
        drop(descriptor);
    }
    assert_eq!(tracker.depth(), 0);
    assert_eq!(tracker.current_rank(), None);
}

#[test]
fn non_lifo_lock_order_release_poisons_tracker() {
    let tracker = LockOrderTracker::new();
    let outer = match tracker.try_enter(LockRank::DescriptorTables) {
        Ok(guard) => guard,
        Err(error) => return assert_eq!(error, SyncError::LockOrderViolation),
    };
    let inner = match tracker.try_enter(LockRank::KernelHeap) {
        Ok(guard) => guard,
        Err(error) => return assert_eq!(error, SyncError::LockOrderViolation),
    };

    assert_eq!(outer.release().err(), Some(SyncError::LockOrderViolation));
    assert!(tracker.is_poisoned());
    assert_eq!(
        tracker.try_enter(LockRank::Telemetry).err(),
        Some(SyncError::LockOrderViolation)
    );
    assert_eq!(inner.release().err(), Some(SyncError::LockOrderViolation));
    assert_eq!(tracker.depth(), 2);
    assert_eq!(tracker.current_rank(), Some(LockRank::KernelHeap));
}

#[test]
fn non_lifo_interrupt_release_poisons_mask() {
    let interrupts = LocalInterruptMask::new_enabled();
    let outer = match interrupts.mask() {
        Ok(guard) => guard,
        Err(error) => return assert_eq!(error, SyncError::LockDepthOverflow),
    };
    let inner = match interrupts.mask() {
        Ok(guard) => guard,
        Err(error) => return assert_eq!(error, SyncError::LockDepthOverflow),
    };

    assert_eq!(outer.release().err(), Some(SyncError::LockOrderViolation));
    assert!(interrupts.is_poisoned());
    assert!(!interrupts.interrupts_enabled());
    assert_eq!(interrupts.mask().err(), Some(SyncError::LockOrderViolation));
    assert_eq!(inner.release().err(), Some(SyncError::LockOrderViolation));
    assert!(!interrupts.interrupts_enabled());
    assert_eq!(interrupts.depth(), 2);
}

#[test]
fn ordered_lock_releases_order_state_after_lock() {
    let tracker = LockOrderTracker::new();
    let lock = EarlyLock::new();
    {
        let guard = match lock.try_lock_ordered(&tracker, LockRank::Scheduler) {
            Ok(guard) => guard,
            Err(error) => return assert_eq!(error, SyncError::AlreadyLocked),
        };
        assert!(lock.is_locked());
        assert_eq!(tracker.current_rank(), Some(LockRank::Scheduler));
        assert!(guard.release().is_ok());
    }
    assert!(!lock.is_locked());
    assert_eq!(tracker.current_rank(), None);
}
