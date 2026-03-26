use std::env;
use std::path::PathBuf;

fn ros_prefixes() -> Vec<PathBuf> {
    let mut prefixes = Vec::new();

    for var in ["AMENT_PREFIX_PATH", "CMAKE_PREFIX_PATH"] {
        if let Ok(value) = env::var(var) {
            for prefix in value.split(':') {
                if prefix.is_empty() {
                    continue;
                }
                let path = PathBuf::from(prefix);
                if !prefixes.contains(&path) {
                    prefixes.push(path);
                }
            }
        }
    }

    if prefixes.is_empty() {
        prefixes.push(PathBuf::from("/opt/ros/jazzy"));
    }

    prefixes
}

fn find_header(prefixes: &[PathBuf], relative_path: &str) -> PathBuf {
    for prefix in prefixes {
        let candidate = prefix.join("include").join(relative_path);
        if candidate.is_file() {
            return candidate;
        }
    }

    let fallback_prefix = prefixes
        .first()
        .cloned()
        .unwrap_or_else(|| PathBuf::from("/opt/ros/jazzy"));
    fallback_prefix.join("include").join(relative_path)
}

fn main() {
    // Generate Rust FFI bindings for the ROS2-generated C message structs.
    //
    // This does not generate full message support; we use it as the low-level
    // rmw struct definitions which we can then pair with typesupport symbols.

    let prefixes = ros_prefixes();

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings_path = out_dir.join("ros_tf_msgs_bindings.rs");

    let mut builder = bindgen::Builder::default()
        .clang_arg("-std=c11")
        // Keep bindgen output stable and minimal.
        .allowlist_type("builtin_interfaces__msg__Time")
        .allowlist_type("std_msgs__msg__Header")
        .allowlist_type("geometry_msgs__msg__Vector3")
        .allowlist_type("geometry_msgs__msg__Quaternion")
        .allowlist_type("geometry_msgs__msg__Transform")
        .allowlist_type("geometry_msgs__msg__TransformStamped")
        .allowlist_type("tf2_msgs__msg__TFMessage")
        // Sequences used by TFMessage.
        .allowlist_type("geometry_msgs__msg__TransformStamped__Sequence")
        // Functions for init/fini.
        .allowlist_function("builtin_interfaces__msg__Time__.*")
        .allowlist_function("std_msgs__msg__Header__.*")
        .allowlist_function("geometry_msgs__msg__Vector3__.*")
        .allowlist_function("geometry_msgs__msg__Quaternion__.*")
        .allowlist_function("geometry_msgs__msg__Transform__.*")
        .allowlist_function("geometry_msgs__msg__TransformStamped__.*")
        .allowlist_function("tf2_msgs__msg__TFMessage__.*")
        // Needed basic C types.
        .allowlist_type("rosidl_runtime_c__String")
        .allowlist_type("rosidl_runtime_c__String__Sequence")
        .allowlist_function("rosidl_runtime_c__String__.*")
        .allowlist_function("rosidl_runtime_c__String__Sequence__.*")
        // Avoid pulling in platform-specific stuff.
        .blocklist_type("max_align_t")
        .derive_default(true)
        .generate_comments(false);

    for prefix in &prefixes {
        let include_root = prefix.join("include");
        if !include_root.is_dir() {
            continue;
        }

        builder = builder
            .clang_arg(format!("-I{}", include_root.display()))
            .clang_arg(format!(
                "-I{}",
                include_root.join("builtin_interfaces").display()
            ))
            .clang_arg(format!("-I{}", include_root.join("std_msgs").display()))
            .clang_arg(format!(
                "-I{}",
                include_root.join("geometry_msgs").display()
            ))
            .clang_arg(format!("-I{}", include_root.join("tf2_msgs").display()))
            .clang_arg(format!(
                "-I{}",
                include_root.join("rosidl_runtime_c").display()
            ))
            .clang_arg(format!("-I{}", include_root.join("rcutils").display()))
            .clang_arg(format!(
                "-I{}",
                include_root.join("rosidl_typesupport_interface").display()
            ));
    }

    // The headers we need.
    let headers = [
        "builtin_interfaces/builtin_interfaces/msg/detail/time__struct.h",
        "builtin_interfaces/builtin_interfaces/msg/detail/time__functions.h",
        "std_msgs/std_msgs/msg/detail/header__struct.h",
        "std_msgs/std_msgs/msg/detail/header__functions.h",
        "geometry_msgs/geometry_msgs/msg/detail/vector3__struct.h",
        "geometry_msgs/geometry_msgs/msg/detail/vector3__functions.h",
        "geometry_msgs/geometry_msgs/msg/detail/quaternion__struct.h",
        "geometry_msgs/geometry_msgs/msg/detail/quaternion__functions.h",
        "geometry_msgs/geometry_msgs/msg/detail/transform__struct.h",
        "geometry_msgs/geometry_msgs/msg/detail/transform__functions.h",
        "geometry_msgs/geometry_msgs/msg/detail/transform_stamped__struct.h",
        "geometry_msgs/geometry_msgs/msg/detail/transform_stamped__functions.h",
        "tf2_msgs/tf2_msgs/msg/detail/tf_message__struct.h",
        "tf2_msgs/tf2_msgs/msg/detail/tf_message__functions.h",
        "rosidl_runtime_c/rosidl_runtime_c/string.h",
    ];

    for h in headers {
        builder = builder.header(find_header(&prefixes, h).to_string_lossy());
    }

    let bindings = builder
        .generate()
        .expect("Unable to generate ROS TF message bindings");
    bindings
        .write_to_file(&bindings_path)
        .expect("Couldn't write bindings");

    println!("cargo:rerun-if-env-changed=AMENT_PREFIX_PATH");
    println!("cargo:rerun-if-env-changed=CMAKE_PREFIX_PATH");
    for h in headers {
        println!(
            "cargo:rerun-if-changed={}",
            find_header(&prefixes, h).display()
        );
    }
}
