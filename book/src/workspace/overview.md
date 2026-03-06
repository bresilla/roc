# Workspace Management Overview

ROC includes a native workspace management system that is aiming to replace `colcon build` over time. The `roc work` command provides package creation, discovery, dependency resolution, and building, but compatibility should be judged from validated behavior rather than the long-term goal.

## Key Features

### Native Workspace Builder
`roc work build` already covers a substantial part of the `colcon build` workflow, with the following strengths:

- **Native Performance**: Written in Rust for superior performance and memory safety
- **Parallel Execution**: Multi-threaded builds with intelligent dependency resolution
- **Environment Isolation**: Clean environment management preventing build contamination
- **Comprehensive Logging**: Detailed build logs and error reporting
- **Growing Compatibility**: supports major `colcon build` options, with validation tracked separately

### Package Management
- **Intelligent Discovery**: Automatic workspace scanning and package.xml parsing
- **Metadata Extraction**: Complete package information including dependencies, maintainers, and build types
- **Build Type Support**: native handling for `ament_cmake`, `ament_python`, and plain `cmake` packages
- **Dependency Validation**: Circular dependency detection and resolution

### Development Workflow
- **Package Creation**: Intelligent wizard for creating properly structured ROS2 packages
- **Build Optimization**: Incremental builds and parallel execution
- **Environment Setup**: Automatic generation of setup scripts for workspace activation

## Architecture

The workspace management system is built on several core components:

1. **Package Discovery Engine**: Recursively scans workspace directories for `package.xml` files
2. **Dependency Graph Resolver**: Builds and validates package dependency graphs
3. **Build Executor**: Manages parallel build execution with proper environment isolation
4. **Environment Manager**: Handles environment variable setup and setup script generation

## Command Structure

```bash
roc work <subcommand> [options]
```

### Available Subcommands

- `build` - Build packages in the workspace with a native colcon-like builder
- `create` - Create new ROS2 packages with templates
- `list` - List and discover packages in the workspace
- `info` - Display detailed package information

## Compatibility

ROC's workspace system is intended to fit existing ROS 2 workflows, but compatibility is still uneven:

- **Colcon Arguments**: major `colcon build` options are supported
- **Package Formats**: Supports package.xml formats 2 and 3
- **Build Systems**: Works with ament_cmake, ament_python, and plain cmake
- **Environment**: Generates standard shell setup scripts, with known differences still documented in `COMPAT_VALIDATION.md`

Current validated status:

- minimal `ament_cmake` workflow: working
- minimal `ament_python` workflow: partially working, with remaining package-registration gaps

## Performance Benefits

Compared to colcon, ROC provides:

- **Faster Startup**: Native binary with minimal overhead
- **Better Parallelization**: More efficient worker thread management
- **Memory Efficiency**: Lower memory usage during builds
- **Cleaner Environment**: Better isolation prevents build environment pollution
- **Superior Error Handling**: More detailed error messages and recovery options

The following sections provide detailed information about each component of the workspace management system.
