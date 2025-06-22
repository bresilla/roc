#include <stdio.h>
#include <dlfcn.h>
#include <stdint.h>

// Define the type support structure based on ROS2 headers
typedef struct rosidl_message_type_support_t {
    const char * typesupport_identifier;
    const void * data;
    void * func;
    void * get_type_hash_func;
    void * get_type_description_func;
    void * get_type_description_sources_func;
} rosidl_message_type_support_t;

typedef const rosidl_message_type_support_t * (*type_support_func)();

void examine_type_support(const char* lib_path, const char* symbol_name, const char* msg_type) {
    printf("\n=== Examining %s ===\n", msg_type);
    
    // Load the library
    void* handle = dlopen(lib_path, RTLD_LAZY);
    if (!handle) {
        printf("Failed to load library: %s\n", dlerror());
        return;
    }
    
    // Get the type support function
    type_support_func get_type_support = (type_support_func)dlsym(handle, symbol_name);
    if (!get_type_support) {
        printf("Failed to find symbol %s: %s\n", symbol_name, dlerror());
        dlclose(handle);
        return;
    }
    
    // Call the function to get type support
    const rosidl_message_type_support_t* ts = get_type_support();
    if (!ts) {
        printf("Type support function returned NULL\n");
        dlclose(handle);
        return;
    }
    
    printf("Type support pointer: %p\n", (void*)ts);
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
    
    // Try to examine the data pointer if it's not null
    if (ts->data) {
        printf("  First 8 bytes of data: ");
        uint8_t* data_bytes = (uint8_t*)ts->data;
        for (int i = 0; i < 8 && i < sizeof(void*); i++) {
            printf("%02x ", data_bytes[i]);
        }
        printf("\n");
    }
    
    dlclose(handle);
}

int main() {
    // Test working message type
    examine_type_support(
        "/opt/ros/jazzy/lib/librcl_interfaces__rosidl_typesupport_c.so",
        "rosidl_typesupport_c__get_message_type_support_handle__rcl_interfaces__msg__Log",
        "rcl_interfaces/msg/Log"
    );
    
    // Test problematic message types
    examine_type_support(
        "/opt/ros/jazzy/lib/libstd_msgs__rosidl_typesupport_c.so",
        "rosidl_typesupport_c__get_message_type_support_handle__std_msgs__msg__String",
        "std_msgs/msg/String"
    );
    
    examine_type_support(
        "/opt/ros/jazzy/lib/libgeometry_msgs__rosidl_typesupport_c.so",
        "rosidl_typesupport_c__get_message_type_support_handle__geometry_msgs__msg__Twist",
        "geometry_msgs/msg/Twist"
    );
    
    return 0;
}