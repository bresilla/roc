#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <dlfcn.h>
#include <signal.h>

// Define minimal structures needed
typedef struct rcl_allocator_t {
    void* (*allocate)(size_t size, void* state);
    void (*deallocate)(void* pointer, void* state);
    void* (*reallocate)(void* pointer, size_t size, void* state);
    void* (*zero_allocate)(size_t number_of_elements, size_t size_of_element, void* state);
    void* state;
} rcl_allocator_t;

typedef struct rosidl_message_type_support_t {
    const char * typesupport_identifier;
    const void * data;
    void * func;
    void * get_type_hash_func;
    void * get_type_description_func;
    void * get_type_description_sources_func;
} rosidl_message_type_support_t;

typedef const rosidl_message_type_support_t * (*type_support_func)();

// Function pointers for RCL functions - we'll load these dynamically
typedef rcl_allocator_t (*get_default_allocator_func)();
typedef int (*rcl_init_func)(int argc, const char * const * argv, void* init_options, void* context);
typedef int (*rcl_shutdown_func)(void* context);
typedef int (*rcl_publisher_init_func)(void* publisher, void* node, const rosidl_message_type_support_t* type_support, const char* topic_name, void* options);
typedef int (*rcl_subscription_init_func)(void* subscription, void* node, const rosidl_message_type_support_t* type_support, const char* topic_name, void* options);

// Global flag for signal handling
volatile sig_atomic_t interrupted = 0;

void signal_handler(int sig) {
    interrupted = 1;
    printf("\nCaught signal %d, exiting gracefully...\n", sig);
}

void compare_type_support_details(const rosidl_message_type_support_t* ts, const char* name) {
    printf("\n=== %s Type Support Analysis ===\n", name);
    printf("  Address: %p\n", (void*)ts);
    
    if (!ts) {
        printf("  ERROR: Type support is NULL\n");
        return;
    }
    
    printf("  typesupport_identifier: %p", (void*)ts->typesupport_identifier);
    if (ts->typesupport_identifier) {
        printf(" = '%s'", ts->typesupport_identifier);
    }
    printf("\n");
    
    printf("  data: %p\n", ts->data);
    printf("  func: %p\n", ts->func);
    printf("  get_type_hash_func: %p\n", ts->get_type_hash_func);
    printf("  get_type_description_func: %p\n", ts->get_type_description_func);
    printf("  get_type_description_sources_func: %p\n", ts->get_type_description_sources_func);
    
    // Try to examine the data structure
    if (ts->data) {
        printf("  First 32 bytes of data: ");
        unsigned char* data_bytes = (unsigned char*)ts->data;
        for (int i = 0; i < 32; i++) {
            printf("%02x ", data_bytes[i]);
            if ((i + 1) % 16 == 0) printf("\n                            ");
        }
        printf("\n");
    }
}

int load_and_test_type_support(const char* lib_path, const char* symbol_name, const char* msg_type) {
    printf("\n\n=== Testing %s ===\n", msg_type);
    printf("Library: %s\n", lib_path);
    printf("Symbol: %s\n", symbol_name);
    
    // Load the library
    void* handle = dlopen(lib_path, RTLD_LAZY);
    if (!handle) {
        printf("ERROR: Failed to load library: %s\n", dlerror());
        return 1;
    }
    
    // Get the type support function
    type_support_func get_type_support = (type_support_func)dlsym(handle, symbol_name);
    if (!get_type_support) {
        printf("ERROR: Failed to find symbol: %s\n", dlerror());
        dlclose(handle);
        return 1;
    }
    
    // Call the function to get type support
    const rosidl_message_type_support_t* type_support = get_type_support();
    compare_type_support_details(type_support, msg_type);
    
    // Now we need to test what happens when we try to use this type support
    // for publisher vs subscription initialization
    
    // TODO: We would need to initialize RCL context and node here
    // For now, just analyze the type support structure
    
    dlclose(handle);
    return 0;
}

int main() {
    printf("=== RCL Publisher vs Subscription Analysis ===\n");
    printf("This program analyzes differences between working and failing type supports\n");
    
    // Install signal handler
    signal(SIGINT, signal_handler);
    signal(SIGTERM, signal_handler);
    
    // Test working message type (rcl_interfaces/msg/Log - this works for both pub and sub)
    load_and_test_type_support(
        "/opt/ros/jazzy/lib/librcl_interfaces__rosidl_typesupport_c.so",
        "rosidl_typesupport_c__get_message_type_support_handle__rcl_interfaces__msg__Log",
        "rcl_interfaces/msg/Log (WORKING)"
    );
    
    // Test problematic message type 1 (std_msgs/msg/String - this causes segfault in subscription)
    if (!interrupted) {
        load_and_test_type_support(
            "/opt/ros/jazzy/lib/libstd_msgs__rosidl_typesupport_c.so",
            "rosidl_typesupport_c__get_message_type_support_handle__std_msgs__msg__String",
            "std_msgs/msg/String (PROBLEMATIC)"
        );
    }
    
    // Test problematic message type 2 (geometry_msgs/msg/Twist - this also causes segfault in subscription)
    if (!interrupted) {
        load_and_test_type_support(
            "/opt/ros/jazzy/lib/libgeometry_msgs__rosidl_typesupport_c.so",
            "rosidl_typesupport_c__get_message_type_support_handle__geometry_msgs__msg__Twist",
            "geometry_msgs/msg/Twist (PROBLEMATIC)"
        );
    }
    
    printf("\n=== Analysis Summary ===\n");
    printf("Key questions to investigate:\n");
    printf("1. Are the type support structures identical between working and failing types?\n");
    printf("2. Does the 'data' pointer contain different structures?\n");
    printf("3. Are there differences in function pointers?\n");
    printf("4. Is the problem in the RCL subscription initialization itself?\n");
    printf("5. Could this be a memory alignment or padding issue?\n");
    printf("6. Are there initialization order dependencies?\n");
    
    printf("\nNext steps:\n");
    printf("1. Compare memory layouts of the 'data' structures\n");
    printf("2. Examine the actual rcl_subscription_init vs rcl_publisher_init implementations\n");
    printf("3. Check for context/node validation differences\n");
    printf("4. Look at RMW layer interactions\n");
    
    return 0;
}