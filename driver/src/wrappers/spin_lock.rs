//! Module providing an RAII wrapper for a spin lock in kernel mode,
//! using the appropriate API based on the IRQL level.

use core::cell::UnsafeCell;
use core::ops::Deref;
use wdk_sys::{KIRQL, KSPIN_LOCK};

#[link(name = "ntoskrnl")]
extern "C" {
    pub fn KeInitializeSpinLock(lock: *mut KSPIN_LOCK);
    pub fn KeReleaseSpinLock(lock: *mut KSPIN_LOCK, old_irql: KIRQL);
    pub fn KeAcquireSpinLockAtDpcLevel(lock: *mut KSPIN_LOCK);
    pub fn KeReleaseSpinLockFromDpcLevel(lock: *mut KSPIN_LOCK);
} 

// This block does NOT need #[link] because your wrappers were compiled and integrated in build.rs
extern "C" {
    /// Acquires the spin lock and raises the IRQL to DISPATCH_LEVEL.
    ///
    /// # Safety
    /// This function is unsafe as it directly manipulates the IRQL and requires
    /// that the caller is in a context where it is safe to raise the IRQL.
    pub fn my_KeAcquireSpinLock(lock: *mut KSPIN_LOCK, old_irql: *mut KIRQL);
}

pub struct SpinLock {
    // We use UnsafeCell to allow internal mutable access.
    lock: UnsafeCell<KSPIN_LOCK>,
}

unsafe impl Send for SpinLock {}
unsafe impl Sync for SpinLock {}

impl SpinLock {
    /// Creates a new spin lock without initialization.
    pub const fn new() -> Self {
        Self {
            // It is assumed that an uninitialized KSPIN_LOCK can be represented as 0.
            lock: UnsafeCell::new(0 as KSPIN_LOCK),
        }
    }

    /// Initializes the spin lock.
    ///
    /// # Safety
    /// This is unsafe as it calls kernel functions and depends on proper initialization.
    pub unsafe fn init(&self) {
        KeInitializeSpinLock(self.lock.get());
    }

    /// Acquires the spin lock using KeAcquireSpinLock,
    /// which raises the IRQL to DISPATCH_LEVEL and saves the previous IRQL.
    ///
    /// # Safety
    /// Must be called in a context where it is safe to raise the IRQL.
    pub unsafe fn lock(&self) -> SpinLockGuard {
        let mut old_irql: KIRQL = 0;
        my_KeAcquireSpinLock(self.lock.get(), &mut old_irql);
        SpinLockGuard {
            lock: self,
            old_irql,
            level: SpinLockLevel::Dispatch,
        }
    }

    /// Acquires the spin lock when already at DISPATCH_LEVEL (DPC context).
    ///
    /// This function does not raise the IRQL.
    ///
    /// # Safety
    /// Must be called when already at DISPATCH_LEVEL (e.g., within a DPC).
    pub unsafe fn lock_at_dpc(&self) -> SpinLockGuard {
        KeAcquireSpinLockAtDpcLevel(self.lock.get());
        SpinLockGuard {
            lock: self,
            // old_irql is not used in this case, as it is already DISPATCH_LEVEL.
            old_irql: 0,
            level: SpinLockLevel::Dpc,
        }
    }
}

/// Indicates the context in which the spin lock was acquired.
pub enum SpinLockLevel {
    Dispatch,
    Dpc,
}

/// RAII guard for the spin lock. The lock is automatically released when the guard goes out of scope.
pub struct SpinLockGuard<'a> {
    lock: &'a SpinLock,
    old_irql: KIRQL,
    level: SpinLockLevel,
}

impl<'a> Drop for SpinLockGuard<'a> {
    fn drop(&mut self) {
        unsafe {
            match self.level {
                SpinLockLevel::Dispatch => {
                    KeReleaseSpinLock(self.lock.lock.get(), self.old_irql);
                }
                SpinLockLevel::Dpc => {
                    KeReleaseSpinLockFromDpcLevel(self.lock.lock.get());
                }
            }
        }
    }
}

impl<'a> Deref for SpinLockGuard<'a> {
    type Target = SpinLock;
    fn deref(&self) -> &Self::Target {
        self.lock
    }
}
