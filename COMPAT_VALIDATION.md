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
- `ros2 pkg prefix demo_python_pkg` succeeded for `colcon`
- `ros2 pkg prefix demo_python_pkg` failed for `roc`

Observed tree deltas:

- `colcon` installed Python payloads to `install/<pkg>/lib/python3.12/site-packages`
- `roc` installed Python payloads to `install/<pkg>/local/lib/python3.12/dist-packages`
- `colcon` installed the package marker and `package.xml` under `install/<pkg>/share/...`
- `roc` installed those artifacts under `install/<pkg>/local/share/...`
- `colcon` generated package hook files like `ament_prefix_path.*`, `pythonpath.*`, and `package.dsv`
- `roc` generated shell setup wrappers, but not the same package hook set

Assessment:

- runtime Python import works
- package registration is still incomplete for standard ROS discovery
- this remains a blocking incompatibility for claiming full `ament_python` replacement

## Current Conclusion

`roc work build` is now close enough to substitute `colcon build` for the validated minimal `ament_cmake` case.

It is not yet a full `colcon` replacement for `ament_python` packages, because the install layout and marker placement still diverge enough to break `ros2 pkg prefix`.

## Next Fixes Suggested By Validation

1. Make isolated Python installs land under `install/<pkg>/lib/pythonX.Y/site-packages` instead of `local/lib/.../dist-packages`.
2. Install the ament resource marker and `package.xml` under `install/<pkg>/share/...` for isolated Python packages.
3. Generate the missing `ament_prefix_path`, `pythonpath`, and `package.dsv` hook set for Python packages.
4. Move or mirror `share/colcon-core/packages/<pkg>` to the package-prefix layout that `colcon` uses.
5. Trim the trailing separator from generated `COLCON_PREFIX_PATH`.
