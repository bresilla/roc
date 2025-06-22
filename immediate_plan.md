# Immediate Plan: Fix Topic Publishing - Topics Not Appearing in Graph

## Problem Statement

When using `roc topic pub`, the published topics don't appear when running `roc topic list`. The message parsing and serialization work correctly, but the topics are not registered in the ROS graph.

## Root Cause Analysis

### Why Topics Don't Appear in `roc topic list`

The fundamental issue is that **we cannot create a real RCL publisher without valid type support**. Here's the exact problem:

1. **RCL Publisher Creation Requires Type Support**: The `rcl_publisher_init()` function requires a valid `rosidl_message_type_support_t*` pointer to register the topic with the correct message type in the DDS layer.

2. **Our Current Type Support Loading is Incomplete**: In our `dynamic_messages.rs`, the functions like `try_get_twist_type_support()` return errors because we haven't implemented the actual dynamic library loading and symbol resolution.

3. **No Publisher = No Topic Registration**: Without calling `rcl_publisher_init()` successfully, the topic never gets registered with the ROS middleware, so it doesn't appear in the graph.

## Current Implementation Status

Right now our implementation:
- ✅ **Parses and validates YAML messages correctly**
- ✅ **Serializes messages to binary format correctly** 
- ❌ **Cannot create real RCL publishers** (missing type support)
- ❌ **Topics don't appear in graph** (no publisher registration)

## Required Implementation Steps

### 1. Complete Dynamic Type Support Loading

We need to implement these missing functions in `rclrs/build.rs`:

```rust
rcutils_load_shared_library()  // Load .so files dynamically
rcutils_get_symbol()           // Extract type support functions
rcutils_unload_shared_library() // Cleanup
```

**Current Issue**: These functions aren't being generated in our bindings.

### 2. Fix the Bindgen Configuration

- The dynamic loading functions aren't being generated in our bindings
- We need to ensure these functions are properly exposed from the RCL headers
- Check why `rcutils_load_shared_library` and related functions are missing from generated bindings

### 3. Implement Actual Type Support Resolution

In `try_get_twist_type_support()`, we need to:

```rust
// 1. Load lib geometry_msgs__rosidl_typesupport_c.so
let library_name = "libgeometry_msgs__rosidl_typesupport_c";
let mut shared_lib = load_shared_library(library_name)?;

// 2. Find symbol: rosidl_typesupport_c__get_message_type_support_handle__geometry_msgs__msg__Twist
let symbol_name = "rosidl_typesupport_c__get_message_type_support_handle__geometry_msgs__msg__Twist";
let type_support_fn = get_symbol(&shared_lib, symbol_name)?;

// 3. Call the function to get the type support pointer
let type_support_ptr = type_support_fn();

// 4. Use that pointer in rcl_publisher_init()
let ret = rcl_publisher_init(&mut publisher, node, type_support_ptr, topic_name, &options);
```

### 4. Alternative Approach - Static Type Support

Instead of dynamic loading, we could:
- Statically link against message packages (geometry_msgs, std_msgs, etc.)
- Add these as dependencies in Cargo.toml
- Directly call their type support functions
- This would be simpler but less flexible

## Technical Deep Dive

### The Missing Link Chain

```
YAML Input → Parse → Validate → Serialize → [MISSING: Type Support] → RCL Publisher → Topic Registration
```

The missing piece is the type support resolution that allows RCL to understand the message structure.

### Required RCL Call Sequence

```rust
// This is what we need to achieve:
let type_support = get_dynamic_type_support("geometry_msgs/msg/Twist")?;
let ret = rcl_publisher_init(
    &mut publisher,
    context.node(),
    type_support,           // This is what we're missing
    topic_name_c.as_ptr(),
    &publisher_options,
);
```

## Recommended Next Steps

### Priority 1: Fix Bindgen Issue
1. Investigate why dynamic loading functions are missing from bindings
2. Update `rclrs/build.rs` and `wrapper.h` to ensure these functions are included
3. Verify that the required RCL libraries are properly linked

### Priority 2: Implement Dynamic Type Support Loading
1. Complete the implementation in `try_get_twist_type_support()` 
2. Test with one message type first (geometry_msgs/msg/Twist)
3. Verify that topics appear in `roc topic list` after publishing

### Priority 3: Expand to Other Message Types
1. Implement type support for std_msgs (String, Int32, Float64)
2. Create a generic type support loading mechanism
3. Test with various message types

### Priority 4: Error Handling and Robustness
1. Handle missing type support libraries gracefully
2. Provide clear error messages when type support can't be loaded
3. Add fallback mechanisms or suggestions

## Success Criteria

1. **`roc topic pub /test_topic geometry_msgs/msg/Twist '{linear: {x: 0.5}}'`** creates a publisher
2. **`roc topic list`** shows `/test_topic` with correct message type
3. **`roc topic info /test_topic`** shows publisher information
4. **Message content is properly serialized and ready for actual publishing**

## Core Blocker

**ROS2 requires valid type support pointers to create publishers**, and we haven't completed the dynamic type support loading infrastructure yet. This is the fundamental blocker preventing topics from appearing in the graph.

## Files That Need Changes

1. **`rclrs/build.rs`** - Fix bindgen configuration for dynamic loading functions
2. **`rclrs/wrapper.h`** - Ensure proper header includes
3. **`src/shared/dynamic_messages.rs`** - Complete type support loading implementation
4. **`src/commands/topic/pub_.rs`** - Use real type support in publisher creation

## Timeline Estimate

- **Bindgen fix**: 1-2 hours
- **Type support implementation**: 4-6 hours  
- **Testing and debugging**: 2-3 hours
- **Total**: 1 full day of focused work

The path forward is clear: we need to complete the dynamic type support loading to create real RCL publishers that register topics in the ROS graph.