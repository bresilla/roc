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
            // Initialize RCL with basic setup
            let mut init_options = rcl_get_zero_initialized_init_options();
            let allocator = rcutils_get_default_allocator();
            
            let ret = rcl_init_options_init(&mut init_options, allocator);
            if ret != 0 {
                return Err(anyhow!("Failed to initialize RCL init options: {}", ret));
            }

            let mut context = rcl_get_zero_initialized_context();
            let ret = rcl_init(0, ptr::null_mut(), &init_options, &mut context);
            if ret != 0 {
                return Err(anyhow!("Failed to initialize RCL: {}", ret));
            }

            Ok(RclGraphContext {
                context,
                is_initialized: true,
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

impl Drop for RclGraphContext {
    fn drop(&mut self) {
        if self.is_initialized {
            unsafe {
                rcl_shutdown(&mut self.context);
            }
            self.is_initialized = false;
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
        
        let context = context.unwrap();
        assert!(context.is_valid());
    }
}
