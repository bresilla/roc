# ROC Parity And CLI Output Plan

## Goal

Keep two tracks moving in parallel:

1. finish the explicit release gate for practical `colcon` workflow parity on Linux/Jazzy
2. modernize `roc` output so the whole CLI feels coherent, dense, and readable

This does not replace `ament`. It replaces workspace orchestration around standard `ament_cmake` and `ament_python` packages, while also improving the user-facing CLI presentation.

## Track A: Colcon Workflow Parity

Current validated scope:

- `roc work build`
- `roc work test`
- `roc work test-result`

Primary validation reference:

- [COMPAT_VALIDATION.md](/doc/code/tools/roc/COMPAT_VALIDATION.md)

Executable validation reference:

- [tests/real_workspace_validation.rs](/doc/code/tools/roc/tests/real_workspace_validation.rs)

Completed slices:

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
- Slice 27: explicit parity release gate

Track A status:

- the release/signoff process is now defined in [RELEASE_GATE.md](/doc/code/tools/roc/RELEASE_GATE.md)
- the repeatable entrypoint is `make release-gate`
- real-workspace validation paths can be overridden with:
  - `ROC_VALIDATION_ROS2_EXAMPLES`
  - `ROC_VALIDATION_ROS2_DEMOS`

Claim policy:

- practical `colcon` parity should only be claimed when the latest release-gate run is green on the validated Linux/Jazzy environment
- after a parity-sensitive change, rerun the gate and refresh [COMPAT_VALIDATION.md](/doc/code/tools/roc/COMPAT_VALIDATION.md)

## Track B: CLI Output Modernization

### Problem Statement

The current CLI output works, but it is inconsistent:

- repeated hand-written headers and totals
- mixed note/warning/error phrasing
- flat list output for commands that really want tables
- duplicated color logic across commands
- poor width handling for long names, types, paths, and error details
- no single shared output model across graph, workspace, bag, and test commands

The issue is not “not enough color”. The issue is lack of structure and a missing shared rendering layer.

### Cargo Library Research

Checked current Rust crates on March 7, 2026.

Recommended stack:

- `comfy-table` 7.2.2
  - strong fit for runtime-built terminal tables
  - dynamic arrangement by width
  - ANSI-aware styling
  - presets, alignment, padding, borders
  - good match for `topic list`, `node list`, `service list`, `work list`, `test-result`
- `console` 0.16.2
  - terminal-aware styling
  - text width measurement
  - padding and truncation helpers
  - stronger primitive layer than the current ad hoc `colored` usage
- `textwrap` 0.16.2
  - wrapping and indentation for notes, help blocks, and verbose failure details
- `indicatif` 0.18.4
  - progress bars, spinners, and human formatting
  - good fit for `work build`, `work test`, bag record/play progress, long discovery waits

Useful optional additions:

- `miette` 7.6.0
  - better user-facing diagnostics for command failures
  - especially useful for parse errors, build/test failures, and invalid user input
- `owo-colors` 4.3.0
  - reasonable upgrade path if replacing `colored`
  - terminal-support-aware styling via `supports-color`
- `supports-color` 3.0.2
  - useful if color policy needs to be managed explicitly
- `unicode-width` 0.2.2
  - direct use only if custom renderers need width logic outside `console` / `comfy-table`

Libraries considered but not the default choice:

- `tabled` 0.20.0
  - good crate, especially for derive-driven static tables
  - weaker fit than `comfy-table` for highly dynamic graph/workspace listings built at runtime
- full TUI crates such as `ratatui`
  - too heavy for the current non-interactive CLI
  - wrong abstraction unless `roc` grows a real interactive dashboard mode

### Recommended Dependency Decision

Primary recommendation:

- keep the CLI non-interactive
- adopt:
  - `comfy-table`
  - `console`
  - `textwrap`
  - `indicatif`
- keep `colored` temporarily during migration
- decide later whether to consolidate on `console` only or switch styling to `owo-colors`

Reason:

- lowest-risk path
- immediate payoff on list/info/build/test outputs
- avoids a big-bang renderer rewrite

### Target Output Model

Introduce a shared `src/ui/` layer:

- `ui/theme.rs`
  - shared colors, emphasis, status chips, fallback styles
- `ui/output_mode.rs`
  - `human`, `plain`, `json`, `ros-style`
- `ui/table.rs`
  - reusable table builder wrappers around `comfy-table`
- `ui/block.rs`
  - headers, key/value sections, notes, warnings, summaries
- `ui/tree.rs`
  - lightweight tree rendering for frames and dependency views
- `ui/progress.rs`
  - `indicatif` wrappers and non-TTY fallbacks

Design rules:

- human mode should be compact, aligned, and colorful
- plain mode should be stable and pipe-friendly
- json mode should be explicit and scripting-safe
- ros-style mode should preserve compatibility where needed
- no spinners or ANSI noise when stdout is not a TTY

### Commands To Improve First

Highest-value first wave:

- `topic list`
- `node list`
- `service list`
- `action list`
- `work list`
- `work test-result`

Second wave:

- `topic info`
- `node info`
- `service info`
- `action info`
- `frame info`
- `bag info`
- `work info`

Third wave:

- `param list`
- `interface list`
- `bag list`
- `frame list`
- long-running monitor commands like `topic hz`, `topic bw`, `topic delay`

### Output Features To Add

Global:

- consistent section headers
- consistent totals/footers
- consistent note/warning/error formatting
- terminal-width-aware truncation and wrapping
- optional icons only if they degrade cleanly in plain mode

Lists:

- aligned tables instead of loose colored lines
- stable column order per command
- path/type truncation with full values still available in JSON mode

Info commands:

- compact key/value blocks
- grouped subsections
- better empty-state rendering

Build and test:

- package progress rows
- clearer failure groups
- summary tables with counts, elapsed time, and log paths

Trees:

- frame hierarchy mode
- workspace/package dependency tree mode

Machine-readable:

- JSON output for all list/info/state commands
- stable field names

### Proposed Rollout Slices

### Slice U0: UI Foundation

Objective:

- create the shared rendering layer without changing every command at once

Tasks:

- add `src/ui/`
- add shared theme/status helpers
- add a minimal table wrapper and block renderer
- add a small `OutputMode` enum

Definition of done:

- one or two commands can render through shared helpers

Suggested commit title:

- `Add shared CLI UI rendering foundation`

### Slice U1: Global Output Modes

Objective:

- standardize how commands choose between human/plain/json/ros-style output

Tasks:

- add global or per-command `--output`
- keep `--ros-style` behavior intact
- disable rich formatting when stdout is not a TTY

Definition of done:

- at least list commands can switch modes consistently

Suggested commit title:

- `Standardize CLI output modes across commands`

### Slice U2: Graph List Commands

Objective:

- convert the most-used discovery commands to structured tables

Tasks:

- migrate:
  - `topic list`
  - `node list`
  - `service list`
  - `action list`
- add columns such as:
  - name
  - type
  - counts where available

Definition of done:

- these commands all share the same header/footer/status style

Suggested commit title:

- `Render graph list commands as structured tables`

### Slice U3: Workspace And Bag Listings

Objective:

- make workspace and bag commands match the same UI standard

Tasks:

- migrate:
  - `work list`
  - `bag list`
  - `interface list`
  - `frame list`
- add status columns where meaningful

Definition of done:

- list-style commands feel like one product instead of separate scripts

Suggested commit title:

- `Unify workspace and bag list output formatting`

### Slice U4: Info Command Cards

Objective:

- replace free-form line dumps with compact, readable sections

Tasks:

- migrate:
  - `topic info`
  - `node info`
  - `service info`
  - `action info`
  - `frame info`
  - `bag info`
  - `work info`

Definition of done:

- info commands use shared key/value blocks and grouped sections

Suggested commit title:

- `Render info commands with shared section blocks`

### Slice U5: Build/Test Progress And Summaries

Objective:

- make the build and test flows easier to scan live and after completion

Tasks:

- add `indicatif` progress/spinner support with non-TTY fallback
- improve package start/finish lines
- render final summary tables for:
  - `work build`
  - `work test`
  - `work test-result`

Definition of done:

- long-running commands are easier to read without losing plain/log-friendly output

Suggested commit title:

- `Improve build and test progress presentation`

### Slice U6: Tree Views

Objective:

- add hierarchical views where flat lists are the wrong shape

Tasks:

- add `frame tree`
- add package dependency tree views for workspace commands
- ensure ASCII fallback exists for non-Unicode terminals

Definition of done:

- hierarchy-heavy commands no longer force users to mentally reconstruct trees from flat text

Suggested commit title:

- `Add tree views for frames and package dependencies`

### Slice U7: JSON Output Coverage

Objective:

- make scripting and machine consumption first-class

Tasks:

- add JSON output for all migrated list/info/state commands
- document stable field names
- ensure no ANSI or presentation-only data leaks into JSON mode

Definition of done:

- users can script against `roc` without scraping terminal text

Suggested commit title:

- `Add JSON output mode for structured CLI commands`

### Slice U8: Diagnostics Cleanup

Objective:

- improve command failures and warnings so they read like one system

Tasks:

- unify warnings/notes/errors through the UI layer
- consider introducing `miette` for richer diagnostic output in selected high-value paths
- remove remaining raw `eprintln!("Error: ...")` formatting drift

Definition of done:

- failures are more readable and consistent across command families

Suggested commit title:

- `Unify command diagnostics and warning presentation`

### Slice U9: Snapshot And Integration Coverage

Objective:

- keep the new output layer from drifting

Tasks:

- add snapshot tests for representative outputs in:
  - human mode
  - plain mode
  - json mode
- keep completion integration tests for command discovery
- add width-sensitive cases to catch wrapping/truncation regressions

Definition of done:

- output regressions are caught automatically

Suggested commit title:

- `Add snapshot coverage for CLI output rendering`

## Delivery Rule

Every completed feature slice must be committed separately.

Commit rule:

- one git commit per completed slice
- commit message title only
- no commit body

## Recommended Next Step

Do Slice U0 first, not another one-off command tweak.

Reason:

- the current problem is systemic inconsistency
- a shared UI layer is what unlocks prettier output everywhere
- once that exists, migrating list/info/build/test commands becomes straightforward
