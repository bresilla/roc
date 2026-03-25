# Development

This project depends on a real ROS 2 environment for many commands, and it vendors a forked `rclrs` checkout under `deps/ros2_rust`.

## Local prerequisites

- Rust toolchain
- ROS 2 installed locally
- a sourced ROS environment before running ROS-aware commands
- `colcon` for parity validation
- clang/libclang for bindgen-driven dependency builds
- Python 3 for `ament_python` package creation and testing

Typical Jazzy setup:

```bash
source /opt/ros/jazzy/setup.bash
```

If the binary fails to start with a runtime library error, verify that your ROS environment and system library paths are present before running `roc`.

## Default workflow

Use the repository `Makefile` for the normal loop:

```bash
make build
make test
```

That covers:

- release build of the CLI
- default unit and integration tests
- smoke coverage for command help and shell completions

## Stronger validation

The strongest workspace parity checks are intentionally kept as ignored integration tests because they need a ROS machine plus local upstream workspaces.

Run them explicitly with:

```bash
cargo test --test real_workspace_validation -- --ignored
```

Those validators currently assume:

- `colcon` is available on `PATH`
- a ROS environment is already sourced
- the referenced upstream workspaces exist under `/tmp`, as described in `COMPAT_VALIDATION.md`

## Working with the forked `rclrs`

This repository currently depends on the vendored fork at:

```text
deps/ros2_rust/rclrs
```

When making `rclrs` changes:

1. commit inside `deps/ros2_rust`
2. push the branch from the submodule checkout
3. commit the updated submodule pointer in this repository when needed

Current local branch policy has been to keep serialization work on `serialized_transport`.

## Troubleshooting

Common issues:

- `roc` fails to load `libspdlog.so.1.12`
  - the runtime library path is incomplete for your shell; re-source ROS and confirm the required libs are reachable
- ROS graph commands fail immediately
  - the ROS environment is not sourced, or the delegated `ros2` command is unavailable
- parity tests are skipped
  - this is expected unless you run the ignored validation tests explicitly
- bag record/play compile regressions appear after touching `rclrs`
  - verify the serialized transport APIs still exist in the forked submodule checkout

## Notes for contributors

- keep behavior changes reflected in `README.md`, `COMPAT_VALIDATION.md`, and this document
- when adding a fix or feature, add the test in the same commit if the change is testable
- avoid replacing the forked `rclrs` dependency unless the serialized transport path is preserved
