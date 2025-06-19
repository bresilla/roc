# Colcon Compatibility

ROC's workspace build system is designed as a comprehensive drop-in replacement for colcon. This chapter details the compatibility features, command-line argument mapping, and behavioral equivalences that make ROC a seamless replacement for existing ROS2 workflows.

## Command-Line Compatibility

### Build Command Mapping

ROC provides full compatibility with colcon's most commonly used build options:

| Colcon Command | ROC Equivalent | Description |
|---------------|----------------|-------------|
| `colcon build` | `roc work build` | Build all packages in workspace |
| `colcon build --packages-select pkg1 pkg2` | `roc work build --packages-select pkg1 pkg2` | Build only specified packages |
| `colcon build --packages-ignore pkg1` | `roc work build --packages-ignore pkg1` | Skip specified packages |
| `colcon build --packages-up-to pkg1` | `roc work build --packages-up-to pkg1` | Build dependencies up to package |
| `colcon build --parallel-workers 4` | `roc work build --parallel-workers 4` | Set number of parallel workers |
| `colcon build --merge-install` | `roc work build --merge-install` | Use merged install directory |
| `colcon build --symlink-install` | `roc work build --symlink-install` | Use symlinks for installation |
| `colcon build --continue-on-error` | `roc work build --continue-on-error` | Continue building after failures |
| `colcon build --cmake-args -DCMAKE_BUILD_TYPE=Debug` | `roc work build --cmake-args -DCMAKE_BUILD_TYPE=Debug` | Pass arguments to CMake |

### Argument Processing

ROC's argument processing mirrors colcon's behavior:

```rust
// Parse command line arguments
if let Some(base_paths) = matches.get_many::<String>("base_paths") {
    config.base_paths = base_paths.map(PathBuf::from).collect();
}

if let Some(packages) = matches.get_many::<String>("packages_select") {
    config.packages_select = Some(packages.map(|s| s.to_string()).collect());
}

if let Some(packages) = matches.get_many::<String>("packages_ignore") {
    config.packages_ignore = Some(packages.map(|s| s.to_string()).collect());
}

if let Some(packages) = matches.get_many::<String>("packages_up_to") {
    config.packages_up_to = Some(packages.map(|s| s.to_string()).collect());
}

if let Some(workers) = matches.get_one::<u32>("parallel_workers") {
    config.parallel_workers = *workers;
}

config.merge_install = matches.get_flag("merge_install");
config.symlink_install = matches.get_flag("symlink_install");
config.continue_on_error = matches.get_flag("continue_on_error");

if let Some(cmake_args) = matches.get_many::<String>("cmake_args") {
    config.cmake_args = cmake_args.map(|s| s.to_string()).collect();
}
```

## Workspace Structure Compatibility

### Directory Layout

ROC maintains the same workspace structure as colcon:

```
workspace/
├── src/                    # Source packages (default discovery path)
│   ├── package1/
│   │   ├── package.xml
│   │   └── CMakeLists.txt
│   └── package2/
│       ├── package.xml
│       └── setup.py
├── build/                  # Build artifacts (created by ROC)
│   ├── package1/
│   └── package2/
├── install/                # Install artifacts (created by ROC)
│   ├── package1/          # Isolated install (default)
│   ├── package2/
│   └── setup.bash         # Workspace setup script
└── log/                    # Build logs (created by ROC)
    └── latest/
        ├── package1/
        └── package2/
```

### Install Space Modes

ROC supports both colcon install modes:

#### Isolated Install (Default)
```rust
let install_prefix = if config.merge_install {
    config.workspace_root.join("install")
} else {
    config.workspace_root.join("install").join(&package.name)
};
```

**Isolated Structure:**
```
install/
├── package1/
│   ├── bin/
│   ├── lib/
│   └── share/
├── package2/
│   ├── bin/
│   ├── lib/
│   └── share/
└── setup.bash
```

#### Merged Install (`--merge-install`)
**Merged Structure:**
```
install/
├── bin/          # All executables
├── lib/          # All libraries  
├── share/        # All shared resources
└── setup.bash
```

## Package Format Compatibility

### Package.xml Support

ROC supports the same package.xml formats as colcon:

#### Format 2 (REP 140)
```xml
<?xml version="1.0"?>
<package format="2">
  <name>my_package</name>
  <version>1.0.0</version>
  <description>Package description</description>
  <maintainer email="maintainer@example.com">Maintainer Name</maintainer>
  <license>Apache-2.0</license>
  
  <buildtool_depend>ament_cmake</buildtool_depend>
  <build_depend>rclcpp</build_depend>
  <exec_depend>rclcpp</exec_depend>
  
  <export>
    <build_type>ament_cmake</build_type>
  </export>
</package>
```

#### Format 3 (REP 149)
```xml
<?xml version="1.0"?>
<package format="3">
  <name>my_package</name>
  <version>1.0.0</version>
  <description>Package description</description>
  <maintainer email="maintainer@example.com">Maintainer Name</maintainer>
  <license>Apache-2.0</license>
  
  <depend>rclcpp</depend>
  <build_depend condition="$ROS_VERSION == 2">ros2_specific_dep</build_depend>
  
  <export>
    <build_type>ament_cmake</build_type>
  </export>
</package>
```

### Build Type Support

ROC supports all major build types used in ROS2:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum BuildType {
    AmentCmake,           // Standard C++ packages
    AmentPython,          // Pure Python packages
    Cmake,                // Plain CMake packages
    Other(String),        // Extensible for future types
}

impl From<&str> for BuildType {
    fn from(s: &str) -> Self {
        match s {
            "ament_cmake" => BuildType::AmentCmake,
            "ament_python" => BuildType::AmentPython,
            "cmake" => BuildType::Cmake,
            other => BuildType::Other(other.to_string()),
        }
    }
}
```

## Build Process Compatibility

### Build System Integration

ROC uses the same build system invocations as colcon:

#### CMake Packages
```rust
// Configure phase
let mut configure_cmd = Command::new("cmake");
configure_cmd
    .arg("-S").arg(&package.path)
    .arg("-B").arg(&build_dir)
    .arg(format!("-DCMAKE_INSTALL_PREFIX={}", install_prefix.display()));

// Build and install phase
let mut build_cmd = Command::new("cmake");
build_cmd
    .arg("--build").arg(&build_dir)
    .arg("--target").arg("install")
    .arg("--")
    .arg(format!("-j{}", config.parallel_workers));
```

#### Python Packages
```rust
// Build phase
Command::new("python3")
    .arg("setup.py")
    .arg("build")
    .arg("--build-base").arg(&build_dir)
    .current_dir(&package.path)

// Install phase
Command::new("python3")
    .arg("setup.py")
    .arg("install")
    .arg("--prefix").arg("")
    .arg("--root").arg(&install_prefix)
    .current_dir(&package.path)
```

### Environment Setup

ROC generates the same environment setup scripts as colcon:

#### Setup Script Structure
```bash
#!/bin/bash
# Generated by roc workspace build tool (colcon compatible)

# Source any parent workspaces
if [ -n "$COLCON_CURRENT_PREFIX" ]; then
    _colcon_current_prefix="$COLCON_CURRENT_PREFIX"
fi
export COLCON_CURRENT_PREFIX="{}"

# Add this workspace to environment
export CMAKE_PREFIX_PATH="$COLCON_CURRENT_PREFIX:${CMAKE_PREFIX_PATH}"
export AMENT_PREFIX_PATH="$COLCON_CURRENT_PREFIX:${AMENT_PREFIX_PATH}"

# Standard paths
if [ -d "$COLCON_CURRENT_PREFIX/bin" ]; then
    export PATH="$COLCON_CURRENT_PREFIX/bin:${PATH}"
fi

if [ -d "$COLCON_CURRENT_PREFIX/lib" ]; then
    export LD_LIBRARY_PATH="$COLCON_CURRENT_PREFIX/lib:${LD_LIBRARY_PATH}"
fi

# Python paths
if [ -d "$COLCON_CURRENT_PREFIX/lib/python3.10/site-packages" ]; then
    export PYTHONPATH="$COLCON_CURRENT_PREFIX/lib/python3.10/site-packages:${PYTHONPATH}"
fi

# Restore previous prefix
if [ -n "$_colcon_current_prefix" ]; then
    export COLCON_CURRENT_PREFIX="$_colcon_current_prefix"
    unset _colcon_current_prefix
else
    unset COLCON_CURRENT_PREFIX
fi
```

## Output and Logging Compatibility

### Console Output Format

ROC matches colcon's console output format:

```
🔧 Building ROS2 workspace with roc (colcon replacement)
Workspace: /home/user/workspace

Discovered 3 packages
  - my_cpp_package (AmentCmake)
  - my_py_package (AmentPython)
  - my_msgs (AmentCmake)

Build order:
  my_msgs
  my_cpp_package
  my_py_package

Starting >>> my_msgs (AmentCmake)
  Configuring with CMake...
  ✅ CMake configure succeeded
  Building and installing...
  ✅ Build and install succeeded
Finished <<< my_msgs [2.34s]

Starting >>> my_cpp_package (AmentCmake)
  Configuring with CMake...
  ✅ CMake configure succeeded
  Building and installing...
  ✅ Build and install succeeded
Finished <<< my_cpp_package [4.12s]

Starting >>> my_py_package (AmentPython)
  Building and installing...
  ✅ Build and install succeeded
Finished <<< my_py_package [1.23s]

Build Summary:
  3 packages succeeded

✅ Build completed successfully!
To use the workspace, run:
  source install/setup.bash
```

### Log Directory Structure

ROC maintains the same logging structure as colcon:

```
log/
├── latest/                 # Symlink to most recent build
│   ├── build.log          # Overall build log
│   ├── my_msgs/
│   │   └── stdout_stderr.log
│   ├── my_cpp_package/
│   │   └── stdout_stderr.log
│   └── my_py_package/
│       └── stdout_stderr.log
└── 2025-06-19_14-30-15/   # Timestamped build logs
    └── ...
```

## Migration Guide

### Switching from Colcon

For existing ROS2 projects, switching to ROC is straightforward:

#### 1. Install ROC
```bash
# From source
git clone https://github.com/your-org/roc.git
cd roc
cargo build --release

# Or from crates.io
cargo install rocc
```

#### 2. Update Build Scripts
Replace colcon commands in scripts:

**Before:**
```bash
#!/bin/bash
source /opt/ros/humble/setup.bash
cd /path/to/workspace
colcon build --parallel-workers 4 --cmake-args -DCMAKE_BUILD_TYPE=Release
source install/setup.bash
```

**After:**
```bash
#!/bin/bash
source /opt/ros/humble/setup.bash
cd /path/to/workspace
roc work build --parallel-workers 4 --cmake-args -DCMAKE_BUILD_TYPE=Release
source install/setup.bash
```

#### 3. CI/CD Integration
Update continuous integration scripts:

**GitHub Actions Example:**
```yaml
- name: Build workspace
  run: |
    source /opt/ros/humble/setup.bash
    roc work build --parallel-workers 2
    source install/setup.bash
```

**Docker Example:**
```dockerfile
RUN source /opt/ros/humble/setup.bash && \
    roc work build --parallel-workers $(nproc) && \
    source install/setup.bash
```

### Behavioral Differences

While ROC maintains high compatibility, there are some differences:

#### Performance Improvements
- **Faster startup**: Native binary vs Python interpreter
- **Better parallelization**: More efficient worker management
- **Memory efficiency**: Lower memory usage during builds

#### Enhanced Error Handling
- **More detailed error messages**: Better context and suggestions
- **Cleaner error output**: Structured error reporting
- **Recovery suggestions**: Actionable advice for common issues

#### Environment Management
- **Cleaner environments**: Better isolation prevents contamination
- **Filtered variables**: Only ROS-relevant variables in setup scripts
- **Windows support**: Better cross-platform environment handling

## Future Compatibility

### Planned Features

ROC's roadmap includes additional colcon compatibility features:

#### Advanced Options
- `--event-handlers`: Custom build event processing
- `--executor`: Different parallel execution strategies
- `--log-base`: Custom log directory locations
- `--install-base`: Custom install directory locations

#### Extensions
- Plugin system for custom build types
- Custom event handlers
- Advanced dependency resolution strategies

### API Compatibility

ROC is designed to maintain API compatibility with colcon's extension points, enabling future integration with existing colcon plugins and extensions where appropriate.

The colcon compatibility layer ensures that ROC can serve as a drop-in replacement for colcon in virtually all ROS2 development workflows, while providing superior performance and enhanced features that improve the developer experience.
