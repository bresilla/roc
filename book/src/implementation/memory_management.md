# Memory Management

This chapter covers how the `roc` tool manages memory when interfacing with ROS 2's C libraries through FFI (Foreign Function Interface).

## Overview

Memory management in FFI bindings is critical for safety and performance. The `roc` tool must carefully handle:

1. **Allocation and deallocation** of C structures
2. **Ownership transfer** between Rust and C code
3. **String handling** across language boundaries
4. **Resource cleanup** to prevent memory leaks

## Memory Safety Principles

### RAII (Resource Acquisition Is Initialization)

The `roc` tool follows Rust's RAII principles by wrapping C resources in Rust structs that implement `Drop`:

```rust
pub struct RclGraphContext {
    context: *mut rcl_context_t,
    node: *mut rcl_node_t,
    // Other fields...
}

impl Drop for RclGraphContext {
    fn drop(&mut self) {
        unsafe {
            if !self.node.is_null() {
                rcl_node_fini(self.node);
                libc::free(self.node as *mut c_void);
            }
            if !self.context.is_null() {
                rcl_context_fini(self.context);
                libc::free(self.context as *mut c_void);
            }
        }
    }
}
```

### Safe Wrappers

All C FFI calls are wrapped in safe Rust functions that handle error checking and memory management:

```rust
impl RclGraphContext {
    pub fn new() -> Result<Self, String> {
        unsafe {
            // Allocate C structures
            let context = libc::malloc(size_of::<rcl_context_t>()) as *mut rcl_context_t;
            if context.is_null() {
                return Err("Failed to allocate context".to_string());
            }

            // Initialize with proper error handling
            let ret = rcl_init(0, ptr::null(), ptr::null(), context);
            if ret != RCL_RET_OK as i32 {
                libc::free(context as *mut c_void);
                return Err(format!("Failed to initialize context: {}", ret));
            }

            // Continue with node allocation and initialization...
        }
    }
}
```

## String Handling

### C String Conversion

Converting between Rust strings and C strings requires careful memory management:

```rust
fn rust_string_to_c_string(s: &str) -> Result<*mut c_char, String> {
    let c_string = CString::new(s).map_err(|e| format!("Invalid string: {}", e))?;
    let ptr = unsafe { libc::malloc(c_string.len() + 1) as *mut c_char };
    if ptr.is_null() {
        return Err("Failed to allocate memory for C string".to_string());
    }
    unsafe {
        libc::strcpy(ptr, c_string.as_ptr());
    }
    Ok(ptr)
}

fn c_string_to_rust_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    unsafe {
        CStr::from_ptr(ptr).to_string_lossy().into_owned().into()
    }
}
```

### Owned vs Borrowed Strings

The code distinguishes between owned and borrowed string data:

```rust
// Borrowed - ROS 2 owns the memory
let topic_name = c_string_to_rust_string(topic_info.topic_name);

// Owned - we must free the memory
unsafe {
    if !owned_string_ptr.is_null() {
        libc::free(owned_string_ptr as *mut c_void);
    }
}
```

## Array and Structure Management

### Dynamic Arrays

When ROS 2 returns arrays of structures, we must carefully manage the memory:

```rust
pub fn get_topic_names_and_types(&self) -> Result<Vec<(String, Vec<String>)>, String> {
    let mut names_and_types = rcl_names_and_types_t {
        names: rcl_string_array_t {
            data: ptr::null_mut(),
            size: 0,
            allocator: rcl_get_default_allocator(),
        },
        types: rcl_string_array_t {
            data: ptr::null_mut(),
            size: 0,
            allocator: rcl_get_default_allocator(),
        },
    };

    unsafe {
        let ret = rcl_get_topic_names_and_types(
            self.node,
            &mut names_and_types.names,
            &mut names_and_types.types,
        );

        if ret != RCL_RET_OK as i32 {
            return Err(format!("Failed to get topic names and types: {}", ret));
        }

        // Convert to Rust types
        let result = self.convert_names_and_types(&names_and_types)?;

        // Clean up ROS 2 allocated memory
        rcl_names_and_types_fini(&mut names_and_types);

        Ok(result)
    }
}
```

### Structure Initialization

C structures must be properly initialized to avoid undefined behavior:

```rust
fn create_topic_endpoint_info() -> rcl_topic_endpoint_info_t {
    rcl_topic_endpoint_info_t {
        node_name: ptr::null(),
        node_namespace: ptr::null(),
        topic_type: ptr::null(),
        endpoint_type: RCL_PUBLISHER_ENDPOINT,
        endpoint_gid: [0; 24], // GID is a fixed-size array
        qos_profile: rcl_qos_profile_t {
            history: RCL_QOS_POLICY_HISTORY_KEEP_LAST,
            depth: 10,
            reliability: RCL_QOS_POLICY_RELIABILITY_RELIABLE,
            durability: RCL_QOS_POLICY_DURABILITY_VOLATILE,
            deadline: rcl_duration_t { nanoseconds: 0 },
            lifespan: rcl_duration_t { nanoseconds: 0 },
            liveliness: RCL_QOS_POLICY_LIVELINESS_AUTOMATIC,
            liveliness_lease_duration: rcl_duration_t { nanoseconds: 0 },
            avoid_ros_namespace_conventions: false,
        },
    }
}
```

## Error Handling and Cleanup

### Consistent Error Handling

All FFI operations follow a consistent pattern for error handling:

```rust
macro_rules! check_rcl_ret {
    ($ret:expr, $msg:expr) => {
        if $ret != RCL_RET_OK as i32 {
            return Err(format!("{}: error code {}", $msg, $ret));
        }
    };
}

// Usage
let ret = unsafe { rcl_some_function(params) };
check_rcl_ret!(ret, "Failed to call rcl_some_function");
```

### Resource Cleanup on Error

When operations fail, we must ensure proper cleanup:

```rust
pub fn initialize_node(name: &str) -> Result<*mut rcl_node_t, String> {
    unsafe {
        let node = libc::malloc(size_of::<rcl_node_t>()) as *mut rcl_node_t;
        if node.is_null() {
            return Err("Failed to allocate node".to_string());
        }

        let c_name = rust_string_to_c_string(name)?;
        let ret = rcl_node_init(node, c_name, self.context);
        
        // Clean up the C string regardless of success/failure
        libc::free(c_name as *mut c_void);

        if ret != RCL_RET_OK as i32 {
            libc::free(node as *mut c_void);
            return Err(format!("Failed to initialize node: {}", ret));
        }

        Ok(node)
    }
}
```

## Performance Considerations

### Memory Pool Reuse

For frequently allocated structures, consider using memory pools:

```rust
pub struct EndpointInfoPool {
    pool: Vec<rcl_topic_endpoint_info_t>,
    next_available: usize,
}

impl EndpointInfoPool {
    pub fn get_endpoint_info(&mut self) -> &mut rcl_topic_endpoint_info_t {
        if self.next_available >= self.pool.len() {
            self.pool.push(create_topic_endpoint_info());
        }
        let info = &mut self.pool[self.next_available];
        self.next_available += 1;
        info
    }

    pub fn reset(&mut self) {
        self.next_available = 0;
    }
}
```

### Minimize Allocations

Reuse string buffers and structures when possible:

```rust
pub struct StringBuffer {
    buffer: Vec<u8>,
}

impl StringBuffer {
    pub fn as_c_string(&mut self, s: &str) -> *const c_char {
        self.buffer.clear();
        self.buffer.extend_from_slice(s.as_bytes());
        self.buffer.push(0); // null terminator
        self.buffer.as_ptr() as *const c_char
    }
}
```

## Common Pitfalls

### Double Free

Never free memory that ROS 2 still owns:

```rust
// BAD - ROS 2 owns this memory
unsafe {
    libc::free(topic_info.topic_name as *mut c_void); // Don't do this!
}

// GOOD - Let ROS 2 clean up its own memory
unsafe {
    rcl_topic_endpoint_info_fini(&mut topic_info);
}
```

### Use After Free

Always set pointers to null after freeing:

```rust
unsafe {
    if !ptr.is_null() {
        libc::free(ptr as *mut c_void);
        ptr = ptr::null_mut(); // Prevent use-after-free
    }
}
```

### Memory Leaks

Use tools like Valgrind to detect memory leaks:

```bash
valgrind --leak-check=full --show-leak-kinds=all ./target/debug/roc topic list
```

## Testing Memory Management

### Unit Tests

Test memory management in isolation:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation_and_cleanup() {
        let context = RclGraphContext::new().expect("Failed to create context");
        // Context should be properly cleaned up when dropped
    }

    #[test]
    fn test_string_conversion() {
        let test_str = "test_topic";
        let c_str = rust_string_to_c_string(test_str).expect("Failed to convert");
        let rust_str = c_string_to_rust_string(c_str).expect("Failed to convert back");
        assert_eq!(test_str, rust_str);
        unsafe {
            libc::free(c_str as *mut c_void);
        }
    }
}
```

### Integration Tests

Test memory management with real ROS 2 operations:

```rust
#[test]
fn test_topic_info_memory_management() {
    let context = RclGraphContext::new().expect("Failed to create context");
    
    // This should not leak memory
    for _ in 0..1000 {
        let topics = context.get_topic_names_and_types()
            .expect("Failed to get topics");
        assert!(!topics.is_empty());
    }
}
```

This comprehensive memory management ensures that the `roc` tool is both safe and efficient when interfacing with ROS 2's C libraries.
