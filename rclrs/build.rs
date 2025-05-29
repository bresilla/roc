use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to look for shared libraries in the ROS 2 installation
    println!("cargo:rustc-link-search=native=/opt/ros/jazzy/lib");
    
    // Link to the RCL and RMW libraries
    println!("cargo:rustc-link-lib=rcl");
    println!("cargo:rustc-link-lib=rmw");
    
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
        // Suppress warnings and use C11
        .clang_arg("-Wno-everything")
        .clang_arg("-std=c11")
        // Tell cargo to invalidate the built crate whenever any of the included header files changed
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Only generate bindings for functions we specify (keep it very minimal)
        .allowlist_function("rcl_get_zero_initialized_context")
        .allowlist_function("rcl_get_default_allocator")
        .allowlist_function("rcl_get_zero_initialized_init_options")
        .allowlist_function("rcl_init")
        .allowlist_function("rcl_shutdown")
        .allowlist_function("rcl_context_is_valid")
        // RMW basic functions
        .allowlist_function("rmw_get_zero_initialized_init_options")
        .allowlist_function("rmw_init")
        .allowlist_function("rmw_shutdown")
        // Also include the basic types we need
        .allowlist_type("rcl_context_t")
        .allowlist_type("rcl_allocator_t")
        .allowlist_type("rcl_init_options_t")
        // RMW basic types
        .allowlist_type("rmw_init_options_t")
        .allowlist_type("rmw_context_t")
        .allowlist_type("rmw_allocator_t")
        .allowlist_type("rmw_ret_t")
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
