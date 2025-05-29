# rclrs - Rust Bindings for ROS 2 RCL and RMW

This crate provides low-level FFI (Foreign Function Interface) bindings to ROS 2's RCL (Robot Control Library) and RMW (ROS Middleware) libraries.

## Features

- **RCL Bindings**: Basic bindings to ROS 2's Robot Control Library
- **RMW Bindings**: Basic bindings to ROS 2's Middleware abstraction layer
- **Zero-cost abstractions**: Minimal Rust wrappers around C types
- **Safe initialization**: Rust-friendly initialization patterns

## Current Status

This is a **very basic** implementation that includes only the most fundamental RCL and RMW functions:

### RCL Functions
- `rcl_get_zero_initialized_context()`
- `rcl_get_default_allocator()`
- `rcl_get_zero_initialized_init_options()`
- `rcl_init()`
- `rcl_shutdown()`
- `rcl_context_is_valid()`

### RMW Functions
- `rmw_get_zero_initialized_init_options()`
- `rmw_init()`
- `rmw_shutdown()`

### Types
- `rcl_context_t`
- `rcl_allocator_t`
- `rcl_init_options_t`
- `rmw_init_options_t`
- `rmw_context_t`
- `rmw_allocator_t`
- `rmw_ret_t`

## Requirements

- ROS 2 Jazzy installed at `/opt/ros/jazzy`
- librcl and librmw libraries available
- bindgen for generating FFI bindings

## Building

```bash
cargo build
```

## Testing

```bash
cargo test
```

## Examples

```bash
cargo run --example basic_rcl
```

## Usage

```rust
use rclrs::{RclContext, RmwInitOptions};

fn main() {
    // Create basic RCL and RMW structures
    let context = RclContext::new();
    let rmw_init_options = RmwInitOptions::new();
    
    // Use raw FFI functions for advanced operations
    unsafe {
        let zero_context = rclrs::rcl_get_zero_initialized_context();
        // ... more advanced usage
    }
}
```

## Future Plans

This is just the beginning! Future iterations will include:

- More comprehensive RCL function bindings
- Publisher and Subscriber bindings
- Service and Action client/server bindings
- Parameter handling
- Higher-level safe Rust abstractions
- Integration with async/await patterns

## Architecture

The crate is structured as follows:
- `build.rs`: Bindgen configuration for generating FFI bindings
- `wrapper.h`: C header file specifying which headers to bind
- `src/lib.rs`: Rust wrapper types and re-exports of generated bindings

## Contributing

This is part of the larger ROCC (Robot Operations Command Center) project. Contributions are welcome!
