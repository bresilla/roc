# Dynamic Message Type Loading

This chapter explains one of the most important and sophisticated features in `roc`: **dynamic runtime loading of ROS2 message type support**. This technique enables `roc` to work with any ROS2 message type without requiring compile-time knowledge or static linking against specific message packages.

## Table of Contents

- [Overview](#overview)
- [The Problem](#the-problem)
- [The Solution: Runtime Dynamic Loading](#the-solution-runtime-dynamic-loading)
- [How Dynamic Loading Works](#how-dynamic-loading-works)
- [Implementation Architecture](#implementation-architecture)
- [Code Walkthrough](#code-walkthrough)
- [Generic Type Support Resolution](#generic-type-support-resolution)
- [Benefits and Trade-offs](#benefits-and-trade-offs)
- [Future Enhancements](#future-enhancements)

## Overview

Dynamic message type loading is a technique that allows `roc` to:

1. **Load ROS2 message type support libraries at runtime** (not compile time)
2. **Resolve type support functions dynamically** using symbol lookup
3. **Create real RCL publishers/subscribers** for any message type
4. **Support custom message types** without code changes
5. **Work with any ROS2 package** that provides proper typesupport libraries

This is what enables commands like:
```bash
# Works with any installed ROS2 message type!
roc topic pub /test geometry_msgs/msg/Twist '{linear: {x: 0.5}}'
roc topic pub /custom custom_msgs/msg/MyMessage '{field: value}'
```

## The Problem

Traditional ROS2 tools face a fundamental challenge:

### Static Linking Approach (Traditional)
```rust
// Traditional approach requires compile-time knowledge
use geometry_msgs::msg::Twist;
use std_msgs::msg::String;
// ... must import every message type you want to use

fn create_publisher() {
    // Must know the exact type at compile time
    let twist_publisher = node.create_publisher::<Twist>("topic", qos);
    let string_publisher = node.create_publisher::<String>("topic", qos); 
}
```

**Problems with static linking:**
- ❌ **Limited to pre-compiled message types**
- ❌ **Huge binary size** (includes all message libraries)
- ❌ **Cannot work with custom/unknown message types**
- ❌ **Requires recompilation** for new message types
- ❌ **Complex dependency management**

### The ROS2 Type Support Challenge

ROS2's architecture requires **type support pointers** to create publishers:

```c
// This is what RCL requires internally
rcl_ret_t rcl_publisher_init(
  rcl_publisher_t * publisher,
  const rcl_node_t * node,
  const rosidl_message_type_support_t * type_support,  // ← This is the key!
  const char * topic_name,
  const rcl_publisher_options_t * options
);
```

The `type_support` pointer contains:
- Message structure layout
- Serialization/deserialization functions  
- Field metadata and types
- Memory management functions

**Without valid type support, you cannot create RCL publishers, and topics won't appear in the ROS graph!**

## The Solution: Runtime Dynamic Loading

`roc` solves this through **dynamic library loading** - a powerful systems programming technique:

### Key Insight: ROS2 Type Support Libraries

ROS2 installations contain pre-compiled type support libraries:
```bash
/opt/ros/jazzy/lib/
├── libgeometry_msgs__rosidl_typesupport_c.so    # Geometry messages
├── libstd_msgs__rosidl_typesupport_c.so         # Standard messages  
├── libsensor_msgs__rosidl_typesupport_c.so      # Sensor messages
├── libcustom_msgs__rosidl_typesupport_c.so      # Your custom messages
└── ...
```

Each library exports **type support functions**:
```bash
$ nm -D libgeometry_msgs__rosidl_typesupport_c.so | grep Twist
rosidl_typesupport_c__get_message_type_support_handle__geometry_msgs__msg__Twist
```

### Dynamic Loading Strategy

Instead of static linking, `roc` uses **runtime dynamic loading**:

1. **Construct library path** from message type: `geometry_msgs/msg/Twist` → `/opt/ros/jazzy/lib/libgeometry_msgs__rosidl_typesupport_c.so`
2. **Load library dynamically** using `dlopen()` (via `rcutils_load_shared_library`)
3. **Resolve type support symbol** using `dlsym()` (via `rcutils_get_symbol`)
4. **Call the function** to get the type support pointer
5. **Create real RCL publishers** with valid type support

## How Dynamic Loading Works

### Step-by-Step Process

#### 1. Message Type Parsing
```rust
// Input: "geometry_msgs/msg/Twist"
let (package, message) = parse_message_type("geometry_msgs/msg/Twist")?;
// package = "geometry_msgs", message = "Twist"
```

#### 2. Library Path Construction
```rust
// Construct library path using naming convention
let library_path = format!(
    "/opt/ros/jazzy/lib/lib{}__rosidl_typesupport_c.so", 
    package
);
// Result: "/opt/ros/jazzy/lib/libgeometry_msgs__rosidl_typesupport_c.so"
```

#### 3. Symbol Name Construction  
```rust
// Construct symbol name using ROS2 naming convention
let symbol_name = format!(
    "rosidl_typesupport_c__get_message_type_support_handle__{}__msg__{}",
    package, message
);
// Result: "rosidl_typesupport_c__get_message_type_support_handle__geometry_msgs__msg__Twist"
```

#### 4. Dynamic Library Loading
```rust
unsafe {
    // Initialize library handle
    let mut shared_lib = rcutils_get_zero_initialized_shared_library();
    
    // Load the shared library
    let ret = rcutils_load_shared_library(
        &mut shared_lib,
        library_path_c.as_ptr(),
        allocator,
    );
    
    if ret != 0 {
        return Err(anyhow!("Failed to load library"));
    }
}
```

#### 5. Symbol Resolution
```rust
unsafe {
    // Get the symbol from the loaded library
    let symbol_ptr = rcutils_get_symbol(&shared_lib, symbol_name_c.as_ptr());
    
    if symbol_ptr.is_null() {
        return Err(anyhow!("Symbol not found"));
    }
    
    // Cast to function pointer and call it
    type TypeSupportGetterFn = unsafe extern "C" fn() -> *const rosidl_message_type_support_t;
    let type_support_fn: TypeSupportGetterFn = std::mem::transmute(symbol_ptr);
    let type_support = type_support_fn();
}
```

#### 6. RCL Publisher Creation
```rust
unsafe {
    // Now we can create a real RCL publisher!
    let ret = rcl_publisher_init(
        &mut publisher,
        node,
        type_support,  // ← Valid type support from dynamic loading
        topic_name_c.as_ptr(),
        &options,
    );
    
    // Publisher is registered in ROS graph and appears in topic lists!
}
```

## Implementation Architecture

### Core Components

#### 1. `DynamicMessageRegistry`
**File**: `src/shared/dynamic_messages.rs`

Central registry for loading and caching message types:
```rust
pub struct DynamicMessageRegistry {
    loaded_types: HashMap<String, DynamicMessageType>,
}

impl DynamicMessageRegistry {
    pub fn load_message_type(&mut self, type_name: &str) -> Result<DynamicMessageType> {
        // 1. Parse message type
        // 2. Load type support dynamically  
        // 3. Cache result
        // 4. Return type info with valid type support pointer
    }
}
```

#### 2. Generic Type Support Loading
```rust
fn try_get_generic_type_support(
    &self,
    package_name: &str,
    message_name: &str,
) -> Result<*const rosidl_message_type_support_t> {
    // Automatic library path construction
    let library_path = format!("/opt/ros/jazzy/lib/lib{}__rosidl_typesupport_c.so", package_name);
    
    // Automatic symbol name construction  
    let symbol_name = format!(
        "rosidl_typesupport_c__get_message_type_support_handle__{}__msg__{}",
        package_name, message_name
    );
    
    // Dynamic loading
    self.load_type_support_from_library(&library_path, &symbol_name)
}
```

#### 3. Bindgen Integration
**File**: `rclrs/build.rs`

Exposes dynamic loading functions to Rust:
```rust
let bindings = bindgen::Builder::default()
    .header("wrapper.h")
    // Dynamic loading functions
    .allowlist_function("rcutils_load_shared_library")
    .allowlist_function("rcutils_get_symbol")  
    .allowlist_function("rcutils_unload_shared_library")
    .allowlist_function("rcutils_get_zero_initialized_shared_library")
    // Type support types
    .allowlist_type("rosidl_message_type_support_t")
    .allowlist_type("rcutils_shared_library_t")
    .generate()?;
```

### Data Flow

```
User Command: roc topic pub /test geometry_msgs/msg/Twist '{linear: {x: 0.5}}'
                                    ↓
                    1. Parse message type: "geometry_msgs/msg/Twist"
                                    ↓  
                    2. Construct library path and symbol name
                                    ↓
                    3. Load: /opt/ros/jazzy/lib/libgeometry_msgs__rosidl_typesupport_c.so
                                    ↓
                    4. Resolve: rosidl_typesupport_c__get_message_type_support_handle__geometry_msgs__msg__Twist
                                    ↓
                    5. Call function → Get type_support pointer
                                    ↓
                    6. Create RCL publisher with valid type_support
                                    ↓
                    7. Topic appears in ROS graph! ✅
```

## Code Walkthrough

### Complete Type Support Loading Function

```rust
fn load_type_support_from_library(
    &self,
    library_name: &str,
    symbol_name: &str,
) -> Result<*const rosidl_message_type_support_t> {
    use std::ffi::CString;
    
    unsafe {
        // Step 1: Initialize shared library handle
        let mut shared_lib = rcutils_get_zero_initialized_shared_library();
        
        // Step 2: Convert library name to C string
        let lib_name_c = CString::new(library_name)
            .map_err(|e| anyhow!("Invalid library name '{}': {}", library_name, e))?;
        
        // Step 3: Load the shared library
        let allocator = rcutils_get_default_allocator();
        let ret = rcutils_load_shared_library(
            &mut shared_lib,
            lib_name_c.as_ptr(),
            allocator,
        );
        
        if ret != 0 { // RCUTILS_RET_OK is 0
            return Err(anyhow!("Failed to load library '{}': return code {}", library_name, ret));
        }
        
        // Step 4: Convert symbol name to C string
        let symbol_name_c = CString::new(symbol_name)
            .map_err(|e| anyhow!("Invalid symbol name '{}': {}", symbol_name, e))?;
        
        // Step 5: Get the symbol from the library
        let symbol_ptr = rcutils_get_symbol(&shared_lib, symbol_name_c.as_ptr());
        
        if symbol_ptr.is_null() {
            rcutils_unload_shared_library(&mut shared_lib);
            return Err(anyhow!("Symbol '{}' not found in library '{}'", symbol_name, library_name));
        }
        
        // Step 6: Cast the symbol to a function pointer and call it
        type TypeSupportGetterFn = unsafe extern "C" fn() -> *const rosidl_message_type_support_t;
        let type_support_fn: TypeSupportGetterFn = std::mem::transmute(symbol_ptr);
        let type_support = type_support_fn();
        
        // Step 7: Validate the result
        if type_support.is_null() {
            return Err(anyhow!("Type support function returned null pointer"));
        }
        
        println!("Successfully loaded type support for symbol: {}", symbol_name);
        Ok(type_support)
    }
}
```

### Publisher Creation with Dynamic Type Support

```rust
fn create_dynamic_publisher(
    context: &RclGraphContext,
    topic_name: &str,
    message_type: &str,
) -> Result<rcl_publisher_t> {
    // Load type support dynamically
    let mut registry = DynamicMessageRegistry::new();
    let message_type_info = registry.load_message_type(message_type)?;
    
    let type_support = message_type_info.type_support
        .ok_or_else(|| anyhow!("Could not load type support for {}", message_type))?;
    
    unsafe {
        let mut publisher = rcl_get_zero_initialized_publisher();
        let options = rcl_publisher_get_default_options();
        let topic_name_c = CString::new(topic_name)?;
        
        // Create publisher with dynamically loaded type support!
        let ret = rcl_publisher_init(
            &mut publisher,
            context.node(),
            type_support,  // ← This comes from dynamic loading
            topic_name_c.as_ptr(),
            &options,
        );
        
        if ret != 0 {
            return Err(anyhow!("Failed to create publisher: {}", ret));
        }
        
        Ok(publisher)
    }
}
```

## Generic Type Support Resolution

### Fallback Hierarchy

`roc` uses a smart fallback strategy:

```rust
fn try_get_type_support(&self, package_name: &str, message_name: &str) -> Result<TypeSupport> {
    let full_type = format!("{}/msg/{}", package_name, message_name);
    
    match full_type.as_str() {
        // 1. Optimized paths for common types
        "geometry_msgs/msg/Twist" => self.try_get_twist_type_support(),
        "std_msgs/msg/String" => self.try_get_string_type_support(), 
        "std_msgs/msg/Int32" => self.try_get_int32_type_support(),
        "std_msgs/msg/Float64" => self.try_get_float64_type_support(),
        
        // 2. Generic fallback for ANY message type
        _ => self.try_get_generic_type_support(package_name, message_name),
    }
}
```

### Automatic Library Discovery

The generic loader automatically constructs paths:

| Message Type | Library Path | Symbol Name |
|--------------|--------------|-------------|
| `geometry_msgs/msg/Twist` | `/opt/ros/jazzy/lib/libgeometry_msgs__rosidl_typesupport_c.so` | `rosidl_typesupport_c__get_message_type_support_handle__geometry_msgs__msg__Twist` |
| `custom_msgs/msg/MyType` | `/opt/ros/jazzy/lib/libcustom_msgs__rosidl_typesupport_c.so` | `rosidl_typesupport_c__get_message_type_support_handle__custom_msgs__msg__MyType` |
| `sensor_msgs/msg/Image` | `/opt/ros/jazzy/lib/libsensor_msgs__rosidl_typesupport_c.so` | `rosidl_typesupport_c__get_message_type_support_handle__sensor_msgs__msg__Image` |

### Testing the Generic Loader

```bash
# These all work automatically:
roc topic pub /test1 geometry_msgs/msg/Twist '{linear: {x: 1.0}}'          # Known type
roc topic pub /test2 geometry_msgs/msg/Point '{x: 1.0, y: 2.0, z: 3.0}'    # Generic loading  
roc topic pub /test3 sensor_msgs/msg/Image '{header: {frame_id: "camera"}}' # Generic loading
roc topic pub /test4 custom_msgs/msg/MyType '{my_field: "value"}'           # Your custom types!
```

Output shows the dynamic loading in action:
```
Attempting generic type support loading:
  Library: /opt/ros/jazzy/lib/libgeometry_msgs__rosidl_typesupport_c.so
  Symbol: rosidl_typesupport_c__get_message_type_support_handle__geometry_msgs__msg__Point
Successfully loaded type support for symbol: ...
Successfully created RCL publisher with real type support!
```

## Benefits and Trade-offs

### Benefits ✅

1. **Universal Message Support**
   - Works with any ROS2 message type
   - Supports custom packages automatically
   - No compilation required for new types

2. **Small Binary Size**
   - No static linking of message libraries
   - Only loads what's actually used
   - Minimal memory footprint

3. **Runtime Flexibility**
   - Discover available message types at runtime
   - Work with packages installed after compilation
   - Perfect for generic tools like `roc`

4. **Performance**
   - Type support loaded once and cached
   - No runtime overhead after initial load
   - Real RCL integration (not simulation)

5. **Maintainability**
   - No manual type definitions required
   - Automatic support for new ROS2 versions
   - Self-discovering architecture

### Trade-offs ⚖️

1. **Runtime Dependencies**
   - Requires ROS2 installation with typesupport libraries
   - Fails gracefully if libraries are missing
   - Error messages help diagnose missing packages

2. **Platform Assumptions**
   - Assumes standard ROS2 installation paths
   - Library naming conventions must match
   - Works with standard ROS2 distributions

3. **Error Handling Complexity**
   - Must handle dynamic loading failures
   - Symbol resolution errors need clear messages
   - Graceful degradation for partial installations

## Future Enhancements

### 1. Introspection-Based Generic Serialization

The next evolution is **fully generic serialization** using ROS2's introspection API:

```rust
// Future: No manual serialization needed!
pub fn serialize_any_message(
    yaml_value: &YamlValue,
    type_support: *const rosidl_message_type_support_t,
) -> Result<Vec<u8>> {
    // 1. Get introspection data from type_support
    let introspection = get_message_introspection(type_support)?;
    
    // 2. Walk the message structure automatically
    let message_ptr = allocate_message_memory(introspection.size_of);
    serialize_fields_recursively(yaml_value, introspection.members, message_ptr)?;
    
    // 3. Use RMW to serialize to CDR format
    let serialized = rmw_serialize(message_ptr, type_support)?;
    Ok(serialized)
}
```

### 2. Automatic Package Discovery

```rust
// Future: Scan filesystem for available message types
pub fn discover_available_message_types() -> Vec<String> {
    let lib_dir = "/opt/ros/jazzy/lib";
    let pattern = "lib*__rosidl_typesupport_c.so";
    
    // Scan libraries and extract symbols
    scan_libraries_for_message_types(lib_dir, pattern)
}
```

### 3. Message Definition Introspection

```rust
// Future: Runtime message structure inspection
pub fn get_message_definition(message_type: &str) -> Result<MessageDefinition> {
    let type_support = load_type_support(message_type)?;
    let introspection = get_introspection_data(type_support)?;
    
    // Return complete message structure info
    Ok(MessageDefinition {
        fields: extract_field_definitions(introspection),
        dependencies: find_nested_types(introspection),
        size: introspection.size_of,
    })
}
```

### 4. Performance Optimizations

- **Library preloading** for common types
- **Symbol caching** across multiple calls  
- **Memory pool** for message allocation
- **Batch operations** for multiple message types

## Conclusion

Dynamic message type loading is a sophisticated technique that gives `roc` **universal ROS2 message support** without the limitations of static linking. By leveraging:

- **Runtime dynamic library loading** (`dlopen`/`dlsym`)
- **ROS2 type support architecture** 
- **Automatic path and symbol construction**
- **Graceful fallback strategies**

`roc` can work with **any ROS2 message type** - including custom packages you create! This makes it a truly generic and powerful tool for ROS2 development.

The implementation demonstrates advanced systems programming concepts while remaining maintainable and extensible. It's a great example of how understanding the underlying architecture (ROS2 type support system) enables building more flexible and powerful tools.

**Key takeaway**: Dynamic loading isn't just a neat trick - it's a fundamental technique that enables building truly generic and extensible systems that can adapt to runtime conditions and work with code that didn't exist at compile time.