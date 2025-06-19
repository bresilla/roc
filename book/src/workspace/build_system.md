# Build System Architecture

ROC's build system is designed as a high-performance, parallel replacement for colcon. This chapter details the internal architecture and implementation of the build system.

## Core Components

### 1. Build Configuration (`BuildConfig`)

The build system is driven by a comprehensive configuration structure that mirrors colcon's options:

```rust
pub struct BuildConfig {
    pub base_paths: Vec<PathBuf>,           // Paths to search for packages
    pub packages_select: Option<Vec<String>>, // Build only selected packages
    pub packages_ignore: Option<Vec<String>>, // Ignore specific packages
    pub packages_up_to: Option<Vec<String>>,  // Build up to specified packages
    pub parallel_workers: u32,               // Number of parallel build workers
    pub merge_install: bool,                 // Use merged vs isolated install
    pub symlink_install: bool,               // Use symlinks for installs
    pub cmake_args: Vec<String>,             // Additional CMake arguments
    pub cmake_target: Option<String>,        // Specific CMake target
    pub continue_on_error: bool,             // Continue building on failures
    pub workspace_root: PathBuf,             // Root of workspace
    pub install_base: PathBuf,               // Install directory
    pub build_base: PathBuf,                 // Build directory
    pub isolated: bool,                      // Isolated vs merged installs
}
```

### 2. Build Orchestrator (`ColconBuilder`)

The main orchestrator manages the entire build process:

```rust
pub struct ColconBuilder {
    config: BuildConfig,
    packages: Vec<PackageMeta>,    // Discovered packages
    build_order: Vec<usize>,       // Topologically sorted build order
}
```

#### Build Process Flow

1. **Package Discovery**: Scan workspace for `package.xml` files
2. **Dependency Resolution**: Build dependency graph and determine build order
3. **Environment Setup**: Prepare build environments for each package
4. **Build Execution**: Execute builds in parallel with proper dependency ordering
5. **Setup Script Generation**: Create workspace activation scripts

### 3. Build Executor (`BuildExecutor`)

The build executor handles the actual compilation process:

#### Sequential vs Parallel Execution

**Sequential Mode** (`parallel_workers = 1`):
- Uses `build_sequential_filtered()` method
- Creates fresh environment for each package to prevent contamination
- Processes packages in strict topological order

**Parallel Mode** (`parallel_workers > 1`):
- Spawns worker threads up to the configured limit
- Uses shared state management for coordination
- Implements work-stealing queue for load balancing

#### Build State Management

```rust
pub struct BuildState {
    package_states: Arc<Mutex<HashMap<String, PackageState>>>,
    install_paths: Arc<Mutex<HashMap<String, PathBuf>>>,
    build_count: Arc<Mutex<(usize, usize)>>, // (successful, failed)
}

#[derive(Debug, Clone, PartialEq)]
pub enum PackageState {
    Pending,    // Waiting for dependencies
    Building,   // Currently being built
    Completed,  // Successfully built
    Failed,     // Build failed
}
```

### 4. Build Type Handlers

The system supports multiple build types through dedicated handlers:

#### CMake Handler (`build_cmake_package_with_env`)
- Configures CMake with appropriate flags and environment
- Supports both ament_cmake and plain cmake packages
- Handles install prefix configuration for isolated/merged installs

```bash
cmake -S <source> -B <build> -DCMAKE_INSTALL_PREFIX=<install>
cmake --build <build> --target install -- -j<workers>
```

#### Python Handler (`build_python_package_with_env`)
- Uses Python setuptools for ament_python packages
- Handles build and install phases separately

```bash
python3 setup.py build --build-base <build>
python3 setup.py install --prefix "" --root <install>
```

## Environment Management

### Build-Time Environment

Each package build receives a carefully constructed environment:

1. **Base Environment**: Inherits from current shell environment
2. **Dependency Paths**: Adds install paths of all built dependencies
3. **Build Tools**: Ensures CMake, Python, and other tools are available
4. **ROS Environment**: Sets up AMENT_PREFIX_PATH, CMAKE_PREFIX_PATH, etc.

### Environment Isolation

The system uses two strategies for environment isolation:

**Sequential Builds**: Each package gets a fresh `EnvironmentManager` instance to prevent environment accumulation that can cause CMake hangs.

**Parallel Builds**: Each worker thread maintains its own environment state, updating it only with completed dependencies.

### Path Management

Environment variables are updated using intelligent path prepending:

```rust
fn update_path_env(&mut self, var_name: &str, new_path: &Path) {
    let separator = if cfg!(windows) { ";" } else { ":" };
    
    if let Some(current) = self.env_vars.get(var_name) {
        // Check for duplicates before adding
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

## Parallel Execution Strategy

### Worker Thread Model

The parallel build system uses a work-stealing approach:

1. **Worker Spawning**: Creates `parallel_workers` threads
2. **Work Discovery**: Each worker scans for packages whose dependencies are satisfied
3. **State Synchronization**: Uses `Arc<Mutex<>>` for thread-safe state sharing
4. **Load Balancing**: Workers dynamically pick up available work

### Dependency Satisfaction

Before building a package, workers verify all dependencies are completed:

```rust
let all_deps_ready = deps.iter().all(|dep| {
    states.get(dep).map(|s| *s == PackageState::Completed).unwrap_or(true)
});
```

External dependencies (not in workspace) are assumed to be available.

### Error Handling

The system supports flexible error handling:

- **Fail Fast** (default): Stop all builds on first failure
- **Continue on Error**: Mark failed packages but continue with independent packages
- **Detailed Logging**: Capture stdout/stderr for debugging

## Performance Optimizations

### Memory Management
- Zero-copy string handling where possible
- Efficient HashMap usage for package lookup
- Minimal cloning of large data structures

### I/O Optimization
- Parallel directory scanning during package discovery
- Asynchronous log writing
- Efficient XML parsing with `roxmltree`

### Build Efficiency
- Leverages CMake's internal dependency checking
- Reuses build directories for incremental builds
- Intelligent environment caching

## Error Recovery

The build system includes comprehensive error handling:

### Build Failures
- Captures complete stdout/stderr output
- Provides detailed error context
- Suggests common fixes for typical issues

### Environment Issues
- Validates required tools (cmake, python) are available
- Checks for common environment problems
- Provides clear error messages for missing dependencies

### Recovery Strategies
- Supports partial rebuilds after fixing issues
- Maintains build state across invocations
- Allows selective package rebuilds

This architecture provides a robust, scalable foundation for workspace builds that significantly outperforms traditional Python-based tools while maintaining full compatibility with existing ROS2 workflows.
