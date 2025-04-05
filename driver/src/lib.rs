#![no_std]
#![no_main]

use core::mem::{size_of, MaybeUninit};
use wdk::println;

// Import allocator and panic handler.
use wdk_alloc::WdkAllocator;
#[global_allocator]
static ALLOCATOR: WdkAllocator = WdkAllocator;

extern crate alloc;
use alloc::vec::Vec;
#[cfg(not(test))]
extern crate wdk_panic;

// Import necessary functions and types from ntddk.
use wdk_sys::ntddk::{
    IoCreateDevice, IoCreateSymbolicLink, IoDeleteSymbolicLink, IoDeleteDevice,
    IofCompleteRequest, KeCancelTimer, KeFlushQueuedDpcs, KeInitializeDpc,
    KeInitializeTimer, KeSetTimerEx,
};

use wdk_sys::{
    DEVICE_OBJECT, DRIVER_OBJECT, IRP, IRP_MJ_CREATE, IRP_MJ_CLOSE, IRP_MJ_DEVICE_CONTROL,
    IO_NO_INCREMENT, FILE_DEVICE_UNKNOWN, STATUS_SUCCESS, STATUS_BUFFER_TOO_SMALL,
    STATUS_NOT_IMPLEMENTED, STATUS_UNSUCCESSFUL, UNICODE_STRING, LARGE_INTEGER, NTSTATUS, 
    PCUNICODE_STRING, KDPC, KTIMER, DO_BUFFERED_IO,
};


// Import our RAII spin lock wrapper.
mod wrappers;
use wrappers::spin_lock::SpinLock;

mod helpers;
use helpers::init_unicode_string;

// Define our IOCTL code.
const IOCTL_GET_COUNTER: u32 = 0x800; // Example IOCTL code

//
// Device Extension Structure
//
// This structure is allocated per-device and holds our timer, DPC,
// spin lock, and a counter that is updated by the DPC.
#[repr(C)]
pub struct DeviceExtension {
    timer: KTIMER,
    dpc: KDPC,
    spin_lock: SpinLock,
    counter: u32,
}

impl DeviceExtension {
    /// Initializes the device extension fields.
    pub unsafe fn init(&mut self) {
        // Zero initialize timer and DPC.
        self.timer = MaybeUninit::zeroed().assume_init();
        self.dpc = MaybeUninit::zeroed().assume_init();
        // Initialize the spin lock.
        self.spin_lock = SpinLock::new();
        self.spin_lock.init();
        self.counter = 0;
    }
}


/// DPC Callback: Called when the timer expires. This function safely increments the counter.
unsafe extern "C" fn dpc_callback(
    _dpc: *mut KDPC,
    deferred_context: *mut core::ffi::c_void,
    _system_arg1: *mut core::ffi::c_void,
    _system_arg2: *mut core::ffi::c_void,
) {
    let dev_ext = &mut *(deferred_context as *mut DeviceExtension);
    let _guard = dev_ext.spin_lock.lock_at_dpc();
    dev_ext.counter = dev_ext.counter.wrapping_add(1);
}

/// Dispatch routine for IRP_MJ_CREATE and IRP_MJ_CLOSE. Completes the IRP with success.
unsafe extern "C" fn dispatch_create_close(
    _device_object: *mut DEVICE_OBJECT,
    irp: *mut IRP,
) -> NTSTATUS {
    (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*irp).IoStatus.Information = 0;
    IofCompleteRequest(irp, IO_NO_INCREMENT as i8);
    STATUS_SUCCESS
}

/// Dispatch routine for IOCTL requests (IRP_MJ_DEVICE_CONTROL).
///
/// For IOCTL_GET_COUNTER, it safely copies the counter value into the output buffer
unsafe extern "C" fn dispatch_device_control(
    device_object: *mut DEVICE_OBJECT,
    irp: *mut IRP,
) -> NTSTATUS {
    let dev_ext = &mut *((*device_object).DeviceExtension.cast::<DeviceExtension>());
    let current_stack = (*irp).Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation;
    let ioctl_code = (*current_stack).Parameters.DeviceIoControl.IoControlCode;
    let mut status = STATUS_SUCCESS;

    match ioctl_code {
        IOCTL_GET_COUNTER => {
            // Acquire the spin lock to safely read the counter.
            let _guard = dev_ext.spin_lock.lock();
            let counter = dev_ext.counter;
            println!("IOCTL_GET_COUNTER: Counter = {}", counter);

            // Check that the output buffer is large enough.
            let out_buffer_length = (*current_stack).Parameters.DeviceIoControl.OutputBufferLength;
            if out_buffer_length < size_of::<u32>() as u32 {
                status = STATUS_BUFFER_TOO_SMALL;
            } else {
                let out_buffer = (*irp).AssociatedIrp.SystemBuffer as *mut u32;
                if !out_buffer.is_null() {
                    *out_buffer = counter;
                    (*irp).IoStatus.Information = size_of::<u32>() as u64;
                } else {
                    status = STATUS_UNSUCCESSFUL;
                }
            }
        },
        _ => {
            status = STATUS_NOT_IMPLEMENTED;
        },
    }

    IofCompleteRequest(irp, 0);
    status
}

/// DriverEntry: Initializes the driver, creates the device and symbolic link,
/// and sets up the device extension, timer, and DPC.
#[export_name = "DriverEntry"]
pub unsafe extern "C" fn driver_entry(
    driver_object: *mut DRIVER_OBJECT,
    _registry_path: PCUNICODE_STRING,
) -> NTSTATUS {
    println!("DriverEntry: Rust Driver starting");

    // Set the unload routine and dispatch routines.
    (*driver_object).DriverUnload = Some(driver_unload);
    (*driver_object).MajorFunction[IRP_MJ_CREATE as usize] = Some(dispatch_create_close);
    (*driver_object).MajorFunction[IRP_MJ_CLOSE as usize] = Some(dispatch_create_close);
    (*driver_object).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] = Some(dispatch_device_control);

    // Initialize the device name and symbolic link.
    let device_name = init_unicode_string("\\Device\\RustDriver");
    let sym_link = init_unicode_string("\\??\\RustDriver");

    let mut device_object: *mut DEVICE_OBJECT = core::ptr::null_mut();
    let status = IoCreateDevice(
        driver_object,
        size_of::<DeviceExtension>() as u32,
        &device_name as *const UNICODE_STRING as *mut _,
        FILE_DEVICE_UNKNOWN,
        0,
        0,
        &mut device_object,
    );

    (*device_object).Flags |= DO_BUFFERED_IO;

    if status != STATUS_SUCCESS {
        println!("DriverEntry: Failed to create device: {:#x}", status);
        return status;
    }
    
    let status = IoCreateSymbolicLink(
        &sym_link as *const UNICODE_STRING as *mut _,
        &device_name as *const UNICODE_STRING as *mut _,
    );
    if status != STATUS_SUCCESS {
        println!("DriverEntry: Failed to create symbolic link: {:#x}", status);
        IoDeleteDevice(device_object);
        return status;
    }

    // Initialize the device extension.
    let dev_ext: &mut DeviceExtension =
        &mut *((*device_object).DeviceExtension.cast::<DeviceExtension>());
    dev_ext.init();

    // Initialize timer and DPC.
    KeInitializeTimer(&mut dev_ext.timer);
    KeInitializeDpc(&mut dev_ext.dpc, Some(dpc_callback), dev_ext as *mut _ as *mut _);

    // Set a periodic timer: due in 1 second, then every 1000 milliseconds.
    let due_time = LARGE_INTEGER { QuadPart: -10_000_000 };
    KeSetTimerEx(&mut dev_ext.timer, due_time, 1000i32, &mut dev_ext.dpc);

    println!("DriverEntry: Device, timer, and DPC initialized.");

    STATUS_SUCCESS
}

/// Driver unload: Cancels the timer, flushes queued DPCs, deletes the symbolic link, and deletes the device.
extern "C" fn driver_unload(driver: *mut DRIVER_OBJECT) {
    unsafe {
        println!("DriverUnload: Unloading driver.");

        let device_object = (*driver).DeviceObject;
        if !device_object.is_null() {
            // Retrieve the device extension.
            let dev_ext: &mut DeviceExtension =
                &mut *((*device_object).DeviceExtension.cast::<DeviceExtension>());
            // Cancel the timer and flush queued DPCs.
            KeCancelTimer(&mut dev_ext.timer);
            KeFlushQueuedDpcs();

            // Delete the symbolic link.
            let sym_link = init_unicode_string("\\??\\RustDriver");
            let _ = IoDeleteSymbolicLink(&sym_link as *const UNICODE_STRING as *mut _);

            // Delete the device object.
            IoDeleteDevice(device_object);
        }
    }
}
