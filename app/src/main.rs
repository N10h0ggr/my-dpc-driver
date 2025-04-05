use std::ffi::c_void;
use std::mem::size_of;
use windows::{
    core::{Error, PCWSTR, Result},
    Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE},
    Win32::Storage::FileSystem::{
        CreateFileW, OPEN_EXISTING, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_MODE, FILE_FLAGS_AND_ATTRIBUTES,
    },
    Win32::System::IO::DeviceIoControl,
};
use shared::IOCTL_GET_COUNTER;

fn main() -> Result<()> {
    // Convert the device name to a null-terminated wide string (UTF-16).
    let device_name_vec: Vec<u16> = "\\\\.\\RustDriver\0".encode_utf16().collect();
    let device_name = PCWSTR(device_name_vec.as_ptr());

    unsafe {
        // Open the device.
        let h_device: HANDLE = CreateFileW(
            device_name,
            FILE_GENERIC_READ.0 | FILE_GENERIC_WRITE.0, // Cast FILE_ACCESS_RIGHTS to u32.
            FILE_SHARE_MODE(0),                           // No sharing.
            None,                                         // No security attributes.
            OPEN_EXISTING,                                // Open existing device.
            FILE_FLAGS_AND_ATTRIBUTES(0),                 // Default flags.
            None,                                         // No template file.
        )?;

        if h_device == INVALID_HANDLE_VALUE {
            eprintln!("Error opening device: {:?}", Error::from_win32());
            return Err(Error::from_win32());
        }

        let mut counter: u32 = 0;
        let mut bytes_returned: u32 = 0;

        // Send the IOCTL to the driver.
        DeviceIoControl(
            h_device,
            IOCTL_GET_COUNTER,
            None,                                         // No input buffer.
            0,                                            // Size of input buffer.
            Some(&mut counter as *mut _ as *mut c_void),    // Output buffer for the counter.
            size_of::<u32>() as u32,                        // Size of output buffer.
            Some(&mut bytes_returned),                      // Pointer to receive number of bytes returned.
            None,                                         // No OVERLAPPED structure.
        )?;

        println!("Counter value: {}", counter);
        CloseHandle(h_device);
    }

    Ok(())
}
