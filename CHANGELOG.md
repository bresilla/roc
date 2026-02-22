# Changelog

## [0.3.0] - 2026-02-22

### <!-- 0 -->⛰️  Features

- Validate create inputs and escape generated metadata
- Unify discovery and improve build/create UX
- Implement MCAP reading and writing for bag commands
- Native Rust tf frame commands
- Add ros2_rust as a submodule
- Add initial TF frame listing functionality
- Implement native `roc bag list` and `roc bag info`
- Implement native ROS interface discovery and parsing
- Implement native action list and info commands
- Implement native Rust topic delay command
- Implement native ROS parameter commands
- Implement revolutionary generic message system
- Implement dynamic message introspection and type support
- Add ROS message serialization for dynamic types
- Implement dynamic type support for RCL publishers
- Support dynamic message type serialization and deserialization
- Introduce dynamic message type support
- Refactor topic handling for improved clarity and robustness
- Consolidate package discovery and enhance protobuf commands
- Integrate IDL and protobuf support for message conversion
- Implement basic topic delay analysis and stub
- Improve communication robustness and consistency
- Implement native ROS 2 topic introspection commands
- Implement core ROS 2 topic commands and graph API

### <!-- 1 -->🐛 Bug Fixes

- Support colcon-style build flags and fail fast on empty discovery
- Disable message publishing

### <!-- 2 -->🚜 Refactor

- Refactor graph module and improve project documentation

### <!-- 3 -->📚 Documentation

- Rewrite README in neutral reference style
- Document dynamic ROS2 message type loading
- Clarify `roc topic` status and limitations in `FEATURES.md`
- Refactor README to highlight workspace management
- Replace all logo images throughout codebase

### <!-- 6 -->🧪 Testing

- Add merged-install e2e flow coverage
- Add temp-workspace create/list/info flow
- Add coverage for status and create validation
- Enhance dynamic type support loading

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Remove unused test and message files

## [0.2.3] - 2025-06-19

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Upgrade development environment to Jazzy OS

## [0.2.2] - 2025-06-19

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Streamline Linux release build process

## [0.2.1] - 2025-06-19

### <!-- 0 -->⛰️  Features

- Add documentation and ROS 2 discovery features
- Display detailed endpoint info with QoS
- Ros2topic: Add verbose mode to topic info
- Add topic info to `rclrs`
- Implement functionality to list topics with types
- Implement topic listing with RCL direct API
- Simplify internal graph API queries
- Add ROS node and graph introspection bindings
- Add RCL graph and node interface functions
- Initialize and shutdown the RCL context

### <!-- 2 -->🚜 Refactor

- Improve topic information and daemon status display
- Refactor 'topic info' to use internal graph API
- Refactor argument handling to use global scope
- Refactor topic commands for shared arguments
- Refactor context initialization and topic discovery

### <!-- 3 -->📚 Documentation

- Document and extend the graph context

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Improve GitHub Actions reliability
- Refactor: Remove debug traces
- Remove unused GitHub Actions workflows and unused print statement

## [0.2.0] - 2025-05-30

### <!-- 0 -->⛰️  Features

- Feat: Implement dynamic shell completions
- Improve ROS 2 launch and run completion with package filtering
- Improve ROS completion path discovery
- Add command completions for `_roc`
- Support finding sourced package executables
- Add shell command completion support
- Introduce RMW bindings and basic examples
- Initial import of the rclrs Rust client library
- Add Rust package templating support
- Implement package parallel builds
- Replace colcon with custom build tool
- Implement workspace and command stubs
- Refactor CLI command execution pattern

### <!-- 2 -->🚜 Refactor

- Refactor completion logic into dedicated module
- Enhance `info` and `list` commands with package data
- Refactor package creation and add new template files

### <!-- 3 -->📚 Documentation

- Add a README for the rclrs crate

### <!-- 6 -->🧪 Testing

- Simplify and test package template generation

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Ignore VS Code settings and clean up configuration files
- Ignore devbox directory in version control
- Refactor environment setup and improve devbox configuration

### Build

- Remove deprecated ROS 2 templates
- Remove unused build code
- Build refactorings

## [0.1.34] - 2025-05-28

### <!-- 0 -->⛰️  Features

- Refactor CLI command execution pattern

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Ignore VS Code settings and clean up configuration files
- Ignore devbox directory in version control
- Refactor environment setup and improve devbox configuration

## [0.1.33] - 2023-11-12

### <!-- 1 -->🐛 Bug Fixes

- `roc topic echo` fix

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Svg_edits

## [0.1.32] - 2023-10-19

### <!-- 1 -->🐛 Bug Fixes

- Github action for x-th time

### <!-- 3 -->📚 Documentation

- Building website/mdbook

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Removed /docs folder
- Gitub action mdbook automatic

## [0.1.30] - 2023-10-19

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Add GitHub Actions workflow for creating releases

### Dosc

- Add image to "Why roc?" section

## [0.1.29] - 2023-10-18

### <!-- 0 -->⛰️  Features

- Finished `roc interfaces show`
- Finished `roc interface model`
- Finished `roc interface all`
- Finished `roc interface package`
- Finished `ros interface list`

### <!-- 3 -->📚 Documentation

- New logo

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Update dependency on RustDDS from GitHub repository
- Update CARGo package name

### Fead

- Finished `roc interface`

## [0.1.28] - 2023-10-16

### <!-- 0 -->⛰️  Features

- Merging finished `roc node`
- Finished `roc node info`
- Finished `roc node list`

### <!-- 3 -->📚 Documentation

- Various updates on mdbook

## [0.1.27] - 2023-10-16

### <!-- 0 -->⛰️  Features

- Namechange to `roc` from `borg`

### <!-- 3 -->📚 Documentation

- Site titile to `roc`
- LOGO change
- Update documentation for BORG installation

## [0.1.26] - 2023-10-13

### <!-- 3 -->📚 Documentation

- Add documentation for BORG tool.

## [0.1.25] - 2023-10-13

### <!-- 1 -->🐛 Bug Fixes

- Github action for x-th time
- Github action

## [0.1.24] - 2023-10-13

### <!-- 3 -->📚 Documentation

- Update documentation and asset paths

## [0.1.23] - 2023-10-13

### <!-- 0 -->⛰️  Features

- Finished `borg param`
- Finished `borg praram import`
- Finished `borg param describe`
- Finished `borg param remove`
- Finished `borg param export`
- Finished `borg param set`
- Finished `borg param list`
- Finished `borg param list`

## [0.1.22] - 2023-10-12

### <!-- 0 -->⛰️  Features

- Add new functionality to the `borg service` command
- Artifacts as archives

## [0.1.20] - 2023-10-12

### <!-- 0 -->⛰️  Features

- Build artifacts on release
- Build workflow changes
- Github workflow
- Github workflow
- Github workflow

## [0.1.19] - 2023-10-12

### <!-- 0 -->⛰️  Features

- Update version, perform miscellaneous tasks, and add deploy command
- MDBOOK
- LOGO
- Finished <borg action goal>
- Finished <borg action list>
- Finished <borg action info>
- Finished <borg topic delay>
- Finished <borg topic find>
- Finished <borg topic bw>
- Finished <borg topic kind>
- Finished <borg topic info>
- Finixhed workning on <borg topic pub>
- Finished <borg topic echo>
- Continue working on "topic" command
- Work on topic command and subcommands
- Command handling for "topics"
- Refactor subcommand handling for "action" command
- Started working on handle of commands
- Separated the CMD into multiple files
- Added subcommand <borg daemon>
- Add CLI subcommands <ros middleware>
- Add new subcommand <borg frame>
- Update command line interface options in builder.rs
- Refactor CLI builder to support new styling options and subcommands
- Revamp the CLI application and improve README formatting
- Add new subcommands and improve documentation for `alli` tool
- Add aliases and options to "node" subcommand in CLI builder
- Add additional commands and subcommands to the CLI builder
- Update CLI builder function to include additional arguments and subcommands
- Refactor CLI imports and initialize matches in main function
- Refactor command line interface and add subcommands
- Initial setup for "borg" command line tool project.

### <!-- 1 -->🐛 Bug Fixes

- Fix path for building mdbook

### <!-- 3 -->📚 Documentation

- Mdbook arrangement
- Improve miscellaneous files and README documentation
- Update README.md and improve command name consistency
- Improve README and ABOUT_STR for alpha state and installation instructions
- Update ASCII
- Update usage instructions and command list in README
- Add ASCII art logo for project
- Add ASCII art for project title

### <!-- 5 -->🎨 Styling

- Adjust image in README file to fit width

### <!-- 7 -->⚙️ Miscellaneous Tasks

- Git-cliff things
- Update version number and modify release process
- Update paths and version number, and modify CHANGELOG.md
- Improve handling of binary image files
- Refactor package name and add new binary
- Refactor package name and publish field in Cargo.toml
- Rename package to "borg_ros"
- Add license specification to Cargo.toml
- Refactor code
- Refactor command line interface commands and arguments
- Remove <derive.rs> file
- Add initial project files and documentation

### Merge

- Finished <borg action>

<!-- BRESILLA -->
