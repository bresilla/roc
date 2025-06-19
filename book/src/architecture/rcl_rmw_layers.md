# RCL and RMW Layers

The RCL (Robot Control Library) and RMW (ROS Middleware) layers form the core of ROS 2's architecture. Understanding these layers is essential for implementing effective bindings and tools.

## RMW Layer (ROS Middleware Interface)

### Purpose and Design
The RMW layer serves as an abstraction barrier between ROS 2 and specific DDS implementations. This design allows ROS 2 to work with different middleware providers without changing upper-layer code.

### Key Data Structures

#### Topic Endpoint Information
```c
typedef struct rmw_topic_endpoint_info_s {
    const char * node_name;                    // Node that owns this endpoint
    const char * node_namespace;               // Node's namespace
    const char * topic_type;                   // Message type name
    rosidl_type_hash_t topic_type_hash;        // Hash of message definition
    rmw_endpoint_type_t endpoint_type;         // PUBLISHER or SUBSCRIPTION
    uint8_t endpoint_gid[RMW_GID_STORAGE_SIZE]; // Global unique identifier
    rmw_qos_profile_t qos_profile;             // Quality of Service settings
} rmw_topic_endpoint_info_t;
```

This structure contains all the detailed information about a topic endpoint that `roc` displays in verbose mode.

#### QoS Profile Structure
```c
typedef struct rmw_qos_profile_s {
    rmw_qos_history_policy_e history;          // KEEP_LAST, KEEP_ALL
    size_t depth;                              // Queue depth for KEEP_LAST
    rmw_qos_reliability_policy_e reliability;  // RELIABLE, BEST_EFFORT
    rmw_qos_durability_policy_e durability;    // VOLATILE, TRANSIENT_LOCAL
    rmw_time_s deadline;                       // Maximum time between messages
    rmw_time_s lifespan;                       // How long messages stay valid
    rmw_qos_liveliness_policy_e liveliness;    // Liveliness assertion policy
    rmw_time_s liveliness_lease_duration;      // Liveliness lease time
    bool avoid_ros_namespace_conventions;      // Bypass ROS naming
} rmw_qos_profile_t;
```

### RMW Functions Used by `roc`

The key RMW functions that our implementation uses:

```c
// Get detailed publisher information
rmw_ret_t rmw_get_publishers_info_by_topic(
    const rmw_node_t * node,
    rcutils_allocator_t * allocator,
    const char * topic_name,
    bool no_mangle,
    rmw_topic_endpoint_info_array_t * publishers_info
);

// Get detailed subscriber information  
rmw_ret_t rmw_get_subscriptions_info_by_topic(
    const rmw_node_t * node,
    rcutils_allocator_t * allocator,
    const char * topic_name,
    bool no_mangle,
    rmw_topic_endpoint_info_array_t * subscriptions_info
);
```

## RCL Layer (Robot Control Library)

### Purpose and Design
The RCL layer provides a C API that manages:
- Context initialization and cleanup
- Node lifecycle management
- Graph introspection
- Resource management

### Key RCL Functions

#### Context and Node Management
```c
// Initialize RCL context
rcl_ret_t rcl_init(
    int argc,
    char const * const * argv,
    const rcl_init_options_t * options,
    rcl_context_t * context
);

// Initialize a node
rcl_ret_t rcl_node_init(
    rcl_node_t * node,
    const char * name,
    const char * namespace_,
    rcl_context_t * context,
    const rcl_node_options_t * options
);
```

#### Graph Introspection
```c
// Get all topics and their types
rcl_ret_t rcl_get_topic_names_and_types(
    const rcl_node_t * node,
    rcutils_allocator_t * allocator,
    bool no_demangle,
    rcl_names_and_types_t * topic_names_and_types
);

// Count publishers for a topic
rcl_ret_t rcl_count_publishers(
    const rcl_node_t * node,
    const char * topic_name,
    size_t * count
);
```

#### Detailed Endpoint Information
```c
// Get detailed publisher info (wraps RMW function)
rcl_ret_t rcl_get_publishers_info_by_topic(
    const rcl_node_t * node,
    rcutils_allocator_t * allocator,
    const char * topic_name,
    bool no_mangle,
    rcl_topic_endpoint_info_array_t * publishers_info
);
```

## Type Mapping and Aliases

RCL often provides type aliases for RMW types:

```c
// RCL aliases for RMW types
typedef rmw_topic_endpoint_info_t rcl_topic_endpoint_info_t;
typedef rmw_topic_endpoint_info_array_t rcl_topic_endpoint_info_array_t;
typedef rmw_names_and_types_t rcl_names_and_types_t;
```

This design means that RCL functions often directly pass through to RMW implementations.

## Error Handling

Both RCL and RMW use integer return codes:

```c
#define RCL_RET_OK                    0
#define RCL_RET_ERROR                 1
#define RCL_RET_BAD_ALLOC            10
#define RCL_RET_INVALID_ARGUMENT     11
#define RCL_RET_NODE_INVALID         200
```

Our Rust bindings convert these into `Result<T, anyhow::Error>` types for idiomatic error handling.

## Memory Management

### Key Principles
1. **Caller allocates, caller deallocates**: The caller must provide allocators and clean up resources
2. **Array finalization**: Arrays returned by RCL/RMW must be finalized with specific functions
3. **String lifecycle**: Strings in returned structures may have complex ownership

### Example: Proper Resource Cleanup
```c
// Initialize array
rcl_topic_endpoint_info_array_t publishers_info = rmw_get_zero_initialized_topic_endpoint_info_array();

// Get data
rcl_get_publishers_info_by_topic(node, &allocator, topic_name, false, &publishers_info);

// Use data...

// Clean up (REQUIRED)
rmw_topic_endpoint_info_array_fini(&publishers_info, &allocator);
```

This pattern is critical for preventing memory leaks in long-running applications like `roc`.

## Integration with DDS

The RMW layer abstracts DDS-specific details, but understanding the mapping helps:

| ROS 2 Concept | DDS Concept | Purpose |
|---------------|-------------|---------|
| Node | DDS Participant | Process-level entity |
| Publisher | DDS Publisher + DataWriter | Sends data |
| Subscription | DDS Subscriber + DataReader | Receives data |
| Topic | DDS Topic | Communication channel |
| QoS Profile | DDS QoS Policies | Communication behavior |
| GID | DDS Instance Handle | Unique endpoint ID |

This layered approach allows `roc` to access both high-level ROS 2 concepts and low-level DDS details through a unified interface.
