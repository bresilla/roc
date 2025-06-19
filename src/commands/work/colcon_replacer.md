# ✅ IMPLEMENTED: ROC Colcon Replacement Build System

## Implementation Status: COMPLETE

ROC now includes a **fully functional colcon replacement build system** implemented in the `roc work build` command. This document has been preserved for historical reference, but the features described have been successfully implemented.

## What Has Been Built

### Complete Build System (`src/commands/work/build/`)

The following modules have been implemented:

1. **Package Discovery** (`package_discovery.rs`)
   - Recursive workspace scanning for `package.xml` files
   - XML parsing with `roxmltree` 
   - Build type inference and validation
   - Support for `COLCON_IGNORE` files

2. **Dependency Resolution** (`dependency_graph.rs`)
   - Topological sorting using Kahn's algorithm
   - Circular dependency detection with detailed error reporting
   - Build order optimization for parallel execution
   - External dependency handling

3. **Build Execution** (`build_executor.rs`)
   - Parallel build execution with configurable worker threads
   - Support for ament_cmake, ament_python, and cmake build types
   - Environment isolation to prevent build contamination
   - Comprehensive error handling and logging

4. **Environment Management** (`environment_manager.rs`)
   - Automatic environment variable setup for builds
   - Setup script generation (bash and batch)
   - Support for both isolated and merged install modes
   - Cross-platform path handling

5. **Build Configuration** (`mod.rs`)
   - Complete colcon command-line compatibility
   - All major build options supported
   - Extensible configuration system

### Command-Line Interface

The `roc work` command provides:

- `build` - Full colcon replacement with parallel execution
- `create` - Package creation wizard for all major build types  
- `list` - Package discovery and listing
- `info` - Package metadata extraction

### Key Features Implemented

✅ **Package Discovery & Parsing**
- Automatic `package.xml` scanning and parsing
- Support for package.xml formats 2 and 3
- Build type detection (ament_cmake, ament_python, cmake)
- Dependency extraction and validation

✅ **Dependency Resolution**
- Topological sorting for correct build order
- Circular dependency detection with clear error messages
- Support for selective building (`--packages-select`, `--packages-ignore`, `--packages-up-to`)
- External dependency handling

✅ **Parallel Build System**
- Multi-threaded execution with configurable worker count
- Intelligent dependency-aware scheduling
- Environment isolation per build
- Build state synchronization

✅ **Environment Management**
- Automatic CMake prefix path setup
- Library path configuration (LD_LIBRARY_PATH, DYLD_LIBRARY_PATH)
- Python path management
- ROS2 environment variable handling
- Setup script generation

✅ **Colcon Compatibility**
- Drop-in replacement for `colcon build`
- All major command-line options supported
- Same workspace structure and output format
- Compatible setup scripts

✅ **Build System Support**
- **ament_cmake**: Full CMake integration with ament macros
- **ament_python**: Python setuptools integration
- **cmake**: Plain CMake package support

✅ **Advanced Features**
- Isolated vs merged install modes
- Symlink installation support
- Continue-on-error option
- Custom CMake arguments
- Detailed build logging

## Performance Improvements Over Colcon

The implemented system provides significant performance benefits:

- **Native Rust performance**: No Python interpreter overhead
- **Efficient parallel execution**: Better worker thread management
- **Memory efficiency**: Lower memory usage during builds
- **Faster startup**: Native binary with minimal initialization time
- **Clean environment isolation**: Prevents build environment pollution

## Usage Examples

```bash
# Basic build (replaces `colcon build`)
roc work build

# Parallel build with 8 workers
roc work build --parallel-workers 8

# Build specific packages
roc work build --packages-select my_package another_package

# Build with merged install
roc work build --merge-install

# Continue on build errors
roc work build --continue-on-error

# Pass custom CMake arguments
roc work build --cmake-args -DCMAKE_BUILD_TYPE=Debug
```

## Architecture Summary

The implementation follows the modular architecture outlined in this document:

```
roc work build
├── Package Discovery     → discover_packages()
├── Dependency Resolution → topological_sort() 
├── Build Execution      → build_all()
├── Environment Setup    → EnvironmentManager
└── Setup Scripts        → generate_setup_scripts()
```

## Testing & Validation

The system has been tested with:
- Various ROS2 package types (C++, Python, mixed)
- Complex dependency graphs
- Parallel execution scenarios
- Both isolated and merged install modes
- Cross-platform compatibility (Linux primary)

## Future Enhancements

While fully functional, potential improvements include:
- Windows batch script generation refinement
- Additional build system support (Cargo, Bazel)
- Enhanced logging and diagnostics
- Build caching and incremental builds
- Plugin system for custom build types

---

## Original Planning Document

Great. I'll dive into how `colcon build` works in detail, specifically in the context of ROS 2 with ament. I'll focus especially on the package compilation aspect and outline how you could replicate this in Rust as a drop-in replacement.reat. I’ll dive into how `colcon build` works in detail, specifically in the context of ROS 2 with ament. I’ll focus especially on the package compilation aspect and outline how you could replicate this in Rust as a drop-in replacement.

I’ll break down the internal stages and describe a step-by-step rewrite procedure including dependency resolution, environment management, and package build execution.
I’ll let you know once I have the detailed explanation and migration plan ready.


# Understanding `colcon build` in ROS 2 (ament) and Designing a Rust-Based Replacement

## How `colcon build` Works in ROS 2 (Ament Build System)

### Package Discovery in Colcon

Colcon is the standard build tool in ROS 2 (replacing ROS 1’s catkin tools and the earlier `ament_tools`). It is designed to operate on a *workspace* containing multiple packages. To discover packages, colcon recursively crawls the workspace directory (by convention, often the `src/` directory) looking for ROS package manifest files (`package.xml`). The presence of a `package.xml` at a directory’s root identifies that directory as a package. Colcon uses the manifest as a marker and reads it to gather metadata. Under the hood, ROS 2 still uses the same manifest format as ROS 1 (REP 127/140); in fact, parsing the XML is done using the **catkin\_pkg** library (from ROS 1), while colcon’s role is to locate these files on disk. Each `package.xml` provides the package’s name (which must be globally unique in the workspace) and lists its dependencies and other metadata.

> **Note:** Colcon adheres to the package manifest specification (REP 149 for format 3, with support for format 2). This means every ROS 2 package must have a proper `package.xml` describing its build, run, and test dependencies, among other information. Colcon will ignore any directory that doesn’t contain a valid manifest, or that is marked to be skipped (e.g. via a `COLCON_IGNORE` file).

When you run `colcon build`, the tool will:

* Search the given workspace paths for `package.xml` files.
* For each found manifest, parse it to identify the package name, version, type (build type), and dependencies.
* Optionally, apply any package selection filters (e.g., `--packages-select` or `--packages-up-to` arguments) to decide which subset of packages to build.
* Build an internal list of packages and their relations.

Colcon’s design is extensible; the **package discovery** step is implemented via an extension point, allowing custom package layouts if needed. By default, it assumes a standard ROS workspace structure (with an optional `src` folder, though you can also point colcon to any directory containing packages).

### Dependency Resolution and Build Order

Once packages are discovered, colcon constructs a directed graph of package dependencies. Each package can declare various dependency types in its manifest:

* **Build dependencies** (things needed to compile the package, e.g. headers or build tools),
* **Build tool dependencies** (e.g. the build system itself like `ament_cmake`),
* **Exec (run) dependencies** (needed at runtime),
* **Test dependencies**, etc.

For determining build order, colcon primarily cares about build-time dependencies. A package A must be built *after* any package B that it depends on for building or exporting interfaces (i.e. after B’s headers, libraries, or CMake config files are ready). Colcon uses the dependency graph to perform a **topological sort** of packages. This ensures that if package `X` depends on `Y`, then `Y` is built before `X`. In the absence of cycles (cyclic dependencies are an error), a valid build order can be found. If some dependencies of a package are *not* present in the workspace (e.g. they are system-installed or in an underlay workspace), colcon does not build those but assumes they are already available in the environment. (Colcon itself does **not** fetch or install missing dependencies – that is out of scope, so the user is expected to have sourced an underlay or installed necessary packages beforehand.)

Colcon’s topological ordering mechanism is robust:

* It uses the dependency declarations from `package.xml` to sort the packages. (In ROS 2’s ament, the manifest must list *all* required dependencies so that colcon can infer the correct order.)
* Colcon can visualize or output this order via commands like `colcon graph` or `colcon list --topological`. There’s also an extension to generate DOT graphs of the dependency tree.
* If a dependency cycle is detected, colcon will report an error (since there is no valid build order).

To illustrate, suppose you have three packages `A`, `B`, and `C` where:

* `A` depends on `B` (build and run depend)
* `B` depends on `C`
* `C` has no dependencies.
  Colcon will determine the order as `C -> B -> A` (C first, then B, then A). Packages with no inter-dependencies can be built in parallel (colcon supports parallel execution where possible), but any dependent packages will only start after their prerequisites finish.

### Invoking Build Commands for Ament Packages

Colcon itself is a **build tool orchestrator**, not a build system. It does not compile code directly but delegates to each package’s own build system. In ROS 2, the two primary build systems (a.k.a. *build types*) are **ament\_cmake** (for C/C++ packages using CMake) and **ament\_python** (for pure Python packages using setuptools). Colcon supports both of these, as well as plain CMake packages and others via its extension mechanism. The package’s `package.xml` usually declares its build type (for example, `<build_type>ament_cmake</build_type>` in the export section for a C++ package, or `ament_python` for a Python package). Colcon uses that to decide which **build task plugin** to invoke for the package.

* **For an** `ament_cmake` **package**: Colcon will call CMake and Make/Ninja under the hood. Specifically, it creates an isolated build directory for the package and runs the equivalent of:

  ```bash
  cmake -S <package_source> -B <workspace>/build/<pkg> -DCMAKE_INSTALL_PREFIX=<workspace>/install/<pkg> [other CMake args]
  cmake --build <workspace>/build/<pkg> --target install -- -jN    # N=parallel jobs
  ```

  In practice, the `ament_cmake` colcon plugin sets up some additional CMake arguments (for example, to find other ament packages via CMAKE\_PREFIX\_PATH, to include debug symbols if requested, etc.), but fundamentally it’s using CMake’s configure and build commands. By the end of this step, the package is compiled and installed into its install prefix.

* **For an** `ament_python` **package**: These are pure Python packages (often containing ROS nodes in Python). Instead of CMake, they use the Python setuptools. Colcon’s ament\_python plugin will invoke the package’s `setup.py`. In ROS 2 Foxy (and earlier releases), this typically means calling:

  ```bash
  python3 setup.py build --build-base <workspace>/build/<pkg>
  python3 setup.py install --root=<workspace>/install/<pkg> --prefix="" 
  ```

  (The exact invocation may differ; newer ROS 2 versions might use `pip` under the hood, but conceptually it’s running the standard setup script.) The result is that Python modules, scripts, and other resources are installed to the package’s install directory. As the ROS 2 docs note, an example of an ament\_python package is **ament\_index\_python**, where the `setup.py` is the primary build entry point.

* **Pure CMake packages** (not using ament macros): Colcon can even handle these, treating them similarly to ament\_cmake (it discovers they have a `package.xml` but perhaps no `<build_type>` or a `<build_type>cmake</build_type>`, and then calls CMake accordingly). This is mainly for integrating non-ROS packages or ROS1 catkin packages in a ROS 2 workspace. The process is the same: configure, build, install via CMake, just without ROS-specific CMake macros.

* **Other build types**: Colcon’s architecture allows additional build system plugins. For example, if there were packages built with Autotools or Bazel, colcon could support them via extensions. In ROS 2’s default setup, ament\_cmake and ament\_python cover most cases, but colcon is not limited to those (it was intended as a “universal build tool”).

Importantly, colcon tries to be as **build-system-agnostic** as possible. It knows how to invoke each type of build, but it doesn’t hardcode language-specific logic beyond that. The build commands and their options are usually provided by plugins like `colcon-cmake` or `colcon-python`. For instance, there are colcon command-line options `--cmake-args` and `--ament-cmake-args` which let you pass arguments through to the CMake invocation of ament\_cmake packages. Similarly, `--pytest-args` could be passed to test runners, etc. This plugin-based approach ensures colcon’s core doesn’t need to know the intricacies of each build system.

### Build Order and Parallelism

After determining the topological order of packages, colcon will build them in sequence by default (ensuring each package’s dependencies are built first). However, colcon can parallelize builds to an extent:

* Independent packages (those with no dependency relations) may be built in parallel if you supply `--parallel-workers` or a similar option. By default, colcon’s execution model might still process one package at a time (especially in earlier ROS 2 releases), but it can be configured for parallel execution of multiple packages. Each package’s internal build (e.g. the `make` step) can of course use multiple threads (CMake will use `-j` with the number of cores).
* Colcon’s executor makes sure that no package starts building until all of its direct *and* transitive build dependencies have been successfully built. This avoids issues where a package might try to find a library or include file that isn’t there yet.

If any package fails to build, colcon will typically stop the build process (unless you’ve used an option to continue with others). It will report the failure and point to logs for details.

### Isolated Build Directories and Installations

One of colcon’s key features (in contrast to the older ROS 1 `catkin_make`) is that it performs **out-of-source, isolated builds** by default. When you run `colcon build` in a workspace with a `src` folder, colcon will create three parallel directories at the root of the workspace:

* **build/** – contains intermediate build artifacts (one subdirectory per package). For each package discovered, colcon creates `build/<package_name>/` and invokes the build system (CMake, etc.) in that directory. This keeps temporary files (object files, CMake cache, etc.) separate from source code.
* **install/** – the target installation directory. By default, colcon uses an **isolated install** for each package: each package gets its own prefix `install/<package_name>/` where its files are installed. For example, after building `my_pkg`, you might have `install/my_pkg/lib/...`, `install/my_pkg/include/...`, etc. This isolation mirrors the behavior of `catkin_make_isolated` or `ament_tools` in earlier systems.
* **log/** – a directory containing build logs and metadata for the current and past build runs. Colcon writes extensive logs here (more on this below).

This default isolation has several advantages:

* Packages only consume what they declare. If package A doesn’t depend on package B, having separate install spaces helps ensure A isn’t accidentally using B’s artifacts (which could happen in a merged environment without proper dependency declaration).
* You can clean or rebuild one package’s artifacts without affecting others.
* On platforms like Windows, isolated installs avoid extremely long file paths that could occur if all packages were merged (Windows has a path length limitation; colcon even suggests using `--merge-install` on Windows only when needed due to this issue).

Colcon **does offer a `--merge-install` option** to use a single unified install directory (all packages install into the same `install/` prefix rather than subfolders). This is useful in some scenarios (e.g., to avoid overly long environment variables with many paths). However, the downside is losing the strict isolation between packages: with a merged install, one package could inadvertently find resources from another package that it didn’t declare as a dependency. By default, it’s safer to keep them separate, and indeed colcon uses isolated installs by default for ROS 2 workspaces.

Finally, note that colcon never touches your source directory (no in-source builds). There is also no intermediate “devel” space as in ROS 1 catkin; the install space plays the role of the final output location. In ROS 2, binaries and libraries are expected to be used from the install space directly (which is why you must source the `install/setup.bash` after building to use them).

### Environment Setup and Handling During Build

Building ROS 2 packages often requires setting up environment variables so that packages can find each other’s resources. Colcon automates this environment management both **during the build** and **after the build** (for runtime use).

**During the build:** When building packages in topological order, colcon ensures that each package’s build sees the installed artifacts of its dependencies. For example, if package **A** depends on **B**, and B has already been built and installed to `install/B`, then while building A, the build system (CMake, in this case) needs to know how to find B. This typically means the path `install/B` (and specifically things like `install/B/share/B/cmake` for CMake config files, or `install/B/lib` for libraries, etc.) must be added to the relevant environment variables so that CMake’s `find_package(B)` works.

Colcon sets up these environment variables automatically in between package builds. In practice:

* Colcon uses each package’s **package manifest and installation contents** to update variables like `CMAKE_PREFIX_PATH`, `LD_LIBRARY_PATH`, `PYTHONPATH`, etc., before building the next package. This prevents the need for the user to manually source setup files between each step of the build. As the colcon design docs state, when building a package on top of its dependencies, “automating the \[environment setup] process is necessary to build packages in topological order without user interaction”.
* After building a package (say B), colcon will incorporate B’s *“environment hook”* scripts or use heuristics to extend the environment for packages that depend on B. ROS 2’s ament build system encourages each package to export environment hooks (scripts that set env vars). Colcon either sources those hooks or replicates their effect. For example, ament\_cmake packages install a file `local_setup.sh` (and `.bash`, `.ps1` for other shells) in `share/<package>` that, when sourced, adds that package’s paths to the environment. Colcon (with the help of the `colcon-ros` extension) knows to call these scripts for dependencies of the next package.
* In addition, colcon has a set of built-in **heuristics** for environment setup: it scans the install directory of each package and if it finds certain files or directories, it sets environment variables accordingly. For instance, if a package installs an executable into a `bin/` directory, colcon will ensure that `install/<pkg>/bin` is added to the `PATH`. If a package installs libraries (`.so` files) into `lib/`, it will add that to `LD_LIBRARY_PATH` on Linux. If it finds CMake package configuration files (`*Config.cmake`), it adds the appropriate path to `CMAKE_PREFIX_PATH`, and so on. These heuristics are provided by extension packages like `colcon-cmake` and `colcon-library-path`.

This automated environment handling means that by the time package A’s build runs, all of B’s headers, libraries, and CMake config files are discoverable. Practically, colcon achieves this by invoking each package’s build in a subshell or process where the environment variables have been adjusted to include all *previous* packages’ install prefixes. The user does not need to intervene and “source” anything manually during the build; colcon does it for you under the hood.

**After the build (for runtime):** Once `colcon build` finishes, the workspace’s `install/` directory contains the installed results of all packages. To use the built packages (run executables, etc.), you need to source the setup script that colcon generated in the install directory. Colcon produces two levels of setup scripts:

* **Package-level setup scripts:** For each package, colcon writes a script at `install/<pkg>/share/<pkg>/package.sh` (and `.bat`, etc. for other shells). This script, if sourced, will add that single package’s paths to your environment (it’s generated using the heuristics and any package-specific hooks, as described above). It’s a standalone environment setup for that package.
* **Workspace (prefix)-level setup scripts:** Colcon also generates a top-level `setup.(bash|sh|zsh|ps1|bat)` in the `install/` directory. This is the one you typically source after a build (`source install/setup.bash`). What this does is invoke all the package-level scripts in the correct order. Specifically, colcon creates a `local_setup.<ext>` which handles *all* packages under that install prefix by calling each package’s script in topological order. The ordering is important because some package’s environment hooks might depend on others (for example, package A’s script might assume B’s variables are already set). Colcon ensures this by storing each package’s **runtime dependencies** in a metadata file (`share/colcon-core/packages/<pkg_name>`), which the `local_setup` script uses to determine the sourcing order. Finally, the `setup.bash` can chain workspaces (underlays): it will first call any parent workspace’s setup (if you have an underlay), then the local `local_setup.bash`. This mechanism allows overlaying multiple workspaces seamlessly.

In summary, colcon’s environment handling is comprehensive:

* **Isolation:** Each package is built and installed in isolation, but colcon stitches the environments together as needed.
* **Build time:** Environment is auto-configured so that dependencies are found (no manual sourcing needed in between).
* **Post build:** Convenient setup scripts are generated so the user can easily switch into the new environment to run the software.

### Logging and Build Output

Colcon provides detailed logging for each build. By default, colcon’s console output is relatively concise – it will typically print each package name as it’s being built and note whether the build succeeded or failed, but it may not show the full compiler output for each file (unless an error occurs or you use verbose settings). Instead, full logs are written to the `log/` directory.

In the `log/` directory (often `log/latest` for the last build, or with timestamp-named folders), you will find:

* **Per-package logs:** e.g. `log/latest/<pkg_name>/stdout_stderr.log` which contains the stdout and stderr from that package’s build commands (compilation output, etc.), or CMake’s configure log. These are invaluable for debugging build failures, as you can inspect exactly what went wrong.
* **Aggregate logs:** colcon also logs overall events, timings, and status in files like `log/latest/colcon.out` or `colcon.log`.
* **Metadata:** There are files describing the build configuration, the package list, dependency graph, etc., as understood in that run.

Colcon uses an event-handler system for output. For instance, there is a `console_cohesion` event handler that can keep a package’s log output hidden until an error occurs, to keep the console tidy. Users can customize this (e.g. `colcon build --event-handlers console_direct+` to see all output live, or use `colcon build --log-base` to control log location). The logging ensures even if you run parallel builds, outputs don’t intermix chaotically; they are captured per package.

If a build fails, colcon will print a summary line like “Failed   \<package\_name>” and point to the log file. This separation of concerns (minimal console output vs. detailed logs on disk) is helpful for managing large workspaces.

## Re-Implementing Colcon’s Functionality in Rust (Step-by-Step Guide)

Creating a Rust-based drop-in replacement for `colcon build` means implementing the same core features:

* Workspace and package discovery
* Dependency graph resolution and topological ordering
* Integration with various build systems (CMake for C++ packages, setuptools for Python packages, etc.)
* Isolated build and install spaces
* Environment setup (both during build and generating the final setup scripts)
* Logging and user feedback (status messages, error reporting)

Below is a step-by-step outline of how to achieve this, including architectural suggestions, relevant Rust libraries, and considerations for edge cases.

### 1. Package Discovery and Manifest Parsing

**Goal:** Identify all ROS 2 packages in the workspace and extract their metadata.

**Approach:** Recursively scan the workspace directory for `package.xml` files. In ROS 2, the presence of `package.xml` is the definitive way to identify a package. Commonly, workspaces have a `src/` directory with many subfolders, but we should not assume it’s only in `src/` – allow the user to specify the root(s) to search (similar to colcon’s `--base-paths` argument).

**Implementation details:**

* Use the Rust `walkdir` crate or `glob` to traverse the directory tree.
* Skip any directories containing a `COLCON_IGNORE` file (to mimic colcon’s ignore mechanism).
* For each `package.xml` found, parse the XML. Rust has good XML libraries like `quick-xml` or `roxmltree` that can parse XML efficiently. For example, using `roxmltree`:

  ```rust
  use roxmltree::Document;
  let xml_text = std::fs::read_to_string(package_xml_path)?;
  let doc = Document::parse(&xml_text)?;
  let name_node = doc.descendants().find(|n| n.has_tag_name("name"))
      .expect("package.xml missing <name>");
  let pkg_name = name_node.text().unwrap().to_string();
  // Similarly, parse <version>, <maintainer>, etc., if needed.
  ```
* While parsing, collect **dependencies**. The manifest format (REP 149) uses tags like `<build_depend>`, `<exec_depend>`, etc. Each dependency tag contains another package’s name. We will want to gather at least:

  * Build dependencies (needed to compile/link)
  * Build tool dependencies (especially to know if a package uses ament\_cmake or ament\_python; e.g. `<buildtool_depend>ament_cmake</buildtool_depend>` often appears in ROS 2 C++ packages)
  * Exported dependencies (if any, though usually those overlap with build/export depends).
  * For simplicity, we might treat all non-exec dependencies as needing to be built before the package.
* Also retrieve the **build type**. In ROS 2, this is indicated in the `<export>` section, e.g.:

  ```xml
  <export>
    <build_type>ament_cmake</build_type>
  </export>
  ```

  If no build\_type is explicitly given, we may infer one:

  * If the package contains a `CMakeLists.txt`, assume `ament_cmake` or `cmake`.
  * If it contains a `setup.py` and no CMake, assume `ament_python`.
  * Or default to `ament_cmake` for C++ packages, as colcon does for ROS 2 (since almost all C++ use ament\_cmake).
* Store the metadata in a struct for each package, e.g.:

  ```rust
  struct PackageMeta {
      name: String,
      path: PathBuf,          // path to package root
      build_type: BuildType,  // enum { AmentCmake, AmentPython, CMake, ... }
      deps: Vec<String>,      // names of packages it depends on (build + build_export deps)
      exec_deps: Vec<String>, // runtime deps (might be needed for environment later)
  }
  ```

**Rust crates to use:**

* `walkdir` for filesystem traversal.
* `roxmltree` or `quick-xml` for XML parsing.
* Possibly `regex` or simple string search if performance is a concern (but XML is more robust to parse properly).
* Optionally, `clap` or `structopt` for command-line argument parsing, to handle user specifying custom paths or package selection (e.g., `--packages-select`).

**Edge Cases & Checks:**

* Duplicate package names: ROS 2 forbids two packages with the same name in one workspace. Your tool should detect if two manifests have the same `<name>` and warn or error.
* Missing `<build_type>`: You might need logic to guess or default the build type.
* Package format versions: ROS 2 supports format 2 and 3 manifests. Ensure the parser can handle both (they’re similar; format 3 just added condition attributes and group dependencies).
* Conditional dependencies (if using format 3, dependencies can have conditions like `condition="$ROS_VERSION == 2"`). A thorough implementation would evaluate conditions (probably using environment variables). At minimum, you might ignore dependencies that are conditionally not meant for this context.
* Non-ROS packages: Colcon can include “packages” that are pure CMake with no manifest if explicitly told. For a drop-in replacement, you may not need that initially. Focusing on actual ROS packages is fine (since ROS 2 ament requires manifest anyway).

### 2. Constructing the Dependency Graph

**Goal:** Determine the build ordering by creating a directed graph of package dependencies.

**Approach:** Using the metadata collected, create a graph where nodes are packages and directed edges indicate “depends on”. That is, an edge from A → B means A depends on B (so B must be built before A).

**Implementation details:**

* Use a graph library like `petgraph` for convenience, or implement a simple topological sort manually (the graph is likely not huge in most cases, so a manual DFS or Kahn’s algorithm is fine).
* Build the graph: for each package meta, for each dependency name in `deps`:

  * If the dependency name exists as another package in the workspace, add an edge from this dependency to the package.
  * If the dependency is **not** in the workspace, mark it as an external dependency. External deps are assumed to be provided by the underlay (or system). They don’t need to appear as nodes that must be built. However, they are still relevant for environment setup (they might have to be in the environment, e.g. if A depends on system library Boost, you’d expect CMake to find Boost via system paths).
  * You might choose to include a node for external dependencies just to detect if something is missing: e.g., an external dep node with no incoming edges, so if it was supposed to be built but isn’t found, you can error. But since colcon doesn’t resolve system deps, you likely assume they exist. Perhaps issue a warning if an external dependency is not found in the environment (this could be complex to check reliably).
* Once the graph is built, perform a **topological sort**. Petgraph has `algo::toposort`, or you can do:

  ```rust
  let sorted = petgraph::algo::toposort(&graph, None)
      .expect("Cycle detected in package dependencies");
  ```

  This returns the nodes in an order such that all dependencies come before dependents.
* If a cycle is detected, print a clear error indicating the cycle (colcon would say something like “Circular dependency between A and B”). You can detect cycles either via the toposort failing or by using something like Tarjan’s strongly connected components algorithm to find loops.

**Parallel builds:** You might at this stage decide how to handle parallelization:

* A simple approach is to build strictly in sequence (which guarantees correctness).
* For performance, you can identify independent subgraphs. For example, if after toposort you know some sets of packages have no inter-dependencies, you could build them concurrently. This requires thread management in Rust:

  * The `rayon` crate could be used to iterate in parallel over independent tasks.
  * Or simply spawn threads for builds, with a mechanism to ensure prerequisites are done (this essentially re-implements what a build executor does – maybe start with sequential execution, then add parallelism once basics work).

**Edge Cases & Checks:**

* If a package lists a dependency that is neither in the workspace nor in the underlay environment, the build will likely fail when attempting to find it. You could proactively check for known environment markers (for example, if dependency name matches a ROS package that should have been in an underlay, perhaps check an environment variable like `AMENT_PREFIX_PATH` or call `ros2 pkg` to see if it’s installed). However, since the question scope is the build tool, you can document that “missing external dependencies will result in build errors – the user should source the appropriate underlays or install missing packages.”
* The graph should incorporate **build tool dependencies** like `ament_cmake` itself. Often, each package declares a buildtool depend on `ament_cmake` or `ament_python`. You might want to ignore those in terms of ordering (because typically `ament_cmake` is an external package provided by ROS itself, and it should be already in the environment). Colcon likely doesn’t attempt to build `ament_cmake` as part of your workspace unless you actually have the source of `ament_cmake` as a package in your workspace (which could happen if building ROS 2 from source). In that case, it needs to treat it like any other package.

### 3. Preparing the Build Environment (Workspace Context)

**Goal:** Ensure that the environment variables are correctly set before building each package, and set up the mechanism for generating the final environment setup scripts.

This step has two facets:

* **Build-time environment:** update `PATH`, `CMAKE_PREFIX_PATH`, etc., as we go from one package to the next during the build process.
* **Post-build environment scripts:** create `package.sh`, `local_setup.sh`, `setup.sh` in the install dir, mimicking colcon’s output, so the user can `source install/setup.bash` and get all the paths.

**Build-Time Environment Automation:**
Your Rust tool can manage environment variables in the process that invokes the build commands:

* In Rust, `std::process::Command` allows you to set env vars for the command execution (via `.env("VAR", "value")` or `.envs(some_hashmap)`).
* Maintain a persistent mapping of environment variables as you iterate through the topologically sorted packages. Initially, this map can be populated from the **current environment** (inherited from the user’s shell, which presumably has the underlay sourced). For example, if the user has an existing `AMENT_PREFIX_PATH` or `LD_LIBRARY_PATH`, carry those forward.
* Before building package *P*, modify the env map to include P’s dependencies’ install locations. One straightforward way:

  * When a package finishes building and installing, record its install prefix (e.g. `/path/to/ws/install/<pkg>`).
  * Also record any specific information from that package that might require special handling (for instance, if the package has an environment hook script, you could actually execute that script in a subshell to see what it exports – but it might be easier to replicate what it would do).
  * For key environment vars:

    * Prepend `install/<dep_pkg>/bin` to `PATH` if exists.
    * Prepend library dirs (e.g. `install/<dep_pkg>/lib`) to `LD_LIBRARY_PATH` (or `PATH` on Windows for DLLs).
    * Prepend Python dirs (`install/<dep_pkg>/lib/pythonX.Y/site-packages` or similar) to `PYTHONPATH`.
    * Prepend CMake prefix (`install/<dep_pkg}`) to `CMAKE_PREFIX_PATH`.
    * Append pkgconfig path (`install/<dep_pkg>/lib/pkgconfig`) to `PKG_CONFIG_PATH`.
  * These are analogous to colcon’s heuristics. You need to be careful to use the right separator (`:` on Linux, `;` on Windows) for PATH-like variables.
* Another approach is to actually **source the package’s `package.sh` (if it exists)** in a shell and capture the env changes. For example, after installing a package, its `share/<pkg>/package.sh` is meant to add env vars. You could run something like `bash -c 'source install/pkg/share/pkg/package.sh && env'` and diff the environment before/after. However, this is complex and inefficient. It’s better to implement the same logic in Rust based on known patterns (which are documented as above).
* Ensure that these environment updates occur *before* invoking the next package’s build. So essentially, loop through sorted packages:

  ```rust
  let mut env_vars = inherit_env(); // clone current process env
  for pkg in sorted_packages {
      // Set up env for this package build
      for dep in pkg.deps {
          if let Some(dep_inst_path) = install_path_for(&dep) {
              augment_env_for_dep(&mut env_vars, &dep_inst_path);
          }
      }
      // Now run the build commands for pkg with env_vars
      let status = Command::new(cmd)
                        .args([...])
                        .envs(&env_vars)
                        .status()?;
      if !status.success() { ... handle error ... }
      // After build, add pkg itself to env (for later packages)
      let pkg_inst_path = workspace_install.join(&pkg.name);
      record_install_path(pkg.name, pkg_inst_path);
      augment_env_for_dep(&mut env_vars, &pkg_inst_path);
  }
  ```

  This simplistic pseudo-code doesn’t account for some nuance (e.g., not all dependencies should be added each time — you might instead maintain a global set of already-added paths), but it sketches the idea.

**Generating Environment Scripts (Post-build):**
Colcon produces `package.<ext>` and `setup.<ext>` scripts. We should do the same to be a drop-in replacement:

* For each package, create `install/<pkg>/share/<pkg>/package.sh` (and a `.bat` and `.ps1` if targeting Windows and PowerShell, for completeness). This script should set environment variables exactly as was done during build for that package. Essentially, it should contain lines to export or prepend paths. For example:

  ```bash
  # install/my_pkg/share/my_pkg/package.sh
  # (assuming Unix shell)
  export CMAKE_PREFIX_PATH="{$CMAKE_PREFIX_PATH:+$CMAKE_PREFIX_PATH:}/path/to/ws/install/my_pkg"
  export LD_LIBRARY_PATH="{$LD_LIBRARY_PATH:+$LD_LIBRARY_PATH:}/path/to/ws/install/my_pkg/lib"
  export PYTHONPATH="{$PYTHONPATH:+$PYTHONPATH:}/path/to/ws/install/my_pkg/lib/python3.10/site-packages"
  export PATH="{$PATH:+$PATH:}/path/to/ws/install/my_pkg/bin"
  ```

  You would generate these lines based on what files were present in the package’s install. This is replicating what colcon’s heuristics do. You don’t necessarily need to replicate it 100% (colcon covers a lot of edge cases), but covering the main ones (bin, lib, share for CMake, etc.) will suffice for most packages.
* Also consider any **package-specific hooks**: If a package installed its own `local_setup.sh` or other scripts (common for ament\_cmake), you might have your `package.sh` source that as well. In colcon, `package.sh` often sources `local_setup.sh` for ament packages. You could adopt a convention: if `share/<pkg>/local_setup.sh` exists, include a line like `. "$COLCON_CURRENT_PREFIX/share/<pkg>/local_setup.sh"` in the generated script (and define `COLCON_CURRENT_PREFIX` accordingly). This ensures that any environment logic provided by the package itself is executed.
* For the entire workspace, create:

  * `install/local_setup.sh`: this script should iterate over all packages in topological order and source each package’s `package.sh`. Colcon actually avoids hardcoding the order in the script; instead it stores dependency info and calculates order at runtime by sourcing each package script in turn (since sourcing 50+ scripts in a shell is not too slow). However, for simplicity, you could generate a static ordered sourcing:

    ```bash
    # in install/local_setup.sh
    . "$(dirname "$0")/share/pkgA/package.sh"
    . "$(dirname "$0")/share/pkgB/package.sh"
    ...
    ```

    The order listed should respect that dependencies come before dependents. (Colcon uses the files in `share/colcon-core/packages/` to do this dynamically, but generating a static list using the known topo order should be fine as long as it’s updated on each build).
  * `install/setup.sh` (and .bash, .zsh variants): This should first source the *underlay* workspace’s setup, if any, then source this workspace’s `local_setup.sh`. In ROS 2, the pattern is:

    ```bash
    #! /usr/bin/env sh
    # Generated file
    # Source parent workspaces if any:
    _colcon_previous_setup="$COLCON_CURRENT_PREFIX" 
    export COLCON_CURRENT_PREFIX="$(dirname "$0")"
    . "$COLCON_CURRENT_PREFIX/local_setup.sh"
    export COLCON_CURRENT_PREFIX="$_colcon_previous_setup"
    ```

    You might not need the complexity of COLCON\_CURRENT\_PREFIX if not supporting multi-workspace overlay, but it’s good to match behavior so that nested overlays work. Essentially, `setup.sh` calls any underlay’s `setup.sh` (this is done by the lines above in colcon’s generated scripts).
    If you want to mirror colcon exactly, you would also generate a `.dsv` (differential setup variables) file and have the shell scripts parse it. But that’s an internal detail that may not be needed if your generator directly writes the shell script lines.

**Rust crates to use:**

* You can generate shell scripts simply by writing text files (no special crate needed, but ensure to get line endings correct for `.bat` vs `.sh`).
* Use `which` crate if you need to find system executables (like finding `cmake` in PATH).
* Possibly `tempfile` if you need to call external shell to source scripts and capture env (but as discussed, better to replicate logic directly).

**Edge Cases & Platform considerations:**

* Windows: On Windows, environment variables like `PATH` are used instead of `LD_LIBRARY_PATH`. Also, `COLCON_CURRENT_PREFIX` logic is slightly different in `.bat` files. If aiming for cross-platform, you need to produce `.bat` and possibly PowerShell `.ps1` scripts. This is a fair amount of extra work; if focusing on Unix-like first, note the limitation.
* Avoid duplicate PATH entries: Colcon’s environment setup tries not to add duplicate entries. A simple approach is to check if a path is already in the variable before adding.
* Shebangs and execution bits: Ensure generated scripts have proper shebangs if needed (though since they are meant to be sourced, not executed directly, shebang is not critical).
* Considering group dependencies (REP 140): If package.xml uses `<group_depend>`, you should treat all dependencies in that group as individual edges (it’s basically sugar in manifest, you can expand them).
* Not interfering with user’s environment: The environment modifications should ideally only affect the subprocess doing the build. Don’t export variables in the parent process running the tool (unless you want the Rust tool itself to modify the user’s shell, which isn’t typical – the user should rather source the generated setup script after the tool finishes).

### 4. Executing Package Builds in Order

**Goal:** For each package in topological order, invoke the appropriate build commands and handle success or failure.

**Approach:** Use Rust’s process management to call external build tools.

For each package (after setting up env for it, as above):

* **Create the build directory** for the package: e.g. `<ws>/build/<pkg>`. Use `std::fs::create_dir_all` to ensure it exists (colcon will have created this).
* **Invoke CMake (for ament\_cmake or plain cmake)**:

  * Ensure `cmake` is installed and available (you might call out to `cmake --version` first to verify, or handle failure gracefully).
  * Construct the command:

    ```rust
    let status = Command::new("cmake")
        .arg("-S").arg(&pkg.path)                // source path
        .arg("-B").arg(format!("build/{}", pkg.name))  // build path
        .arg(format!("-DCMAKE_INSTALL_PREFIX={}", install_prefix))
        .args(&additional_cmake_args)
        .envs(&env_vars_for_pkg)  // important: pass the environment
        .status()?;
    if !status.success() { /* handle error */ }
    ```

    The additional CMake args might include build type (Release/Debug), or any user-provided args (maybe your tool can accept `--cmake-args "..."` to pass through, similar to colcon).
  * Then run the build:

    ```rust
    Command::new("cmake")
        .arg("--build").arg(format!("build/{}", pkg.name))
        .arg("--target").arg("install")
        .args(&["--", "-j", num_jobs])  // if using Makefile or Ninja, pass parallel jobs
        .envs(&env_vars_for_pkg)
        .status()?;
    ```

    (The syntax for `--` and `-j` might differ if Ninja is the generator. Possibly use `ninja` directly if you know Ninja is being used. But using `cmake --build` is a nice generator-agnostic way.)
* **Invoke Python setup (for ament\_python)**:

  * Make sure Python is available (maybe use `PYTHON_EXECUTABLE` from environment or just call `python3`).
  * Change directory to the package’s source (or specify working directory via `Command::current_dir`).
  * You can either call `python setup.py bdist_wheel` and then install, or directly `setup.py install`. Colcon likely does an install to the install prefix. A modern approach might be:

    ```rust
    Command::new("python3")
        .arg("setup.py")
        .arg("build")
        .arg("--build-base").arg(format!("../../build/{}", pkg.name)) // relative path or absolute
        .current_dir(&pkg.path)
        .envs(&env_vars_for_pkg)
        .status()?;
    Command::new("python3")
        .arg("setup.py")
        .arg("install")
        .arg("--prefix").arg("")      // prefix empty when using --root
        .arg("--root").arg(format!("../../install/{}", pkg.name))
        .current_dir(&pkg.path)
        .envs(&env_vars_for_pkg)
        .status()?;
    ```

    Here, we simulate what `--prefix "" --root install/pkg` does: it installs into our install directory. Another approach is using `pip`: `pip install . --target install/pkg/lib/pythonX.Y/site-packages` etc., but using setup.py directly is closer to how ament\_python works in Foxy.
  * Be mindful of entry-point scripts: ament\_python’s installation will place console scripts into `install/<pkg>/lib/<pkg>` or `install/<pkg>/bin`. Ensure those are handled by your environment (they will be, if you add `bin` to PATH as above).
* **Other build systems**: If you encounter a package with `build_type="ament_cmake_auto"` or others, they usually still use CMake under the hood. So treat them as ament\_cmake. If it’s `catkin` (ROS 1), that’s outside ROS 2 ament context, but colcon *can* build ROS 1 catkin packages by treating them via a plugin. For a drop-in in ROS 2, you might not need to handle catkin, but if you wanted to, you’d call `catkin_make_isolated` or better use `catkin_tools` logic. This gets complicated, so probably skip ROS 1 support unless required.

**Monitoring and Logging:**

* While executing these commands, capture their output. In Rust, you can use `Command::stdout` and `stderr` to capture output streams. However, capturing everything in memory is not ideal for large compile outputs. Instead, you can direct output to files:

  * Open a file like `log/<pkg>/stdout_stderr.log` (create the directory structure similar to colcon’s).
  * Use `Command::stdout(File)` and `Command::stderr(File)` to pipe outputs there. Alternatively, run the command and stream its output line by line, echoing minimal info to console (e.g., you could print a single line “\[pkg] compiled X.cpp” if needed).
* Consider using a logging crate. The \[`flexi_logger`], \[`log`] + \[`simplelog`], or even just writing to file manually with `std::fs::File` and `write!` as you capture output from the child process.
* One idea: run the build command via `std::process::Command::spawn` (so it’s asynchronous), then read from its `stdout` pipe in real-time. You can then both write to the log file and optionally echo to terminal. If doing parallel builds, tag each line with package name or use a mutex to serialize console output per package to avoid jumbled text.

**Handling Failures:**

* If any build command returns a non-zero exit code, record that as a failure. You might want to stop the build of further packages (since others depending on it cannot succeed). Colcon stops on failure by default.
* Provide a clear error message: e.g., “❌ Build failed for package X. See log/X/stdout\_stderr.log for details.” Possibly also print the last few lines of that log to give a hint.
* Optionally, implement an argument like `--continue-on-error` to mimic colcon’s `--continue-on-error` behavior (if it exists), but not mandatory.

**Edge Cases & Additional Features:**

* **Rebuild/Incremental**: If the user runs the tool again, you might want to avoid re-building packages that haven’t changed. Colcon always runs through, but CMake will no-op if nothing’s changed. For efficiency, you could check timestamps or package manifest changes. However, a first implementation can simply call through and rely on CMake’s internal checks.
* **Cleaning**: Colcon doesn’t automatically clean unless asked. If you need a clean build, the user would delete the build/ and install/ directories or use a separate tool or invocation. You could implement a `--clean` flag to wipe before building.
* **Testing**: Colcon has `colcon test` as a separate verb. You don’t need to implement that for a build tool replacement, but you could allow an option to run tests after building (basically iterate with `ctest` for each package).
* **Install Package to prefix**: The approach above always installs into the workspace’s install dir. That’s correct for ROS. Just make sure the CMake configure step uses the intended `CMAKE_INSTALL_PREFIX` (some ament\_cmake packages might rely on environment variable `AMENT_PREFIX_PATH` as well, which your env handling will cover).
* **Console output formatting**: For user-friendliness, print when a package starts and finishes building, similar to colcon. For example:

  ```
  --- Building package: foo_msgs (ROS package) ---
  ... (maybe print nothing or minimal stuff during build) ...
  --- Finished package: foo_msgs [0.4s] ---
  ```

  Use color (via `termcolor` crate) or simple ASCII banners to highlight failures vs successes.

### 5. Providing Colcon-like User Output and Logs

Mimicking colcon’s user experience involves clear log management and console messages.

**Console UI:**

* Print a summary of what will be built (e.g., “Found 12 packages in workspace, building 10 (skipped 2 already up-to-date or not selected)…” if implementing such logic).
* As mentioned, print each package’s progress. Colcon by default prints lines like:

  ```
  Starting >>> my_package
  Finished <<< my_package [3.2s] 
  Starting >>> next_package
  ```

  You can do similar. Keep the format consistent if possible so that it feels like a drop-in replacement.
* If parallel builds are on, you might print “Starting >>> pkg1 & >>> pkg2 (in parallel)” etc., but careful with output mixing. Colcon’s `console_cohesion` ensures logs from parallel builds don’t intermix; you may have to implement something similar (perhaps buffer each package’s output and only flush it on completion or on error). This can be achieved by having each build run in a separate thread and capturing its output separately.

**Logs:**

* Write logs to a `log/` directory in the workspace. Possibly mimic colcon’s structure:

  * `log/latest` symlink to the latest timestamped log dir.
  * Or just always use `log/` for current to simplify.
* As mentioned, create per-package log files capturing stdout/stderr.
* Log the environment variables for each build if possible (this helps debugging environment issues). Colcon often logs a file listing the exact env for each package’s build.
* Also log a top-level build summary (list of packages built, time taken, any failed).

**Rust tools for logging:**

* The `log` crate with a backend (like `env_logger` or `fern` or `simplelog`) could be used, but since we want per-package separation, manual file management might be easier.
* You could use `indicatif` crate to show a progress bar or spinner for each package – but in a TTY with multiple packages, that might be overkill. Simpler textual output is fine.

**Error Reporting:**

* If a package fails, ensure the exit code of your tool is non-zero (so CI systems know the build failed).
* Possibly implement an option to summarize results at end: e.g., “9 packages succeeded, 1 failed.” Colcon has `colcon build --event-handlers summary` by default which prints that.
* Provide guidance: e.g., “Check \[path to log] for details” so user knows where to look.

### 6. Key Architecture and Libraries in Rust

To build this tool in Rust effectively, consider organizing the code into components akin to colcon’s extension points:

* **Package Discovery Module:** Responsible for scanning directories and parsing `package.xml`.
  *Rust helpers:* `walkdir`, `roxmltree`.
  *Output:* a list of `PackageMeta` structs.
* **Dependency Graph Module:** Takes `PackageMeta` list and constructs a graph/topological order.
  *Rust helpers:* `petgraph` for graph and topo sort, or custom DFS.
  *Output:* an ordered list of packages (or an iterator that yields in order).
* **Build Executor Module:** Handles invoking external processes for each package.
  *Rust helpers:* `std::process::Command`, maybe `duct` crate (which provides a nicer interface for piping processes), `indicatif` for progress.
  Consider defining a trait like:

  ```rust
  trait BuildRunner {
      fn configure(&self, pkg: &PackageMeta, env: &HashMap<String,String>) -> Result<()>;
      fn build(&self, pkg: &PackageMeta, env: &HashMap<String,String>) -> Result<()>;
      // Perhaps a combined run that does both configure & build
  }
  ```

  and implement it for each BuildType (AmentCmakeRunner, AmentPythonRunner, etc.). This mimics colcon’s extensibility: you could select the runner based on `pkg.build_type`.
* **Environment Manager:** Utility functions for environment variable management and script generation.
  *Rust helpers:* none specific, just careful string handling and file writing.
* **Logging/Output Module:** Could encapsulate writing to log files and printing to console. For example, a struct that holds `stdout: File` and `stderr: File` for the current package’s log and implements `std::io::Write`, allowing you to easily write to both console and file by wrapping it.

**Crates and Libraries Summary:** *(We compile some of the above into a quick reference table for clarity.)*

| Concern                          | Rust Libraries/Tools                         | Notes                                                                                                         |
| -------------------------------- | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| CLI argument parsing             | `clap` or `structopt` (derive-based)         | e.g. to handle flags like `--merge-install`, `--parallel`                                                     |
| XML parsing (`package.xml`)      | `roxmltree`, `quick-xml`, or `xml-rs`        | `roxmltree` is high-level and easy for small XML files.                                                       |
| File traversal                   | `walkdir`                                    | Recursively find package.xml files.                                                                           |
| Dependency graph & topo          | `petgraph` (for complex needs) or custom     | Petgraph’s DAG algorithms simplify topo sort.                                                                 |
| Process execution                | `std::process::Command` or `duct` crate      | `duct` can simplify piping outputs. Otherwise, use `Command` for full control.                                |
| Concurrency (if parallel builds) | `rayon` or manual threads + channels         | Could also use `tokio` if async, but not necessary here.                                                      |
| Output capture & logging         | `log` with `simplelog` or manual file writes | Could use `simplelog` to log to file and console, but interleaving might be tricky – manual might be simpler. |
| Terminal output styling          | `termcolor`, `indicatif`                     | For colored statuses or progress bars.                                                                        |
| Shell script generation          | No special crate (just format strings)       | Pay attention to OS differences.                                                                              |

**Memory and performance considerations:** Building can be heavy on CPU but the build tool itself will not be memory-intensive. The largest overhead might be reading many small XML files and spawning many processes. Rust is well-suited for this as it has low overhead in these areas. Ensure not to copy large strings unnecessarily (but again, package.xml files are tiny, so not an issue). Perhaps implement some caching of parsed manifests if you anticipate re-running many times (but likely not needed).

**Extensibility:** If desired, design your Rust tool such that adding a new build system is straightforward. For example, you could have an enum `BuildType` and match on it in a centralized build function, or use the trait approach as mentioned. The trait approach would be closer to colcon’s plugin system (you could even load dynamic libraries for new build types, though that’s probably overkill). For a drop-in replacement focused on ROS 2, supporting `ament_cmake`, `ament_python`, and `cmake` covers 99%. Possibly also support `ament_gradle` if any Java ROS 2 packages exist, or `ament_ros` which is more of an umbrella.

**Testing the tool:** Use a sample workspace (perhaps ROS 2 demo packages) to verify that:

* The build completes successfully and the resulting `install/` works when sourced (try running a `ros2 run` on an executable).
* Introduce a failure to see if logging catches it.
* Test with `--merge-install` scenario (your tool should respect a config to install all into one prefix if requested — that means you’d adjust `CMAKE_INSTALL_PREFIX` to the same path for all and tweak environment logic accordingly).
* Ensure idempotency: running the tool twice without cleaning should ideally not rebuild everything from scratch (unless code changed). This can be checked by seeing that the second run is very fast or that CMake says “up-to-date”.

### 7. Edge Cases and Constraints

Be aware of the following while implementing:

* **Workspace overlays:** In ROS 2, you can have an underlay workspace (sourced before building the overlay). Your tool should allow that (which it does by inheriting environment). If someone wanted to build two workspaces and overlay them, your environment scripts logic should handle chaining (the way we generate `setup.sh` above).
* **Large number of packages:** Colcon can handle hundreds of packages. Make sure to optimize accordingly:

  * Reading XML and building graph is trivial even for 1000 packages.
  * Spawning processes for each package is fine; just avoid spawning more processes than necessary (don’t, for example, spawn a shell just to call another command if you can call it directly).
  * If parallelizing, watch out for CPU oversubscription (building 8 heavy C++ projects simultaneously on 4 cores may thrash performance, so perhaps default parallel jobs to number of cores or allow user override).
* **Path lengths on Windows:** If supporting Windows, consider using `--merge-install` automatically if many packages to avoid MAX\_PATH issues (colcon does this for you with a warning).
* **Output on Windows vs Linux:** The command invocations for `.bat` may differ (calling `vcvars64.bat` for Visual C++ environment, etc.). Colcon requires running in a Visual Studio Developer shell for Windows builds. Your tool could detect OS = Windows and verify `CL.exe` or appropriate env is present.
* **Rust tool distribution:** If you intend to truly replace colcon, you’ll distribute this tool as a binary (colcon is Python, distributed via pip/apt). Ensure your binary can be easily installed (maybe via `cargo install`). Also ensure it accepts similar CLI arguments as colcon for familiarity.

### 8. Example Workflow of the Rust Tool

To tie it all together, here’s what an example run might look like (pseudocode mixing our steps):

```rust
fn main() -> Result<()> {
    // 1. Discover packages
    let packages = discover_packages("./src")?;
    println!("Found {} packages in workspace", packages.len());
    // 2. Build dependency graph and order
    let build_order = order_packages(&packages)?;
    println!("Building packages in topological order: {:?}", build_order.iter().map(|p| p.name).collect::<Vec<_>>());
    // 3. Prepare environment
    let mut env = std::env::vars().collect::<HashMap<_,_>>();  // start from current env
    let mut install_paths = HashMap::new();
    // 4. Iterate and build
    for pkg in build_order {
        println!("Starting >>> {} ({})", pkg.name, pkg.build_type);
        let pkg_build = BuildRunner::for_type(pkg.build_type);
        // Augment env with all deps already built:
        for dep in &pkg.deps {
            if let Some(dep_path) = install_paths.get(dep) {
                augment_env(&mut env, dep_path);
            }
        }
        // Configure & build
        pkg_build.configure(pkg, &env).map_err(|e| format!("Failed configuring {}: {}", pkg.name, e))?;
        let status = pkg_build.build(pkg, &env)?;
        if !status.success() {
            log_failure(&pkg.name);
            return Err(format!("Package {} failed to build", pkg.name).into());
        }
        // Mark this package as built
        let inst = format!("{}/install/{}", workspace_root, pkg.name);
        install_paths.insert(pkg.name.clone(), inst);
        augment_env(&mut env, install_paths.get(&pkg.name).unwrap());
        println!("Finished <<< {} [{:.2}s]", pkg.name, pkg_build.last_build_duration());
    }
    // 5. After loop, generate environment scripts
    generate_all_setup_scripts(&install_paths, &packages, workspace_root)?;
    println!("Build Succeeded. Output at {}/install", workspace_root);
}
```

*Note:* The above is highly simplified pseudocode, but it shows the general flow aligned with our plan.

By following these steps and recommendations, you can implement a robust Rust-based build tool that mirrors the functionality of `colcon build` for ROS 2 ament packages. The end result will allow ROS developers to build their entire workspace with a single command, achieving the same outcomes (artifacts and environment setup) as colcon, but leveraging Rust’s performance and safety for the implementation.
