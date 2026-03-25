<img align="right" width="32%" src="./misc/logo.png">

# ROC - Robot Operations Command

ROC is a ROS 2 command-line tool implemented in Rust. It implements many ROS 2 workflows directly and uses command naming close to the `ros2` CLI.

The project is usable today, but it is not a blanket replacement for the full `ros2` CLI or `colcon` on every machine. The strongest validation in this repository is currently scoped to Linux with ROS 2 Jazzy.

For detailed command-by-command implementation status, see `COMPAT.md`.
For workspace-build validation results against `colcon`, see `COMPAT_VALIDATION.md`.

## Installation

### From crates.io

```bash
cargo install rocc
```

This only installs the `roc` binary. ROS-dependent commands still require a working ROS 2 installation in the environment where you run them.

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

## Environment prerequisites

For ROS-aware commands, the practical baseline is:

- a sourced ROS 2 environment, for example `source /opt/ros/jazzy/setup.bash`
- `ros2` available on `PATH` for delegated commands
- standard ROS runtime libraries available to the `roc` binary

For source builds and validation:

- Rust toolchain
- clang/libclang for bindgen-based builds
- Python 3 for `ament_python` package workflows

If the ROS environment is not sourced, command discovery and delegated subcommands will fail early.

## Workspace commands

`roc work` includes package and workspace utilities:

- `roc work create` - scaffold packages (`ament_cmake`, `ament_python`, `cmake`)
- `roc work list` - list discovered packages and build state
- `roc work info <package>` - print package metadata
- `roc work build` - build workspace packages with dependency ordering and colcon-like setup generation

Current validation status for `roc work build`:

- validated against a minimal `ament_cmake` workspace: build and `ros2 pkg prefix` worked
- validated against a minimal `ament_python` workspace: build and Python import worked, but package registration is still incomplete

The strongest direct parity checks live in ignored integration tests and local validation notes; they are not part of the default `cargo test` gate yet.

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

Bag command caveats:

- `roc bag record` and `roc bag play` use serialized transport through the forked `rclrs` submodule in this repository.
- these commands are intended for MCAP-based workflows and are not presented as feature-complete replacements for all `ros2 bag` behavior
- validation is currently strongest for compile-time coverage and focused unit tests; end-to-end ROS bag validation still depends on local ROS setup

Refer to:

- `COMPAT.md` for feature status
- `COMPAT_VALIDATION.md` for direct `colcon` vs `roc` build results
- `FEATURES.md` for high-level feature notes
- `book/` for extended project documentation
- `DEVELOPMENT.md` for the local developer workflow and troubleshooting notes

## Development

Default build/test loop:

```bash
cargo build --release
cargo test
```

Run the heavier workspace parity validators explicitly:

```bash
cargo test --test real_workspace_validation -- --ignored
```

Those ignored tests assume a sourced ROS 2 environment, `colcon`, and local upstream workspaces described in [COMPAT_VALIDATION.md](/home/bresilla/data/code/tools/roc/COMPAT_VALIDATION.md).

## License

Licensed under MIT. See `LICENSE.md`.
