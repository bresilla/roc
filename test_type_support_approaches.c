#include <stdio.h>
#include <dlfcn.h>
#include <string.h>

// ROS2 type support structures
typedef struct rosidl_message_type_support_t {
    const char * typesupport_identifier;
    const void * data;
    void * func;
    void * get_type_hash_func;
    void * get_type_description_func;
    void * get_type_description_sources_func;
} rosidl_message_type_support_t;

typedef const rosidl_message_type_support_t * (*type_support_func)();
typedef const rosidl_message_type_support_t * (*dispatch_func)(const rosidl_message_type_support_t *, const char *);

void test_type_support(const char* lib_path, const char* symbol_name, const char* msg_type) {
    printf("\n=== Testing %s ===\n", msg_type);
    
    // Load library
    void* handle = dlopen(lib_path, RTLD_LAZY);
    if (!handle) {
        printf("Failed to load library: %s\n", dlerror());
        return;
    }
    
    // Get type support function
    type_support_func get_type_support = (type_support_func)dlsym(handle, symbol_name);
    if (!get_type_support) {
        printf("Failed to find symbol: %s\n", dlerror());
        dlclose(handle);
        return;
    }
    
    // Get the type support
    const rosidl_message_type_support_t* ts = get_type_support();
    printf("Type support: %p\n", (void*)ts);
    printf("  identifier: %s\n", ts->typesupport_identifier);
    
    // Try to load the dispatch function from rosidl_typesupport_c
    void* typesupport_c_handle = dlopen("librosidl_typesupport_c.so", RTLD_LAZY);
    if (typesupport_c_handle) {
        // Get the identifier string
        const char** identifier_ptr = (const char**)dlsym(typesupport_c_handle, "rosidl_typesupport_c__typesupport_identifier");
        if (identifier_ptr) {
            printf("rosidl_typesupport_c identifier: %s\n", *identifier_ptr);
            
            // Check if we need to dispatch
            if (strcmp(ts->typesupport_identifier, *identifier_ptr) != 0) {
                printf("Type support identifier mismatch! Need to dispatch.\n");
                
                // Try to get dispatch function
                dispatch_func get_handle = (dispatch_func)dlsym(typesupport_c_handle, 
                    "rosidl_typesupport_c__get_message_typesupport_handle_function");
                if (get_handle) {
                    printf("Found dispatch function, attempting dispatch...\n");
                    const rosidl_message_type_support_t* dispatched_ts = get_handle(ts, *identifier_ptr);
                    if (dispatched_ts) {
                        printf("Dispatched type support: %p\n", (void*)dispatched_ts);
                        printf("  identifier: %s\n", dispatched_ts->typesupport_identifier);
                    } else {
                        printf("Dispatch failed - returned NULL\n");
                    }
                } else {
                    printf("Could not find dispatch function\n");
                }
            } else {
                printf("Type support already has correct identifier\n");
            }
        }
        dlclose(typesupport_c_handle);
    }
    
    dlclose(handle);
}

int main() {
    // Test rcl_interfaces/msg/Log (working)
    test_type_support(
        "/opt/ros/jazzy/lib/librcl_interfaces__rosidl_typesupport_c.so",
        "rosidl_typesupport_c__get_message_type_support_handle__rcl_interfaces__msg__Log",
        "rcl_interfaces/msg/Log"
    );
    
    // Test std_msgs/msg/String (problematic)
    test_type_support(
        "/opt/ros/jazzy/lib/libstd_msgs__rosidl_typesupport_c.so",
        "rosidl_typesupport_c__get_message_type_support_handle__std_msgs__msg__String",
        "std_msgs/msg/String"
    );
    
    // Test geometry_msgs/msg/Twist (problematic)
    test_type_support(
        "/opt/ros/jazzy/lib/libgeometry_msgs__rosidl_typesupport_c.so",
        "rosidl_typesupport_c__get_message_type_support_handle__geometry_msgs__msg__Twist",
        "geometry_msgs/msg/Twist"
    );
    
    // Also try introspection type support
    printf("\n\n=== Testing introspection type support ===\n");
    
    test_type_support(
        "/opt/ros/jazzy/lib/libstd_msgs__rosidl_typesupport_introspection_c.so",
        "rosidl_typesupport_introspection_c__get_message_type_support_handle__std_msgs__msg__String",
        "std_msgs/msg/String (introspection)"
    );
    
    return 0;
}