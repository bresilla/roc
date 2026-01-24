# Command Compatibility Matrix

Legend:
- **Native Rust**: implemented in Rust (rclrs / direct graph APIs / filesystem), no `ros2 ...` subprocess.
- **ROS 2 CLI**: implemented by spawning `ros2 ...` (or `bash -c "ros2 ..."`).
- **Hybrid**: mostly native, but may fall back to `ros2 ...` for some cases.
- **Stub/WIP**: command exists but is not fully implemented and/or not currently wired in CLI args.

| Command | Status | Notes / Implementation |
|---|---|---|
| `roc topic list` | Native Rust | `src/commands/topic/list.rs` (graph discovery) |
| `roc topic info` | Native Rust | `src/commands/topic/info.rs` |
| `roc topic kind` (`type`) | Native Rust | `src/commands/topic/kind.rs` |
| `roc topic find` | Native Rust | `src/commands/topic/find.rs` |
| `roc topic echo` | Native Rust | Native dynamic subscription + YAML/CSV formatting; field selectors support dotted paths (e.g. `--field data`); `src/commands/topic/echo.rs` |
| `roc topic hz` | Native Rust | `src/commands/topic/hz.rs` |
| `roc topic bw` | Native Rust | `src/commands/topic/bw.rs` (uses string-length as size estimate) |
| `roc topic pub` | Native Rust | `src/commands/topic/pub_.rs` (dynamic publisher) |
| `roc topic delay` | Native Rust | Dynamic subscribe + buffer + dynamic publish after delay; `src/commands/topic/delay.rs` |
| `roc node list` | Native Rust | Graph discovery via `rclrs`; `src/commands/node/list.rs` |
| `roc node info` | Native Rust | Uses `rclrs` per-node graph queries; `src/commands/node/info.rs` |
| `roc service list` | Native Rust | Graph discovery via `rclrs`; `src/commands/service/list.rs` |
| `roc service find` | Native Rust | Graph discovery via `rclrs`; `src/commands/service/find.rs` |
| `roc service kind` (`type`) | Native Rust | Graph discovery via `rclrs`; `src/commands/service/kind.rs` |
| `roc service call` | ROS 2 CLI | `src/commands/service/call.rs` |
| `roc param list` | Native Rust | Calls parameter services (`/list_parameters`, etc.) using `rclrs::vendor::rcl_interfaces`; `src/commands/param/list.rs` |
| `roc param get` | Native Rust | Calls `/get_parameters`; `src/commands/param/get.rs` |
| `roc param set` | Native Rust | Calls `/set_parameters`; `src/commands/param/set.rs` |
| `roc param remove` (`delete`) | Native Rust | Removes by setting value type to `PARAMETER_NOT_SET` via `/set_parameters`; `src/commands/param/remove.rs` |
| `roc param describe` | Native Rust | Calls `/describe_parameters`; `src/commands/param/describe.rs` |
| `roc param export` (`dump`) | Native Rust | Exports a ROS2-style YAML (approx) by listing + getting all params; `src/commands/param/export.rs` |
| `roc param import` (`load`) | Native Rust | Loads ROS2-style YAML (approx) and sets params via `/set_parameters`; `src/commands/param/import.rs` |
| `roc action list` | Native Rust | Action discovery via service graph scan; `src/commands/action/list.rs`, `src/shared/action_operations.rs` |
| `roc action info` | Native Rust | Best-effort type inference via `*_SendGoal` service type; `src/commands/action/info.rs`, `src/shared/action_operations.rs` |
| `roc action goal` (`send_goal`) | ROS 2 CLI | `src/commands/action/goal.rs` |
| `roc interface list` | ROS 2 CLI | `src/commands/interface/list.rs` |
| `roc interface package` | ROS 2 CLI | `src/commands/interface/package.rs` |
| `roc interface all` (`packages`) | ROS 2 CLI | `src/commands/interface/all.rs` |
| `roc interface show` | ROS 2 CLI | `src/commands/interface/show.rs` |
| `roc interface model` (`proto`) | ROS 2 CLI | `src/commands/interface/model.rs` |
| `roc frame list` | ROS 2 CLI | wraps `ros2 run tf2_tools view_frames`; `src/commands/frame/list.rs` |
| `roc frame echo` | ROS 2 CLI | wraps `ros2 run tf2_ros tf2_echo`; `src/commands/frame/echo.rs` |
| `roc frame info` | ROS 2 CLI | wraps `ros2 run tf2_ros buffer_client`; `src/commands/frame/info.rs` |
| `roc frame pub` | ROS 2 CLI | wraps `ros2 run tf2_ros static_transform_publisher`; `src/commands/frame/pub_.rs` |
| `roc run <pkg> <exe>` | Native Rust | Finds executable in workspace/install and runs it directly; `src/commands/run/mod.rs` |
| `roc launch <pkg> <launch>` | ROS 2 CLI | always executes `ros2 launch ...`; `src/commands/launch/mod.rs` |
| `roc work create` | Native Rust | generates package skeletons; `src/commands/work/create/command.rs` |
| `roc work list` | Native Rust | workspace package discovery; `src/commands/work/list/command.rs` |
| `roc work info` | Native Rust | package.xml parsing + status; `src/commands/work/info/command.rs` |
| `roc work build` | Native Rust | colcon-replacement build system; `src/commands/work/build/*` |
| `roc idl protobuf` (`proto`, `pb`) | Native Rust | bidirectional `.proto` ↔ `.msg`; `src/commands/idl/protobuf.rs` |
| `roc idl ros2msg` (`msg`, `ros2`) | Native Rust | `.msg` → `.proto`; `src/commands/idl/ros2msg.rs` |
| `roc bag record` | ROS 2 CLI | wraps `ros2 bag record`; `src/commands/bag/record.rs` |
| `roc bag list` | Stub/WIP | prints placeholder; `src/commands/bag/list.rs` |
| `roc bag info` | Stub/WIP | prints placeholder; `src/commands/bag/info.rs` |
| `roc bag play` | Stub/WIP | prints placeholder; `src/commands/bag/play.rs` |
| `roc daemon start` | Stub/WIP | prints placeholder; `src/commands/daemon/start.rs` |
| `roc daemon stop` | Stub/WIP | prints placeholder; `src/commands/daemon/stop.rs` |
| `roc daemon status` | Stub/WIP | prints placeholder; `src/commands/daemon/status.rs` |
| `roc middleware list` | Stub/WIP | prints placeholder; `src/commands/middleware/list.rs` |
| `roc middleware set` | Stub/WIP | prints placeholder; `src/commands/middleware/set.rs` |
| `roc middleware get` | Stub/WIP | prints placeholder; `src/commands/middleware/get.rs` |
