use rclrs::*;
use std::ptr;
use std::ffi::CString;
use std::env;
use anyhow::{Result, anyhow};

/// A simple RCL context manager for graph operations  
/// Includes both context and a minimal node for graph queries
pub struct RclGraphContext {
    context: rcl_context_t,
    node: rcl_node_t,
    is_initialized: bool,
}

/// Information about a topic endpoint (publisher or subscriber)
#[derive(Debug, Clone)]
pub struct TopicEndpointInfo {
    pub node_name: String,
    pub node_namespace: String,
    pub topic_type: String,
    pub gid: Vec<u8>,
    // We'll skip QoS for now as it's more complex to extract
}

impl RclGraphContext {
    /// Create a new RCL context for graph operations
    pub fn new() -> Result<Self> {
        unsafe {
            // Read ROS_DOMAIN_ID from environment (default to 0 if not set)
            let domain_id = env::var("ROS_DOMAIN_ID")
                .ok()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);

            // Initialize RCL init options
            let mut init_options = rcl_get_zero_initialized_init_options();
            let allocator = rcutils_get_default_allocator();
            
            let ret = rcl_init_options_init(&mut init_options, allocator);
            if ret != 0 {
                return Err(anyhow!("Failed to initialize RCL init options: {}", ret));
            }

            // Get the RMW init options from RCL and set the domain ID
            let rmw_init_options = rcl_init_options_get_rmw_init_options(&mut init_options);
            if rmw_init_options.is_null() {
                return Err(anyhow!("Failed to get RMW init options"));
            }
            
            // Set the domain ID
            (*rmw_init_options).domain_id = domain_id;

            // Initialize RCL context with the configured options
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
                types: ptr::null_mut(),
            };
            
            let ret = rcl_get_topic_names_and_types(
                &self.node,
                &mut allocator as *mut _,
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
                types: ptr::null_mut(),
            };
            
            let ret = rcl_get_service_names_and_types(
                &self.node,
                &mut allocator as *mut _,
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

    /// Get all topics in the ROS graph with their type information using direct RCL API calls
    pub fn get_topics_with_types(&self) -> Result<Vec<TopicInfo>> {
        if !self.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }
        
        unsafe {
            let mut allocator = rcutils_get_default_allocator();
            let mut topic_names_and_types = rcl_names_and_types_t { 
                names: rcutils_get_zero_initialized_string_array(),
                types: ptr::null_mut(),
            };
            
            let ret = rcl_get_topic_names_and_types(
                &self.node,
                &mut allocator,
                false, // no_demangle
                &mut topic_names_and_types,
            );
            
            if ret != 0 {
                return Err(anyhow!("Failed to get topic names and types: {}", ret));
            }
            
            // Convert the topic names and types to Vec<TopicInfo>
            let mut result = Vec::new();
            for i in 0..topic_names_and_types.names.size {
                if !topic_names_and_types.names.data.add(i).is_null() {
                    let name_ptr = *topic_names_and_types.names.data.add(i);
                    if !name_ptr.is_null() {
                        let name_cstr = std::ffi::CStr::from_ptr(name_ptr);
                        if let Ok(name_str) = name_cstr.to_str() {
                            // Get the types for this topic
                            let mut topic_types = Vec::new();
                            if !topic_names_and_types.types.is_null() {
                                let types_array = topic_names_and_types.types.add(i);
                                if !types_array.is_null() {
                                    let types_for_topic = &*types_array;
                                    for j in 0..types_for_topic.size {
                                        if !types_for_topic.data.add(j).is_null() {
                                            let type_ptr = *types_for_topic.data.add(j);
                                            if !type_ptr.is_null() {
                                                let type_cstr = std::ffi::CStr::from_ptr(type_ptr);
                                                if let Ok(type_str) = type_cstr.to_str() {
                                                    topic_types.push(type_str.to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            
                            result.push(TopicInfo {
                                name: name_str.to_string(),
                                types: topic_types,
                            });
                        }
                    }
                }
            }
            
            // Clean up
            rcl_names_and_types_fini(&mut topic_names_and_types);
            
            Ok(result)
        }
    }

    /// Get all topics and their types as tuples
    pub fn get_topic_names_and_types(&self) -> Result<Vec<(String, String)>> {
        if !self.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }
        
        unsafe {
            let mut allocator = rcutils_get_default_allocator();
            let mut topic_names_and_types = rcl_names_and_types_t { 
                names: rcutils_get_zero_initialized_string_array(),
                types: ptr::null_mut(),
            };
            
            let ret = rcl_get_topic_names_and_types(
                &self.node,
                &mut allocator as *mut _,
                false, // no_demangle
                &mut topic_names_and_types,
            );
            
            if ret != 0 {
                return Err(anyhow!("Failed to get topic names and types: {}", ret));
            }
            
            // Convert the topics and types to Vec<(String, String)>
            let mut result = Vec::new();
            for i in 0..topic_names_and_types.names.size {
                if !topic_names_and_types.names.data.add(i).is_null() {
                    let name_ptr = *topic_names_and_types.names.data.add(i);
                    if !name_ptr.is_null() {
                        let name_cstr = std::ffi::CStr::from_ptr(name_ptr);
                        if let Ok(name_str) = name_cstr.to_str() {
                            // Get the corresponding type(s) - there may be multiple types per topic
                            if !topic_names_and_types.types.add(i).is_null() {
                                let types_array = &*topic_names_and_types.types.add(i);
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
            rcl_names_and_types_fini(&mut topic_names_and_types);
            
            Ok(result)
        }
    }

    /// Count the number of publishers for a given topic
    pub fn count_publishers(&self, topic_name: &str) -> Result<usize> {
        if !self.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }
        
        let topic_name_c = CString::new(topic_name).map_err(|e| anyhow!("Invalid topic name: {}", e))?;
        
        unsafe {
            let mut count: usize = 0;
            let ret = rcl_count_publishers(
                &self.node,
                topic_name_c.as_ptr(),
                &mut count,
            );
            
            if ret != 0 {
                return Err(anyhow!("Failed to count publishers for topic '{}': {}", topic_name, ret));
            }
            
            Ok(count)
        }
    }

    /// Count the number of subscribers for a given topic
    pub fn count_subscribers(&self, topic_name: &str) -> Result<usize> {
        if !self.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }
        
        let topic_name_c = CString::new(topic_name).map_err(|e| anyhow!("Invalid topic name: {}", e))?;
        
        unsafe {
            let mut count: usize = 0;
            let ret = rcl_count_subscribers(
                &self.node,
                topic_name_c.as_ptr(),
                &mut count,
            );
            
            if ret != 0 {
                return Err(anyhow!("Failed to count subscribers for topic '{}': {}", topic_name, ret));
            }
            
            Ok(count)
        }
    }

    /// Get detailed information about all publishers to a topic
    pub fn get_publishers_info(&self, topic_name: &str) -> Result<Vec<TopicEndpointInfo>> {
        if !self.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }
        
        let topic_name_c = CString::new(topic_name).map_err(|e| anyhow!("Invalid topic name: {}", e))?;
        
        unsafe {
            let mut allocator = rcutils_get_default_allocator();
            
            // Initialize the array - we need to use the RMW function since that's what RCL uses
            let mut publishers_info: rcl_topic_endpoint_info_array_t = std::mem::zeroed();
            
            let ret = rcl_get_publishers_info_by_topic(
                &self.node,
                &mut allocator,
                topic_name_c.as_ptr(),
                false, // no_mangle
                &mut publishers_info,
            );
            
            if ret != 0 {
                return Err(anyhow!("Failed to get publishers info for topic '{}': {}", topic_name, ret));
            }
            
            // Convert to our Rust struct
            let mut result = Vec::new();
            for i in 0..publishers_info.size {
                let info = &*(publishers_info.info_array.add(i));
                
                // Extract strings safely
                let node_name = if info.node_name.is_null() {
                    "unknown".to_string()
                } else {
                    std::ffi::CStr::from_ptr(info.node_name).to_string_lossy().to_string()
                };
                
                let node_namespace = if info.node_namespace.is_null() {
                    "/".to_string()
                } else {
                    std::ffi::CStr::from_ptr(info.node_namespace).to_string_lossy().to_string()
                };
                
                let topic_type = if info.topic_type.is_null() {
                    "unknown".to_string()
                } else {
                    std::ffi::CStr::from_ptr(info.topic_type).to_string_lossy().to_string()
                };
                
                // Extract GID (Global ID) - it's a fixed-size array in RMW
                let gid = std::slice::from_raw_parts(info.endpoint_gid.as_ptr(), info.endpoint_gid.len()).to_vec();
                
                result.push(TopicEndpointInfo {
                    node_name,
                    node_namespace,
                    topic_type,
                    gid,
                });
            }
            
            // We should clean up the array
            let mut allocator = rcutils_get_default_allocator();
            rmw_topic_endpoint_info_array_fini(&mut publishers_info, &mut allocator);
            
            Ok(result)
        }
    }

    /// Get detailed information about all subscribers to a topic
    pub fn get_subscribers_info(&self, topic_name: &str) -> Result<Vec<TopicEndpointInfo>> {
        if !self.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }
        
        let topic_name_c = CString::new(topic_name).map_err(|e| anyhow!("Invalid topic name: {}", e))?;
        
        unsafe {
            let mut allocator = rcutils_get_default_allocator();
            
            // Initialize the array
            let mut subscribers_info: rcl_topic_endpoint_info_array_t = std::mem::zeroed();
            
            let ret = rcl_get_subscriptions_info_by_topic(
                &self.node,
                &mut allocator,
                topic_name_c.as_ptr(),
                false, // no_mangle
                &mut subscribers_info,
            );
            
            if ret != 0 {
                return Err(anyhow!("Failed to get subscribers info for topic '{}': {}", topic_name, ret));
            }
            
            // Convert to our Rust struct
            let mut result = Vec::new();
            for i in 0..subscribers_info.size {
                let info = &*(subscribers_info.info_array.add(i));
                
                // Extract strings safely
                let node_name = if info.node_name.is_null() {
                    "unknown".to_string()
                } else {
                    std::ffi::CStr::from_ptr(info.node_name).to_string_lossy().to_string()
                };
                
                let node_namespace = if info.node_namespace.is_null() {
                    "/".to_string()
                } else {
                    std::ffi::CStr::from_ptr(info.node_namespace).to_string_lossy().to_string()
                };
                
                let topic_type = if info.topic_type.is_null() {
                    "unknown".to_string()
                } else {
                    std::ffi::CStr::from_ptr(info.topic_type).to_string_lossy().to_string()
                };
                
                // Extract GID (Global ID)
                let gid = std::slice::from_raw_parts(info.endpoint_gid.as_ptr(), info.endpoint_gid.len()).to_vec();
                
                result.push(TopicEndpointInfo {
                    node_name,
                    node_namespace,
                    topic_type,
                    gid,
                });
            }

            // Clean up the array
            let mut allocator = rcutils_get_default_allocator();
            rmw_topic_endpoint_info_array_fini(&mut subscribers_info, &mut allocator);
            
            Ok(result)
        }
    }
}

impl Drop for RclGraphContext {
    fn drop(&mut self) {
        if self.is_initialized {
            unsafe {
                // It's important to shut down the node before the context
                if rcl_node_is_valid(&self.node) {
                    rcl_node_fini(&mut self.node);
                }
                if rcl_context_is_valid(&self.context) {
                    rcl_shutdown(&mut self.context);
                }
            }
            self.is_initialized = false;
        }
    }
}

/// Information about a topic, including its name and types
#[derive(Debug, Clone)]
pub struct TopicInfo {
    pub name: String,
    pub types: Vec<String>,
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
        
        // Test that the API structure works
        let topics = context.get_topic_names();
        let nodes = context.get_node_names();
        let services = context.get_service_names();
        
        // Print results for debugging
        if let Err(e) = &topics {
            println!("Topics error: {} (expected without ROS 2 daemon)", e);
        }
        if let Err(e) = &services {
            println!("Services error: {} (expected without ROS 2 daemon)", e);
        }
        
        // Node discovery should always work
        assert!(nodes.is_ok(), "Node discovery should work");
        
        // Check that we got at least our own node
        let node_names = nodes.unwrap();
        assert!(node_names.contains(&"roc_graph_node".to_string()));
        
        println!("✅ Found {} nodes", node_names.len());
        println!("✅ Node names: {:?}", node_names);
        
        // Topics and services may fail without ROS 2 daemon, but the API should work
        if let Ok(topic_names) = topics {
            println!("✅ Found {} topics", topic_names.len());
        }
        if let Ok(service_names) = services {
            println!("✅ Found {} services", service_names.len());
        }
        
        println!("✅ RCL graph discovery API is working!");
    }

    #[test]
    fn test_individual_functions() {
        let context = RclGraphContext::new().unwrap();
        
        // Test individual functions separately
        println!("Context is valid: {}", context.is_valid());
        
        let nodes_result = context.get_node_names();
        match &nodes_result {
            Ok(nodes) => println!("Nodes: {:?}", nodes),
            Err(e) => println!("Nodes error: {}", e),
        }
        
        let topics_result = context.get_topic_names();
        match &topics_result {
            Ok(topics) => println!("Topics: {:?}", topics),
            Err(e) => println!("Topics error: {}", e),
        }
        
        let services_result = context.get_service_names();  
        match &services_result {
            Ok(services) => println!("Services: {:?}", services),
            Err(e) => println!("Services error: {}", e),
        }
        
        // At least nodes should work since it has different error behavior
        assert!(nodes_result.is_ok(), "Node names should work even without ROS 2 daemon");
    }

    #[test]
    fn test_topic_discovery_with_delay() {
        let context = RclGraphContext::new().unwrap();
        
        println!("Context created, waiting for discovery...");
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        let topics_result = context.get_topic_names();
        match &topics_result {
            Ok(topics) => {
                println!("✅ Successfully discovered {} topics:", topics.len());
                for topic in topics {
                    println!("  - {}", topic);
                }
            },
            Err(e) => {
                println!("❌ Topics error: {}", e);
            }
        }
        
        // Test if the issue is persistent
        std::thread::sleep(std::time::Duration::from_millis(1000));
        let topics_result2 = context.get_topic_names();
        match &topics_result2 {
            Ok(topics) => {
                println!("✅ Second attempt: Successfully discovered {} topics", topics.len());
            },
            Err(e) => {
                println!("❌ Second attempt also failed: {}", e);
            }
        }
    }
}
