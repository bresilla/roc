use rclrs::*;
use std::ptr;
use anyhow::{Result, anyhow};

/// A simple RCL context manager for graph operations  
pub struct RclGraphContext {
    context: rcl_context_t,
    is_initialized: bool,
}

impl RclGraphContext {
    /// Create a new RCL context for graph operations
    pub fn new() -> Result<Self> {
        unsafe {
            // For now, just create a zero-initialized context
            // We'll expand this step by step as we add more functions to rclrs
            let context = rcl_get_zero_initialized_context();

            Ok(RclGraphContext {
                context,
                is_initialized: false, // Set to false until we can properly initialize
            })
        }
    }

    /// Check if the context is valid
    pub fn is_valid(&self) -> bool {
        if !self.is_initialized {
            return false;
        }
        unsafe {
            rcl_context_is_valid(&self.context)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rcl_context_creation() {
        let context = RclGraphContext::new();
        assert!(context.is_ok());
        
        // For now, just test that we can create the context
        // We'll add proper initialization tests as we expand the bindings
    }
}
