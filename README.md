<img align="right" width="32%" src="./misc/logo.png">

# ROC - Robot Operations Command

ROC is a ROS 2 command-line tool implemented in Rust. It implements many ROS 2 workflows directly and uses command naming close to the `ros2` CLI.

For detailed command-by-command implementation status, see `COMPAT.md`.
For workspace-build validation results against `colcon`, see `COMPAT_VALIDATION.md`.

## Installation

### From crates.io

```bash
cargo install rocc
```

### From source

```bash
git clone https://github.com/bresilla/roc.git
cd roc
cargo build --release
```

## Basic usage

```bash
roc <COMMAND> [SUBCOMMAND] [OPTIONS]
```

Main command groups:

- `topic` - topic discovery, publish, echo, bandwidth/rate/delay tools
- `service` - service discovery; `service call` currently delegates to `ros2`
- `action` - action discovery; `action goal` currently delegates to `ros2`
- `node` - node discovery and introspection
- `param` - parameter operations
- `interface` - ROS interface inspection
- `frame` - TF frame tools
- `bag` - MCAP recording/playback and rosbag metadata tools
- `run` - executable discovery and execution
- `launch` - launch file discovery with execution delegated to `ros2 launch`
- `work` - workspace package create/list/info/build
- `idl` - protobuf and ROS message conversion tools

Use command help to inspect options:

```bash
roc --help
roc work --help
roc topic pub --help
```

## Workspace commands

`roc work` includes package and workspace utilities:

- `roc work create` - scaffold packages (`ament_cmake`, `ament_python`, `cmake`)
- `roc work list` - list discovered packages and build state
- `roc work info <package>` - print package metadata
- `roc work build` - build workspace packages with dependency ordering and colcon-like setup generation

Current validation status for `roc work build`:

- validated against a minimal `ament_cmake` workspace: build and `ros2 pkg prefix` worked
- validated against a minimal `ament_python` workspace: build and Python import worked, but package registration is still incomplete

Examples:

```bash
# Build all packages in the current workspace
roc work build

# Build selected packages
roc work build --packages-select my_pkg other_pkg

# Build with merged install layout
roc work build --merge-install

# Create a CMake package
roc work create my_pkg --build_type ament_cmake --node_name talker

# Inspect package details
roc work info my_pkg
```

## IDL and protobuf conversion

`roc idl protobuf` converts between `.proto` and `.msg` files.

```bash
# Proto -> msg
roc idl protobuf robot.proto

# Msg -> proto
roc idl protobuf RobotStatus.msg

# Write generated files to a directory
roc idl protobuf --output ./generated robot.proto
```

## Architecture notes

The project is organized into command modules under `src/commands` and argument definitions under `src/arguments`.

- ROS interactions are implemented through Rust code and ROS 2 bindings.
- Some subcommands intentionally delegate to `ros2` today. Current delegated paths are:
  - `roc service call`
  - `roc action goal`
  - `roc launch <pkg> <launch>`
- Workspace package discovery logic is shared across `work` commands.

Refer to:

- `COMPAT.md` for feature status
- `COMPAT_VALIDATION.md` for direct `colcon` vs `roc` build results
- `FEATURES.md` for high-level feature notes
- `book/` for extended project documentation

## Development

Requirements:

- Rust toolchain
- ROS 2 environment available/sourced for ROS-dependent commands
- clang/libclang for bindgen-based builds

Build and test:

```bash
cargo build --release
cargo test
```

## License

Licensed under MIT. See `LICENSE.md`.
