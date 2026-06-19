use aesynx_sync::{
    ArchIrqDisableProof, EarlyLock, LocalInterruptMask, LockOrderTracker, LockRank, SyncError,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConcurrencySmokeStatus {
    pub irq_guard_ok: bool,
    pub nested_irq_guard_ok: bool,
    pub early_lock_ok: bool,
    pub irq_lock_ok: bool,
    pub lock_order_ok: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConcurrencySmokeError {
    Sync(SyncError),
    ContractFailed,
}

pub fn run() -> Result<ConcurrencySmokeStatus, ConcurrencySmokeError> {
    let irq_guard_ok = irq_guard_smoke()?;
    let nested_irq_guard_ok = nested_irq_guard_smoke()?;
    let early_lock_ok = early_lock_smoke()?;
    let irq_lock_ok = irq_lock_smoke()?;
    let lock_order_ok = lock_order_smoke()?;

    if !(irq_guard_ok && nested_irq_guard_ok && early_lock_ok && irq_lock_ok && lock_order_ok) {
        return Err(ConcurrencySmokeError::ContractFailed);
    }

    Ok(ConcurrencySmokeStatus {
        irq_guard_ok,
        nested_irq_guard_ok,
        early_lock_ok,
        irq_lock_ok,
        lock_order_ok,
    })
}

fn irq_guard_smoke() -> Result<bool, ConcurrencySmokeError> {
    let interrupts = LocalInterruptMask::new_enabled();
    {
        let guard = interrupts.mask().map_err(ConcurrencySmokeError::Sync)?;
        if !guard.snapshot().interrupts_were_enabled() || interrupts.interrupts_enabled() {
            return Ok(false);
        }
    }
    Ok(interrupts.interrupts_enabled() && interrupts.depth() == 0)
}

fn nested_irq_guard_smoke() -> Result<bool, ConcurrencySmokeError> {
    let interrupts = LocalInterruptMask::new_enabled();
    {
        let outer = interrupts.mask().map_err(ConcurrencySmokeError::Sync)?;
        let inner = interrupts.mask().map_err(ConcurrencySmokeError::Sync)?;
        if !outer.snapshot().interrupts_were_enabled()
            || inner.snapshot().interrupts_were_enabled()
            || interrupts.interrupts_enabled()
            || interrupts.depth() != 2
        {
            return Ok(false);
        }
        drop(inner);
        if interrupts.interrupts_enabled() || interrupts.depth() != 1 {
            return Ok(false);
        }
    }
    Ok(interrupts.interrupts_enabled() && interrupts.depth() == 0)
}

fn early_lock_smoke() -> Result<bool, ConcurrencySmokeError> {
    let lock = EarlyLock::new();
    let guard = lock.try_lock().map_err(ConcurrencySmokeError::Sync)?;
    if !lock.is_locked() || !matches!(lock.try_lock(), Err(SyncError::AlreadyLocked)) {
        return Ok(false);
    }
    guard.release().map_err(ConcurrencySmokeError::Sync)?;
    Ok(!lock.is_locked() && lock.try_lock().is_ok())
}

fn irq_lock_smoke() -> Result<bool, ConcurrencySmokeError> {
    let lock = EarlyLock::new();
    let interrupts = LocalInterruptMask::new_enabled();
    let proof = ArchIrqDisableProof::model_only_for_single_core_smoke();
    let guard = lock
        .try_lock_irq(&interrupts, proof)
        .map_err(ConcurrencySmokeError::Sync)?;
    if !lock.is_locked() || interrupts.interrupts_enabled() {
        return Ok(false);
    }
    guard.release().map_err(ConcurrencySmokeError::Sync)?;
    Ok(!lock.is_locked() && interrupts.interrupts_enabled())
}

fn lock_order_smoke() -> Result<bool, ConcurrencySmokeError> {
    let tracker = LockOrderTracker::new();
    {
        let descriptor = tracker
            .try_enter(LockRank::DescriptorTables)
            .map_err(ConcurrencySmokeError::Sync)?;
        if !matches!(
            tracker.try_enter(LockRank::InterruptController),
            Err(SyncError::LockOrderViolation)
        ) {
            return Ok(false);
        }
        let heap = tracker
            .try_enter(LockRank::KernelHeap)
            .map_err(ConcurrencySmokeError::Sync)?;
        if tracker.depth() != 2 || tracker.current_rank() != Some(LockRank::KernelHeap) {
            return Ok(false);
        }
        drop(heap);
        if tracker.current_rank() != Some(LockRank::DescriptorTables) {
            return Ok(false);
        }
        drop(descriptor);
    }
    Ok(tracker.depth() == 0 && tracker.current_rank().is_none())
}

#[cfg(test)]
mod tests {
    use super::run;

    #[test]
    fn concurrency_smoke_validates_early_contracts() {
        let status = match run() {
            Ok(status) => status,
            Err(error) => return assert_eq!(format!("{error:?}"), ""),
        };

        assert!(status.irq_guard_ok);
        assert!(status.nested_irq_guard_ok);
        assert!(status.early_lock_ok);
        assert!(status.irq_lock_ok);
        assert!(status.lock_order_ok);
    }
}
