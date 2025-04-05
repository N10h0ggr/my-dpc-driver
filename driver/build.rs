//! Build script for the `my-dpc-driver` crate.
//!
//! This build script configures the build process for the driver binary,
//! including setting up linker flags, compiling C source files, and
//! configuring the Windows Driver Kit (WDK) environment.

use glob::glob;
use std::env;
use std::path::PathBuf;

fn get_windows_sdk_km_include_path() -> PathBuf {
    let sdk_dir = env::var("WindowsSdkDir")
        .unwrap_or_else(|_| "C:/Program Files (x86)/Windows Kits/10/".into());

    let sdk_version_raw = env::var("WindowsSDKVersion")
        .unwrap_or_else(|_| "10.0.26100.0".into());
    let sdk_version = sdk_version_raw.trim_end_matches('\\');

    let mut include_path = PathBuf::from(sdk_dir);
    include_path.push("Include");
    include_path.push(sdk_version);
    include_path.push("km");
    include_path
}

fn main() -> Result<(), wdk_build::ConfigError> {
    println!("cargo:rerun-if-env-changed=WindowsSdkDir");
    println!("cargo:rerun-if-env-changed=WindowsSDKVersion");

    let mut build = cc::Build::new();

    // Define target architecture explicitly
    build.define("_AMD64_", None);

    // Include Windows kernel-mode headers
    let include_path = get_windows_sdk_km_include_path();
    println!("cargo:warning=Using WDK include path: {}", include_path.display());
    build.include(include_path);

    // Compile all .c files in c_wrappers/
    for entry in glob("c_wrappers/*.c").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                println!("cargo:rerun-if-changed={}", path.display());
                build.file(path);
            }
            Err(e) => eprintln!("Glob error: {:?}", e),
        }
    }

    build.compile("c_wrappers");

    // Proceed with WDK driver configuration
    wdk_build::configure_wdk_binary_build()
}
