# ROC Colcon Parity Plan

## Goal

Make `roc` a high-confidence replacement for the validated `colcon` workflow on Linux/Jazzy:

- `roc work build`
- `roc work test`
- `roc work test-result`

This does not replace `ament`. It replaces the workspace orchestration around standard `ament_cmake` and `ament_python` packages.

## Current State

Already implemented and validated to varying depth:

- build discovery, ordering, install layout, setup scripts, hooks, selectors, logs, and resume behavior
- `work test` package execution and test log/result artifact generation
- `work test-result` result discovery, aggregate summaries, verbose testcase failures, and delete semantics
- shell completions and completion integration coverage for the newer test commands
- ignored real-workspace validation for build flows
- ignored real-workspace validation for selected test and test-result flows

Primary validation reference:

- [COMPAT_VALIDATION.md](/doc/code/tools/roc/COMPAT_VALIDATION.md)

Executable validation reference:

- [tests/real_workspace_validation.rs](/doc/code/tools/roc/tests/real_workspace_validation.rs)

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
- Slice 20: real workspace build parity matrix
- Slice 21: native `work test`
- Slice 22: test execution defaults and result artifacts
- Slice 23: native `work test-result`
- Slice 24: `test-result` parser and verbose output parity
- Slice 25: `test-result` delete flag parity
- Slice 26: real workspace test/test-result validation

## Remaining Slices

### Slice 27: Release Gate

Objective:

- define a hard gate before claiming full practical `colcon` parity

Tasks:

- create one explicit checklist covering:
  - build parity matrix
  - test parity matrix
  - test-result parity matrix
  - shell completion sanity for the newer verbs
  - required ignored real-workspace validations
- document exactly which ignored validations must be run before release
- keep product docs conservative until that gate is green

Definition of done:

- parity is backed by an explicit release process instead of scattered notes and ignored tests

Suggested commit title:

- `Add release gate for validated colcon workflow parity`

## Recommended Next Step

Do Slice 27.

Reason:

- the implementation is already broad
- the remaining weakness is confidence and repeatability, not an obvious missing core feature
- the project now needs a release-quality parity gate more than another ad hoc capability
