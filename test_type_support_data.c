#include <stdio.h>
#include <dlfcn.h>
#include <string.h>

typedef struct rosidl_message_type_support_t {
    const char * typesupport_identifier;
    const void * data;
    void * func;
    void * get_type_hash_func;
    void * get_type_description_func;
    void * get_type_description_sources_func;
} rosidl_message_type_support_t;

typedef const rosidl_message_type_support_t * (*type_support_func)();

// Type support map structure (from examining ROS2 source)
typedef struct type_support_map_entry_t {
    const char * package_name;
    const rosidl_message_type_support_t * typesupport;
} type_support_map_entry_t;

typedef struct type_support_map_t {
    unsigned int size;
    const type_support_map_entry_t * entries;
} type_support_map_t;

void examine_type_support_data(const char* lib_path, const char* symbol_name, const char* msg_type) {
    printf("\n=== Examining %s ===\n", msg_type);
    
    void* handle = dlopen(lib_path, RTLD_LAZY | RTLD_GLOBAL);
    if (!handle) {
        printf("Failed to load library: %s\n", dlerror());
        return;
    }
    
    type_support_func get_type_support = (type_support_func)dlsym(handle, symbol_name);
    if (!get_type_support) {
        printf("Failed to find symbol: %s\n", dlerror());
        dlclose(handle);
        return;
    }
    
    const rosidl_message_type_support_t* ts = get_type_support();
    printf("Type support: %p\n", (void*)ts);
    printf("  identifier: %s\n", ts->typesupport_identifier);
    printf("  data: %p\n", ts->data);
    
    // Try to interpret the data as a type support map
    if (ts->data) {
        const type_support_map_t* map = (const type_support_map_t*)ts->data;
        printf("  Interpreting data as type_support_map:\n");
        printf("    size: %u\n", map->size);
        printf("    entries: %p\n", (void*)map->entries);
        
        if (map->entries && map->size > 0 && map->size < 10) {  // Sanity check
            for (unsigned int i = 0; i < map->size; i++) {
                printf("    Entry %u:\n", i);
                printf("      package: %s\n", map->entries[i].package_name ? map->entries[i].package_name : "(null)");
                printf("      typesupport: %p\n", (void*)map->entries[i].typesupport);
                if (map->entries[i].typesupport) {
                    printf("        identifier: %s\n", map->entries[i].typesupport->typesupport_identifier);
                }
            }
        }
    }
    
    // Check if there's a difference in symbol visibility
    printf("\nChecking for other type support symbols in library:\n");
    
    // Try FastRTPS type support
    char fastrtps_symbol[256];
    snprintf(fastrtps_symbol, sizeof(fastrtps_symbol), 
        "%s__fastrtps_c", symbol_name);
    void* fastrtps_ts = dlsym(handle, fastrtps_symbol);
    printf("  FastRTPS variant (%s): %s\n", fastrtps_symbol, fastrtps_ts ? "FOUND" : "not found");
    
    // Try introspection type support
    char intro_symbol[256];
    snprintf(intro_symbol, sizeof(intro_symbol),
        "rosidl_typesupport_introspection_c__get_message_type_support_handle__%s", 
        strstr(symbol_name, "__") + 2);
    void* intro_ts = dlsym(handle, intro_symbol);
    printf("  Introspection variant: %s\n", intro_ts ? "FOUND" : "not found");
    
    dlclose(handle);
}

int main() {
    examine_type_support_data(
        "/opt/ros/jazzy/lib/librcl_interfaces__rosidl_typesupport_c.so",
        "rosidl_typesupport_c__get_message_type_support_handle__rcl_interfaces__msg__Log",
        "rcl_interfaces/msg/Log"
    );
    
    examine_type_support_data(
        "/opt/ros/jazzy/lib/libstd_msgs__rosidl_typesupport_c.so",
        "rosidl_typesupport_c__get_message_type_support_handle__std_msgs__msg__String",
        "std_msgs/msg/String"
    );
    
    examine_type_support_data(
        "/opt/ros/jazzy/lib/libgeometry_msgs__rosidl_typesupport_c.so",
        "rosidl_typesupport_c__get_message_type_support_handle__geometry_msgs__msg__Twist",
        "geometry_msgs/msg/Twist"
    );
    
    return 0;
}