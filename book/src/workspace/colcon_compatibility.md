# Colcon Compatibility

`roc work build` is aiming to replace `colcon build`, but it is not yet a blanket drop-in replacement for every ROS 2 package layout.

This chapter reflects the current verified state, not the long-term goal.

## Verified Today

Validation source:

- [COMPAT_VALIDATION.md](/doc/code/tools/roc/COMPAT_VALIDATION.md)
- [tests/real_workspace_validation.rs](/doc/code/tools/roc/tests/real_workspace_validation.rs)

Validated on March 6, 2026 against ROS 2 Jazzy:

- minimal `ament_cmake` workspace
- minimal `ament_python` workspace

## Command-Line Coverage

The following `colcon build` options are implemented in `roc work build`:

| Colcon Command | ROC Equivalent | Status |
|---------------|----------------|--------|
| `colcon build` | `roc work build` | implemented |
| `colcon build --packages-select pkg1 pkg2` | `roc work build --packages-select pkg1 pkg2` | implemented |
| `colcon build --packages-ignore pkg1` | `roc work build --packages-ignore pkg1` | implemented |
| `colcon build --packages-skip pkg1` | `roc work build --packages-skip pkg1` | implemented |
| `colcon build --packages-up-to pkg1` | `roc work build --packages-up-to pkg1` | implemented |
| `colcon build --parallel-workers 4` | `roc work build --parallel-workers 4` | implemented |
| `colcon build --merge-install` | `roc work build --merge-install` | implemented |
| `colcon build --symlink-install` | `roc work build --symlink-install` | implemented, strongest for Python payloads |
| `colcon build --continue-on-error` | `roc work build --continue-on-error` | implemented |
| `colcon build --cmake-args ...` | `roc work build --cmake-args ...` | implemented |

Additional build-state selectors implemented in `roc`:

- `--packages-select-build-failed`
- `--packages-skip-build-finished`

## Compatibility Summary

### `ament_cmake`

What is currently verified:

- build succeeds
- workspace setup scripts are generated
- `ros2 pkg prefix <pkg>` resolves after sourcing `install/setup.bash`
- package metadata and per-package logs are generated

Known differences vs `colcon`:

- PowerShell wrappers are not generated
- helper files like `_local_setup_util_*.py` are not generated
- `share/colcon-core/packages/<pkg>` is currently written at the workspace root instead of only at the package prefix layout used by `colcon`
- generated `COLCON_PREFIX_PATH` still has a trailing separator

### `ament_python`

What is currently verified:

- build succeeds
- generated setup scripts make Python import work after sourcing `install/setup.bash`

Current blocking gaps:

- `ros2 pkg prefix <pkg>` can fail after a `roc` build
- isolated Python installs currently land under `local/lib/.../dist-packages`
- package markers and `package.xml` can land under `local/share/...`
- `ament_prefix_path`, `pythonpath`, and `package.dsv` hook generation is incomplete compared to `colcon`

Because of those gaps, `ament_python` should still be treated as partially compatible.

## Practical Guidance

Use `roc work build` today when:

- the workspace is primarily `ament_cmake`
- you want native package discovery, dependency ordering, and logging
- you are comfortable with a compatibility layer that is still converging on `colcon`

Be cautious when:

- the workspace relies on `ament_python` package registration behavior
- downstream tools expect byte-for-byte `colcon` install trees
- PowerShell setup support is required

## Next Work

The highest-value compatibility fixes from current validation are:

1. install isolated Python artifacts under `lib/pythonX.Y/site-packages`
2. place Python package markers and `package.xml` under `share/...`
3. generate the missing Python hook set (`ament_prefix_path`, `pythonpath`, `package.dsv`)
4. align `share/colcon-core/packages/<pkg>` placement with `colcon`
5. remove the trailing separator from `COLCON_PREFIX_PATH`
