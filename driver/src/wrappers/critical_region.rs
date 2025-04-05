//! RAII wrapper for critical and guarded regions.

use wdk_sys::ntddk::{KeEnterCriticalRegion, KeLeaveCriticalRegion};

/// RAII guard for a critical region.
/// On creation, it calls KeEnterCriticalRegion, and on drop it calls KeLeaveCriticalRegion.
pub struct CriticalRegionGuard;

impl CriticalRegionGuard {
    /// Enters a critical region.
    ///
    /// # Safety
    ///
    /// Entering a critical region affects APC delivery, so it must be used with care.
    pub unsafe fn new() -> Self {
        KeEnterCriticalRegion();
        CriticalRegionGuard
    }
}

impl Drop for CriticalRegionGuard {
    fn drop(&mut self) {
        unsafe {
            KeLeaveCriticalRegion();
        }
    }
}
