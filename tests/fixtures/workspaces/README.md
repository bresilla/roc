# Workspace Fixtures

These fixture workspaces provide stable compatibility cases for `roc work build`.

## Fixtures

- `ament_cmake_minimal`
  - one minimal `ament_cmake` package
  - used for discovery and manifest parsing checks

- `ament_python_minimal`
  - one minimal `ament_python` package
  - used for discovery and Python package layout checks

- `ament_python_setup_cfg_only`
  - one `ament_python` package that declares `setup.cfg` but omits `setup.py`
  - used for unsupported Python layout coverage

- `unsupported_build_type`
  - one package with an explicit non-supported build type
  - used for unsupported build-type discovery coverage

- `ignored_packages`
  - one normal package and one package hidden by `COLCON_IGNORE`
  - used for ignore-marker discovery coverage

- `dependency_chain`
  - two packages with a simple dependency edge
  - used for topological ordering checks

- `merged_install_layout`
  - a synthetic merged install tree
  - used for install-layout invariant checks

- `overlay_layout`
  - a synthetic underlay + overlay workspace pair
  - used for overlay layout and future setup-script checks
