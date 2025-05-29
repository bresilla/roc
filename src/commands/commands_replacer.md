Understood. I’ll dig into the `rcl` C-based Graph APIs in detail, covering all key functions for accessing node names, topic and service types, publishers/subscribers, actions, and graph event handling. I’ll also include how these APIs interface with the `rmw` layer and how to use them effectively via FFI in a Rust-based CLI tool.


# ROS 2 `rcl` Graph APIs and the `rmw` Layer Interface

**Overview:** In ROS 2, the **graph** represents the collection of all nodes, topics, services, and their connections in the runtime system. The C-language ROS Client Library (`rcl`) provides Graph APIs to introspect this information. Under the hood, `rcl` relies on the ROS Middleware Interface (`rmw`) to query the underlying DDS or ROS 2 middleware for discovery data (node names, topic lists, etc.). Each `rcl` Graph function typically maps to a corresponding `rmw` function that actually retrieves the data from the middleware. This reference explains each Graph API in `rcl`, how they use `rmw`, the data structures and memory management involved, how graph changes are signaled via guard conditions, and typical usage patterns in C (with notes for Rust FFI bindings).

&#x20;*ROS 2 layered architecture: user code calls into client libraries (rclcpp, rclpy, etc.), which use `rcl` (C API). The `rcl` layer in turn uses the `rmw` abstraction to interface with the specific DDS or middleware implementation. Graph information (nodes, topics, services) is gathered via the middleware’s discovery mechanisms and exposed through the `rcl` Graph APIs.*

## Data Structures and Memory Management in Graph APIs

Before diving into each API, it's important to understand the common data structures used for graph information and how memory is managed. All Graph query functions allocate memory for their results, and the caller is responsible for freeing this memory via the appropriate *fini* functions.

* **String Array (`rcutils_string_array_t`):** Used for lists of node names and namespaces. This struct contains a dynamic array of C strings. It has fields for the array size, a pointer to an array of `char*` strings, and an allocator for memory management. You must initialize a `rcutils_string_array_t` to zero before use (e.g. with `rcutils_get_zero_initialized_string_array()`), and finalize it with `rcutils_string_array_fini()` to free the allocated strings and array.

* **Names and Types (`rcl_names_and_types_t` / `rmw_names_and_types_t`):** Used for topic and service listings. `rcl_names_and_types_t` is a typedef alias of `rmw_names_and_types_t`, essentially a structure containing an array of names and a parallel array of type lists. Specifically, it holds an `rcutils_string_array_t` of names, and an array of `rcutils_string_array_t` for the types corresponding to each name. Each name\[i] has a list of types in types\[i], and `types` array length equals the number of names. A zero-initialized instance can be obtained via `rcl_get_zero_initialized_names_and_types()` (which uses the RMW zero-initializer) and must be freed with `rcl_names_and_types_fini()`. Finalizing frees all internally allocated strings and arrays.

**Memory allocation:** All these functions take an `rcl_allocator_t` (usually the default allocator) to allocate memory for strings. For example, `rcl_get_topic_names_and_types` will allocate space for each discovered topic name and its types using the given allocator. In C, you typically pass `rcutils_get_default_allocator()` as the allocator. **Important:** when using these APIs via FFI (e.g., in Rust), do not manually free or modify the returned pointers. Instead, call the appropriate `*_fini` function (from C via FFI) to release memory using the same allocator. Failing to finalize results in memory leaks. Also ensure that output structures are zero-initialized in Rust before calling (to avoid undefined behavior) and not moved or dropped until finalized.

**Pitfalls:** One subtle issue is that some RMW implementations historically could return empty or `NULL` names for nodes if the name was not yet discovered. The documentation notes that entries in the node name array might be `NULL` in some cases, though newer implementations try to avoid this. When iterating results, always check for empty strings. Additionally, none of these functions automatically apply name remapping rules – they return the names as known in the graph. Creating publishers/subscribers or services using these returned names without considering remap rules may not yield the expected entity.

## rcl\_get\_node\_names

**Purpose:** Retrieve a list of all node names currently in the ROS graph, along with their namespaces. This gives a snapshot of all nodes discovered by the local node.

**Signature:** `rcl_ret_t rcl_get_node_names(const rcl_node_t *node, rcl_allocator_t allocator, rcutils_string_array_t *node_names, rcutils_string_array_t *node_namespaces);`

**Behavior:** This function populates two parallel string arrays: one for node names and one for their corresponding namespaces. Both `node_names` and `node_namespaces` must be allocated and zero-initialized by the caller before the call. After the call, each index `i` in these arrays represents a node: `node_names.data[i]` is the name and `node_namespaces.data[i]` is that node’s namespace.

**rmw Interface:** Internally, `rcl_get_node_names` uses the `rmw_get_node_names` function of the middleware. The `rcl_node_t` is essentially a handle containing an `rmw_node_t`. The implementation will pass the underlying `rmw_node` to `rmw_get_node_names` to get the actual data from the DDS layer. The RMW function returns node names and namespaces as string arrays. Thus, the results are ultimately provided by discovery data from the middleware. (Notably, ROS 2 also has an `rcl_get_node_names_with_enclaves` variant that additionally returns each node’s security enclave name, which similarly uses `rmw_get_node_names_with_enclaves`.)

**Memory:** Memory for strings is allocated via the given allocator. The caller must finalize both `node_names` and `node_namespaces` with `rcutils_string_array_fini()` when done. Each of those functions will free the internal char\*\* array and each name string. Use caution in FFI: these strings are owned by the `rcl` layer; if you need them beyond the lifetime of the C structures, copy them into Rust-owned memory. Also note, if the node is invalid or the arrays are not zero-initialized, the function returns error codes without allocating memory.

**Usage example (C):**

```c
rcl_allocator_t alloc = rcl_get_default_allocator();
rcutils_string_array_t node_names = rcutils_get_zero_initialized_string_array();
rcutils_string_array_t node_ns = rcutils_get_zero_initialized_string_array();

rcl_ret_t ret = rcl_get_node_names(my_node, alloc, &node_names, &node_ns);
if (ret != RCL_RET_OK) {
    // error handling
}
for (size_t i = 0; i < node_names.size; ++i) {
    const char *name = node_names.data[i];
    const char *ns   = node_ns.data[i];
    printf("Discovered node: %s (namespace: %s)\n", name ? name : "(null)", ns ? ns : "(null)");
}
rcutils_string_array_fini(&node_names);
rcutils_string_array_fini(&node_ns);
```

This will print all nodes (and their namespaces) visible to `my_node`. Remember that if any entry was returned as NULL or empty, it should be handled (here we print "(null)" if so).

**Error handling:** If the `node` is not valid, `RCL_RET_NODE_INVALID` is returned. If any argument is invalid, `RCL_RET_INVALID_ARGUMENT` is returned. If the `node_name` list cannot be retrieved (e.g. a lower-level error), a generic `RCL_RET_ERROR` is returned. If using a *by-name* query (described next) and the specified node name is not found in the graph, `RCL_RET_NODE_NAME_NON_EXISTENT` is returned instead.

## rcl\_get\_topic\_names\_and\_types

**Purpose:** Retrieve all topic names in the ROS graph along with their type names. This includes any topic that has at least one publisher or subscriber discovered by the given node.

**Signature:** `rcl_ret_t rcl_get_topic_names_and_types(const rcl_node_t *node, rcl_allocator_t *allocator, bool no_demangle, rcl_names_and_types_t *topic_names_and_types);`

**Behavior:** This function populates a `rcl_names_and_types_t` structure (names-and-types list) with every topic name currently known, and the list of type names for each topic. If multiple ROS message types are published on the same topic name, all those type names will appear in the types list for that topic. The `no_demangle` boolean controls whether to return **ROS 2 demangled names** or raw middleware topic names. In ROS 2, certain internal topics or hidden names use a mangling scheme (for example, DDS topic names for ROS services, or the ROS “hidden” topics that start with `_`). By default (`no_demangle = false`), the RMW implementation may translate or filter topic names to match conventional ROS names (e.g. hiding the ROS service request/reply topics, or translating DDS-specific prefixes). If `no_demangle` is `true`, the names are returned exactly as seen at the middleware level, with no filtering. (See the ROS 2 design article on topic/service name mapping for details.)

**rmw Interface:** `rcl_get_topic_names_and_types` directly calls the `rmw_get_topic_names_and_types` function for the underlying middleware. The `rmw_node_t` of the given node is passed in, along with the demangle flag, and the RMW populates an `rmw_names_and_types_t` (which `rcl` treats as `rcl_names_and_types_t`). The RMW implementation queries its discovery data for all topics and their types. For instance, in a DDS-based RMW, this might iterate over discovered DataReaders/DataWriters to collect topic names. It is a synchronous, non-blocking call that should return quickly with cached discovery info. The RMW is responsible for populating the names and types arrays (likely using the provided allocator).

**Memory:** The `topic_names_and_types` output must be zero-initialized before calling (e.g. using `rcl_get_zero_initialized_names_and_types()`). On success, it will contain allocated memory (each topic name string and each type string is allocated). The caller must free this by calling `rcl_names_and_types_fini(&topic_names_and_types)`. That will free all name strings and type strings. **Important for FFI:** The structure contains pointers to allocated C strings; do not mutate these from Rust. Instead, iterate and copy if needed, then call the fini function. Also, ensure that `rcl_names_and_types_t` in Rust has the same memory layout as the C struct (which it will if you use the C definition, since it’s just two pointers and a size). Typically, you can call these functions from Rust using the bindings, then wrap the result in a safe Rust structure that knows to call fini in its Drop.

**Usage example (C):**

```c
rcl_names_and_types_t topic_names_types = rcl_get_zero_initialized_names_and_types();
rcl_ret_t ret = rcl_get_topic_names_and_types(node, &allocator, false, &topic_names_types);
if (ret != RCL_RET_OK) {
    // handle error
}
// Iterate through topics
for (size_t i = 0; i < topic_names_types.names.size; ++i) {
    const char *topic = topic_names_types.names.data[i];
    printf("Topic: %s\n", topic);
    // List all types for this topic
    rcutils_string_array_t types_list = topic_names_types.types[i];
    for (size_t j = 0; j < types_list.size; ++j) {
        printf("  - Type: %s\n", types_list.data[j]);
    }
}
rcl_names_and_types_fini(&topic_names_types);
```

This prints each topic and the associated type(s). If `no_demangle` were `true`, the topic names might include ROS internal topics or be fully qualified with type mangling (as defined by the middleware).

**Notes:** The returned topic names are not automatically remapped or resolved; they are exactly as known in the graph. If you use a returned name to create a new subscription/publisher, ROS remapping rules could still remap it at creation time. Also, this function only lists topics that *currently* have at least one publisher or subscriber. It won’t list latent topic names with no entities.

## rcl\_get\_service\_names\_and\_types

**Purpose:** Retrieve all service names in the ROS graph and their service type. This lists services that are available (i.e., at least one node advertises a service of that name).

**Signature:** `rcl_ret_t rcl_get_service_names_and_types(const rcl_node_t *node, rcl_allocator_t *allocator, rcl_names_and_types_t *service_names_and_types);`

**Behavior:** It populates a names-and-types list where each name is a **service name** and the types array contains the service’s type (each service has exactly one type, consisting of request/response pair, but presented as a single service type name). If multiple services with the same name but different types were somehow in the graph, they would appear as additional types for the name (though this situation is unlikely in normal ROS usage since service names are typically unique).

**rmw Interface:** This calls the RMW’s `rmw_get_service_names_and_types` implementation. The middleware will use its discovery information to find all service servers and gather their service names and types. In DDS-based RMWs, services are implemented with two topics (request and response topics), but the RMW knows how to derive the ROS service name from those. It then populates an `rmw_names_and_types_t` structure for services.

**Memory:** Like the topic case, the output `service_names_and_types` must be zero-initialized and later freed with `rcl_names_and_types_fini()`. Each service name string and type string is allocated. The same FFI considerations apply: use the provided C API to manage the memory. The example usage is analogous to topics.

**Note:** The returned names are not remapped. If you take a service name from here to create a client, it may be subject to remapping at that creation time, but the introspection itself doesn’t apply remap rules. Typically, this function is used to list available services (e.g., for a CLI tool that lists all services in the system along with their type).

## rcl\_get\_publisher\_names\_and\_types\_by\_node

**Purpose:** For a given remote node (specified by name and namespace), list all topic names that the node publishes, along with the type of each topic. This allows you to inspect what a specific node is publishing.

**Signature:** `rcl_ret_t rcl_get_publisher_names_and_types_by_node(const rcl_node_t *node, rcl_allocator_t *allocator, bool no_demangle, const char *node_name, const char *node_namespace, rcl_names_and_types_t *topic_names_and_types);`

**Behavior:** You must supply the target node’s name and namespace as strings (`node_name`, `node_namespace`). The function will search the ROS graph for a node matching that name/namespace, and if found, fill `topic_names_and_types` with all the topic names that node is publishing, and the list of type names on each of those topics. The `no_demangle` option works the same as in `rcl_get_topic_names_and_types`: if false, the topic names may be filtered or processed to hide ROS-specific mangling (e.g., it will exclude the hidden service request/reply topics from this list, since those are not user-facing topics). If true, you get the raw topic names exactly as the middleware knows them.

**rmw Interface:** This relies on `rmw_get_publisher_names_and_types_by_node` in the middleware. The RMW will look for the node in its discovery data and collect all publishers associated with that node (often by matching the node's unique ID or name in the discovery info). It then returns the topic names and types. If the specified node name is not found, the RMW returns an indication of that, which `rcl` translates to `RCL_RET_NODE_NAME_NON_EXISTENT`.

**Memory:** The output is an `rcl_names_and_types_t` that must be zero-initialized and later finalized with `rcl_names_and_types_fini()`. Each string is allocated via the provided allocator. If the node isn’t found, the function returns an error and leaves `topic_names_and_types` zero-initialized (no allocation performed). After a successful call, free the results with the fini function. In an FFI scenario, ensure the `node_name` and `node_namespace` C strings you pass in are null-terminated and valid (lifetime long enough for the call). The returned strings reside in C memory.

**Usage example (C):**

```c
rcl_names_and_types_t pub_topics = rcl_get_zero_initialized_names_and_types();
const char *target_node_name = "my_robot_controller";
const char *target_node_ns   = "/robot";  // e.g. namespace of the target node
rcl_ret_t ret = rcl_get_publisher_names_and_types_by_node(my_node, &alloc, false,
                                                         target_node_name, target_node_ns,
                                                         &pub_topics);
if (ret == RCL_RET_OK) {
    printf("Node %s publishes %zu topics:\n", target_node_name, pub_topics.names.size);
    for (size_t i = 0; i < pub_topics.names.size; ++i) {
        printf("  %s : [", pub_topics.names.data[i]);
        for (size_t j = 0; j < pub_topics.types[i].size; ++j) {
            printf("%s%s", pub_topics.types[i].data[j], (j+1 < pub_topics.types[i].size ? ", " : ""));
        }
        printf("]\n");
    }
}
rcl_names_and_types_fini(&pub_topics);
```

This would output each topic that `"my_robot_controller"` node (in `/robot` namespace) is publishing, along with the message types for each. If the node isn’t found in the graph, the return value would be `RCL_RET_NODE_NAME_NON_EXISTENT` and you should handle that (e.g., notify that the node name was not found).

**Thread-safety:** Note that all the “by node” query functions are *not* marked thread-safe in `rcl`. This means you should not call them concurrently on the same `rcl_node_t` from multiple threads without external synchronization. The underlying RMW calls are generally thread-safe (the RMW documentation indicates that querying the graph can be done concurrently on the same node), but `rcl` may have its own state or simply doesn’t guarantee safety in its API contract. If you are writing a Rust FFI wrapper, it’s wise to ensure these functions are called in a thread-safe manner (e.g., by locking around calls if using the same node in multiple threads).

## rcl\_get\_subscriber\_names\_and\_types\_by\_node

**Purpose:** Similar to the above, but for subscriptions. It lists all topics that a given remote node subscribes to, with the type of each topic.

**Signature:** `rcl_ret_t rcl_get_subscriber_names_and_types_by_node(const rcl_node_t *node, rcl_allocator_t *allocator, bool no_demangle, const char *node_name, const char *node_namespace, rcl_names_and_types_t *topic_names_and_types);`

**Behavior:** You provide the target node’s name/namespace, and it returns all topic names that node has subscriptions on (i.e. that it is listening to), along with their types. As with the publisher version, `no_demangle` controls whether ROS-specific name mangling is undone or not. Typically, you will leave `no_demangle=false` to get a clean list of topics (excluding hidden/internal names). If true, you might see subscription topics that include hidden names (like parameter events or even the service response topics if a node has an internal subscription to those).

**rmw Interface:** Uses `rmw_get_subscriber_names_and_types_by_node` underneath. The middleware finds all subscriptions for the identified node and returns their topic names and types. If the node doesn’t exist, you get `RCL_RET_NODE_NAME_NON_EXISTENT`. If the node exists but simply has no subscriptions, you would get an empty list (the call would still return `RCL_RET_OK` with `topic_names_and_types.names.size == 0`).

**Memory:** Managed exactly like the publisher case. Initialize the output with zero, and finalize it after use. Each subscription topic name and its types are allocated. In Rust FFI usage, treat these the same way: call fini to free, and avoid modifying the data.

**Usage:** Usage is analogous to the publisher case. For example, to list what topics `"my_robot_controller"` is subscribing to, you’d call `rcl_get_subscriber_names_and_types_by_node` and iterate the result. Each topic name returned is one that the target node has a subscription on. (This is useful for tools that introspect what data a node is consuming.)

**Note:** These “by node” functions (`rcl_get_publisher_names_and_types_by_node` and `rcl_get_subscriber_names_and_types_by_node`) are often used together to display a node’s pub/sub interface. Keep in mind that if a node has no publishers or no subscriptions, the corresponding result will be an empty list (but still needs to be fini’d). Also, *only nodes that this local context has discovered* are considered. In a large system, discovery might take a moment; if you call immediately on startup, some nodes might not be known yet.

## rcl\_get\_service\_names\_and\_types\_by\_node

**Purpose:** List all *service servers* provided by a given node, including the service names and their types. In other words, for a specified node, get the names of services that node offers (as a server) and the service type for each.

**Signature:** `rcl_ret_t rcl_get_service_names_and_types_by_node(const rcl_node_t *node, rcl_allocator_t *allocator, const char *node_name, const char *node_namespace, rcl_names_and_types_t *service_names_and_types);`

**Behavior:** Provide the target node’s name and namespace; the function will fill `service_names_and_types` with an entry for each service server advertised by that node. Each name in the list is a service name, and the corresponding type array contains the interface type (e.g., `example_interfaces/srv/AddTwoInts`) that the service uses. A node that provides multiple services will have multiple entries. If the node has no services, you get an empty list. If the node name doesn’t exist in the graph, you get `RCL_RET_NODE_NAME_NON_EXISTENT`.

**rmw Interface:** This uses `rmw_get_service_names_and_types_by_node` under the hood. The RMW queries its discovery info for services offered by the given node. In DDS, this might involve looking at discovered DataReaders/DataWriters with certain prefixes used for ROS services (e.g., request topics). The RMW knows how to infer the service name and type from those and returns them.

**Memory:** As with other names-and-types outputs, allocate with zero init and free with `rcl_names_and_types_fini()` when done. Each service name and type string is allocated. From Rust, one approach is to wrap this call in a safe abstraction that yields a Rust vector of (String, Vec<String>) for name and types, then immediately finalize the C structure.

**Usage:** For example, to inspect what services a node offers, you would call this function and print each service name and type. This is useful for introspection tools (like `ros2 node info` command). The code pattern is similar to the pub/sub case:

```c
rcl_names_and_types_t services = rcl_get_zero_initialized_names_and_types();
rcl_ret_t ret = rcl_get_service_names_and_types_by_node(node, &alloc,
                                                       "my_robot_controller", "/robot",
                                                       &services);
if (ret == RCL_RET_OK) {
    for (size_t i = 0; i < services.names.size; ++i) {
        printf("Service %s : type %s\n", services.names.data[i],
               (services.types[i].size > 0 ? services.types[i].data[0] : "unknown"));
    }
}
rcl_names_and_types_fini(&services);
```

Here we expect each `services.types[i]` to have exactly one type (the service type), so we take the first element. (The API is generalized as names->types list for consistency, even though a service server should only have one type.) If a node had both a service `Foo` and another service `Bar`, both would be listed.

**Note:** This is for *service servers*. There is a separate concept of service clients (nodes that call services). Those are introspected by the next function.

## rcl\_get\_client\_names\_and\_types\_by\_node

**Purpose:** List all *service clients* used by a given node, including the service names they call and the type of each service. In other words, for a specified node, get the names of services that node is a client of.

**Signature:** `rcl_ret_t rcl_get_client_names_and_types_by_node(const rcl_node_t *node, rcl_allocator_t *allocator, const char *node_name, const char *node_namespace, rcl_names_and_types_t *service_names_and_types);`

**Behavior:** Provide a target node name/namespace; this fills the `service_names_and_types` output with each service name that node has created a client for, along with the service type. For instance, if node `"teleop"` has a client for `/robot/set_speed` (type `robot_interfaces/srv/SetSpeed`), you would get an entry for that. If the node has no service clients, the list will be empty.

**rmw Interface:** Corresponds to `rmw_get_client_names_and_types_by_node` in the middleware. The RMW finds all service clients associated with the given node and returns their service names and types. Internally, since ROS service clients are implemented with DDS entities (a requester pairing), the middleware uses discovery to identify those. If the node is not found, `RCL_RET_NODE_NAME_NON_EXISTENT` is returned as usual.

**Memory:** Managed similarly to the service-servers case. Initialize `service_names_and_types` to zero, call the function, then free with `rcl_names_and_types_fini()`. Each name and type string is allocated. For Rust FFI, ensure proper string handling; you might use `CString` for input node name/namespace, and remember to finalize the output to avoid leaks.

**Notes:** This function allows introspecting which services a node is calling (for example, you could detect that node `A` calls the `/some_service` service of type `X`). One possible pitfall is that if a node has a client for a service that no server is currently available for, the client still exists and will be listed here. So this is not a list of *available* services (that would be the global service list from `rcl_get_service_names_and_types`), but specifically the services that node intends to use. Also, as with others, names are not remapped or formatted – they are raw service names. If the node name isn’t found, you get an error. If found but no clients, you get a valid empty list.

## Graph Guard Condition and Updates (rcl\_node\_get\_graph\_guard\_condition)

While the above functions provide snapshots of the graph state, ROS 2 also provides a mechanism to be **notified of graph changes**. Each `rcl_node_t` maintains an internal *graph guard condition* that is triggered whenever the ROS graph changes (e.g., a new node appears, a publisher/subscription is created or destroyed, a service becomes available or disappears, etc.). You can access this via `rcl_node_get_graph_guard_condition()`.

**Signature:** `const rcl_guard_condition_t * rcl_node_get_graph_guard_condition(const rcl_node_t *node);`

This returns a pointer to an internal guard condition associated with the node. A guard condition in ROS 2 is a kind of wake-up signal that can be waited on (similar to a conditional variable or event). The returned guard condition is **triggered anytime there is a change in the ROS graph** as seen by this node. Graph changes include (but are not limited to):

* A node joining or leaving the graph (discovery of a new node or a node going offline).
* A publisher being created or destroyed (which changes the topic list).
* A subscription being created or destroyed.
* A service server or client appearing or disappearing.
* Matching events (e.g., a publisher matching or unmatching a subscription due to QoS compatibility).

Whenever any of these events occur, the guard condition will be set (triggered). In the ROS wait set API, this guard condition can be added to a wait set alongside subscriptions, timers, etc. When `rcl_wait()` returns, if the guard condition is triggered, you know the graph has changed. The typical usage pattern is: a node’s executor (or a tool) will wait on the graph guard condition, and when it triggers, the code will then call the above graph query functions (like `rcl_get_topic_names_and_types`, etc.) to get the updated info.

**rmw Interface:** The guard condition is actually created and managed by the `rmw` implementation. When you create a node (`rcl_node_init`), the underlying `rmw_create_node` typically also creates a corresponding `rmw_guard_condition_t` for graph changes. `rmw_node_get_graph_guard_condition(rmw_node)` is used to retrieve it. The RMW implementation is responsible for signaling this guard condition when it detects graph changes (for example, in DDS, when discovery data changes, it will signal the guard condition). The `rcl_node_get_graph_guard_condition` simply returns the `rcl_guard_condition_t` wrapper around that RMW guard condition pointer.

**Important usage notes:** The returned guard condition is owned by the node; you should **not** destroy it manually (it will be cleaned up when the node is destroyed). Also, it becomes invalid if the node is finalized or if `rcl_shutdown` is called. In C, you typically do not need to manipulate the guard condition directly except to add it to a wait set. For example:

```c
rcl_guard_condition_t * graph_gc = (rcl_guard_condition_t*) rcl_node_get_graph_guard_condition(node);
// ... initialize a wait set with space for guard conditions ...
rcl_wait_set_t wait_set = rcl_get_zero_initialized_wait_set();
rcl_wait_set_init(&wait_set, 0, 1, 0, 0, 0, 0, context, alloc);
// (0 subscriptions, 1 guard condition, 0 timers, etc. in this example)
rcl_wait_set_add_guard_condition(&wait_set, graph_gc, NULL);
// Wait with a timeout
rcl_ret_t ret = rcl_wait(&wait_set, RCL_MS_TO_NS(500));  // wait up to 500ms
if (ret == RCL_RET_OK && wait_set.guard_conditions[0] == graph_gc) {
    // The graph guard condition was triggered – graph changed
    // We can now call rcl_get_topic_names_and_types or others to see the new state.
}
```

The above pattern attaches the graph guard condition to a wait set and waits. When awakened due to graph change, you can then fetch updated lists of nodes/topics/services as needed. This is exactly how higher-level ROS libraries (rclcpp, etc.) monitor for graph changes to update their internal caches. For a Rust FFI tool, you could similarly wait on the guard condition (perhaps via rclrs or via binding to rcl wait sets) or periodically poll the graph state if simplicity is okay.

**Thread-safety:** The guard condition is thread-safe to trigger (signaling is handled by RMW). Waiting on it uses the standard `rcl_wait` which is thread-safe as long as you use separate wait sets in each thread. Typically, you only need one thread waiting on a guard condition. The `rcl_node_get_graph_guard_condition` itself is not thread-safe if the node is being modified concurrently, but usually you call it once to get the handle.

**Summary of Graph updates:** The guard condition approach means you don’t have to continuously poll `rcl_get_*` functions to detect changes. Instead, you wait for the guard to trigger, then use the query functions to get the latest state. This is efficient. The kinds of graph events that trigger it include new publishers/subscribers (which affect the output of `rcl_get_topic_names_and_types` and the by-node topic lists), new services/clients (affecting `rcl_get_service_names_and_types` outputs), and new or removed nodes (affecting `rcl_get_node_names`).

## Typical Usage Patterns

In a C program (or via Rust FFI), a common pattern to utilize these graph APIs is:

1. **Initialize ROS and create a node:** Use `rcl_init` and `rcl_node_init` to create an `rcl_node_t`. Without a valid node, you cannot query the graph (all these functions require a valid `rcl_node_t *` as they operate within a context and namespace).

2. **Query initial graph state:** Call `rcl_get_node_names`, `rcl_get_topic_names_and_types`, etc., to get the current state of the world. This can be useful for an initial listing (for example, a CLI tool that prints all topics and nodes at start).

3. **Optionally, wait for updates:** If you need to monitor changes (e.g., a dynamic tool that watches for new topics), use the graph guard condition. Add it to a wait set or use an executor that listens for graph events. When it triggers, use the APIs again to get the updated lists. For instance, you might notice a new node was added, or a topic disappeared.

4. **Resource cleanup:** After finishing, free all allocated structures (all string arrays and names-and-types structures via their fini functions). Finally, shut down the node (`rcl_node_fini`) and ROS (`rcl_shutdown`). The guard condition will be automatically destroyed with the node.

By following this pattern, you ensure that memory is managed properly and that you are responding to graph changes efficiently. Here is a brief pseudo-code combining some of these steps:

```c
// ... assume rcl_init and node creation done ...

// 1. Initial list of nodes:
rcutils_string_array_t node_names = rcutils_get_zero_initialized_string_array();
rcutils_string_array_t node_namespaces = rcutils_get_zero_initialized_string_array();
rcl_get_node_names(node, alloc, &node_names, &node_namespaces);
// Print nodes
for (size_t i = 0; i < node_names.size; ++i) {
    printf("Node: %s (ns: %s)\n", node_names.data[i], node_namespaces.data[i]);
}
rcutils_string_array_fini(&node_names);
rcutils_string_array_fini(&node_namespaces);

// 2. Initial list of topics:
rcl_names_and_types_t topics = rcl_get_zero_initialized_names_and_types();
rcl_get_topic_names_and_types(node, &alloc, false, &topics);
// Print topics
rcl_names_and_types_fini(&topics);

// 3. Wait for graph changes using guard condition:
const rcl_guard_condition_t * graph_cond = rcl_node_get_graph_guard_condition(node);
rcl_wait_set_t wait_set = rcl_get_zero_initialized_wait_set();
rcl_wait_set_init(&wait_set, 0, 1, 0, 0, 0, 0, node->context, alloc);
rcl_wait_set_add_guard_condition(&wait_set, graph_cond, NULL);
while (running) {
    if (rcl_wait(&wait_set, RCL_S_TO_NS(1)) == RCL_RET_OK) {
        if (wait_set.guard_conditions[0] == graph_cond) {
            // Graph changed, query again or handle accordingly
            printf("Graph update detected!\n");
            // (Could re-call rcl_get_topic_names_and_types or others here)
        }
    }
}
// Cleanup:
rcl_wait_set_fini(&wait_set);
```

In a Rust setting, you might use the `rcl` FFI directly or use an existing ROS 2 Rust client library (like *rclrs*). If using FFI, the same logic applies: call the C functions via FFI, ensure proper allocations (you can use the helpers to get zero-initialized structs), and ensure you call the fini functions in a `Drop` implementation or manually to free memory. Be wary of the fact that the `rcl_allocator_t` might not match Rust’s global allocator – it’s safest to use the default provided by `rcutils_get_default_allocator()` unless you have a special reason to use a custom one.

## Mapping of rcl Graph APIs to rmw Functions (Summary)

For quick reference, here is how each `rcl` Graph function interfaces with the `rmw` layer:

* **`rcl_get_node_names`** – Calls **`rmw_get_node_names`** to collect node names and namespaces. The results are returned in `rcutils_string_array_t` structures for names and namespaces.

* **`rcl_get_topic_names_and_types`** – Calls **`rmw_get_topic_names_and_types`** (with the `no_demangle` flag passed through) to get all topic names and their types. Returns data in an `rcl_names_and_types_t` (alias of `rmw_names_and_types_t`). Each name corresponds to a topic; types array for that name contains one or more types of that topic.

* **`rcl_get_service_names_and_types`** – Calls **`rmw_get_service_names_and_types`** to get all service names and types. Returns an `rcl_names_and_types_t` listing each service name and its type(s). Usually one type per service.

* **`rcl_get_publisher_names_and_types_by_node`** – Calls **`rmw_get_publisher_names_and_types_by_node`** to get all topic names and types that the specified remote node publishes. Returns an `rcl_names_and_types_t` for those topics. Requires the target node name/namespace; returns `RCL_RET_NODE_NAME_NON_EXISTENT` if not found.

* **`rcl_get_subscriber_names_and_types_by_node`** – Calls **`rmw_get_subscriber_names_and_types_by_node`** to get all topics that the specified node subscribes to. Returns an `rcl_names_and_types_t` for those topics (topics and their types that the node is listening on). Also uses `RCL_RET_NODE_NAME_NON_EXISTENT` if the node isn’t discovered.

* **`rcl_get_service_names_and_types_by_node`** – Calls **`rmw_get_service_names_and_types_by_node`** for all service servers offered by the specified node. Returns an `rcl_names_and_types_t` of service names and types. (This is essentially the remote node’s advertised servers.)

* **`rcl_get_client_names_and_types_by_node`** – Calls **`rmw_get_client_names_and_types_by_node`** for all service clients used by the specified node. Returns an `rcl_names_and_types_t` of service names and types that the node is a client of.

* **Graph guard condition (`rcl_node_get_graph_guard_condition`)** – Uses **`rmw_node_get_graph_guard_condition`** to get the middleware’s guard condition handle for graph changes. `rmw` triggers this condition on graph events, and `rcl` provides it to the user for waiting. It is a const pointer in `rcl` API since you should not modify it—just wait on it.

Each `rcl` function essentially validates parameters, maybe does some minor preprocessing (for example, ensuring the output struct is zero-initialized, or filtering out any null names), and then delegates to the `rmw` layer. The close alignment between `rcl` and `rmw` is reflected in the data types (e.g., the reuse of `rmw_names_and_types_t` in `rcl`). This design allows `rcl` to remain independent of any particular middleware implementation: all middleware-specific logic (like DDS discovery details, demangling of topic names, etc.) lives in the `rmw` implementation, while `rcl` provides a consistent API to the rest of the ROS stack.

## Conclusion

The `rcl` Graph APIs provide powerful introspection capabilities for ROS 2 tools and libraries. When building FFI bindings (such as a Rust CLI tool), understanding these functions and their contracts is crucial. Remember to manage memory explicitly: initialize before use, finalize after use. Use the guard condition to efficiently react to changes rather than polling. With proper usage, you can query the ROS graph to list nodes, topics, and services, track when new entities appear or vanish, and integrate that information into your tool. The combination of `rcl` and `rmw` layers abstracts away the details of DDS discovery, giving you a relatively simple C API to the complex distributed system that is ROS 2’s runtime graph.

**References:**

* ROS 2 Client Library (rcl) documentation for Graph functions. These describe the usage and requirements (initialization, finalization, etc.) for each function.

* ROS 2 Middleware Interface (rmw) documentation for corresponding graph queries and guard condition, which illustrate how graph changes map to the data returned by these functions and how the guard condition covers various graph events.

* ROS 2 design article on Topic and Service name mapping (for understanding name demangling).

* Internal ROS 2 interfaces overview for context on how `rcl` and `rmw` interact in the ROS 2 architecture.
