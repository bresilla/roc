# ROC Full Colcon Parity Plan

## Goal

Achieve full practical parity for `roc work build` with `colcon build` for standard ROS 2 workspaces on Linux first.

“Full parity” here means:

- the same workspace inputs produce install trees close enough that standard ROS 2 tools behave the same
- standard `ament_cmake` and `ament_python` packages work without package-specific hacks
- sourcing `install/setup.*` yields equivalent downstream discovery behavior
- common `colcon` package-selection and resume workflows behave the same

This plan still does **not** replace `ament` itself. It replaces the workspace orchestration, environment setup, and install metadata behavior normally provided by `colcon`.

## Upstream Parity References

This plan is anchored to upstream `colcon` and ROS 2 behavior, not local guesses.

Primary references:

- `colcon build` and package selection arguments:
  - https://colcon.readthedocs.io/en/released/reference/package-selection-arguments.html
  - https://colcon.readthedocs.io/en/main/reference/verb/build.html
- `colcon` environment setup model:
  - https://colcon.readthedocs.io/en/released/developer/environment.html
  - https://colcon.readthedocs.io/en/released/user/isolated-vs-merged-workspaces.html
- ROS 2 / ament package expectations:
  - https://docs.ros.org/en/rolling/How-To-Guides/Ament-CMake-Documentation.html
  - https://docs.ros.org/en/rolling/How-To-Guides/Ament-CMake-Python-Documentation.html

Important consequences from those docs:

- package-selection arguments combine with logical `AND`
- `--packages-ignore` / `--packages-skip` semantics must match `colcon`
- setup generation is driven by package metadata, package hooks, and helper scripts like `_local_setup_util_sh.py`
- standard ROS Python packages must register through the ament resource index and install `package.xml` in the expected `share/...` location
- isolated and merged installs are both first-class behaviors, not approximations

## Hard Success Criteria

`roc work build` should not be called parity-complete until all of the following are true:

- `ros2 pkg prefix <pkg>` works after `roc` builds for both minimal `ament_cmake` and minimal `ament_python`
- package imports, CMake downstream discovery, and overlay sourcing behave the same as `colcon`
- install layout matches `colcon` closely enough that only non-functional differences remain
- package hooks and `.dsv` processing match `colcon` ordering and effect
- workspace setup scripts produce the same practical environment variables without malformed separators
- package selectors and resume selectors behave the same as `colcon`
- validation passes against representative real workspaces, not just synthetic fixtures

## Current State

Already implemented:

- fixture workspaces and compatibility tests
- configurable `build`, `install`, and `log` bases
- richer dependency parsing
- package setup and workspace setup script generation
- artifact scanning and `.dsv` support
- package logs and machine-readable build state
- some build-state selectors
- direct validation against real minimal `ament_cmake` and `ament_python` workspaces

Known remaining parity gaps from direct validation:

- release gating still needs to be made explicit before claiming full parity

## Non-Negotiable Constraints

- keep `ament_cmake` and `ament_python` packages unchanged
- prefer exact `colcon` behavior over internal elegance
- treat real downstream behavior as the source of truth
- validate Linux first before widening platform claims

## Delivery Rule

Every completed feature slice must be committed separately.

Commit rule:

- one git commit per completed slice
- commit message title only
- no commit body

## Completed Slices

- Slice 0: baseline and compatibility fixtures
- Slice 1: configurable build/install/log base paths
- Slice 2: richer package metadata model
- Slice 3: colcon package metadata generation
- Slice 4: package-level setup scripts
- Slice 5: workspace setup chaining
- Slice 6: installed artifact scanning
- Slice 7: environment hooks and `.dsv`
- Slice 8: `--symlink-install`
- Slice 9: build execution fidelity
- Slice 10: logging and build state
- Slice 11: package selection and resume semantics
- Slice 12: validation against real ROS 2 packages
- Slice 13: documentation cleanup
- Slice 14: `ament_python` install layout
- Slice 15: Python package registration and resource index layout
- Slice 16: full hook set and helper scripts
- Slice 17: package metadata placement and prefix chaining edge cases
- Slice 18: selector and build-state parity
- Slice 19: PowerShell and install-layout metadata parity
- Slice 20: real workspace parity matrix

Those slices got the implementation close. They do **not** finish full parity.

## Remaining Slices For Full Parity

### Slice 14: Fix `ament_python` Install Layout

Objective:

- make isolated Python package installs match `colcon` and ament expectations

Tasks:

- install Python payloads under `install/<pkg>/lib/pythonX.Y/site-packages`
- stop routing isolated Python payloads through `local/lib/.../dist-packages`
- confirm merged-install behavior also matches `colcon`
- validate with direct tree comparisons against `colcon`

Definition of done:

- minimal `ament_python` install tree matches `colcon` at the package payload level
- sourced `PYTHONPATH` points to the expected location

Suggested commit title:

- `Align ament python install layout with colcon`

### Slice 15: Fix Python Package Registration and Resource Index

Objective:

- make Python packages discoverable through standard ROS 2 mechanisms after a `roc` build

Tasks:

- install the ament resource marker in the same place `colcon` does
- install `package.xml` under the expected `share/<pkg>/package.xml` location
- validate `ros2 pkg prefix <pkg>` for isolated and merged installs
- compare ament index layout directly against `colcon`

Definition of done:

- `ros2 pkg prefix demo_python_pkg` works after `roc work build`
- ament index layout for the validated Python fixture matches `colcon` closely

Suggested commit title:

- `Register ament python packages with standard ROS discovery`

### Slice 16: Generate Full Hook Set and Helper Scripts

Objective:

- match `colcon` setup behavior more exactly instead of relying on shell-only approximations

Tasks:

- generate or reuse the expected hook set:
  - `package.dsv`
  - `ament_prefix_path.*`
  - `pythonpath.*`
  - other standard package hook files where applicable
- add `_local_setup_util_sh.py`
- add `_local_setup_util_ps1.py`
- ensure package and workspace setup use the same chaining model as `colcon`

Definition of done:

- validated fixtures contain the expected helper files and hook family
- setup script behavior matches `colcon` more closely when diffed and sourced

Suggested commit title:

- `Generate colcon helper scripts and standard package hooks`

### Slice 21: Release Gate

Objective:

- define a hard stop before calling `roc work build` a full `colcon` replacement

Tasks:

- create a checklist that must pass before making the claim:
  - `ament_cmake` parity
  - `ament_python` parity
  - selector parity
  - setup-script parity
  - real workspace validation
- keep the docs conservative until all items are green
- only then update docs to say full replacement

Definition of done:

- the project has an explicit parity gate
- “full parity” is a tested claim, not a goal statement

Suggested commit title:

- `Add release gate for full colcon parity`

## Execution Order

Recommended order from here:

1. Slice 14
2. Slice 15
3. Slice 16
4. Slice 17
5. Slice 18
6. Slice 19
7. Slice 20
8. Slice 21

## Rules For Each Remaining Slice

Before coding:

- identify the exact upstream `colcon` behavior being matched
- define the expected install-tree and runtime behavior
- add a direct comparison case if one does not already exist

During implementation:

- keep each slice narrow
- use `colcon` output as the baseline for parity-sensitive files
- do not accept “close enough” if downstream behavior still differs

Before committing:

- run targeted tests first
- run the ignored real-workspace validator when the slice affects parity-sensitive behavior
- update `COMPAT_VALIDATION.md` whenever observed behavior changes

Commit:

- create exactly one commit for the completed slice
- commit title only
- no body

## Immediate Next Step

Start with Slice 14.

Reason:

- the remaining blocker to a full replacement claim is still `ament_python`
- direct validation already shows the first hard failure: `ros2 pkg prefix` breaks after a `roc` Python build
- until Python install layout and registration match `colcon`, the parity claim is false
