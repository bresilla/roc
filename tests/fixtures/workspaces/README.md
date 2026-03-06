# Workspace Fixtures

These fixture workspaces provide stable compatibility cases for `roc work build`.

## Fixtures

- `ament_cmake_minimal`
  - one minimal `ament_cmake` package
  - used for discovery and manifest parsing checks

- `ament_python_minimal`
  - one minimal `ament_python` package
  - used for discovery and Python package layout checks

- `dependency_chain`
  - two packages with a simple dependency edge
  - used for topological ordering checks

- `merged_install_layout`
  - a synthetic merged install tree
  - used for install-layout invariant checks

- `overlay_layout`
  - a synthetic underlay + overlay workspace pair
  - used for overlay layout and future setup-script checks
