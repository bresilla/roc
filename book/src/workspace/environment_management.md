# Environment Management

ROC's environment management system handles the complex task of setting up proper build and runtime environments for ROS2 packages. This chapter details how environments are constructed, maintained, and used throughout the build process.

## Environment Architecture

### Core Components

The environment management system consists of several key components:

```rust
pub struct EnvironmentManager {
    env_vars: HashMap<String, String>,    // Current environment variables
    install_prefix: PathBuf,              // Install prefix directory
    isolated: bool,                       // Whether using isolated installs
}
```

### Environment Lifecycle

1. **Initialization**: Start with current shell environment
2. **Package Setup**: Add package-specific paths and variables
3. **Dependency Integration**: Include paths from built dependencies
4. **Build Execution**: Provide clean environment to build processes
5. **Script Generation**: Create setup scripts for workspace activation

## Build-Time Environment Management

### Environment Isolation Strategy

ROC uses different isolation strategies based on build mode:

#### Sequential Builds
- Each package gets a fresh `EnvironmentManager` instance
- Prevents environment accumulation that can cause CMake hangs
- Ensures clean, predictable builds

```rust
// Create a fresh environment manager for this package
let mut package_env_manager = EnvironmentManager::new(
    self.config.install_base.clone(),
    self.config.isolated
);

// Setup environment for this package
package_env_manager.setup_package_environment(&package.name, &package.path)?;
```

#### Parallel Builds
- Each worker thread maintains its own environment state
- Synchronizes with shared build state for dependency tracking
- Updates environment only with completed dependencies

### PATH-Like Variable Management

The system handles PATH-like environment variables with sophisticated logic:

```rust
fn update_path_env(&mut self, var_name: &str, new_path: &Path) {
    let separator = if cfg!(windows) { ";" } else { ":" };
    let new_path_str = new_path.to_string_lossy();
    
    if let Some(current) = self.env_vars.get(var_name) {
        // Check if path is already in the variable
        let paths: Vec<&str> = current.split(separator).collect();
        if !paths.contains(&new_path_str.as_ref()) {
            let updated = format!("{}{}{}", new_path_str, separator, current);
            self.env_vars.insert(var_name.to_string(), updated);
        }
    } else {
        self.env_vars.insert(var_name.to_string(), new_path_str.to_string());
    }
}
```

This approach:
- **Prevents Duplicates**: Avoids adding the same path multiple times
- **Maintains Order**: New paths are prepended for priority
- **Cross-Platform**: Uses appropriate path separators

### Key Environment Variables

The system manages these critical environment variables:

#### ROS2-Specific Variables
```rust
// Core ROS2 environment
CMAKE_PREFIX_PATH     // CMake package discovery
AMENT_PREFIX_PATH     // Ament package discovery
COLCON_PREFIX_PATH    // Colcon compatibility

// Build and execution paths
PATH                  // Executable discovery
LD_LIBRARY_PATH      // Library loading (Linux)
DYLD_LIBRARY_PATH    // Library loading (macOS)
PYTHONPATH           // Python module discovery

// Build configuration
PKG_CONFIG_PATH      // pkg-config discovery
CMAKE_MODULE_PATH    // CMake module discovery
```

#### ROS Environment Detection
```rust
fn is_ros_relevant_env_var(key: &str) -> bool {
    match key {
        // Core ROS2 environment variables
        "CMAKE_PREFIX_PATH" | "AMENT_PREFIX_PATH" | "COLCON_PREFIX_PATH" => true,
        
        // System library paths
        "PATH" | "LD_LIBRARY_PATH" | "DYLD_LIBRARY_PATH" => true,
        
        // Python paths
        "PYTHONPATH" => true,
        
        // ROS-specific variables
        key if key.starts_with("ROS_") => true,
        key if key.starts_with("AMENT_") => true,
        key if key.starts_with("COLCON_") => true,
        key if key.starts_with("RCUTILS_") => true,
        key if key.starts_with("RMW_") => true,
        
        // Build-related variables
        "PKG_CONFIG_PATH" | "CMAKE_MODULE_PATH" => true,
        
        _ => false,
    }
}
```

## Package Environment Setup

### Per-Package Configuration

For each package, the environment manager configures:

```rust
pub fn setup_package_environment(&mut self, package_name: &str, _package_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let install_dir = if self.isolated {
        self.install_prefix.join(package_name)  // Isolated: install/package_name/
    } else {
        self.install_prefix.clone()             // Merged: install/
    };
    
    // Update CMAKE_PREFIX_PATH
    self.update_path_env("CMAKE_PREFIX_PATH", &install_dir);
    
    // Update AMENT_PREFIX_PATH  
    self.update_path_env("AMENT_PREFIX_PATH", &install_dir);
    
    // Update PATH to include bin directories
    let bin_dir = install_dir.join("bin");
    if bin_dir.exists() {
        self.update_path_env("PATH", &bin_dir);
    }
    
    // Update library paths
    #[cfg(target_os = "linux")]
    {
        let lib_dir = install_dir.join("lib");
        if lib_dir.exists() {
            self.update_path_env("LD_LIBRARY_PATH", &lib_dir);
        }
    }
    
    // Update Python path
    let python_lib_dirs = [
        install_dir.join("lib").join("python3").join("site-packages"),
        install_dir.join("local").join("lib").join("python3").join("site-packages"),
    ];
    
    for python_dir in &python_lib_dirs {
        if python_dir.exists() {
            self.update_path_env("PYTHONPATH", python_dir);
        }
    }
    
    Ok(())
}
```

### Directory Structure Handling

The system adapts to different install directory structures:

#### Isolated Installs (`--isolated`)
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
```

#### Merged Installs (`--merge-install`)
```
install/
├── bin/          # All executables
├── lib/          # All libraries
├── share/        # All shared resources
```

### Build Tool Integration

Environment setup integrates with different build systems:

#### CMake Integration
- Sets `CMAKE_PREFIX_PATH` for `find_package()` commands
- Configures `CMAKE_INSTALL_PREFIX` for install locations
- Provides environment for CMake's build and install phases

#### Python Integration
- Updates `PYTHONPATH` for module discovery
- Sets up virtual environment compatibility
- Handles setuptools installation requirements

## Setup Script Generation

### Script Architecture

ROC generates comprehensive setup scripts that mirror colcon's behavior:

#### Bash Setup Scripts
```bash
#!/bin/bash
# ROS2 workspace setup script generated by roc

_roc_prepend_path() {
    local var_name="$1"
    local new_path="$2"
    
    if [ -z "${!var_name}" ]; then
        export "$var_name"="$new_path"
    else
        # Check if path is already present
        if [[ ":${!var_name}:" != *":$new_path:"* ]]; then
            export "$var_name"="$new_path:${!var_name}"
        fi
    fi
}

# Environment variable exports
export CMAKE_PREFIX_PATH="/workspace/install:/opt/ros/humble"
export AMENT_PREFIX_PATH="/workspace/install:/opt/ros/humble"

# Mark workspace as sourced
export ROC_WORKSPACE_SOURCED=1
```

#### Windows Batch Scripts
```batch
@echo off
REM ROS2 workspace setup script generated by roc

set "CMAKE_PREFIX_PATH=C:\workspace\install;C:\opt\ros\humble"
set "AMENT_PREFIX_PATH=C:\workspace\install;C:\opt\ros\humble"

REM Mark workspace as sourced
set "ROC_WORKSPACE_SOURCED=1"
```

### Script Generation Process

#### Per-Package Scripts (Isolated Mode)
```rust
// Generate individual package setup scripts
for package in packages {
    if let Some(pkg_install_path) = self.install_paths.get(&package.name) {
        let package_dir = pkg_install_path.join("share").join(&package.name);
        fs::create_dir_all(&package_dir)?;
        
        let package_setup = package_dir.join("package.bash");
        let package_setup_content = format!(r#"#!/bin/bash
# Generated setup script for package {}

export CMAKE_PREFIX_PATH="{}:${{CMAKE_PREFIX_PATH}}"
export AMENT_PREFIX_PATH="{}:${{AMENT_PREFIX_PATH}}"

if [ -d "{}/bin" ]; then
    export PATH="{}/bin:${{PATH}}"
fi

if [ -d "{}/lib" ]; then
    export LD_LIBRARY_PATH="{}/lib:${{LD_LIBRARY_PATH}}"
fi
"#, 
            package.name,
            pkg_install_path.display(),
            pkg_install_path.display(),
            pkg_install_path.display(),
            pkg_install_path.display(),
            pkg_install_path.display(),
            pkg_install_path.display()
        );
        
        fs::write(&package_setup, package_setup_content)?;
    }
}
```

#### Workspace Setup Script
```rust
// Generate workspace setup script
let setup_bash = install_dir.join("setup.bash");
let mut setup_content = String::from(r#"#!/bin/bash
# Generated by roc workspace build tool

if [ -n "$COLCON_CURRENT_PREFIX" ]; then
    _colcon_current_prefix="$COLCON_CURRENT_PREFIX"
fi
export COLCON_CURRENT_PREFIX="{}"

"#);

// Source each package in dependency order
for package in packages {
    if self.install_paths.contains_key(&package.name) {
        setup_content.push_str(&format!(
            r#"if [ -f "$COLCON_CURRENT_PREFIX/{}/share/{}/package.bash" ]; then
    source "$COLCON_CURRENT_PREFIX/{}/share/{}/package.bash"
fi
"#,
            package.name, package.name, package.name, package.name
        ));
    }
}
```

### Cross-Platform Considerations

#### Unix Systems (Linux/macOS)
- Uses bash syntax with `export` commands
- Sets executable permissions on script files
- Handles library path differences (LD_LIBRARY_PATH vs DYLD_LIBRARY_PATH)

#### Windows Systems
- Generates `.bat` files with `set` commands
- Uses Windows path separators (`;` instead of `:`)
- Handles different library path conventions

## Environment Debugging

### Diagnostic Features

The environment manager includes debugging capabilities:

#### Environment Variable Inspection
```rust
pub fn get_env_vars(&self) -> &HashMap<String, String> {
    &self.env_vars
}

pub fn get_env_var(&self, key: &str) -> Option<&String> {
    self.env_vars.get(key)
}
```

#### ROS-Specific Filtering
Only ROS-relevant environment variables are included in setup scripts to avoid pollution:

```rust
// Add environment variable exports with ROS-specific filtering
for (key, value) in &self.env_vars {
    // Only export ROS-related and essential environment variables
    if Self::is_ros_relevant_env_var(key) {
        script.push_str(&format!("export {}=\"{}\"\n", key, value));
    }
}
```

### Common Environment Issues

#### Build Environment Pollution
- **Problem**: Accumulated environment variables cause CMake hangs
- **Solution**: Fresh environment instances for each package

#### Missing Dependencies
- **Problem**: Required tools not found in PATH
- **Solution**: Comprehensive environment validation

#### Path Duplication
- **Problem**: Same paths added multiple times
- **Solution**: Duplicate detection in path management

## Performance Optimizations

### Memory Efficiency
- Environment variables stored as `HashMap<String, String>`
- Minimal copying of environment data between processes
- Efficient string operations for path manipulation

### I/O Optimization
- Batch file operations for script generation
- Minimal filesystem operations during environment setup
- Efficient script template generation

### Parallelization
- Thread-safe environment management for parallel builds
- Independent environment instances prevent contention
- Shared state only for coordination, not environment data

The environment management system provides a robust foundation for ROS2 workspace builds, ensuring that packages have access to their dependencies while maintaining clean, predictable build environments that scale from single-package builds to large, complex workspaces.
