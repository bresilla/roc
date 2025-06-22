#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <dlfcn.h>
#include <rcl/rcl/rcl.h>
#include <rcl/rcl/subscription.h>
#include <rosidl_runtime_c/message_type_support_struct.h>

typedef const rosidl_message_type_support_t * (*type_support_func)();

int test_subscription_init(const char* lib_path, const char* symbol_name, const char* topic_name, const char* msg_type) {
    printf("\n=== Testing subscription init for %s ===\n", msg_type);
    
    // Initialize RCL
    rcl_allocator_t allocator = rcl_get_default_allocator();
    rcl_context_t context = rcl_get_zero_initialized_context();
    rcl_init_options_t init_options = rcl_get_zero_initialized_init_options();
    
    rcl_ret_t ret = rcl_init_options_init(&init_options, allocator);
    if (ret != RCL_RET_OK) {
        printf("Failed to initialize init options\n");
        return 1;
    }
    
    int argc = 0;
    char** argv = NULL;
    ret = rcl_init(argc, argv, &init_options, &context);
    if (ret != RCL_RET_OK) {
        printf("Failed to initialize RCL\n");
        return 1;
    }
    
    // Create node
    rcl_node_t node = rcl_get_zero_initialized_node();
    rcl_node_options_t node_options = rcl_node_get_default_options();
    ret = rcl_node_init(&node, "test_node", "", &context, &node_options);
    if (ret != RCL_RET_OK) {
        printf("Failed to initialize node\n");
        rcl_shutdown(&context);
        return 1;
    }
    
    // Load type support
    void* handle = dlopen(lib_path, RTLD_LAZY);
    if (!handle) {
        printf("Failed to load library: %s\n", dlerror());
        rcl_node_fini(&node);
        rcl_shutdown(&context);
        return 1;
    }
    
    type_support_func get_type_support = (type_support_func)dlsym(handle, symbol_name);
    if (!get_type_support) {
        printf("Failed to find symbol: %s\n", dlerror());
        dlclose(handle);
        rcl_node_fini(&node);
        rcl_shutdown(&context);
        return 1;
    }
    
    const rosidl_message_type_support_t* type_support = get_type_support();
    printf("Type support loaded: %p\n", (void*)type_support);
    printf("  identifier: %s\n", type_support->typesupport_identifier);
    
    // Create subscription
    rcl_subscription_t subscription = rcl_get_zero_initialized_subscription();
    rcl_subscription_options_t subscription_options = rcl_subscription_get_default_options();
    
    printf("About to call rcl_subscription_init...\n");
    fflush(stdout);
    
    ret = rcl_subscription_init(
        &subscription,
        &node,
        type_support,
        topic_name,
        &subscription_options
    );
    
    if (ret != RCL_RET_OK) {
        printf("rcl_subscription_init failed with code: %d\n", ret);
    } else {
        printf("rcl_subscription_init succeeded!\n");
        rcl_subscription_fini(&subscription, &node);
    }
    
    // Cleanup
    dlclose(handle);
    rcl_node_fini(&node);
    rcl_shutdown(&context);
    rcl_context_fini(&context);
    
    return ret == RCL_RET_OK ? 0 : 1;
}

int main() {
    // Test with working message type
    test_subscription_init(
        "/opt/ros/jazzy/lib/librcl_interfaces__rosidl_typesupport_c.so",
        "rosidl_typesupport_c__get_message_type_support_handle__rcl_interfaces__msg__Log",
        "/rosout",
        "rcl_interfaces/msg/Log"
    );
    
    // Test with problematic message type
    test_subscription_init(
        "/opt/ros/jazzy/lib/libstd_msgs__rosidl_typesupport_c.so",
        "rosidl_typesupport_c__get_message_type_support_handle__std_msgs__msg__String",
        "/test_string",
        "std_msgs/msg/String"
    );
    
    return 0;
}