# ROC - Feature Status

This document tracks ROC feature status at a high level.

Source of truth for command-by-command compatibility:

- `COMPAT.md`

## Legend

- ✅ Native: implemented in Rust (no `ros2 ...` subprocess)
- 🔄 Wrapper: delegates to `ros2 ...` for execution
- 🧱 Stub/WIP: placeholder behavior

## Core Commands Overview

| Command | Status | Notes |
|---------|--------|-------|
| `roc topic` | ✅ | Native graph + dynamic message tooling |
| `roc service` | ✅/🔄 | Native discovery; `call` currently delegates to `ros2` |
| `roc param` | ✅ | Native parameter service client |
| `roc node` | ✅ | Native graph introspection |
| `roc action` | ✅/🔄 | Native discovery; `goal` currently delegates to `ros2` |
| `roc interface` | ✅ | Native interface scanning + parsing |
| `roc frame` | ✅ | Native TF graph + query + static publish |
| `roc bag` | ✅ | Native MCAP record/play + rosbag2 metadata parsing |
| `roc run` | ✅ | Native executable discovery + execution |
| `roc launch` | 🔄 | Launch file discovery + `ros2 launch` execution |
| `roc work` | ✅ | Native workspace management + build system |
| `roc idl` | ✅ | Native IDL/protobuf tooling |
| `roc completion` | ✅ | Native completion generation |
| `roc daemon` | 🧱 | Placeholder |
| `roc middleware` | 🧱 | Placeholder |

## Notable Implementations

- Dynamic topic tools: `echo`, `pub`, `hz`, `bw`, `delay` are implemented natively using `rclrs` dynamic subscriptions/publishers.
- TF tooling: subscribes to `/tf` and `/tf_static`, builds a TF graph, resolves multi-hop transforms.
- Bag tooling: records serialized CDR bytes into MCAP; plays MCAP back to ROS topics.
- Workspace tooling: package discovery, dependency resolution, and a native colcon-like builder. Minimal `ament_cmake` validation is in place; `ament_python` still has known compatibility gaps.

## Near-Term Priorities

- Replace remaining `ros2` wrappers (`roc service call`, `roc action goal`, and potentially `roc launch`).
- Improve lifecycle control for long-running commands (graceful ctrl-c, flush/finalize outputs).
- Implement real dynamic completions using native graph queries (topics/services/nodes/params).
