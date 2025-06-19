
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
- `work` `[w]` - Workspace management and build tools

### Utility Commands
- `bag` `[b]` - ROS bag recording and playback
- `daemon` `[d]` - Daemon and bridge management
- `middleware` `[m]` - RMW configuration and diagnostics
- `frame` `[f]` - Transform frame utilities

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

## 🛠️ Development

### Prerequisites
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
- 🚧 **Action Operations**: Basic functionality (expanding)
- 🚧 **Bag Operations**: Recording and playback (in progress)
- ⏳ **Launch Operations**: Planning phase
- ⏳ **Workspace Operations**: Planning phase

## 🤝 Why ROC?

ROC was created to address limitations in the existing ROS2 toolchain:
- **Performance**: Native Rust implementation eliminates Python overhead
- **Reliability**: Strong typing and memory safety reduce runtime errors  
- **Completeness**: Direct RCL/RMW access enables features not available in the standard CLI
- **Developer Experience**: Better error messages, shell completions, and debugging tools

## 📄 License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.
