# Release Gate

This document defines the explicit release gate for practical `colcon` workflow parity claims on the validated Linux/Jazzy scope.

## Claim Policy

Do not describe `roc` as a practical `colcon` replacement unless the latest release-gate run is green and the result has been reflected in [COMPAT_VALIDATION.md](/doc/code/tools/roc/COMPAT_VALIDATION.md).

This gate covers:

- `roc work build`
- `roc work test`
- `roc work test-result`
- shell completion sanity for the newer workflow verbs

## Prerequisites

Required commands:

- `cargo`
- `colcon`

Required local paths:

- `/opt/ros/jazzy/setup.bash`
- `/tmp/roc_ros2_examples` by default
- `/tmp/roc_ros2_demos` by default

Workspace overrides:

- `ROC_VALIDATION_ROS2_EXAMPLES=/path/to/ros2/examples`
- `ROC_VALIDATION_ROS2_DEMOS=/path/to/ros2/demos`

The ignored real-workspace validators in [tests/real_workspace_validation.rs](/doc/code/tools/roc/tests/real_workspace_validation.rs) honor those two environment variables.

## One Command

Run the full release gate with:

```bash
make release-gate
```

That executes [scripts/release_gate.sh](/doc/code/tools/roc/scripts/release_gate.sh), which runs:

```bash
cargo test
ROC_VALIDATION_ROS2_EXAMPLES=... ROC_VALIDATION_ROS2_DEMOS=... \
  cargo test --test real_workspace_validation -- --ignored --nocapture
```

The repository also includes a matching GitHub Actions workflow in
[.github/workflows/parity.yaml](/doc/code/tools/roc/.github/workflows/parity.yaml),
which clones the same upstream validation workspaces and runs the same gate on Ubuntu 24.04 with ROS 2 Jazzy.

## Checklist

1. `make release-gate` passes on the target Linux/Jazzy environment.
2. [COMPAT_VALIDATION.md](/doc/code/tools/roc/COMPAT_VALIDATION.md) is updated with the current validation date and any material result changes.
3. Public docs remain scoped to the validated environment and do not overstate coverage.
4. If parity-sensitive behavior changed, re-run the gate before release even if CI/unit tests are already green.

## What The Gate Covers

`cargo test` covers:

- workspace build/test/test-result unit and integration coverage
- shell completion integration coverage
- help and output integration coverage

The ignored real-workspace validation suite covers:

- minimal `ament_cmake` and `ament_python` parity checks
- merged install and overlay chaining
- failed-build resume behavior
- real upstream `ros2/examples` test execution comparison
- real upstream `ros2/demos` test-result comparison

## If The Gate Fails

- Fix the parity regression first.
- Re-run `make release-gate`.
- Only then update [COMPAT_VALIDATION.md](/doc/code/tools/roc/COMPAT_VALIDATION.md) and release-facing docs.
