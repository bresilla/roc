
<img align="right" width="32%" src="./misc/logo.png">

# ROC - Robot Operations Command

**A high-performance ROS2 command-line tool written in Rust**

ROC is a modern replacement for the ROS2 CLI toolchain, built with Rust for performance and reliability. Unlike other implementations, ROC directly interfaces with the RCL (ROS Client Library) and RMW (ROS Middleware) layers through FFI bindings, providing native-level performance and detailed introspection capabilities.

## 🚀 Features

- **Direct RCL/RMW Integration**: Native bindings to ROS2's core libraries
- **Comprehensive Topic Information**: Detailed QoS profiles, endpoint discovery, and type introspection
- **High Performance**: Built in Rust for speed and memory safety
- **Complete CLI Compatibility**: Drop-in replacement for `ros2` commands
- **Advanced Debugging**: Detailed endpoint information including GIDs and type hashes
- **Shell Completions**: Dynamic completions for bash, zsh, and fish

## 📋 Installation

### From Crates.io
```bash
cargo install rocc
```

### From Source
```bash
git clone https://github.com/your-org/roc.git
cd roc
cargo build --release
```

### Binary Releases
Download pre-built binaries from our [releases page](https://github.com/your-org/roc/releases).

## 🔧 Usage

```bash
roc <COMMAND> [SUBCOMMAND] [OPTIONS] [ARGS]
```

### Monitor Commands
- `action` `[a]` - Action server introspection and interaction
- `topic` `[t]` - Topic monitoring, publishing, and detailed info
- `service` `[s]` - Service discovery and calling
- `param` `[p]` - Parameter management and introspection
- `node` `[n]` - Node discovery and information
- `interface` `[i]` - Message/service type introspection

### Workspace Commands
- `run` `[r]` - Execute ROS2 packages and nodes
- `launch` `[l]` - Launch file execution
- `work` `[w]` - **Complete workspace management suite**
  - `build` - **Colcon replacement build system** with parallel builds, dependency resolution, and environment management
  - `create` - Package creation wizard for ament_cmake, ament_python, and cmake packages
  - `list` - Package discovery and listing
  - `info` - Package metadata and dependency information

### Utility Commands
- `bag` `[b]` - ROS bag recording and playback
- `daemon` `[d]` - Daemon and bridge management
- `middleware` `[m]` - RMW configuration and diagnostics
- `frame` `[f]` - Transform frame utilities
- `idl` `[interface-def]` - **Interface Definition Language tools**
  - `protobuf` - **Bidirectional conversion between Protobuf (.proto) and ROS2 (.msg) files**

## 🎯 Key Advantages

### Native Performance
ROC bypasses the Python layer entirely, interfacing directly with RCL/RMW through optimized Rust FFI bindings. This provides:
- Faster startup times
- Lower memory usage
- More reliable operation
- Better error handling

### Enhanced Topic Information
Get comprehensive topic details that exceed the standard ROS2 CLI:

```bash
roc topic info /chatter --verbose
```

This provides detailed QoS profiles, endpoint information, GIDs, type hashes, and publisher/subscriber discovery data.

### Architecture
ROC is built on a layered architecture:
- **RCL/RMW FFI Layer**: Direct bindings to ROS2 core libraries
- **Graph Context**: Efficient ROS graph introspection
- **Command Interface**: Familiar CLI matching ROS2 tools
- **Shell Integration**: Dynamic completions and scripting support

## 📚 Documentation

Comprehensive documentation is available in the [ROC Book](./book/), including:
- **Architecture Overview**: How ROC interfaces with ROS2
- **RCL/RMW Integration**: Technical details of the FFI bindings
- **Implementation Guide**: Deep dives into graph context and endpoint discovery
- **Examples**: Practical usage patterns and integration examples

To build and serve the documentation locally:
```bash
cd book
mdbook serve
```

## � Interface Definition Language (IDL) Tools

ROC includes powerful tools for working with different interface definition languages, enabling seamless interoperability between ROS2 and other systems.

### Protobuf ↔ ROS2 Conversion (`roc idl protobuf`)

Convert between Protobuf (.proto) and ROS2 (.msg) formats with automatic direction detection:

```bash
# Convert .proto files to .msg files
roc idl protobuf robot.proto sensor_data.proto

# Convert .msg files to .proto files  
roc idl protobuf RobotStatus.msg SensorData.msg

# Specify output directory
roc idl protobuf --output ./generated robot.proto

# Dry run to preview output
roc idl protobuf --dry-run robot.proto
```

**Key Features:**
- **Automatic Direction Detection**: Detects conversion direction based on file extensions
- **Advanced Protobuf Support**: Handles nested messages, enums, oneofs, maps, and comments
- **Dependency Resolution**: Generates files in correct dependency order
- **Inplace Output**: Generates files in the same directory as input by default
- **Type Mapping**: Intelligent conversion between Protobuf and ROS2 types
- **Pure Rust Implementation**: No external dependencies or tools required

**Supported Protobuf Features:**
- Primitive types (int32, string, bool, etc.)
- Repeated fields (arrays)
- Nested messages and custom types
- Enums with value mapping
- Oneof fields
- Map types
- Comments and documentation
- Proto3 syntax

**Example Conversion:**
```protobuf
// robot.proto
syntax = "proto3";
package robotics;

message RobotStatus {
  bool active = 1;
  string name = 2;
  repeated double joint_positions = 3;
}

message Robot {
  RobotStatus status = 1;
  int32 id = 2;
}
```

Converts to:
```msg
# RobotStatus.msg
bool active
string name
float64[] joint_positions

# Robot.msg  
RobotStatus status
int32 id
```

## �🛠️ Workspace Management

ROC includes a complete workspace management system that serves as a **drop-in replacement for colcon**:

### Build System (`roc work build`)
- **Full colcon compatibility**: All major colcon build options supported
- **Parallel builds**: Multi-threaded compilation with automatic dependency resolution
- **Package discovery**: Automatic scanning and parsing of package.xml manifests
- **Environment management**: Automatic setup of build and runtime environments
- **Isolated/merged installs**: Support for both colcon install modes
- **Build types supported**: ament_cmake, ament_python, cmake

```bash
# Build entire workspace (like colcon build)
roc work build

# Build specific packages
roc work build --packages-select my_package another_package

# Parallel builds with custom worker count
roc work build --parallel-workers 8

# Build with merged install space
roc work build --merge-install

# Continue on errors
roc work build --continue-on-error
```

### Package Creation (`roc work create`)
Intelligent package creation wizard that generates properly structured ROS2 packages:

```bash
# Create C++ package
roc work create my_cpp_package --build-type ament_cmake

# Create Python package  
roc work create my_py_package --build-type ament_python

# Create with dependencies and metadata
roc work create my_package \
  --build-type ament_cmake \
  --dependencies rclcpp std_msgs \
  --description "My awesome ROS2 package" \
  --maintainer-name "Your Name" \
  --maintainer-email "you@domain.com"
```

### Package Management
```bash
# List all packages in workspace
roc work list

# Get detailed package information
roc work info my_package --xml
```

## 🛠️ Development
- Rust 1.70+ 
- ROS2 (Humble, Iron, or Rolling)
- clang/libclang (for bindgen)

### Building
```bash
cargo build --release
```

### Testing
```bash
cargo test
```

### Contributing
We welcome contributions! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## 📊 Status

ROC is actively developed and production-ready for most ROS2 workflows. Current implementation status:

- ✅ **Topic Operations**: Full feature parity with enhanced diagnostics
- ✅ **Service Operations**: Complete service introspection and calling
- ✅ **Node Operations**: Node discovery and detailed information
- ✅ **Parameter Operations**: Full parameter management
- ✅ **Interface Operations**: Message and service type introspection
- ✅ **IDL Tools**: **Complete Protobuf ↔ ROS2 conversion**
  - ✅ Bidirectional conversion with automatic direction detection
  - ✅ Advanced Protobuf feature support (nested messages, enums, oneofs, maps)
  - ✅ Pure Rust implementation with no external dependencies
  - ✅ Intelligent type mapping and dependency resolution
- ✅ **Workspace Operations**: **Complete colcon replacement build system**
  - ✅ Build system with parallel execution and dependency resolution
  - ✅ Package creation wizard for all major build types
  - ✅ Environment management and setup script generation
  - ✅ Package discovery and metadata extraction
- 🚧 **Action Operations**: Basic functionality (expanding)
- 🚧 **Bag Operations**: Recording and playbook (in progress)
- ⏳ **Launch Operations**: Planning phase

## 🤝 Why ROC?

ROC was created to address limitations in the existing ROS2 toolchain:
- **Performance**: Native Rust implementation eliminates Python overhead
- **Reliability**: Strong typing and memory safety reduce runtime errors  
- **Completeness**: Direct RCL/RMW access enables features not available in the standard CLI
- **Developer Experience**: Better error messages, shell completions, and debugging tools
- **Build System Innovation**: Modern colcon replacement with superior dependency resolution, parallel execution, and cleaner environment management

## 📄 License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.
