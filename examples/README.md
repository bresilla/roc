# Example Workspace

This workspace is intentionally larger than the default package templates.
It is meant to exercise `roc work` commands against a realistic mix of:

- reusable C++ libraries
- dependent C++ executables
- multiple ROS nodes in one package
- an `ament_python` package with console entry points and tests
- installed config files and package-to-package dependencies

Packages:

- `demo_math_cpp`
  - `ament_cmake`
  - exports a reusable analytics library
  - installs headers, config, and two executables
  - generates synthetic named signal streams for other packages

- `demo_telemetry_cpp`
  - `ament_cmake`
  - depends on `demo_math_cpp`
  - publishes enriched telemetry reports
  - subscribes to generated summaries and computes health-style rollups

- `demo_python_tools`
  - `ament_python`
  - provides a ROS node, a CLI report tool, and pure-Python helpers
  - includes pytest coverage for scenario loading and report rendering

Suggested workflow:

```bash
cd examples
roc work list
roc work build --merge-install
roc work test
roc work test-result --all
```

If you have a sourced ROS 2 environment, you can inspect the result with:

```bash
source install/setup.bash
ros2 pkg list | grep demo_
ros2 run demo_math_cpp stats_node
ros2 run demo_telemetry_cpp telemetry_monitor_node
ros2 run demo_python_tools metrics_cli --series burst --window 4
```

Useful things to inspect after building:

- `install/demo_math_cpp/share/demo_math_cpp/config/pipeline.yaml`
- `install/demo_telemetry_cpp/share/demo_telemetry_cpp/config/telemetry.yaml`
- `log/latest/`
- generated package metadata under `install/share/colcon-core/packages/`
