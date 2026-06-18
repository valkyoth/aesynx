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
    CoreTopology = 25,
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
    poisoned: Cell<bool>,
}

impl LockOrderTracker {
    pub const fn new() -> Self {
        Self {
            current: Cell::new(None),
            depth: Cell::new(0),
            poisoned: Cell::new(false),
        }
    }

    pub fn try_enter(&self, rank: LockRank) -> Result<LockOrderGuard<'_>, SyncError> {
        if self.poisoned.get() {
            return Err(SyncError::LockOrderViolation);
        }

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

    #[must_use]
    pub fn is_poisoned(&self) -> bool {
        self.poisoned.get()
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

        let expected_depth = match self.previous_depth.checked_add(1) {
            Some(depth) => depth,
            None => {
                self.tracker.poisoned.set(true);
                self.active = false;
                return Err(SyncError::LockDepthOverflow);
            }
        };
        if self.tracker.poisoned.get() || self.tracker.depth.get() != expected_depth {
            self.tracker.poisoned.set(true);
            self.active = false;
            return Err(SyncError::LockOrderViolation);
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
/// Local model of interrupt masking state used by host tests and early
/// synchronization policy.
///
/// This type does not execute architecture interrupt instructions. Kernel code
/// that needs hardware IRQ masking must pair this model with an arch-backed
/// guard that actually disables interrupts on the owning core.
pub struct LocalInterruptMask {
    enabled: Cell<bool>,
    depth: Cell<u8>,
    poisoned: Cell<bool>,
}

impl LocalInterruptMask {
    pub const fn new_enabled() -> Self {
        Self {
            enabled: Cell::new(true),
            depth: Cell::new(0),
            poisoned: Cell::new(false),
        }
    }

    pub const fn new_disabled() -> Self {
        Self {
            enabled: Cell::new(false),
            depth: Cell::new(0),
            poisoned: Cell::new(false),
        }
    }

    pub fn mask(&self) -> Result<InterruptGuard<'_>, SyncError> {
        if self.poisoned.get() {
            return Err(SyncError::LockOrderViolation);
        }

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

    #[must_use]
    pub fn is_poisoned(&self) -> bool {
        self.poisoned.get()
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

        let expected_depth = match self.previous_depth.checked_add(1) {
            Some(depth) => depth,
            None => {
                self.mask.poisoned.set(true);
                self.active = false;
                return Err(SyncError::LockDepthOverflow);
            }
        };
        if self.mask.poisoned.get() || self.mask.depth.get() != expected_depth {
            self.mask.poisoned.set(true);
            self.active = false;
            return Err(SyncError::LockOrderViolation);
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

/// Evidence that hardware IRQ delivery is disabled on the current core.
///
/// The current constructor is intentionally named as a single-core model/smoke
/// escape hatch. It must not be treated as a production architecture proof; the
/// x86_64 integration needs to replace this with a real IF/CLI-backed token
/// before IRQ locks protect hardware interrupt handlers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArchIrqDisableProof {
    model_only: bool,
}

impl ArchIrqDisableProof {
    #[must_use]
    pub const fn model_only_for_single_core_smoke() -> Self {
        Self { model_only: true }
    }

    #[must_use]
    pub const fn is_model_only(self) -> bool {
        self.model_only
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

    /// Takes the lock while updating the local software interrupt model.
    ///
    /// This is not, by itself, a hardware IRQ-safe lock. `LocalInterruptMask`
    /// records the policy state only; an architecture integration must provide
    /// the real interrupt-disable proof before this pattern is used around
    /// interrupt handlers.
    pub fn try_lock_irq<'a>(
        &'a self,
        interrupts: &'a LocalInterruptMask,
        _proof: ArchIrqDisableProof,
    ) -> Result<IrqLockGuard<'a>, SyncError> {
        let irq = interrupts.mask()?;
        let lock = self.try_lock()?;
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
        let lock = self.try_lock()?;
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
mod tests;
