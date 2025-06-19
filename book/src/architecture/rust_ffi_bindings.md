# Rust FFI Bindings

Creating effective Rust bindings for ROS 2's C libraries requires careful handling of Foreign Function Interface (FFI) concepts, memory management, and type safety.

## Overview of Our Binding Strategy

The `roc` project uses a custom FFI binding approach located in the `rclrs/` subdirectory. This provides direct access to RCL and RMW functions without the overhead of higher-level abstractions.

## Project Structure

```
rclrs/
├── build.rs              # Build script for bindgen
├── Cargo.toml             # Crate configuration
├── wrapper.h              # C header wrapper
└── src/
    └── lib.rs             # Rust bindings and wrappers
```

## Build System (`build.rs`)

Our build script uses `bindgen` to automatically generate Rust bindings from C headers:

```rust
use bindgen;
use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to look for ROS 2 installation
    println!("cargo:rustc-link-search=native=/opt/ros/jazzy/lib");
    
    // Link against RCL libraries
    println!("cargo:rustc-link-lib=rcl");
    println!("cargo:rustc-link-lib=rmw");
    println!("cargo:rustc-link-lib=rcutils");
    
    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-I/opt/ros/jazzy/include")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
```

## Header Wrapper (`wrapper.h`)

We create a minimal wrapper that includes only the headers we need:

```c
#ifndef WRAPPER_H
#define WRAPPER_H

// Core RCL headers
#include "rcl/rcl/allocator.h"
#include "rcl/rcl/context.h"
#include "rcl/rcl/graph.h"
#include "rcl/rcl/init.h"
#include "rcl/rcl/init_options.h"
#include "rcl/rcl/node.h"

// RMW headers for detailed topic information
#include "rmw/rmw/allocators.h"
#include "rmw/rmw/init.h"
#include "rmw/rmw/init_options.h"
#include "rmw/rmw/ret_types.h"
#include "rmw/rmw/types.h"
#include "rmw/rmw/topic_endpoint_info.h"

#endif // WRAPPER_H
```

This selective inclusion keeps compilation fast and only exposes the APIs we actually use.

## Generated Bindings

The `bindgen` tool generates Rust equivalents for C types and functions:

### C Structs → Rust Structs
```rust
// C: rmw_topic_endpoint_info_t
#[repr(C)]
pub struct rmw_topic_endpoint_info_s {
    pub node_name: *const ::std::os::raw::c_char,
    pub node_namespace: *const ::std::os::raw::c_char,
    pub topic_type: *const ::std::os::raw::c_char,
    pub topic_type_hash: rosidl_type_hash_t,
    pub endpoint_type: rmw_endpoint_type_t,
    pub endpoint_gid: [u8; 16usize],
    pub qos_profile: rmw_qos_profile_t,
}
```

### C Enums → Rust Constants
```rust
// C: rmw_endpoint_type_e
pub const rmw_endpoint_type_e_RMW_ENDPOINT_INVALID: rmw_endpoint_type_e = 0;
pub const rmw_endpoint_type_e_RMW_ENDPOINT_PUBLISHER: rmw_endpoint_type_e = 1;
pub const rmw_endpoint_type_e_RMW_ENDPOINT_SUBSCRIPTION: rmw_endpoint_type_e = 2;
pub type rmw_endpoint_type_e = ::std::os::raw::c_uint;
```

### C Functions → Rust Extern Functions
```rust
extern "C" {
    pub fn rcl_get_publishers_info_by_topic(
        node: *const rcl_node_t,
        allocator: *mut rcutils_allocator_t,
        topic_name: *const ::std::os::raw::c_char,
        no_mangle: bool,
        publishers_info: *mut rcl_topic_endpoint_info_array_t,
    ) -> rcl_ret_t;
}
```

## Safe Rust Wrappers

Our implementation wraps the raw FFI with safe Rust abstractions:

### String Handling
```rust
// Convert C strings to Rust strings safely
let node_name = if info.node_name.is_null() {
    "unknown".to_string()
} else {
    std::ffi::CStr::from_ptr(info.node_name)
        .to_string_lossy()
        .to_string()
};
```

### Error Handling
```rust
// Convert C return codes to Rust Results
let ret = rcl_get_publishers_info_by_topic(
    &self.node,
    &mut allocator,
    topic_name_c.as_ptr(),
    false,
    &mut publishers_info,
);

if ret != 0 {
    return Err(anyhow!("Failed to get publishers info: {}", ret));
}
```

### Memory Management
```rust
// Ensure proper cleanup with RAII
unsafe {
    let mut allocator = rcutils_get_default_allocator();
    let mut publishers_info: rcl_topic_endpoint_info_array_t = std::mem::zeroed();
    
    // ... use the data ...
    
    // Automatic cleanup when leaving scope
    rmw_topic_endpoint_info_array_fini(&mut publishers_info, &mut allocator);
}
```

## Type Conversions

We provide safe conversions between C types and idiomatic Rust types:

### Enum Conversions
```rust
impl EndpointType {
    fn from_rmw(endpoint_type: rmw_endpoint_type_t) -> Self {
        match endpoint_type {
            rmw_endpoint_type_e_RMW_ENDPOINT_PUBLISHER => EndpointType::Publisher,
            rmw_endpoint_type_e_RMW_ENDPOINT_SUBSCRIPTION => EndpointType::Subscription,
            _ => EndpointType::Invalid,
        }
    }
}
```

### Complex Structure Conversions
```rust
impl QosProfile {
    fn from_rmw(qos: &rmw_qos_profile_t) -> Self {
        QosProfile {
            history: QosHistoryPolicy::from_rmw(qos.history),
            depth: qos.depth,
            reliability: QosReliabilityPolicy::from_rmw(qos.reliability),
            durability: QosDurabilityPolicy::from_rmw(qos.durability),
            deadline_sec: qos.deadline.sec,
            deadline_nsec: qos.deadline.nsec,
            // ... other fields
        }
    }
}
```

## Challenges and Solutions

### 1. Null Pointer Handling
**Challenge**: C APIs can return null pointers
**Solution**: Check for null before dereferencing
```rust
let topic_type = if info.topic_type.is_null() {
    "unknown".to_string()
} else {
    std::ffi::CStr::from_ptr(info.topic_type).to_string_lossy().to_string()
};
```

### 2. Memory Ownership
**Challenge**: Complex ownership semantics between C and Rust
**Solution**: Clear ownership boundaries and explicit cleanup
```rust
// C owns the memory in the array, we just read it
let gid = std::slice::from_raw_parts(
    info.endpoint_gid.as_ptr(), 
    info.endpoint_gid.len()
).to_vec(); // Copy to Rust-owned Vec
```

### 3. Type Size Mismatches
**Challenge**: C `int` vs Rust `i32` vs `c_int`
**Solution**: Use `std::os::raw` types consistently
```rust
use std::os::raw::{c_char, c_int, c_uint};
```

### 4. Array Handling
**Challenge**: C arrays with separate size fields
**Solution**: Safe iteration with bounds checking
```rust
for i in 0..publishers_info.size {
    let info = &*(publishers_info.info_array.add(i));
    // ... process info safely
}
```

## Testing FFI Code

FFI code requires careful testing:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let context = RclGraphContext::new();
        assert!(context.is_ok());
    }

    #[test]
    fn test_topic_discovery() {
        let context = RclGraphContext::new().unwrap();
        let topics = context.get_topic_names();
        assert!(topics.is_ok());
    }
}
```

## Performance Considerations

1. **Minimize FFI Calls**: Batch operations when possible
2. **Avoid String Conversions**: Cache converted strings
3. **Memory Locality**: Process data in the order it's laid out in memory
4. **Error Path Optimization**: Fast paths for common success cases

This FFI design provides the foundation for `roc`'s powerful introspection capabilities while maintaining safety and performance.
