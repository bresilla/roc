# Dependency Resolution

ROC's dependency resolution system builds a comprehensive dependency graph from discovered packages and determines the optimal build order. This chapter explains the algorithms and strategies used for dependency management.

## Dependency Graph Construction

### Graph Representation

The dependency system uses a directed graph where:
- **Nodes**: Represent packages in the workspace
- **Edges**: Represent dependencies (A → B means A depends on B)
- **Direction**: Dependencies point from dependent to dependency

```rust
pub fn topological_sort(packages: &[PackageMeta]) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    let mut name_to_index: HashMap<String, usize> = HashMap::new();
    let mut graph: Vec<Vec<usize>> = vec![Vec::new(); packages.len()];
    let mut in_degree: Vec<usize> = vec![0; packages.len()];
    
    // Build name to index mapping
    for (idx, package) in packages.iter().enumerate() {
        name_to_index.insert(package.name.clone(), idx);
    }
    
    // Build dependency graph
    for (pkg_idx, package) in packages.iter().enumerate() {
        for dep_name in &package.build_deps {
            if let Some(&dep_idx) = name_to_index.get(dep_name) {
                graph[dep_idx].push(pkg_idx);
                in_degree[pkg_idx] += 1;
            }
            // External dependencies are ignored (assumed available)
        }
    }
}
```

### Dependency Types

The system considers multiple types of dependencies when building the graph:

#### Build Dependencies (`build_depend`)
- Required for compilation/building
- Must be built before dependent package
- Include headers, libraries, and build tools

#### Build Tool Dependencies (`buildtool_depend`)
- Build system tools (cmake, ament_cmake, etc.)
- Usually external to workspace
- Considered for ordering if present in workspace

#### Runtime Dependencies (`exec_depend`)
- Required at runtime
- Not directly used for build ordering
- Important for environment setup

### External Dependencies

Dependencies not found in the workspace are treated as external:
- Assumed to be available in the environment
- Not included in build ordering
- May trigger warnings if expected but missing

## Topological Sorting Algorithm

### Kahn's Algorithm Implementation

ROC uses Kahn's algorithm for topological sorting, which is efficient and provides clear cycle detection:

```rust
// Kahn's algorithm for topological sorting
let mut queue: VecDeque<usize> = VecDeque::new();
let mut result: Vec<usize> = Vec::new();

// Add all nodes with no incoming edges
for (idx, &degree) in in_degree.iter().enumerate() {
    if degree == 0 {
        queue.push_back(idx);
    }
}

while let Some(current) = queue.pop_front() {
    result.push(current);
    
    // Remove this node and update in-degrees
    for &neighbor in &graph[current] {
        in_degree[neighbor] -= 1;
        if in_degree[neighbor] == 0 {
            queue.push_back(neighbor);
        }
    }
}
```

### Algorithm Benefits

Kahn's algorithm provides several advantages:

1. **Cycle Detection**: Incomplete result indicates circular dependencies
2. **Efficiency**: O(V + E) time complexity
3. **Stability**: Consistent ordering for the same input
4. **Parallelization**: Can identify independent packages for parallel builds

## Cycle Detection and Resolution

### Circular Dependency Detection

Circular dependencies are detected when the topological sort fails to include all packages:

```rust
// Check for cycles
if result.len() != packages.len() {
    // Find the cycle
    let remaining: Vec<String> = packages
        .iter()
        .enumerate()
        .filter(|(idx, _)| !result.contains(idx))
        .map(|(_, pkg)| pkg.name.clone())
        .collect();
    
    return Err(format!("Circular dependency detected among packages: {:?}", remaining).into());
}
```

### Common Cycle Scenarios

Typical circular dependency patterns:

1. **Direct Cycles**: A depends on B, B depends on A
2. **Indirect Cycles**: A → B → C → A
3. **Build Tool Cycles**: Package depends on build tool that depends on package

### Resolution Strategies

When cycles are detected, users can:

1. **Review Dependencies**: Examine package.xml files for unnecessary dependencies
2. **Split Packages**: Break large packages into smaller, independent pieces
3. **Use Interface Packages**: Create interface-only packages to break cycles
4. **Dependency Inversion**: Restructure dependencies using abstract interfaces

## Package Filtering

### Build Selection Filters

The dependency resolver supports various package selection strategies:

#### Selective Building (`--packages-select`)
Build only specified packages plus their dependencies:

```rust
if let Some(ref selected) = self.config.packages_select {
    self.packages.retain(|pkg| selected.contains(&pkg.name));
}
```

#### Package Exclusion (`--packages-ignore`)
Exclude specific packages from builds:

```rust
if let Some(ref ignored) = self.config.packages_ignore {
    self.packages.retain(|pkg| !ignored.contains(&pkg.name));
}
```

#### Build Up To (`--packages-up-to`)
Build dependencies up to specified packages:

```rust
if let Some(ref up_to) = self.config.packages_up_to {
    let mut packages_to_build = std::collections::HashSet::new();
    
    // Add target packages
    for target in up_to {
        if let Some(pkg) = self.packages.iter().find(|p| &p.name == target) {
            packages_to_build.insert(pkg.name.clone());
            // Add all dependencies recursively
            self.add_dependencies_recursive(&pkg.name, &mut packages_to_build);
        }
    }
    
    self.packages.retain(|pkg| packages_to_build.contains(&pkg.name));
}
```

### Recursive Dependency Collection

For `--packages-up-to`, dependencies are collected recursively:

```rust
fn add_dependencies_recursive(&self, pkg_name: &str, packages_to_build: &mut std::collections::HashSet<String>) {
    if let Some(pkg) = self.packages.iter().find(|p| &p.name == pkg_name) {
        for dep in &pkg.build_deps {
            if !packages_to_build.contains(dep) {
                if self.packages.iter().any(|p| &p.name == dep) {
                    packages_to_build.insert(dep.clone());
                    self.add_dependencies_recursive(dep, packages_to_build);
                }
            }
        }
    }
}
```

## Parallel Build Optimization

### Independent Package Identification

The topological sort naturally identifies packages that can be built in parallel:

- Packages with no dependencies can start immediately
- Packages with the same dependency level can build concurrently
- Only direct dependencies need to complete before a package starts

### Dependency Satisfaction Checking

During parallel builds, each worker verifies dependencies before starting:

```rust
let all_deps_ready = deps.iter().all(|dep| {
    states.get(dep).map(|s| *s == PackageState::Completed).unwrap_or(true)
});

if all_deps_ready {
    states.insert(pkg_name.clone(), PackageState::Building);
    ready_package = Some(pkg_name);
    break;
}
```

### Load Balancing

The work-stealing approach ensures optimal resource utilization:

1. **Dynamic Work Assignment**: Workers pick up available packages as dependencies complete
2. **No Static Partitioning**: Avoids idle workers when some builds take longer
3. **Dependency-Aware**: Respects build order constraints while maximizing parallelism

## Advanced Dependency Scenarios

### Cross-Package Dependencies

Handling complex dependency relationships:

#### Message/Service Dependencies
```xml
<build_depend>my_interfaces</build_depend>
<exec_depend>my_interfaces</exec_depend>
```

#### Metapackage Dependencies
```xml
<buildtool_depend>ament_cmake</buildtool_depend>
<exec_depend>package1</exec_depend>
<exec_depend>package2</exec_depend>
```

#### Conditional Dependencies (Format 3)
```xml
<depend condition="$ROS_VERSION == 2">ros2_specific_pkg</depend>
```

### Build Tool Resolution

Special handling for build tools:

1. **External Build Tools**: ament_cmake, cmake, python3-setuptools
2. **Workspace Build Tools**: Custom ament extensions built from source
3. **Version Constraints**: Ensuring compatible tool versions

## Error Handling and Diagnostics

### Dependency Validation

The system performs comprehensive validation:

```rust
// Check for missing dependencies
for package in &packages {
    for dep in &package.build_deps {
        if !name_to_index.contains_key(dep) && !is_external_dependency(dep) {
            warnings.push(format!("Package {} depends on missing package {}", 
                                package.name, dep));
        }
    }
}
```

### Diagnostic Output

Detailed information for troubleshooting:

- **Build Order Visualization**: Show the determined build sequence
- **Dependency Tree**: Display complete dependency relationships
- **Cycle Analysis**: Identify specific packages involved in cycles
- **Missing Dependencies**: List external dependencies that may be missing

### Recovery Strategies

When dependency issues are encountered:

1. **Graceful Degradation**: Continue with buildable packages
2. **Partial Builds**: Build independent subgraphs
3. **Dependency Suggestions**: Recommend missing packages to install
4. **Alternative Orderings**: Provide multiple valid build orders when possible

The dependency resolution system provides a robust foundation for workspace builds, ensuring correct build order while maximizing parallel execution opportunities and providing clear diagnostics for troubleshooting dependency issues.
