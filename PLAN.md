# ROC Hardening Plan

## Goal

Move `roc` from "featureful but uneven" to "predictable, debuggable, and safe to extend".

This plan focuses on:

1. eliminating the highest-risk runtime and orchestration bugs
2. making default build/test workflows trustworthy
3. tightening environment handling and process execution
4. reducing the gap between implemented behavior and documented promises
5. improving validation so regressions are caught earlier

The current codebase already has useful breadth. The problem is not lack of features. The problem is that several important paths rely on optimistic assumptions, partial implementations, or validation that only exists in ignored/infrequently-run tests.

## Current Assessment

### Major strengths

- native ROS graph and dynamic message support is already substantial
- workspace build/test parity work is more advanced than most of the CLI surface
- the codebase is modular enough to harden incrementally
- compile-time health is decent:
  - `cargo check` passes
  - `cargo test --no-run` passes

### Major weaknesses

- default `cargo test` does not run successfully in the current environment
- parallel workspace build logic has correctness risks
- long-running ROS helpers have weak lifecycle control
- subprocess argument/env handling is lossy or heuristic-heavy in key paths
- some commands are still obviously partial or placeholder-grade
- runtime parity confidence depends too heavily on ignored ROS/colcon tests

## Hardening Principles

- Prefer deterministic behavior over clever behavior.
- Prefer explicit unsupported/error states over partial silent behavior.
- Treat environment construction as part of correctness, not convenience.
- Make long-running helpers stoppable and observable.
- Make the default dev loop (`build`, `test`, targeted command checks) work without tribal knowledge.
- Add tests around failures and edge conditions, not just happy paths.

## Priority Order

## Phase 0: Stabilize The Base

Objective:

- stop carrying known runtime hazards while continuing normal feature work

Deliverables:

- one tracked hardening backlog
- clear separation between:
  - correctness bugs
  - reliability issues
  - missing functionality
  - documentation drift

Definition of done:

- this plan exists and is the working backlog for hardening work

## Phase 1: Fix High-Risk Correctness Bugs

These are the first items to implement because they can produce hangs, invalid builds, or broken user execution even when the CLI appears to work.

### Slice 1.1: Parallel Build Deadlock And Failure Propagation [complete]

Problem:

- the parallel build scheduler can leave packages stuck in `Pending` forever if one of their dependencies fails
- workers then keep polling because `all_done` never becomes true

Relevant code:

- [src/commands/work/build/build_executor.rs](/home/bresilla/data/code/tools/roc/src/commands/work/build/build_executor.rs)

Tasks:

- model blocked packages explicitly
  - add a `Blocked` or `SkippedDueToDependencyFailure` package state
- when a package fails:
  - mark direct and transitive dependents as blocked, or
  - make scheduler detect that a pending package can never become ready
- terminate worker loop when all packages are in terminal states
  - `Completed`
  - `Failed`
  - `Blocked`
- make the build summary distinguish:
  - failed packages
  - unbuilt packages blocked by dependency failures
- add tests for:
  - dependency failure with `continue_on_error = false`
  - dependency failure with `continue_on_error = true`
  - multi-worker build where one worker fails early

Definition of done:

- no infinite polling after dependency failures
- summaries accurately reflect blocked downstream packages

### Slice 1.2: Hermetic Package Environment Per Build Job [complete]

Problem:

- parallel workers reuse a mutable `EnvironmentManager` across packages
- this can leak state from unrelated packages into later builds on the same worker

Relevant code:

- [src/commands/work/build/build_executor.rs](/home/bresilla/data/code/tools/roc/src/commands/work/build/build_executor.rs)
- [src/commands/work/build/environment_manager.rs](/home/bresilla/data/code/tools/roc/src/commands/work/build/environment_manager.rs)

Tasks:

- stop reusing a single environment manager inside worker threads
- construct a fresh package-specific environment for each package build
- ensure dependency-derived env is built only from:
  - base shell environment
  - selected underlay/install prefixes
  - current package context
- add tests that verify:
  - worker A building package X does not pollute package Y
  - package env contents are stable regardless of worker assignment

Definition of done:

- package build environment is reproducible and independent of worker history

### Slice 1.3: Fix `roc run` Argument And Prefix Parsing [complete]

Problem:

- `argv` and `prefix` are split with `split_whitespace()`
- quoted values and escaped spaces are broken

Relevant code:

- [src/commands/run/mod.rs](/home/bresilla/data/code/tools/roc/src/commands/run/mod.rs)

Tasks:

- redesign CLI parsing for executable args
  - prefer repeated trailing args captured by clap
  - avoid storing shell-like command strings where possible
- redesign prefix handling
  - either parse with a shellwords-compatible parser
  - or model prefix as repeated arguments too
- add tests for:
  - spaces in arguments
  - nested quotes
  - prefixed execution
  - package executable plus user args

Definition of done:

- `roc run` preserves user intent for normal quoted arguments and prefixes

### Slice 1.4: Add Lifecycle Control To Long-Running ROS Helpers [complete]

Problem:

- helper threads and loops are effectively immortal
- `DynamicSubscriber` spins forever in a dedicated thread
- bag record/play loops have weak or missing stop/finalization behavior depending on command version

Relevant code:

- [src/shared/dynamic_messages.rs](/home/bresilla/data/code/tools/roc/src/shared/dynamic_messages.rs)
- [src/commands/bag/record.rs](/home/bresilla/data/code/tools/roc/src/commands/bag/record.rs)
- [src/commands/bag/play.rs](/home/bresilla/data/code/tools/roc/src/commands/bag/play.rs)

Tasks:

- add explicit shutdown signaling to dynamic subscriber helper
- ensure dropped helpers stop their executor threads cleanly
- standardize signal handling in bag commands
  - ctrl-c
  - loop termination
  - summary flush/final writer drop
- add tests where feasible for:
  - helper drop semantics
  - command stop behavior
  - no lost file-finalization on interrupt

Definition of done:

- no helper relies on "thread ends when process exits" as its primary lifecycle model

## Phase 2: Make Environment And Runtime Assumptions Explicit

Objective:

- reduce "works only in the right shell" failures

### Slice 2.1: Python Install Layout Detection [complete]

Problem:

- Python site-packages paths are guessed using `python3/site-packages`
- many systems install under versioned dirs such as `python3.11/site-packages`

Relevant code:

- [src/commands/work/build/environment_manager.rs](/home/bresilla/data/code/tools/roc/src/commands/work/build/environment_manager.rs)

Tasks:

- detect versioned Python lib dirs dynamically
- prefer scanning install prefix for `site-packages` or `dist-packages`
- verify both isolated and merged layouts
- validate against at least:
  - typical system Python layout
  - local install layout produced by `ament_python`

Definition of done:

- Python package discovery/import behavior is not pinned to a fake `python3/` directory shape

### Slice 2.2: Runtime Library Preconditions [complete]

Problem:

- `cargo test` currently fails because runtime libraries such as `libspdlog.so.1.12` are not resolvable in the default environment

Tasks:

- identify the minimal runtime env required for executing unit/integration binaries
- decide on one of:
  - make tests source/setup the necessary runtime env
  - skip/runtime-gate ROS-linked tests more explicitly
  - document and codify a dev-shell contract
- ensure `cargo test` or a clearly documented alternative becomes a trustworthy default

Definition of done:

- test execution expectations are explicit and reproducible

### Slice 2.3: Tighten Build/Run Preflight Checks [complete]

Problem:

- several commands assume ROS env, runtime libraries, or external tools are available and only fail deep into execution

Tasks:

- add targeted preflight checks for commands that require:
  - ROS environment
  - `colcon`
  - `ros2`
  - `ctest`
  - `python3`
- provide actionable errors that say exactly what is missing
- avoid placeholder or vague output for missing dependencies

Definition of done:

- the first error a user sees is close to the real root cause

## Phase 3: Reduce Half-Baked Surface Area

Objective:

- stop exposing commands that look finished but are not

### Slice 3.1: Audit Placeholder And Delegated Commands [complete]

Problem:

- some commands are intentionally delegated to `ros2`
- some are minimal wrappers
- some are placeholders

Tasks:

- classify every command into:
  - native and production-worthy
  - delegated by design
  - partial/experimental
  - placeholder
- make placeholder paths explicit in help and docs
- ensure delegated commands report that they delegate
- either remove or hide obviously misleading stubs
  - especially daemon-related placeholders

Definition of done:

- users can tell which commands are fully native, delegated, or unfinished

### Slice 3.2: Harden Bag Record/Play Surface

Problem:

- bag commands are powerful but operationally fragile
- they depend on serialized transport behavior, raw loops, and MCAP assumptions

Tasks:

- add signal-safe shutdown and summary behavior
- validate type resolution errors and empty topic cases
- add playback validation for:
  - missing schema/channel metadata
  - unknown message type names
  - empty inputs
  - loop mode interruption
- add record validation for:
  - non-existent output directories
  - duplicate topics
  - unresolved topic types
  - separated-output edge cases

Definition of done:

- bag commands fail clearly and stop cleanly

### Slice 3.3: Dynamic Message Publish/Echo Capability Matrix

Problem:

- dynamic publish and echo paths are useful, but some field types and nested structures are only partially supported

Relevant code:

- [src/commands/topic/pub_.rs](/home/bresilla/data/code/tools/roc/src/commands/topic/pub_.rs)
- [src/commands/topic/echo.rs](/home/bresilla/data/code/tools/roc/src/commands/topic/echo.rs)

Tasks:

- document what message shapes are supported today
- add explicit unsupported errors for complex unhandled cases
- expand support in the most common missing categories:
  - nested structures
  - sequence handling in publish path
  - bounded strings/messages
- add focused tests around message conversion behavior

Definition of done:

- topic publish/echo behavior is predictable and documented for dynamic messages

## Phase 4: Strengthen Validation

Objective:

- catch real regressions with normal development workflows

### Slice 4.1: Rebalance Test Pyramid

Current state:

- completion tests run by default
- the strongest workspace parity checks are ignored and environment-dependent

Tasks:

- add default-running tests for pure logic:
  - dependency filtering
  - build state transitions
  - env var construction
  - bag metadata parsing
  - argument parsing
- keep ROS-heavy parity tests, but separate them clearly as validation/integration gates
- add deterministic fixtures for:
  - blocked dependency propagation
  - Python env path detection
  - run/prefix argument parsing

Definition of done:

- meaningful correctness coverage exists without requiring a full ROS machine

### Slice 4.2: Add Command-Focused Smoke Matrix

Tasks:

- create a small smoke matrix for the most important commands:
  - `roc topic list`
  - `roc topic echo --help`
  - `roc topic pub --help`
  - `roc work build --help`
  - `roc work test --help`
  - `roc work test-result --help`
  - `roc run --help`
  - `roc bag info --help`
- add lightweight assertions on exit status and stable output markers

Definition of done:

- obvious command breakage is caught before deeper validation

### Slice 4.3: Add Hardening Regression Tests

Target regressions to lock down:

- no parallel build deadlock after dependency failure
- no environment leakage between parallel jobs
- quoted args preserved in `roc run`
- long-running helpers shut down correctly
- Python install path handling supports versioned site-packages

Definition of done:

- each major hardening fix gets at least one regression test

## Phase 5: Documentation And Operational Clarity

Objective:

- make the real maturity level legible

### Slice 5.1: Align README And Docs With Reality

Tasks:

- audit README claims against actual behavior
- call out commands that still delegate to `ros2`
- describe environment prerequisites more concretely
- document bag command limitations and workspace validation scope
- document how to run the strongest validation suite

Definition of done:

- docs are conservative, specific, and operationally useful

### Slice 5.2: Add Developer Runbooks

Tasks:

- add a short developer workflow doc covering:
  - build
  - test
  - ROS env setup
  - parity validation
  - how submodule/forked `rclrs` changes are handled
- document expected local dependencies and troubleshooting steps

Definition of done:

- a new contributor can reproduce the intended dev/test environment without guessing

## Immediate Execution Plan

Recommended next implementation order:

1. fix parallel build deadlock and dependency-failure propagation
2. make worker build environments fresh per package
3. fix `roc run` argument/prefix parsing
4. add shutdown/lifecycle control to dynamic subscriber and bag commands
5. make Python env detection version-aware
6. make default test execution story explicit and reproducible

## Release Gate For Hardening Phase

Before calling the codebase materially hardened, the following should be true:

- no known infinite-loop or deadlock condition in workspace build/test orchestration
- `roc run` preserves quoted arguments correctly
- bag record/play stop cleanly and finalize outputs
- dynamic subscriber helper has explicit shutdown semantics
- Python package env handling works for versioned site-packages layouts
- default compile/test workflow is documented and reproducible
- the highest-risk fixes have regression tests

## Suggested Commit Strategy

Keep hardening work sliced by failure mode, not by module.

Recommended commit titles:

- `Fix parallel build deadlock after dependency failures`
- `Reset worker build environments per package`
- `Preserve quoted argv and prefix handling in roc run`
- `Add shutdown control to dynamic ROS subscription helpers`
- `Detect versioned Python site-packages in workspace env setup`
- `Document and gate runtime prerequisites for test execution`
- `Add regression tests for hardening fixes`
