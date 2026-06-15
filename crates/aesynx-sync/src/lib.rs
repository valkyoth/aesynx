#![no_std]
#![forbid(unsafe_code)]

use core::cell::Cell;
use core::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SyncError {
    AlreadyLocked,
    NotLocked,
    LockOrderViolation,
    LockDepthOverflow,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum LockRank {
    InterruptController = 10,
    DescriptorTables = 20,
    AddressSpace = 30,
    FrameAllocator = 40,
    KernelHeap = 50,
    Scheduler = 60,
    Ipc = 70,
    Telemetry = 80,
    AiPolicy = 90,
}

#[derive(Debug)]
pub struct LockOrderTracker {
    current: Cell<Option<LockRank>>,
    depth: Cell<u8>,
}

impl LockOrderTracker {
    pub const fn new() -> Self {
        Self {
            current: Cell::new(None),
            depth: Cell::new(0),
        }
    }

    pub fn try_enter(&self, rank: LockRank) -> Result<LockOrderGuard<'_>, SyncError> {
        if let Some(current) = self.current.get()
            && rank <= current
        {
            return Err(SyncError::LockOrderViolation);
        }

        let depth = self.depth.get();
        let Some(next_depth) = depth.checked_add(1) else {
            return Err(SyncError::LockDepthOverflow);
        };
        let previous = self.current.get();
        self.current.set(Some(rank));
        self.depth.set(next_depth);

        Ok(LockOrderGuard {
            tracker: self,
            previous,
            previous_depth: depth,
            active: true,
        })
    }

    #[must_use]
    pub fn current_rank(&self) -> Option<LockRank> {
        self.current.get()
    }

    #[must_use]
    pub fn depth(&self) -> u8 {
        self.depth.get()
    }
}

impl Default for LockOrderTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct LockOrderGuard<'a> {
    tracker: &'a LockOrderTracker,
    previous: Option<LockRank>,
    previous_depth: u8,
    active: bool,
}

impl LockOrderGuard<'_> {
    pub fn release(mut self) -> Result<(), SyncError> {
        self.release_inner()
    }

    fn release_inner(&mut self) -> Result<(), SyncError> {
        if !self.active {
            return Err(SyncError::NotLocked);
        }

        self.tracker.current.set(self.previous);
        self.tracker.depth.set(self.previous_depth);
        self.active = false;
        Ok(())
    }
}

impl Drop for LockOrderGuard<'_> {
    fn drop(&mut self) {
        let _ = self.release_inner();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InterruptSnapshot {
    interrupts_were_enabled: bool,
}

impl InterruptSnapshot {
    #[must_use]
    pub const fn new(interrupts_were_enabled: bool) -> Self {
        Self {
            interrupts_were_enabled,
        }
    }

    #[must_use]
    pub const fn interrupts_were_enabled(self) -> bool {
        self.interrupts_were_enabled
    }
}

#[derive(Debug)]
pub struct LocalInterruptMask {
    enabled: Cell<bool>,
    depth: Cell<u8>,
}

impl LocalInterruptMask {
    pub const fn new_enabled() -> Self {
        Self {
            enabled: Cell::new(true),
            depth: Cell::new(0),
        }
    }

    pub const fn new_disabled() -> Self {
        Self {
            enabled: Cell::new(false),
            depth: Cell::new(0),
        }
    }

    pub fn mask(&self) -> Result<InterruptGuard<'_>, SyncError> {
        let depth = self.depth.get();
        let Some(next_depth) = depth.checked_add(1) else {
            return Err(SyncError::LockDepthOverflow);
        };
        let snapshot = InterruptSnapshot::new(self.enabled.get());
        self.enabled.set(false);
        self.depth.set(next_depth);
        Ok(InterruptGuard {
            mask: self,
            snapshot,
            previous_depth: depth,
            active: true,
        })
    }

    #[must_use]
    pub fn interrupts_enabled(&self) -> bool {
        self.enabled.get()
    }

    #[must_use]
    pub fn depth(&self) -> u8 {
        self.depth.get()
    }
}

#[derive(Debug)]
pub struct InterruptGuard<'a> {
    mask: &'a LocalInterruptMask,
    snapshot: InterruptSnapshot,
    previous_depth: u8,
    active: bool,
}

impl InterruptGuard<'_> {
    #[must_use]
    pub const fn snapshot(&self) -> InterruptSnapshot {
        self.snapshot
    }

    pub fn release(mut self) -> Result<(), SyncError> {
        self.release_inner()
    }

    fn release_inner(&mut self) -> Result<(), SyncError> {
        if !self.active {
            return Err(SyncError::NotLocked);
        }

        if self.snapshot.interrupts_were_enabled {
            self.mask.enabled.set(true);
        }
        self.mask.depth.set(self.previous_depth);
        self.active = false;
        Ok(())
    }
}

impl Drop for InterruptGuard<'_> {
    fn drop(&mut self) {
        let _ = self.release_inner();
    }
}

#[derive(Debug)]
pub struct EarlyLock {
    locked: AtomicBool,
}

impl EarlyLock {
    pub const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }

    pub fn try_lock(&self) -> Result<EarlyLockGuard<'_>, SyncError> {
        self.locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .map_err(|_current| SyncError::AlreadyLocked)?;
        Ok(EarlyLockGuard {
            locked: &self.locked,
            active: true,
        })
    }

    pub fn try_lock_irq<'a>(
        &'a self,
        interrupts: &'a LocalInterruptMask,
    ) -> Result<IrqLockGuard<'a>, SyncError> {
        let irq = interrupts.mask()?;
        let lock = match self.try_lock() {
            Ok(lock) => lock,
            Err(error) => {
                drop(irq);
                return Err(error);
            }
        };
        Ok(IrqLockGuard {
            lock: Some(lock),
            irq: Some(irq),
        })
    }

    pub fn try_lock_ordered<'a>(
        &'a self,
        tracker: &'a LockOrderTracker,
        rank: LockRank,
    ) -> Result<OrderedLockGuard<'a>, SyncError> {
        let order = tracker.try_enter(rank)?;
        let lock = match self.try_lock() {
            Ok(lock) => lock,
            Err(error) => {
                drop(order);
                return Err(error);
            }
        };
        Ok(OrderedLockGuard {
            lock: Some(lock),
            order: Some(order),
        })
    }

    #[must_use]
    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Acquire)
    }
}

impl Default for EarlyLock {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct EarlyLockGuard<'a> {
    locked: &'a AtomicBool,
    active: bool,
}

impl EarlyLockGuard<'_> {
    pub fn release(mut self) -> Result<(), SyncError> {
        self.release_inner()
    }

    fn release_inner(&mut self) -> Result<(), SyncError> {
        if !self.active {
            return Err(SyncError::NotLocked);
        }
        self.locked.store(false, Ordering::Release);
        self.active = false;
        Ok(())
    }
}

impl Drop for EarlyLockGuard<'_> {
    fn drop(&mut self) {
        let _ = self.release_inner();
    }
}

#[derive(Debug)]
pub struct OrderedLockGuard<'a> {
    lock: Option<EarlyLockGuard<'a>>,
    order: Option<LockOrderGuard<'a>>,
}

impl OrderedLockGuard<'_> {
    pub fn release(mut self) -> Result<(), SyncError> {
        self.release_inner()
    }

    fn release_inner(&mut self) -> Result<(), SyncError> {
        if self.lock.is_none() || self.order.is_none() {
            return Err(SyncError::NotLocked);
        }
        let _ = self.lock.take();
        let _ = self.order.take();
        Ok(())
    }
}

impl Drop for OrderedLockGuard<'_> {
    fn drop(&mut self) {
        let _ = self.release_inner();
    }
}

#[derive(Debug)]
pub struct IrqLockGuard<'a> {
    lock: Option<EarlyLockGuard<'a>>,
    irq: Option<InterruptGuard<'a>>,
}

impl IrqLockGuard<'_> {
    pub fn release(mut self) -> Result<(), SyncError> {
        self.release_inner()
    }

    fn release_inner(&mut self) -> Result<(), SyncError> {
        if self.lock.is_none() || self.irq.is_none() {
            return Err(SyncError::NotLocked);
        }
        let _ = self.lock.take();
        let _ = self.irq.take();
        Ok(())
    }
}

impl Drop for IrqLockGuard<'_> {
    fn drop(&mut self) {
        let _ = self.release_inner();
    }
}

#[cfg(test)]
mod tests {
    use super::{EarlyLock, LocalInterruptMask, LockOrderTracker, LockRank, SyncError};

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
        let guard = match lock.try_lock_irq(&interrupts) {
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
}
