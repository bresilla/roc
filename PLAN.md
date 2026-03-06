# ROC Colcon Replacement Plan

## Goal

Turn `roc work build` into a practical replacement for `colcon build` for standard ROS 2 workspaces while preserving compatibility with existing `ament_cmake`, `ament_python`, and plain `cmake` packages.

This plan does **not** attempt to replace `ament` itself. It replaces the workspace orchestrator layer currently provided by `colcon`.

## Non-Goals

- Replacing `ament_cmake` macros or `ament_python` package conventions
- Supporting every historical `colcon` extension from day one
- Supporting ROS 1 / catkin in the first implementation
- Rewriting package build systems that already work through CMake or setuptools

## Success Criteria

`roc work build` should be considered a viable `colcon build` replacement when:

- standard `ament_cmake` packages build and install correctly
- standard `ament_python` packages build and install correctly
- isolated and merged install modes behave correctly
- workspace setup scripts can be sourced and used as expected
- downstream packages can discover upstream packages through normal ROS 2 mechanisms
- overlays behave correctly enough for common developer workflows
- compatibility is validated against fixture workspaces and real packages

## Current State Summary

The current implementation already has:

- package discovery from `package.xml`
- basic dependency ordering
- CMake and Python build invocation
- parallel execution
- coarse workspace setup generation

The current implementation is still missing or incomplete in the following critical areas:

- colcon-style package metadata output
- package-level setup scripts
- workspace `local_setup.*` / `setup.*` chaining
- `.dsv` and environment hook behavior
- accurate dependency semantics for runtime/setup ordering
- real `--symlink-install`
- colcon-compatible logging and state layout
- compatibility tests against actual ament package behavior

## Constraints

- Keep `ament_cmake` and `ament_python` packages unchanged
- Prefer matching `colcon` output layout and sourcing behavior rather than inventing a new one
- Use installed artifacts and package hooks as the source of truth
- Preserve cross-platform structure where practical, but prioritize Linux first

## Delivery Rule

Every completed feature slice must be committed separately.

Commit rule:

- one git commit per completed slice
- commit message title only
- no commit body

Examples:

- `Implement package metadata generation for workspace installs`
- `Add package-level local_setup and package scripts`
- `Support merged and isolated install environment chaining`

## Work Slices

### Slice 0: Baseline and Compatibility Fixtures

Objective:

- establish a repeatable way to compare `roc work build` against `colcon build`

Tasks:

- create fixture workspaces for:
  - minimal `ament_cmake` package
  - minimal `ament_python` package
  - two-package dependency chain
  - merged-install workspace
  - overlay workspace
- add test helpers that inspect install trees, setup scripts, and package metadata
- document expected outputs for each fixture

Definition of done:

- fixture workspaces exist in-repo
- tests can assert install-tree invariants
- tests clearly show current gaps

Suggested commit title:

- `Add build compatibility fixtures for ament workspaces`

### Slice 1: Build Output Layout and Base Paths

Objective:

- make workspace path handling match `colcon` more closely

Tasks:

- add explicit support for:
  - `--build-base`
  - `--install-base`
  - `--log-base`
- ensure `build/`, `install/`, and `log/` layout matches expected conventions
- create ignore markers where appropriate to prevent recursive rediscovery
- stop hardcoding path assumptions in command setup

Definition of done:

- build, install, and log base directories are configurable
- current tests pass with non-default bases
- workspace scanning does not rediscover generated trees

Suggested commit title:

- `Support configurable build install and log base paths`

### Slice 2: Package Metadata Model

Objective:

- separate build-time, runtime, and environment dependencies correctly

Tasks:

- extend `package.xml` parsing to include:
  - `depend`
  - `build_depend`
  - `buildtool_depend`
  - `build_export_depend`
  - `exec_depend`
  - `test_depend`
  - group dependencies where feasible
  - conditional dependencies where feasible
- define explicit dependency sets for:
  - build ordering
  - runtime/setup ordering
  - exported downstream usage
- stop overloading a single dependency list for all phases

Definition of done:

- internal package metadata clearly distinguishes dependency roles
- topological sort uses the right dependency set
- runtime/setup metadata can be emitted from this model

Suggested commit title:

- `Refine package manifest parsing for build and runtime dependencies`

### Slice 3: Colcon Package Metadata Files

Objective:

- emit workspace metadata required for correct setup chaining

Tasks:

- generate `share/colcon-core/packages/<pkg>` files
- write runtime dependency information in dependency order
- ensure metadata is generated for isolated and merged installs
- make metadata generation part of successful package installation

Definition of done:

- installed workspace contains package metadata files for every built package
- runtime dependency chain can be reconstructed from generated metadata

Suggested commit title:

- `Generate colcon package metadata for installed packages`

### Slice 4: Package-Level Setup Scripts

Objective:

- generate package-scoped setup entry points compatible with colcon-style sourcing

Tasks:

- generate package-level scripts such as:
  - `share/<pkg>/package.sh`
  - `share/<pkg>/package.bash`
  - `share/<pkg>/package.zsh`
  - `share/<pkg>/local_setup.sh`
- source package-provided hooks when present
- ensure scripts are generated from package metadata and installed artifacts

Definition of done:

- every built package installs its own setup entry points
- package scripts can be sourced directly and update the environment correctly

Suggested commit title:

- `Add package-level setup and local setup scripts`

### Slice 5: Workspace Setup Pipeline

Objective:

- replace the current coarse environment dump with real workspace sourcing logic

Tasks:

- generate workspace-level:
  - `local_setup.sh`
  - `setup.sh`
  - `setup.bash`
  - `setup.zsh`
- chain package setup in dependency order
- preserve overlay behavior using prefix chaining conventions
- stop exporting a static snapshot of the parent shell as the main environment strategy

Definition of done:

- `source install/setup.bash` works for fixture workspaces
- overlays source underlays correctly in supported workflows
- the old environment-dump model is removed or relegated to a fallback

Suggested commit title:

- `Implement workspace setup chaining for installed packages`

### Slice 6: Installed Artifact Scanning and Environment Heuristics

Objective:

- derive environment changes from installed outputs instead of assumptions

Tasks:

- inspect installed package prefixes for:
  - `bin`
  - `lib`
  - `lib/pkgconfig`
  - Python site-packages
  - CMake package config files
  - resource index markers
- use discovered artifacts to generate environment modifications
- support both isolated and merged installs

Definition of done:

- setup generation is driven by actual installed contents
- downstream package discovery works in fixture workspaces

Suggested commit title:

- `Generate environment entries from installed package artifacts`

### Slice 7: Environment Hooks and DSV Support

Objective:

- support standard package-provided environment customization

Tasks:

- detect and source installed environment hooks
- support `.dsv` processing or an equivalent normalized internal representation
- ensure package hooks are applied in dependency order
- define deterministic precedence when multiple hooks modify the same variables

Definition of done:

- package hooks are honored during workspace sourcing
- common ament hook-based packages behave correctly in fixture tests

Suggested commit title:

- `Support package environment hooks and dsv processing`

### Slice 8: True `--symlink-install`

Objective:

- implement real symlink install behavior instead of a flag placeholder

Tasks:

- for supported package types, install symlinks where colcon would
- preserve expected behavior for Python modules, scripts, and shared resources
- define fallbacks for unsupported files or platforms

Definition of done:

- fixture workspaces validate symlink behavior
- user-facing docs clearly describe platform-specific caveats

Suggested commit title:

- `Implement symlink install mode for workspace builds`

### Slice 9: Build Execution Fidelity

Objective:

- tighten build invocation behavior so more packages succeed without special handling

Tasks:

- review and align CMake invocation flags with common colcon behavior
- support more accurate per-package environment construction
- improve handling for custom CMake targets
- review Python package build/install invocation for current ROS 2 expectations
- capture and expose exact subprocess failures

Definition of done:

- fixtures and representative packages build without manual intervention
- execution behavior is documented and test-covered

Suggested commit title:

- `Align package build execution with colcon behavior`

### Slice 10: Logging and Build State

Objective:

- make the build output debuggable and script-friendly

Tasks:

- write logs under `log/latest/...`
- store per-package stdout/stderr
- emit a clear end-of-build summary
- keep machine-readable state where useful for future selectors and tooling

Definition of done:

- each package has persisted logs
- failures can be diagnosed without rerunning interactively

Suggested commit title:

- `Add colcon-style logging and per-package build output`

### Slice 11: Package Selection Semantics

Objective:

- improve selector behavior until it matches common `colcon` workflows

Tasks:

- review and align semantics for:
  - `--packages-select`
  - `--packages-ignore`
  - `--packages-up-to`
- add support for additional selectors only if justified by user demand
- ensure selectors combine predictably

Definition of done:

- selector behavior is covered by tests
- behavior is documented and stable

Suggested commit title:

- `Align package selection behavior with colcon semantics`

### Slice 12: Compatibility Validation Against Real Workspaces

Objective:

- prove the implementation against real ROS 2 usage, not just synthetic tests

Tasks:

- build one or more representative real ROS 2 workspaces with `roc`
- compare install tree, setup behavior, and downstream discovery against `colcon`
- record gaps and either fix them or explicitly defer them

Definition of done:

- compatibility report exists in-repo
- major blocking incompatibilities are either resolved or documented

Suggested commit title:

- `Validate roc workspace builds against real ROS 2 packages`

### Slice 13: Documentation and Positioning Cleanup

Objective:

- make project claims accurate and keep docs aligned with reality

Tasks:

- update README, book, and compatibility docs
- remove exaggerated “fully functional” claims until verified
- document supported and unsupported behaviors explicitly

Definition of done:

- docs match actual implementation
- users can tell what is safe to rely on

Suggested commit title:

- `Update build documentation for verified colcon replacement support`

## Suggested Execution Order

Recommended order:

1. Slice 0
2. Slice 1
3. Slice 2
4. Slice 3
5. Slice 4
6. Slice 5
7. Slice 6
8. Slice 7
9. Slice 8
10. Slice 9
11. Slice 10
12. Slice 11
13. Slice 12
14. Slice 13

## Rules for Each Slice

Before coding:

- identify the exact fixture or compatibility case the slice should satisfy
- define the install-tree or runtime behavior expected at the end

During implementation:

- keep each slice narrowly scoped
- add or extend tests with the implementation
- avoid mixing unrelated cleanup into the same change

Before committing:

- run targeted tests first
- run the broadest practical validation available in the current environment
- ensure docs reflect any user-visible behavior changes

Commit:

- create exactly one commit for the completed slice
- commit title only
- no body

## Immediate Next Step

Start with Slice 0.

Reason:

- the project currently lacks a strong compatibility harness
- without fixture-based comparison, later “colcon replacement” work will drift or regress
- the next slices depend on being able to verify install trees and setup behavior quickly
