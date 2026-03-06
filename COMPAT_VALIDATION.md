# Build Compatibility Validation

Last validated: March 6, 2026

This document records direct `colcon` vs `roc work build` comparisons run in this repository against representative ament workspaces.

Validation environment:

- host: local development machine
- ROS distribution: Jazzy Jalisco
- `colcon`: `/usr/bin/colcon`
- `ros2`: `/opt/ros/jazzy/bin/ros2`
- `roc`: local debug binary from this repository

Repeatable validator:

- [tests/real_workspace_validation.rs](/doc/code/tools/roc/tests/real_workspace_validation.rs)

The ignored integration test copies the fixture workspace to a temp directory, builds it once with `colcon`, builds it again with `roc`, and validates the resulting runtime behavior.

## Cases

### `ament_cmake_minimal`

Commands executed:

- `source /opt/ros/jazzy/setup.bash && colcon build --base-paths src`
- `source /opt/ros/jazzy/setup.bash && roc work build --base-paths src`
- `source install/setup.bash && ros2 pkg prefix demo_cmake_pkg`

Observed result:

- `colcon build` succeeded
- `roc work build` succeeded
- `ros2 pkg prefix demo_cmake_pkg` succeeded for both installs
- `AMENT_PREFIX_PATH` and `CMAKE_PREFIX_PATH` were equivalent at the package-prefix level after sourcing

Observed tree deltas:

- `colcon` generated `.ps1` wrappers and `_local_setup_util_*.py`
- `colcon` wrote package metadata below `install/<pkg>/share/colcon-core/packages/<pkg>`
- `roc` currently writes workspace-level metadata below `install/share/colcon-core/packages/<pkg>`
- `roc` left `COLCON_PREFIX_PATH` with a trailing `:`

Assessment:

- usable for the validated `ament_cmake` case
- not yet byte-for-byte compatible with `colcon`

### `ament_python_minimal`

Commands executed:

- `source /opt/ros/jazzy/setup.bash && colcon build --base-paths src`
- `source /opt/ros/jazzy/setup.bash && roc work build --base-paths src`
- `source install/setup.bash && python3 -c "import demo_python_pkg"`
- `source install/setup.bash && ros2 pkg prefix demo_python_pkg`

Observed result:

- `colcon build` succeeded
- `roc work build` succeeded
- importing `demo_python_pkg` succeeded for both installs
- `ros2 pkg prefix demo_python_pkg` succeeded for both installs

Observed tree deltas:

- `colcon` and `roc` now both install Python payloads under `install/<pkg>/lib/python3.12/site-packages`
- `colcon` and `roc` now both install the package marker and `package.xml` under `install/<pkg>/share/...`
- `colcon` generated package hook files like `ament_prefix_path.*`, `pythonpath.*`, and `package.dsv`
- `roc` generated shell setup wrappers, but not the same package hook set

Assessment:

- runtime Python import works
- ROS package discovery now works for the validated minimal case
- remaining differences are concentrated in hook generation and metadata fidelity

## Current Conclusion

`roc work build` is now close enough to substitute `colcon build` for the validated minimal `ament_cmake` case and the validated minimal `ament_python` case.

It is still not full parity, because helper-script generation, hook fidelity, metadata placement, and shell-family coverage still differ from `colcon`.

## Next Fixes Suggested By Validation

1. Generate the missing `ament_prefix_path`, `pythonpath`, and `package.dsv` hook set for Python packages.
2. Move or mirror `share/colcon-core/packages/<pkg>` to the package-prefix layout that `colcon` uses.
3. Trim the trailing separator from generated `COLCON_PREFIX_PATH`.
4. Add `_local_setup_util_*.py` and `.ps1` parity where `colcon` emits them.
