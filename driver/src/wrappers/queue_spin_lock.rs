//! RAII wrapper for a queued spin lock.

use core::mem::MaybeUninit;
use wdk_sys::{KSPIN_LOCK, KLOCK_QUEUE_HANDLE};
use wdk_sys::ntddk::{
    KeAcquireInStackQueuedSpinLock, KeReleaseInStackQueuedSpinLock,
};

/// RAII guard for a queued spin lock.
/// This guard calls KeAcquireInStackQueuedSpinLock on creation and
/// automatically releases the lock with KeReleaseInStackQueuedSpinLock when dropped.
pub struct QueuedSpinLockGuard<'a> {
    // A reference to the spin lock being protected.
    lock: &'a mut KSPIN_LOCK,
    // The queue handle used for the lock; its lifetime is tied to the guard.
    lock_handle: KLOCK_QUEUE_HANDLE,
}

impl<'a> QueuedSpinLockGuard<'a> {
    /// Acquires the queued spin lock.
    ///
    /// # Safety
    ///
    /// Must be called in a context where it is safe to acquire a spin lock.
    pub unsafe fn new(lock: &'a mut KSPIN_LOCK) -> Self {
        // Initialize an uninitialized KLOCK_QUEUE_HANDLE.
        let mut lock_handle = MaybeUninit::<KLOCK_QUEUE_HANDLE>::uninit();
        KeAcquireInStackQueuedSpinLock(lock, lock_handle.as_mut_ptr());
        let lock_handle = lock_handle.assume_init();
        Self { lock, lock_handle }
    }
}

impl<'a> Drop for QueuedSpinLockGuard<'a> {
    fn drop(&mut self) {
        unsafe {
            KeReleaseInStackQueuedSpinLock(&mut self.lock_handle);
        }
    }
}
