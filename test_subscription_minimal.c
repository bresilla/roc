#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <dlfcn.h>

// Minimal type definitions to avoid header dependencies
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

// Function pointers for RCL functions
typedef rcl_allocator_t (*get_default_allocator_func)();
typedef int (*rcl_init_func)(int argc, const char * const * argv, void* init_options, void* context);
typedef int (*rcl_shutdown_func)(void* context);
typedef int (*rcl_subscription_init_func)(void* subscription, void* node, const rosidl_message_type_support_t* type_support, const char* topic_name, void* options);

void print_type_support_details(const rosidl_message_type_support_t* ts, const char* name) {
    printf("\n%s type support details:\n", name);
    printf("  Address: %p\n", (void*)ts);
    printf("  Identifier: %s\n", ts->typesupport_identifier);
    printf("  Data: %p\n", ts->data);
    printf("  Func: %p\n", ts->func);
    
    // Try to peek at the data structure
    if (ts->data) {
        unsigned char* data_bytes = (unsigned char*)ts->data;
        printf("  First 16 bytes of data: ");
        for (int i = 0; i < 16; i++) {
            printf("%02x ", data_bytes[i]);
        }
        printf("\n");
    }
}

int test_subscription_minimal(const char* lib_path, const char* symbol_name, const char* msg_type) {
    printf("\n=== Testing %s ===\n", msg_type);
    
    // Load type support
    void* handle = dlopen(lib_path, RTLD_LAZY);
    if (!handle) {
        printf("Failed to load library: %s\n", dlerror());
        return 1;
    }
    
    type_support_func get_type_support = (type_support_func)dlsym(handle, symbol_name);
    if (!get_type_support) {
        printf("Failed to find symbol: %s\n", dlerror());
        dlclose(handle);
        return 1;
    }
    
    const rosidl_message_type_support_t* type_support = get_type_support();
    print_type_support_details(type_support, msg_type);
    
    // Compare the type support structures more closely
    static const rosidl_message_type_support_t* previous_ts = NULL;
    static const char* previous_name = NULL;
    
    if (previous_ts) {
        printf("\nComparing with previous (%s):\n", previous_name);
        printf("  Same address: %s\n", (previous_ts == type_support) ? "YES" : "NO");
        printf("  Same identifier: %s\n", 
            (strcmp(previous_ts->typesupport_identifier, type_support->typesupport_identifier) == 0) ? "YES" : "NO");
        printf("  Same data: %s\n", (previous_ts->data == type_support->data) ? "YES" : "NO");
        printf("  Same func: %s\n", (previous_ts->func == type_support->func) ? "YES" : "NO");
    }
    
    previous_ts = type_support;
    previous_name = msg_type;
    
    dlclose(handle);
    return 0;
}

int main() {
    // Test the three message types
    test_subscription_minimal(
        "/opt/ros/jazzy/lib/librcl_interfaces__rosidl_typesupport_c.so",
        "rosidl_typesupport_c__get_message_type_support_handle__rcl_interfaces__msg__Log",
        "rcl_interfaces/msg/Log"
    );
    
    test_subscription_minimal(
        "/opt/ros/jazzy/lib/libstd_msgs__rosidl_typesupport_c.so",
        "rosidl_typesupport_c__get_message_type_support_handle__std_msgs__msg__String",
        "std_msgs/msg/String"
    );
    
    test_subscription_minimal(
        "/opt/ros/jazzy/lib/libgeometry_msgs__rosidl_typesupport_c.so",
        "rosidl_typesupport_c__get_message_type_support_handle__geometry_msgs__msg__Twist",
        "geometry_msgs/msg/Twist"
    );
    
    return 0;
}