use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to look for shared libraries in the ROS 2 installation
    println!("cargo:rustc-link-search=native=/opt/ros/jazzy/lib");

    // Link to the RCL and RMW libraries
    println!("cargo:rustc-link-lib=rcl");
    println!("cargo:rustc-link-lib=rmw");
    println!("cargo:rustc-link-lib=rcutils");
    
    // Additional libraries for dynamic type support
    println!("cargo:rustc-link-lib=rosidl_dynamic_typesupport");
    println!("cargo:rustc-link-lib=rosidl_typesupport_c");
    println!("cargo:rustc-link-lib=rosidl_typesupport_cpp");
    println!("cargo:rustc-link-lib=rosidl_typesupport_introspection_c");
    println!("cargo:rustc-link-lib=rosidl_typesupport_introspection_cpp");
    println!("cargo:rustc-link-lib=rcl_yaml_param_parser");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point to bindgen
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate bindings for
        .header("wrapper.h")
        // Add include paths for ROS 2 headers - try to find the correct path
        .clang_arg("-I/opt/ros/jazzy/include")
        .clang_arg("-I/opt/ros/jazzy/include/rcl")
        .clang_arg("-I/opt/ros/jazzy/include/rcutils")
        .clang_arg("-I/opt/ros/jazzy/include/rmw")
        .clang_arg("-I/opt/ros/jazzy/include/rcl_yaml_param_parser")
        .clang_arg("-I/opt/ros/jazzy/include/rosidl_runtime_c")
        .clang_arg("-I/opt/ros/jazzy/include/rosidl_typesupport_interface")
        .clang_arg("-I/opt/ros/jazzy/include/rosidl_dynamic_typesupport")
        .clang_arg("-I/opt/ros/jazzy/include/rosidl_typesupport_c")
        .clang_arg("-I/opt/ros/jazzy/include/rosidl_typesupport_cpp")
        .clang_arg("-I/opt/ros/jazzy/include/rosidl_typesupport_introspection_c")
        .clang_arg("-I/opt/ros/jazzy/include/rosidl_typesupport_introspection_cpp")
        .clang_arg("-I/opt/ros/jazzy/include/type_description_interfaces")
        .clang_arg("-I/opt/ros/jazzy/include/service_msgs")
        .clang_arg("-I/opt/ros/jazzy/include/builtin_interfaces")
        .clang_arg("-I/opt/ros/jazzy/include/std_msgs")
        .clang_arg("-I/opt/ros/jazzy/include/sensor_msgs")
        .clang_arg("-I/opt/ros/jazzy/include/geometry_msgs")
        .clang_arg("-I/opt/ros/jazzy/include/action_msgs")
        // Suppress warnings and use C11
        .clang_arg("-Wno-everything")
        .clang_arg("-std=c11")
        // Tell cargo to invalidate the built crate whenever any of the included header files changed
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Only generate bindings for functions we specify (keep it very minimal)
        .allowlist_function("rcl_get_zero_initialized_context")
        .allowlist_function("rcl_get_default_allocator")
        .allowlist_function("rcutils_get_default_allocator")
        .allowlist_function("rcl_get_zero_initialized_init_options")
        .allowlist_function("rcl_init_options_init")
        .allowlist_function("rcl_init_options_get_rmw_init_options")
        .allowlist_function("rcl_init")
        .allowlist_function("rcl_shutdown")
        .allowlist_function("rcl_context_is_valid")
        // RMW basic functions
        .allowlist_function("rmw_get_zero_initialized_init_options")
        .allowlist_function("rmw_init_options_init")
        .allowlist_function("rmw_init")
        .allowlist_function("rmw_shutdown")
        // Node functions
        .allowlist_function("rcl_get_zero_initialized_node")
        .allowlist_function("rcl_node_init")
        .allowlist_function("rcl_node_fini")
        .allowlist_function("rcl_node_is_valid")
        .allowlist_function("rcl_node_get_default_options")
        // Graph discovery functions
        .allowlist_function("rcl_get_node_names")
        .allowlist_function("rcl_get_node_names_with_enclaves")
        .allowlist_function("rcl_get_topic_names_and_types")
        .allowlist_function("rcl_get_service_names_and_types")
        .allowlist_function("rcl_names_and_types_fini")
        // Topic info functions
        .allowlist_function("rcl_count_publishers")
        .allowlist_function("rcl_count_subscribers")
        .allowlist_function("rcl_get_publishers_info_by_topic")
        .allowlist_function("rcl_get_subscriptions_info_by_topic")
        .allowlist_function("rmw_topic_endpoint_info_array_fini")
        // Subscription and Publisher functions for interception
        .allowlist_function("rcl_subscription_init")
        .allowlist_function("rcl_subscription_fini")
        .allowlist_function("rcl_take")
        .allowlist_function("rcl_publisher_init")
        .allowlist_function("rcl_publisher_fini")
        .allowlist_function("rcl_publisher_is_valid")
        .allowlist_function("rcl_publisher_get_default_options")
        .allowlist_function("rcl_get_zero_initialized_publisher")
        .allowlist_function("rcl_publish")
        // RMW-level functions for direct access
        .allowlist_function("rmw_create_subscription")
        .allowlist_function("rmw_destroy_subscription")
        .allowlist_function("rmw_take")
        .allowlist_function("rmw_take_with_info")
        .allowlist_function("rmw_create_publisher")
        .allowlist_function("rmw_destroy_publisher")
        .allowlist_function("rmw_publish")
        .allowlist_function("rmw_subscription_set_on_new_message_callback")
        // String array functions
        .allowlist_function("rcutils_get_zero_initialized_string_array")
        .allowlist_function("rcutils_string_array_fini")
        // Also include the basic types we need
        .allowlist_type("rcl_context_t")
        .allowlist_type("rcl_allocator_t")
        .allowlist_type("rcl_init_options_t")
        // Node types
        .allowlist_type("rcl_node_t")
        .allowlist_type("rcl_node_options_t")
        // Graph discovery types
        .allowlist_type("rcl_names_and_types_t")
        .allowlist_type("rcutils_string_array_t")
        // Topic info types
        .allowlist_type("rcl_topic_endpoint_info_array_t")
        .allowlist_type("rcl_topic_endpoint_info_t")
        // Subscription and Publisher types
        .allowlist_type("rcl_subscription_t")
        .allowlist_type("rcl_subscription_options_t")
        .allowlist_type("rcl_publisher_t")
        .allowlist_type("rcl_publisher_options_t")
        // RMW message types
        .allowlist_type("rmw_subscription_t")
        .allowlist_type("rmw_publisher_t")
        .allowlist_type("rmw_message_info_t")
        .allowlist_type("rmw_event_callback_t")
        // Message type support
        .allowlist_type("rosidl_message_type_support_t")
        .allowlist_type("rosidl_typesupport_interface__MessageTypeSupport")
        // Dynamic type support functions
        .allowlist_function("rosidl_get_typesupport_target")
        .allowlist_function("rosidl_typesupport_c__get_message_typesupport_handle_function")
        .allowlist_function("rosidl_get_message_type_support_handle")
        // Dynamic type support loading
        .allowlist_function("rcutils_get_symbol")
        .allowlist_function("rcutils_load_shared_library")
        .allowlist_function("rcutils_unload_shared_library")
        .allowlist_function("rcutils_get_zero_initialized_shared_library")
        .allowlist_function("rcutils_has_symbol")
        // Message introspection functions
        .allowlist_function("rosidl_typesupport_introspection_c__get_message_type_support_handle")
        .allowlist_function("rosidl_get_typesupport_handle")
        // RMW serialization functions for any message type
        .allowlist_function("rmw_get_zero_initialized_serialized_message")
        .allowlist_function("rmw_serialized_message_init")
        .allowlist_function("rmw_serialized_message_fini")
        .allowlist_function("rmw_serialized_message_resize")
        // RMW serialization functions
        .allowlist_function("rmw_serialize")
        .allowlist_function("rmw_deserialize")
        .allowlist_function("rmw_get_serialized_message_size")
        .allowlist_function("rmw_serialized_message_init")
        .allowlist_function("rmw_serialized_message_fini")
        .allowlist_function("rmw_serialized_message_resize")
        .allowlist_function("rmw_publish_serialized_message")
        .allowlist_function("rmw_take_serialized_message")
        .allowlist_function("rmw_take_serialized_message_with_info")
        // YAML parameter parsing functions
        .allowlist_function("rcl_yaml_param_parser_load_file")
        .allowlist_function("rcl_yaml_param_parser_parse_string")
        .allowlist_function("rcutils_yaml_param_value_fini")
        // Type description and hash functions
        .allowlist_function("rosidl_runtime_c__type_description__create")
        .allowlist_function("rosidl_runtime_c__type_description__destroy")
        .allowlist_function("rosidl_runtime_c__type_hash__create")
        .allowlist_function("rosidl_runtime_c__type_hash__destroy")
        // RMW basic types
        .allowlist_type("rmw_init_options_t")
        .allowlist_type("rmw_context_t")
        .allowlist_type("rmw_allocator_t")
        .allowlist_type("rmw_ret_t")
        // Dynamic loading types
        .allowlist_type("rcutils_shared_library_t")
        .allowlist_type("rcutils_ret_t")
        // Message introspection types
        .allowlist_type("rosidl_typesupport_introspection_c__MessageMembers")
        .allowlist_type("rosidl_typesupport_introspection_c__MessageMember")
        // Introspection field type constants
        .allowlist_var("rosidl_typesupport_introspection_c__ROS_TYPE_.*")
        // Serialized message type
        .allowlist_type("rmw_serialized_message_t")
        // Type description types
        .allowlist_type("rosidl_runtime_c__type_description__TypeDescription")
        .allowlist_type("rosidl_runtime_c__type_hash__TypeHash")
        // YAML parameter types
        .allowlist_type("rcutils_yaml_param_value_t")
        .allowlist_type("rcutils_yaml_param_node_t")
        // Generate the bindings
        .generate()
        // Unwrap the Result and panic on failure
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
