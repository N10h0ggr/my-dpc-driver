//! RAII wrapper for an executive resource (ERESOURCE).

use core::mem::zeroed;
use wdk_sys::ERESOURCE;
use wdk_sys::ntddk::{
    ExInitializeResourceLite, ExAcquireResourceExclusiveLite, ExReleaseResourceLite,
};

/// Wrapper for an executive resource.
pub struct ExecutiveResource {
    resource: ERESOURCE,
}

impl ExecutiveResource {
    /// Creates and initializes a new executive resource.
    ///
    /// # Safety
    ///
    /// Caller must ensure this resource is used appropriately.
    pub unsafe fn new() -> Self {
        let mut res: ERESOURCE = zeroed();
        let _ = ExInitializeResourceLite(&mut res);
        ExecutiveResource { resource: res }
    }

    /// Acquires the resource exclusively and returns a guard that will release it on drop.
    ///
    /// The `wait` parameter indicates if the call should wait for the resource.
    ///
    /// # Safety
    ///
    /// Caller must ensure that exclusive access is appropriate for the resource.
    pub unsafe fn acquire_exclusive<'a>(&'a mut self, wait: bool) -> ExecutiveResourceGuard<'a> {
        ExAcquireResourceExclusiveLite(&mut self.resource, wait as u8);
        ExecutiveResourceGuard { resource: &mut self.resource }
    }
}

/// RAII guard for an executive resource.
pub struct ExecutiveResourceGuard<'a> {
    resource: &'a mut ERESOURCE,
}

impl<'a> Drop for ExecutiveResourceGuard<'a> {
    fn drop(&mut self) {
        unsafe {
            ExReleaseResourceLite(self.resource);
        }
    }
}
