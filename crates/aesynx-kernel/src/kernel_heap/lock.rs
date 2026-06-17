use core::sync::atomic::{AtomicBool, Ordering};

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
use aesynx_arch::ArchCpu;

use super::stats::KernelHeapError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum HeapLockError {
    ReentrantLock,
}

pub(super) struct HeapLockGuard<'a> {
    locked: &'a AtomicBool,
    interrupts_were_enabled: bool,
}

impl<'a> HeapLockGuard<'a> {
    pub(super) fn lock(locked: &'a AtomicBool) -> Result<Self, HeapLockError> {
        let interrupts_were_enabled = mask_interrupts_for_heap_lock();
        if locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            restore_interrupts_after_heap_lock(interrupts_were_enabled);
            return Err(HeapLockError::ReentrantLock);
        }

        Ok(Self {
            locked,
            interrupts_were_enabled,
        })
    }
}

impl Drop for HeapLockGuard<'_> {
    fn drop(&mut self) {
        self.locked.store(false, Ordering::Release);
        restore_interrupts_after_heap_lock(self.interrupts_were_enabled);
    }
}

fn mask_interrupts_for_heap_lock() -> bool {
    #[cfg(all(target_arch = "x86_64", target_os = "none"))]
    {
        let interrupts_were_enabled =
            aesynx_arch_x86_64::X86_64::interrupts_enabled().unwrap_or(false);
        let _ = aesynx_arch_x86_64::X86_64::disable_interrupts();
        interrupts_were_enabled
    }

    #[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
    {
        false
    }
}

fn restore_interrupts_after_heap_lock(interrupts_were_enabled: bool) {
    #[cfg(all(target_arch = "x86_64", target_os = "none"))]
    {
        if interrupts_were_enabled {
            let _ = aesynx_arch_x86_64::X86_64::enable_interrupts();
        }
    }

    #[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
    {
        let _ = interrupts_were_enabled;
    }
}

impl From<HeapLockError> for KernelHeapError {
    fn from(error: HeapLockError) -> Self {
        match error {
            HeapLockError::ReentrantLock => Self::ReentrantLock,
        }
    }
}
