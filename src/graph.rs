use rclrs::*;
use std::ptr;
use std::ffi::CString;
use anyhow::{Result, anyhow};

/// A simple RCL context manager for graph operations  
/// Includes both context and a minimal node for graph queries
pub struct RclGraphContext {
    context: rcl_context_t,
    node: rcl_node_t,
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

            // Create a minimal node for graph queries
            let mut node = rcl_get_zero_initialized_node();
            let node_name = CString::new("roc_graph_node").map_err(|e| anyhow!("Failed to create node name: {}", e))?;
            let namespace = CString::new("/").map_err(|e| anyhow!("Failed to create namespace: {}", e))?;
            let node_options = rcl_node_get_default_options();
            
            let ret = rcl_node_init(
                &mut node,
                node_name.as_ptr(),
                namespace.as_ptr(),
                &mut context,
                &node_options,
            );
            if ret != 0 {
                rcl_shutdown(&mut context);
                return Err(anyhow!("Failed to initialize node: {}", ret));
            }

            Ok(RclGraphContext {
                context,
                node,
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
            rcl_context_is_valid(&self.context) && rcl_node_is_valid(&self.node)
        }
    }

    /// Get all topics in the ROS graph using direct RCL API calls
    pub fn get_topic_names(&self) -> Result<Vec<String>> {
        if !self.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }
        
        unsafe {
            let mut allocator = rcutils_get_default_allocator();
            let mut topic_names_and_types = rcl_names_and_types_t { 
                names: rcutils_get_zero_initialized_string_array(),
                types: rcutils_get_zero_initialized_string_array(),
            };
            
            let ret = rcl_get_topic_names_and_types(
                &self.node,
                &mut allocator,
                false, // no_demangle
                &mut topic_names_and_types,
            );
            
            if ret != 0 {
                return Err(anyhow!("Failed to get topic names: {}", ret));
            }
            
            // Convert the topic names to Vec<String>
            let mut result = Vec::new();
            for i in 0..topic_names_and_types.names.size {
                if !topic_names_and_types.names.data.add(i).is_null() {
                    let name_ptr = *topic_names_and_types.names.data.add(i);
                    if !name_ptr.is_null() {
                        let name_cstr = std::ffi::CStr::from_ptr(name_ptr);
                        if let Ok(name_str) = name_cstr.to_str() {
                            result.push(name_str.to_string());
                        }
                    }
                }
            }
            
            // Clean up
            rcl_names_and_types_fini(&mut topic_names_and_types);
            
            Ok(result)
        }
    }

    /// Get all nodes in the ROS graph using direct RCL API calls
    pub fn get_node_names(&self) -> Result<Vec<String>> {
        if !self.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }
        
        unsafe {
            let allocator = rcutils_get_default_allocator();
            let mut node_names = rcutils_get_zero_initialized_string_array();
            let mut node_namespaces = rcutils_get_zero_initialized_string_array();
            
            let ret = rcl_get_node_names(
                &self.node,
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

    /// Get all services in the ROS graph using direct RCL API calls
    pub fn get_service_names(&self) -> Result<Vec<String>> {
        if !self.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }
        
        unsafe {
            let mut allocator = rcutils_get_default_allocator();
            let mut service_names_and_types = rcl_names_and_types_t { 
                names: rcutils_get_zero_initialized_string_array(),
                types: rcutils_get_zero_initialized_string_array(),
            };
            
            let ret = rcl_get_service_names_and_types(
                &self.node,
                &mut allocator,
                &mut service_names_and_types,
            );
            
            if ret != 0 {
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
}

impl Drop for RclGraphContext {
    fn drop(&mut self) {
        if self.is_initialized {
            unsafe {
                rcl_node_fini(&mut self.node);
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

    #[test]
    fn test_graph_api_structure() {
        let context = RclGraphContext::new().unwrap();
        
        // Test that the API structure works and returns results
        let topics = context.get_topic_names();
        let nodes = context.get_node_names();
        let services = context.get_service_names();
        
        assert!(topics.is_ok());
        assert!(nodes.is_ok());
        assert!(services.is_ok());
        
        // Check that we got at least our own node
        let node_names = nodes.unwrap();
        assert!(node_names.contains(&"roc_graph_node".to_string()));
        
        println!("Found {} nodes", node_names.len());
        println!("Found {} topics", topics.unwrap().len());
        println!("Found {} services", services.unwrap().len());
    }
}
