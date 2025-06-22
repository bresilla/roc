use rclrs::*;
use anyhow::{Result, anyhow};

use crate::shared::graph_context::RclGraphContext;

/// Get all nodes in the ROS graph using direct RCL API calls
#[allow(dead_code)]
pub fn get_node_names(context: &RclGraphContext) -> Result<Vec<String>> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }
    
    unsafe {
        let allocator = rcutils_get_default_allocator();
        let mut node_names = rcutils_get_zero_initialized_string_array();
        let mut node_namespaces = rcutils_get_zero_initialized_string_array();
        
        let ret = rcl_get_node_names(
            context.node(),
            allocator,
            &mut node_names,
            &mut node_namespaces,
        );
        
        if ret != 0 {
            return Err(anyhow!("Failed to get node names: {}", ret));
        }
        
        // Convert the string array to Vec<String>
        let mut result = Vec::new();
        for i in 0..node_names.size {
            if !node_names.data.add(i).is_null() {
                let name_ptr = *node_names.data.add(i);
                if !name_ptr.is_null() {
                    let name_cstr = std::ffi::CStr::from_ptr(name_ptr);
                    if let Ok(name_str) = name_cstr.to_str() {
                        result.push(name_str.to_string());
                    }
                }
            }
        }
        
        // Clean up
        rcutils_string_array_fini(&mut node_names);
        rcutils_string_array_fini(&mut node_namespaces);
        
        Ok(result)
    }
}

/// Get all nodes with their namespaces in the ROS graph using direct RCL API calls
#[allow(dead_code)]
pub fn get_node_names_with_namespaces(context: &RclGraphContext) -> Result<Vec<(String, String)>> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }
    
    unsafe {
        let allocator = rcutils_get_default_allocator();
        let mut node_names = rcutils_get_zero_initialized_string_array();
        let mut node_namespaces = rcutils_get_zero_initialized_string_array();
        
        let ret = rcl_get_node_names(
            context.node(),
            allocator,
            &mut node_names,
            &mut node_namespaces,
        );
        
        if ret != 0 {
            return Err(anyhow!("Failed to get node names: {}", ret));
        }
        
        // Convert to Vec<(String, String)> with (name, namespace) pairs
        let mut result = Vec::new();
        for i in 0..node_names.size {
            if !node_names.data.add(i).is_null() && !node_namespaces.data.add(i).is_null() {
                let name_ptr = *node_names.data.add(i);
                let namespace_ptr = *node_namespaces.data.add(i);
                if !name_ptr.is_null() && !namespace_ptr.is_null() {
                    let name_cstr = std::ffi::CStr::from_ptr(name_ptr);
                    let namespace_cstr = std::ffi::CStr::from_ptr(namespace_ptr);
                    if let (Ok(name_str), Ok(namespace_str)) = (name_cstr.to_str(), namespace_cstr.to_str()) {
                        result.push((name_str.to_string(), namespace_str.to_string()));
                    }
                }
            }
        }
        
        // Clean up
        rcutils_string_array_fini(&mut node_names);
        rcutils_string_array_fini(&mut node_namespaces);
        
        Ok(result)
    }
}