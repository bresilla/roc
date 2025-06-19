# Graph Context Implementation

The `RclGraphContext` is the core component that manages ROS 2 graph introspection in `roc`. It provides a safe Rust wrapper around RCL and RMW APIs for discovering and querying the ROS 2 computation graph.

## Core Design Principles

### 1. RAII (Resource Acquisition Is Initialization)
The context automatically manages RCL resources:
```rust
pub struct RclGraphContext {
    context: rcl_context_t,      // RCL context handle
    node: rcl_node_t,            // Minimal node for graph queries
    is_initialized: bool,        // Safety flag
}
```

### 2. Direct DDS Discovery
Unlike `ros2` CLI tools that may use the daemon, `roc` always performs direct DDS discovery:
```rust
/// Note: This implementation always performs direct DDS discovery 
/// (equivalent to --no-daemon)
pub fn new() -> Result<Self> {
    Self::new_with_discovery(std::time::Duration::from_millis(150))
}
```

### 3. Type Safety
All unsafe C interactions are wrapped in safe Rust APIs that return `Result` types.

## Initialization Process

The initialization follows a specific sequence required by RCL:

```rust
pub fn new_with_discovery(discovery_time: std::time::Duration) -> Result<Self> {
    unsafe {
        // 1. Read ROS_DOMAIN_ID from environment
        let domain_id = env::var("ROS_DOMAIN_ID")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);

        // 2. Initialize RCL init options
        let mut init_options = rcl_get_zero_initialized_init_options();
        let allocator = rcutils_get_default_allocator();
        
        let ret = rcl_init_options_init(&mut init_options, allocator);
        if ret != 0 {
            return Err(anyhow!("Failed to initialize RCL init options: {}", ret));
        }

        // 3. Configure RMW init options with domain ID
        let rmw_init_options = rcl_init_options_get_rmw_init_options(&mut init_options);
        if rmw_init_options.is_null() {
            return Err(anyhow!("Failed to get RMW init options"));
        }
        (*rmw_init_options).domain_id = domain_id;

        // 4. Initialize RCL context
        let mut context = rcl_get_zero_initialized_context();
        let ret = rcl_init(0, ptr::null_mut(), &init_options, &mut context);
        if ret != 0 {
            return Err(anyhow!("Failed to initialize RCL: {}", ret));
        }

        // 5. Create minimal node for graph queries
        let mut node = rcl_get_zero_initialized_node();
        let node_name = CString::new("roc_graph_node")?;
        let namespace = CString::new("/")?;
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

        // 6. Wait for DDS discovery
        let graph_context = RclGraphContext { context, node, is_initialized: true };
        graph_context.wait_for_graph_discovery(discovery_time)?;
        
        Ok(graph_context)
    }
}
```

## Graph Discovery Operations

### Basic Topic Listing
```rust
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
            false, // no_demangle: use ROS topic name conventions
            &mut topic_names_and_types,
        );
        
        if ret != 0 {
            return Err(anyhow!("Failed to get topic names: {}", ret));
        }
        
        // Convert C string array to Rust Vec<String>
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
        
        // Critical: clean up allocated memory
        rcl_names_and_types_fini(&mut topic_names_and_types);
        
        Ok(result)
    }
}
```

### Counting Publishers/Subscribers
```rust
pub fn count_publishers(&self, topic_name: &str) -> Result<usize> {
    if !self.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }
    
    let topic_name_c = CString::new(topic_name)?;
    
    unsafe {
        let mut count: usize = 0;
        let ret = rcl_count_publishers(
            &self.node, 
            topic_name_c.as_ptr(), 
            &mut count
        );
        
        if ret != 0 {
            return Err(anyhow!("Failed to count publishers: {}", ret));
        }
        
        Ok(count)
    }
}
```

## Detailed Endpoint Information

The most complex operation is getting detailed endpoint information with QoS profiles:

```rust
pub fn get_publishers_info(&self, topic_name: &str) -> Result<Vec<TopicEndpointInfo>> {
    if !self.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }
    
    let topic_name_c = CString::new(topic_name)?;
    
    unsafe {
        let mut allocator = rcutils_get_default_allocator();
        let mut publishers_info: rcl_topic_endpoint_info_array_t = std::mem::zeroed();
        
        let ret = rcl_get_publishers_info_by_topic(
            &self.node,
            &mut allocator,
            topic_name_c.as_ptr(),
            false, // no_mangle: follow ROS conventions
            &mut publishers_info,
        );
        
        if ret != 0 {
            return Err(anyhow!("Failed to get publishers info: {}", ret));
        }
        
        // Process each endpoint info structure
        let mut result = Vec::new();
        for i in 0..publishers_info.size {
            let info = &*(publishers_info.info_array.add(i));
            
            // Extract and convert all fields safely
            let endpoint_info = TopicEndpointInfo {
                node_name: self.extract_string(info.node_name)?,
                node_namespace: self.extract_string(info.node_namespace)?,
                topic_type: self.extract_string(info.topic_type)?,
                topic_type_hash: format_topic_type_hash(&info.topic_type_hash),
                endpoint_type: EndpointType::from_rmw(info.endpoint_type),
                gid: self.extract_gid(&info.endpoint_gid),
                qos_profile: QosProfile::from_rmw(&info.qos_profile),
            };
            
            result.push(endpoint_info);
        }
        
        // Critical: cleanup allocated memory
        rmw_topic_endpoint_info_array_fini(&mut publishers_info, &mut allocator);
        
        Ok(result)
    }
}
```

## Memory Management Strategy

### Allocation Pattern
1. **Zero-initialize** all structures before use
2. **Pass allocators** to RCL/RMW functions
3. **Extract/copy** data before cleanup
4. **Finalize** structures to free memory

### Helper Methods for Safe Extraction
```rust
impl RclGraphContext {
    unsafe fn extract_string(&self, ptr: *const c_char) -> Result<String> {
        if ptr.is_null() {
            Ok("unknown".to_string())
        } else {
            Ok(std::ffi::CStr::from_ptr(ptr).to_string_lossy().to_string())
        }
    }
    
    unsafe fn extract_gid(&self, gid_array: &[u8; 16]) -> Vec<u8> {
        gid_array.to_vec() // Copy the array to owned Vec
    }
}
```

## Error Handling and Validation

### Context Validation
```rust
pub fn is_valid(&self) -> bool {
    if !self.is_initialized {
        return false;
    }
    unsafe {
        rcl_context_is_valid(&self.context) && rcl_node_is_valid(&self.node)
    }
}
```

### Comprehensive Error Mapping
```rust
fn map_rcl_error(ret: i32, operation: &str) -> anyhow::Error {
    match ret {
        0 => panic!("Success code passed to error mapper"),
        1 => anyhow!("{}: Generic error", operation),
        10 => anyhow!("{}: Memory allocation failed", operation),
        11 => anyhow!("{}: Invalid argument", operation),
        200 => anyhow!("{}: Node is invalid", operation),
        _ => anyhow!("{}: Unknown error code {}", operation, ret),
    }
}
```

## Resource Cleanup (Drop Implementation)

Proper cleanup is critical for long-running applications:

```rust
impl Drop for RclGraphContext {
    fn drop(&mut self) {
        if self.is_initialized {
            unsafe {
                // Order matters: node before context
                if rcl_node_is_valid(&self.node) {
                    rcl_node_fini(&mut self.node);
                }
                
                if rcl_context_is_valid(&self.context) {
                    rcl_shutdown(&mut self.context);
                }
                
                self.is_initialized = false;
            }
        }
    }
}
```

## Discovery Timing

Since we use direct DDS discovery, we must wait for the discovery protocol:

```rust
fn wait_for_graph_discovery(&self, discovery_time: std::time::Duration) -> Result<()> {
    if !self.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }

    // DDS discovery is asynchronous - we need to wait for network convergence
    std::thread::sleep(discovery_time);
    
    Ok(())
}
```

The default 150ms timeout balances discovery completeness with startup speed.

## Thread Safety

The `RclGraphContext` is **not** thread-safe. RCL contexts and nodes are not designed for concurrent access. For multi-threaded applications, create separate contexts per thread or use synchronization primitives.

## Performance Characteristics

- **Initialization**: ~150ms (dominated by DDS discovery)
- **Topic listing**: ~1-5ms (depends on graph size)
- **Detailed queries**: ~5-20ms (depends on topic complexity)
- **Memory usage**: ~1MB base + proportional to graph size

This implementation provides the foundation for all of `roc`'s graph introspection capabilities while maintaining safety and performance.
