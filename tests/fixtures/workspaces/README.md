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

- `ament_python_missing_resource`
  - one `ament_python` package missing `resource/<pkg>`
  - used for unsupported Python resource-marker coverage

- `ament_cmake_missing_cmakelists`
  - one `ament_cmake` package missing `CMakeLists.txt`
  - used for unsupported CMake layout coverage

- `ambiguous_inferred_build`
  - one package with both CMake and Python markers but no declared build type
  - used for ambiguous inferred build-type coverage

- `unknown_inferred_build`
  - one package with no declared build type and no build markers
  - used for unknown inferred build-type coverage

- `unsupported_build_type`
  - one package with an explicit non-supported build type
  - used for unsupported build-type discovery coverage

- `ignored_packages`
  - one normal package and packages hidden by `COLCON_IGNORE` and `AMENT_IGNORE`
  - used for ignore-marker discovery coverage

- `duplicate_name_collision`
  - one source package and one installed package copy with the same package name
  - used for source-vs-install duplicate resolution coverage

- `dependency_chain`
  - two packages with a simple dependency edge
  - used for topological ordering checks

- `merged_install_layout`
  - a synthetic merged install tree
  - used for install-layout invariant checks

- `overlay_layout`
  - a synthetic underlay + overlay workspace pair
  - used for overlay layout and future setup-script checks
