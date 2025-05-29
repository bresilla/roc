//! Basic example demonstrating RCL and RMW bindings
//! 
//! This example shows how to use the basic RCL and RMW FFI bindings.

use rclrs::{RclContext, RmwInitOptions};

fn main() {
    println!("Basic RCL and RMW bindings example");
    
    // Create an RCL context
    let context = RclContext::new();
    println!("✓ Created RCL context");
    
    // Create RMW init options
    let rmw_init_options = RmwInitOptions::new();
    println!("✓ Created RMW init options");
    
    // Test basic FFI calls
    unsafe {
        let _zero_context = rclrs::rcl_get_zero_initialized_context();
        println!("✓ Called rcl_get_zero_initialized_context()");
        
        let _zero_init_options = rclrs::rcl_get_zero_initialized_init_options();
        println!("✓ Called rcl_get_zero_initialized_init_options()");
        
        let _rmw_zero_init_options = rclrs::rmw_get_zero_initialized_init_options();
        println!("✓ Called rmw_get_zero_initialized_init_options()");
    }
    
    println!("🎉 All basic RCL and RMW bindings working!");
}
