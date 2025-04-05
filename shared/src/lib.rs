#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "wdk-panic"))]
extern crate wdk_panic;


// Define constants for buffered I/O.
const METHOD_BUFFERED: u32 = 0;
const FILE_ANY_ACCESS: u32 = 0;
const FILE_DEVICE_UNKNOWN: u32 = 22;

/// This macro creates a control code for device I/O operations, similar to the Windows CTL_CODE macro.
/// The control code is a 32-bit value constructed by combining several parameters that specify
/// various characteristics of the I/O control operation.
///
/// The parameters are combined as follows:
///   Bits 31-16: Device Type
///   Bits 15-14: Required Access
///   Bits 13-2:  Function Code
///   Bits 1-0:   Method (buffering mechanism)
///
/// Parameters:
/// - `device_type`: A constant representing the type of device (for example, FILE_DEVICE_UNKNOWN).
///   This value is shifted left by 16 bits to occupy the upper 16 bits of the control code.
/// - `function`: A unique function code for the I/O operation (e.g., 0x800 for IOCTL_GET_COUNTER).
///   It is shifted left by 2 bits to position it in bits 2 to 13 of the control code.
/// - `method`: The buffering method to be used for the I/O operation. For buffered I/O,
///   this is typically `METHOD_BUFFERED` (which is 0). This occupies the lowest 2 bits.
/// - `access`: Specifies the required access permissions (commonly `FILE_ANY_ACCESS` for buffered I/O).
///   This value is shifted left by 14 bits so that it occupies bits 14 and 15.
///
/// The final IOCTL control code is computed by OR-ing together the shifted values.
///
/// Example usage:
/// The following creates an IOCTL code for getting a counter value using buffered I/O:
macro_rules! ctl_code {
    ($device_type:expr, $function:expr, $method:expr, $access:expr) => {
        (($device_type as u32) << 16) |      // Device Type occupies bits 16-31.
        (($access as u32) << 14) |           // Access required occupies bits 14-15.
        (($function as u32) << 2)  |         // Function code occupies bits 2-13.
        ($method as u32)                    // Method occupies bits 0-1.
    };
}

// Create the IOCTL code using buffered I/O.
pub const IOCTL_GET_COUNTER: u32 = ctl_code!(FILE_DEVICE_UNKNOWN, 0x800, METHOD_BUFFERED, FILE_ANY_ACCESS);