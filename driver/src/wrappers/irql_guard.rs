//! Module providing an RAII guard for IRQL elevation.
//!
//! When an instance is created, the guard raises the current IRQL to the target level,
//! and when the instance goes out of scope, it automatically lowers the IRQL to its original level.

use wdk_sys::ntddk::{KfRaiseIrql as KeRaiseIrql, KeLowerIrql};
use wdk_sys::KIRQL;

pub struct IrqlGuard {
    old_irql: KIRQL,
}

impl IrqlGuard {
    /// Raises the IRQL to the specified target level and returns an RAII guard.
    ///
    /// # Safety
    ///
    /// This function is unsafe because raising the IRQL is a critical operation that must be
    /// done in a controlled manner. Incorrect usage may cause system instability.
    pub unsafe fn new(target_irql: KIRQL) -> Self {
        let mut old_irql: KIRQL = 0;
        old_irql = KeRaiseIrql(target_irql);
        IrqlGuard { old_irql }
    }
}

impl Drop for IrqlGuard {
    fn drop(&mut self) {
        // Optional: Under a debug flag, you could log a message here.
        // However, keep in mind that debug printing in kernel mode can affect performance and behavior.
        unsafe {
            KeLowerIrql(self.old_irql);
        }
    }
}
