# Colcon Compatibility Validation

Last validated: March 7, 2026

This document records direct `colcon` vs `roc` comparisons for both workspace build and test flows.

Validation environment:

- host: local development machine
- ROS distribution: Jazzy Jalisco
- `colcon`: `/usr/bin/colcon`
- `ros2`: `/opt/ros/jazzy/bin/ros2`
- `roc`: local debug binary from this repository

Repeatable validators:

- [tests/real_workspace_validation.rs](/doc/code/tools/roc/tests/real_workspace_validation.rs)
- [tests/completion_integration.rs](/doc/code/tools/roc/tests/completion_integration.rs)

## Build Validation

Validated against ignored end-to-end comparisons:

- isolated `ament_cmake`
- isolated `ament_python`
- merged install
- underlay/overlay chaining
- failed-build resume with `--continue-on-error` and `--packages-select-build-failed`

Observed result:

- `roc work build` now matches the validated `colcon build` behaviors closely enough for normal ROS package discovery and sourcing flows
- `ros2 pkg prefix` works in the validated minimal `ament_cmake` and `ament_python` cases
- merged-install metadata and overlay sourcing behave the same in the validated fixture matrix

Real upstream workspace pressure tests previously run in `/tmp`:

- `/tmp/roc_ros2_examples`
  - full `ros2/examples` build succeeded with `roc`
- `/tmp/roc_demos_ws`
  - `pendulum_msgs` + `pendulum_control` succeeded with both `colcon` and `roc`
- `/tmp/roc_ros2_demos`
  - failing packages matched `colcon` failures in the local environment rather than exposing builder-specific breakage

## Test Validation

Validated `roc` functionality:

- `roc work test`
- `roc work test-result`

Current test-flow behavior:

- `roc work test` runs `ctest` for CMake packages and `python3 -m pytest` for Python packages
- per-package logs, `status.txt`, `test_summary.log`, and `colcon_test.rc` are written into the build/log trees
- `roc work test-result` reads:
  - `Testing/.../Test.xml`
  - `pytest.xml`
  - package xUnit files under `test_results/...`
  - `colcon_test.rc` as a fallback when no XML results exist
- verbose output now includes testcase-level failure blocks similar to `colcon test-result --verbose`
- delete semantics now use `--delete` and `--delete-yes`, matching `colcon`’s CLI surface

Real upstream workspace validation now exists as ignored integration tests for:

- `/tmp/roc_ros2_examples`
  - compare `colcon build/test` vs `roc work build/test` on selected example packages
- `/tmp/roc_ros2_demos`
  - compare `colcon build/test/test-result` vs `roc work build/test/test-result` on selected demo packages

Observed result from the current direct checks:

- `roc work test` fails and succeeds on the same selected upstream packages that `colcon test` does in this environment
- `roc work test-result --all --verbose` now matches `colcon test-result` much more closely on:
  - result discovery
  - aggregate totals
  - testcase-level failure detail presence

## Current Conclusion

`roc` now has high practical parity with `colcon` for the validated Linux/Jazzy scope across:

- `work build`
- `work test`
- `work test-result`

This is still not an unconditional blanket replacement claim.

What remains weaker than the build parity story:

- the newest `work test` and `work test-result` real-workspace validations are present as ignored tests, but they have not yet been promoted into the same routinely-run release gate as the build matrix
- some output formatting still differs from `colcon`, even where the underlying results match

## Remaining Work Before A Stronger Claim

1. Turn the current ignored real-workspace test-flow checks into part of an explicit release gate.
2. Re-run the full build+test validation matrix after any parity-sensitive change.
3. Keep docs scoped to the validated Linux/Jazzy environment unless wider validation is added.
