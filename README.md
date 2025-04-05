# DPC Driver

A Windows kernel driver written in Rust using the Windows Rust WDK. This project combines modern Rust safety with traditional kernel development techniques, including IRQL management, Deferred Procedure Calls (DPCs), and safe handling of user buffers. A small C wrapper bridges inline functions provided by the Windows WDK, enabling Rust to call these critical routines.

## Project Structure

- **driver/**  
  Contains the kernel driver code:
  - **src/**: The main driver source files.
  - **wrappers/**: Modules wrapping kernel components (e.g., IRQL guards).
  - **c_wrappers/**: C code that provides bindings for inline functions.
  - **build.rs**: Build script that compiles both Rust and C components.

- **app/**  
  A user-mode application for testing and interacting with the driver.

- **shared/**  
  Contains common code and definitions shared between the driver and the user-mode application.

## Key Concepts

- **IRQL (Interrupt Request Level):**  
  The driver uses an RAII pattern (as seen in [`driver/src/wrappers/irql_guard.rs`](driver/src/wrappers/irql_guard.rs)) to safely raise and lower the IRQL, ensuring critical sections execute without interruption.

- **Deferred Procedure Calls (DPCs):**  
  DPCs enable the driver to schedule non-urgent tasks to be executed at a lower IRQL, thereby keeping high-priority operations responsive.

- **User Buffers:**  
  When handling data from user-mode applications, the driver validates buffers to prevent security issues, ensuring safe data transfers between user and kernel mode.

- **C Wrappers:**  
  Some Windows kernel functions are provided as inline functions in C, which Rust cannot directly call. A C wrapper (found in [`driver/c_wrappers/spinlock_wrapper.c`](driver/c_wrappers/spinlock_wrapper.c)) exposes these inline functions as callable routines, with Cargo Make managing the build process for both Rust and C code.

## Getting Started

### Prerequisites

- A Windows development environment with the Windows Driver Kit (WDK) installed.
- Rust and Cargo installed.
- A C compiler compatible with the WDK (e.g., MSVC).
- [Cargo Make](https://sagiegurari.github.io/cargo-make/) installed. If not, install it with:
  
  ```bash
  cargo install cargo-make
  ```

### Building the Project

This project is built using Cargo Make, which orchestrates the build process for both Rust and C components.

1. **Clone the repository:**

   ```bash
   git clone https://github.com/yourusername/my-dpc-driver.git
   cd my-dpc-driver
   ```

2. **Build the project using Cargo Make:**

   ```bash
   cargo make build
   ```

   This command runs the tasks defined in the `Makefile.toml`, compiling the C wrappers and linking them with the Rust driver code.


### Deploying the Driver

Follow standard Windows driver deployment practices:

- Use Windows Driver Kit tools to sign and install the driver.
- Ensure that you have the correct driver signing certificates.
- Consult the WDK documentation for detailed deployment instructions.

## Usage

This project serves as an educational tool for Windows kernel driver development in Rust. It demonstrates how to:
- Manage IRQL levels using RAII guards.
- Schedule deferred tasks with DPCs.
- Safely interact with user-mode applications by validating user buffers.
- Bridge Rust with essential Windows inline functions via C wrappers.

The accompanying user-mode app in the **app/** folder can be used to send IOCTL commands and interact with the driver for testing purposes.
