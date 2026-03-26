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

### Slice 3.2: Harden Bag Record/Play Surface [complete]

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

### Slice 3.3: Dynamic Message Publish/Echo Capability Matrix [complete]

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

### Slice 4.1: Rebalance Test Pyramid [complete]

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

### Slice 4.2: Add Command-Focused Smoke Matrix [complete]

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

### Slice 4.3: Add Hardening Regression Tests [complete]

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

### Slice 5.1: Align README And Docs With Reality [complete]

Tasks:

- audit README claims against actual behavior
- call out commands that still delegate to `ros2`
- describe environment prerequisites more concretely
- document bag command limitations and workspace validation scope
- document how to run the strongest validation suite

Definition of done:

- docs are conservative, specific, and operationally useful

### Slice 5.2: Add Developer Runbooks [complete]

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

## Phase 6: Deep Hardening Of `roc work build`

Objective:

- make the workspace build path robust enough to trust under failure, scale, and real user environments

Current assessment:

- the build path has improved materially, but several important parts still rely on optimistic assumptions
- the highest remaining risks are in:
  - generated setup script correctness
  - subprocess execution model
  - scheduler architecture
  - discovery strictness
  - Python packaging assumptions

### Current Findings

1. the generated Python workspace helper appears to emit literal placeholders instead of resolved package/script paths
2. workspace setup ordering still relies too much on direct iteration order instead of runtime dependency metadata
3. build subprocesses are fully buffered via `Command::output()` with no timeout, cancellation, or streaming logs
4. parallel worker scheduling is polling-based and lock-heavy
5. `roc work build` currently requires a sourced ROS environment earlier and more broadly than `colcon` would
6. Python builds still depend on `setup.py build/install`
7. discovery is tolerant in ways that can hide bad workspace state
8. build-type inference can silently misclassify unknown packages as `ament_cmake`
9. previous-build selection relies on mutable log state as the source of truth
10. setup-script generation is large, hand-rolled, and only partially validated relative to its complexity

### Slice 6.1: Fix Generated Workspace Helper Script [complete]

Problem:

- the generated `_local_setup_util_*.py` helper is likely emitting literal placeholders for package/script paths
- this undermines ordered setup execution and install usability

Relevant code:

- [src/commands/work/build/build_executor.rs](/home/bresilla/data/code/tools/roc/src/commands/work/build/build_executor.rs)

Tasks:

- verify the generated helper script output against a real fixture install tree
- fix placeholder rendering so emitted commands contain actual resolved paths
- add a test that executes the generated helper and asserts:
  - package ordering is real
  - sourced script paths are real
  - no literal `{package_prefix}` or `{script}` placeholders remain

Definition of done:

- generated helper scripts emit executable path lines, not template artifacts

### Slice 6.2: Make Setup Ordering Metadata-Driven Everywhere [complete]

Problem:

- workspace local setup generation iterates package discovery/build order directly
- setup sourcing should be driven by runtime dependency metadata, not incidental vector order

Relevant code:

- [src/commands/work/build/build_executor.rs](/home/bresilla/data/code/tools/roc/src/commands/work/build/build_executor.rs)

Tasks:

- define one source of truth for package setup order
  - generated `share/colcon-core/packages/*` metadata
  - topologically sorted runtime dependency graph
- make `local_setup.sh` and `local_setup.ps1` follow that order
- add tests that prove a package with runtime deps is sourced after its prerequisites
- validate both merged and isolated install layouts

Definition of done:

- workspace setup ordering is deterministic and metadata-driven

### Slice 6.3: Replace Buffered Subprocess Execution With Streaming Phase Execution [complete]

Problem:

- `Command::output()` buffers stdout/stderr in memory and only returns on process exit
- long or hung builds are opaque and operationally weak

Relevant code:

- [src/commands/work/build/build_executor.rs](/home/bresilla/data/code/tools/roc/src/commands/work/build/build_executor.rs)

Tasks:

- replace phase execution with streamed subprocess handling
- tee stdout/stderr into:
  - terminal output
  - per-phase log files
- keep failure summaries concise while preserving full logs
- add tests for:
  - phase log creation
  - non-zero exit handling
  - visible partial output capture

Definition of done:

- users can see build progress while logs remain complete and usable

### Slice 6.4: Add Timeouts And Interrupt Handling To Build Phases

Problem:

- a wedged `cmake` or `python3 setup.py` process can stall the build forever
- there is no standard interrupt propagation for build subprocesses

Tasks:

- add configurable per-phase timeout support
- propagate ctrl-c into active build subprocesses
- mark interrupted packages distinctly from normal failures when possible
- ensure partial logs and status files still persist on interruption
- add tests around:
  - timeout classification
  - interrupted builds
  - summary/status output

Definition of done:

- hung or interrupted build phases terminate predictably and leave debuggable state behind

### Slice 6.5: Replace Polling Scheduler With Ready-Queue Coordination

Problem:

- the current parallel scheduler repeatedly polls shared state and sleeps
- this is simple but scales poorly and is harder to reason about than an event-driven queue

Tasks:

- redesign parallel scheduling around:
  - explicit ready queue
  - dependency counters
  - worker wakeup signaling
- minimize lock scope and lock frequency
- keep blocked/failure propagation semantics from the hardened version
- add regression tests for:
  - early failure
  - multi-worker fairness
  - no busy-loop waiting when idle

Definition of done:

- worker coordination is event-driven and easier to reason about than shared polling

### Slice 6.6: Tighten Discovery Diagnostics And Add Strict Mode [complete]

Problem:

- missing paths, parse failures, duplicate package names, and other discovery anomalies mostly degrade into warnings
- this hides real workspace problems

Relevant code:

- [src/shared/package_discovery.rs](/home/bresilla/data/code/tools/roc/src/shared/package_discovery.rs)
- [src/commands/work/build/mod.rs](/home/bresilla/data/code/tools/roc/src/commands/work/build/mod.rs)

Tasks:

- add a strict discovery mode for `roc work build`
- surface structured diagnostics for:
  - duplicate package names
  - unreadable/invalid manifests
  - missing requested packages
  - ignored packages due to hide/exclude rules
- keep a permissive mode when desired, but make strictness available for CI and debugging
- add tests for duplicate-package and malformed-manifest scenarios

Definition of done:

- users can choose between permissive discovery and fail-fast discovery with explicit diagnostics

### Slice 6.7: Harden Build-Type Inference And Unsupported-Type Reporting [complete]

Problem:

- unknown or underspecified packages can be silently inferred as `ament_cmake`
- that delays the real error and makes diagnosis worse

Tasks:

- make build-type inference more conservative
- distinguish:
  - explicitly declared build types
  - inferred build types
  - unsupported/unknown build types
- improve errors for `BuildType::Other`
- add tests for:
  - missing `build_type` plus clear source indicators
  - ambiguous packages
  - unsupported declared build types

Definition of done:

- unsupported or ambiguous packages fail early with a precise reason

### Slice 6.8: Revisit ROS Preflight Scope For `work build`

Problem:

- `roc work build` currently requires ROS env preflight before any real build work
- that is stricter than necessary for some local/package-only workflows

Tasks:

- separate:
  - absolute prerequisites for builder execution
  - prerequisites only needed for specific package types or underlays
- decide whether `work build` should:
  - always require sourced ROS
  - warn but continue in some cases
  - gate only when a package actually needs the missing environment
- document and test the chosen policy

Definition of done:

- `roc work build` has a deliberate preflight policy instead of a blanket assumption

### Slice 6.9: Reduce Reliance On `setup.py install` Assumptions

Problem:

- `ament_python` builds still rely on `setup.py build/install`
- newer Python packaging layouts and tooling are gradually making this path weaker

Tasks:

- document the currently supported Python package shapes explicitly
- identify the minimal compatibility target for `ament_python`
- harden install-layout detection around the outputs we actually rely on
- add fixtures covering:
  - versioned site-packages
  - egg-info metadata
  - resource/package.xml installation
- evaluate a forward path away from legacy assumptions without breaking current parity

Definition of done:

- Python package behavior is explicit, better tested, and less dependent on luck

### Slice 6.10: Strengthen Build-State Persistence And Resume Semantics [complete]

Problem:

- resume/select-skip behavior relies on `log/latest/*/status.txt`
- that is simple, but fragile if the log tree is stale, partially deleted, or copied

Tasks:

- define a more explicit machine-readable build state contract
- decide whether `latest/` is enough or whether a dedicated state file is needed
- validate resume behavior for:
  - partially deleted logs
  - mixed completed/failed/blocked state
  - stale state from a different workspace root
- add tests for build-state loading and filtering under those conditions

Definition of done:

- resume/select-skip behavior uses explicit, trustworthy state rather than opportunistic log parsing

## Recommended Next Order For `roc work` Hardening

1. fix the generated workspace helper script
2. make setup ordering metadata-driven
3. replace buffered subprocess execution with streaming logs
4. add timeouts and interrupt handling
5. replace polling scheduler with a ready queue
6. tighten discovery diagnostics and build-type inference
7. revisit ROS preflight scope
8. harden Python packaging behavior
9. strengthen resume/build-state persistence semantics
