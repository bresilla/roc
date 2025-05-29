//! Rust bindings for ROS 2 RCL (Robot Control Library) and RMW (ROS Middleware)
//! 
//! This crate provides low-level FFI bindings to ROS 2's RCL and RMW libraries.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

// Include the generated bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// A simple wrapper for RCL context 
pub struct RclContext {
    context: rcl_context_t,
}

impl RclContext {
    /// Create a new uninitialized RCL context
    pub fn new() -> Self {
        unsafe {
            Self {
                context: rcl_get_zero_initialized_context(),
            }
        }
    }
}

/// A simple wrapper for RMW init options
pub struct RmwInitOptions {
    init_options: rmw_init_options_t,
}

impl RmwInitOptions {
    /// Create a new uninitialized RMW init options
    pub fn new() -> Self {
        unsafe {
            Self {
                init_options: rmw_get_zero_initialized_init_options(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let _context = RclContext::new();
        // Just test that we can create a context without crashing
    }

    #[test]
    fn test_rmw_init_options_creation() {
        let _init_options = RmwInitOptions::new();
        // Just test that we can create RMW init options without crashing
    }

    #[test]
    fn test_basic_ffi_functions() {
        // Test that we can call basic RCL functions
        unsafe {
            let _context = rcl_get_zero_initialized_context();
            let _init_options = rcl_get_zero_initialized_init_options();
        }
    }

    #[test]
    fn test_basic_rmw_ffi_functions() {
        // Test that we can call basic RMW functions
        unsafe {
            let _rmw_init_options = rmw_get_zero_initialized_init_options();
        }
    }
}
