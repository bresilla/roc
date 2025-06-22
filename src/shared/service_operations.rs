use rclrs::*;
use std::ptr;
use anyhow::{Result, anyhow};

use crate::shared::graph_context::RclGraphContext;

/// Get all services in the ROS graph using direct RCL API calls
#[allow(dead_code)]
pub fn get_service_names(context: &RclGraphContext) -> Result<Vec<String>> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }
    
    unsafe {
        let mut allocator = rcutils_get_default_allocator();
        let mut service_names_and_types = rcl_names_and_types_t { 
            names: rcutils_get_zero_initialized_string_array(),
            types: ptr::null_mut(),
        };
        
        let ret = rcl_get_service_names_and_types(
            context.node(),
            &mut allocator as *mut _,
            &mut service_names_and_types,
        );
        
        if ret != 0 {
            crate::shared::graph_context::RclGraphContext::reset_error_state();
            return Err(anyhow!("Failed to get service names: {}", ret));
        }
        
        // Convert the service names to Vec<String>
        let mut result = Vec::new();
        for i in 0..service_names_and_types.names.size {
            if !service_names_and_types.names.data.add(i).is_null() {
                let name_ptr = *service_names_and_types.names.data.add(i);
                if !name_ptr.is_null() {
                    let name_cstr = std::ffi::CStr::from_ptr(name_ptr);
                    if let Ok(name_str) = name_cstr.to_str() {
                        result.push(name_str.to_string());
                    }
                }
            }
        }
        
        // Clean up
        rcl_names_and_types_fini(&mut service_names_and_types);
        
        Ok(result)
    }
}

/// Get all services and their types as tuples
#[allow(dead_code)]
pub fn get_service_names_and_types(context: &RclGraphContext) -> Result<Vec<(String, String)>> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }
    
    unsafe {
        let mut allocator = rcutils_get_default_allocator();
        let mut service_names_and_types = rcl_names_and_types_t { 
            names: rcutils_get_zero_initialized_string_array(),
            types: ptr::null_mut(),
        };
        
        let ret = rcl_get_service_names_and_types(
            context.node(),
            &mut allocator as *mut _,
            &mut service_names_and_types,
        );
        
        if ret != 0 {
            crate::shared::graph_context::RclGraphContext::reset_error_state();
            return Err(anyhow!("Failed to get service names and types: {}", ret));
        }
        
        // Convert to Vec<(String, String)> for service name and type pairs
        let mut result = Vec::new();
        for i in 0..service_names_and_types.names.size {
            if !service_names_and_types.names.data.add(i).is_null() {
                let name_ptr = *service_names_and_types.names.data.add(i);
                if !name_ptr.is_null() {
                    let name_cstr = std::ffi::CStr::from_ptr(name_ptr);
                    if let Ok(name_str) = name_cstr.to_str() {
                        // Get the corresponding type(s) - there may be multiple types per service
                        if !service_names_and_types.types.add(i).is_null() {
                            let types_array = &*service_names_and_types.types.add(i);
                            for j in 0..types_array.size {
                                if !types_array.data.add(j).is_null() {
                                    let type_ptr = *types_array.data.add(j);
                                    if !type_ptr.is_null() {
                                        let type_cstr = std::ffi::CStr::from_ptr(type_ptr);
                                        if let Ok(type_str) = type_cstr.to_str() {
                                            result.push((name_str.to_string(), type_str.to_string()));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Clean up
        rcl_names_and_types_fini(&mut service_names_and_types);
        
        Ok(result)
    }
}