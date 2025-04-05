// spinlock_wrapper.c
//
// This file contains C wrappers for the Inline Windows functions
// because the Inline Windows functions are not exported by the kernel
// and cannot be used directly in the driver code. 

#include <ntddk.h>

void my_KeAcquireSpinLock(KSPIN_LOCK *SpinLock, KIRQL *OldIrql) {
    KeAcquireSpinLock(SpinLock, OldIrql);
}

PVOID my_GetMdlAddressWrapper(PMDL Mdl) {
    return MmGetSystemAddressForMdlSafe(Mdl, NormalPagePriority);
}
