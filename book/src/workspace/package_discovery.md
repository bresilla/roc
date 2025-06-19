# Package Discovery

ROC's package discovery system automatically scans workspace directories to find and parse ROS2 packages. This chapter details how the discovery process works and how it handles various package configurations.

## Discovery Process

### 1. Workspace Scanning

The discovery process begins by recursively scanning the configured base paths (default: `src/`):

```rust
pub fn discover_packages(base_paths: &[PathBuf]) -> Result<Vec<PackageMeta>, Box<dyn std::error::Error>> {
    let mut packages = Vec::new();
    
    for base_path in base_paths {
        if base_path.exists() {
            discover_packages_in_path(base_path, &mut packages)?;
        } else {
            println!("Warning: Base path {} does not exist", base_path.display());
        }
    }
    
    Ok(packages)
}
```

### 2. Package Identification

Packages are identified by the presence of a `package.xml` file in the directory root. The discovery engine:

- Recursively walks directory trees using the `walkdir` crate
- Skips directories containing `COLCON_IGNORE` files
- Parses each `package.xml` file found
- Extracts comprehensive package metadata

### 3. Manifest Parsing

Each `package.xml` is parsed using the `roxmltree` XML parser to extract:

```rust
pub struct PackageMeta {
    pub name: String,                    // Package name
    pub path: PathBuf,                   // Package directory path
    pub build_type: BuildType,           // Build system type
    pub version: String,                 // Package version
    pub description: String,             // Package description
    pub maintainers: Vec<String>,        // Package maintainers
    pub build_deps: Vec<String>,         // Build dependencies
    pub buildtool_deps: Vec<String>,     // Build tool dependencies
    pub exec_deps: Vec<String>,          // Runtime dependencies
    pub test_deps: Vec<String>,          // Test dependencies
}
```

## XML Parsing Implementation

### Dependency Extraction

The parser extracts different types of dependencies from the manifest:

```rust
// Build dependencies
let build_deps: Vec<String> = root
    .descendants()
    .filter(|n| n.has_tag_name("build_depend"))
    .filter_map(|n| n.text())
    .map(|s| s.to_string())
    .collect();

// Build tool dependencies (cmake, ament_cmake, etc.)
let buildtool_deps: Vec<String> = root
    .descendants()
    .filter(|n| n.has_tag_name("buildtool_depend"))
    .filter_map(|n| n.text())
    .map(|s| s.to_string())
    .collect();

// Runtime dependencies
let exec_deps: Vec<String> = root
    .descendants()
    .filter(|n| n.has_tag_name("exec_depend") || n.has_tag_name("run_depend"))
    .filter_map(|n| n.text())
    .map(|s| s.to_string())
    .collect();
```

### Build Type Detection

Build type is determined through multiple strategies:

1. **Explicit Declaration**: Check for `<build_type>` in the `<export>` section
2. **File-Based Inference**: Examine files in the package directory
3. **Default Assignment**: Fall back to `ament_cmake`

```rust
fn infer_build_type(package_path: &Path) -> BuildType {
    if package_path.join("CMakeLists.txt").exists() {
        BuildType::AmentCmake
    } else if package_path.join("setup.py").exists() {
        BuildType::AmentPython
    } else {
        BuildType::AmentCmake // Default
    }
}
```

## Supported Package Formats

### Package.xml Format Support

ROC supports both package.xml formats used in ROS2:

- **Format 2**: Standard format inherited from ROS1
- **Format 3**: Enhanced format with conditional dependencies and groups

### Build Type Support

The discovery system recognizes these build types:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum BuildType {
    AmentCmake,           // Standard C++ packages
    AmentPython,          // Pure Python packages  
    Cmake,                // Plain CMake packages
    Other(String),        // Extension point for future types
}
```

#### AmentCmake Packages
- Use CMake as the build system
- Include ament_cmake macros for ROS2 integration
- Typically contain C++ source code
- Most common package type in ROS2

#### AmentPython Packages
- Use Python setuptools for building
- Contain Python modules and scripts
- Use `setup.py` for build configuration
- Common for pure Python ROS2 nodes

#### Plain CMake Packages
- Use standard CMake without ament extensions
- Useful for integrating non-ROS libraries
- Less common but fully supported

## Error Handling and Validation

### XML Parsing Errors

The discovery system handles various XML parsing issues:

```rust
match parse_package_xml(&package_xml) {
    Ok(package_meta) => {
        packages.push(package_meta);
    }
    Err(e) => {
        eprintln!("Warning: Failed to parse {}: {}", package_xml.display(), e);
    }
}
```

Common issues addressed:
- Malformed XML syntax
- Missing required elements (`<name>`, `<version>`)
- Invalid dependency declarations
- Encoding issues

### Package Validation

During discovery, several validation checks are performed:

1. **Unique Names**: Ensure no duplicate package names in workspace
2. **Required Elements**: Verify presence of essential package.xml elements
3. **Path Validity**: Confirm package paths are accessible
4. **Build Type Consistency**: Validate build type matches package contents

### Duplicate Package Handling

If multiple packages with the same name are discovered:

```rust
// Check for duplicate package names
let mut seen_names = std::collections::HashSet::new();
for package in &packages {
    if !seen_names.insert(&package.name) {
        return Err(format!("Duplicate package name found: {}", package.name).into());
    }
}
```

## Performance Optimizations

### Efficient Directory Traversal

The discovery system uses optimized directory traversal:

- **Parallel Scanning**: Multiple base paths scanned concurrently
- **Early Termination**: Stop scanning ignored directories immediately
- **Memory Efficiency**: Stream processing of directory entries

### XML Parser Selection

ROC uses `roxmltree` for XML parsing because:

- **Performance**: Faster than alternatives for small XML files
- **Memory Efficiency**: Low memory overhead
- **Safety**: Memory-safe with proper error handling
- **Simplicity**: Clean API for tree traversal

### Caching Strategy

While not currently implemented, the architecture supports future caching:

- **Manifest Checksums**: Cache parsed results based on file modification time
- **Incremental Discovery**: Only re-scan changed directories
- **Metadata Persistence**: Save/restore package metadata across invocations

## Integration with Build System

### Package Filtering

Discovery results can be filtered based on build configuration:

```rust
// Apply packages_select filter
if let Some(ref selected) = self.config.packages_select {
    self.packages.retain(|pkg| selected.contains(&pkg.name));
}

// Apply packages_ignore filter  
if let Some(ref ignored) = self.config.packages_ignore {
    self.packages.retain(|pkg| !ignored.contains(&pkg.name));
}
```

### Dependency Graph Input

The discovered packages serve as input to the dependency resolution system:

- Package names become graph nodes
- Dependencies become directed edges
- Build types determine build strategies
- Metadata guides environment setup

## Future Enhancements

### Conditional Dependencies

Support for package.xml format 3 conditional dependencies:

```xml
<depend condition="$ROS_VERSION == 2">ros2_specific_pkg</depend>
```

### Package Groups

Enhanced support for dependency groups:

```xml
<group_depend>navigation_stack</group_depend>
```

### Extended Metadata

Additional metadata extraction for:
- License information
- Repository URLs
- Bug tracker links
- Documentation links

The package discovery system provides a solid foundation for workspace management, efficiently finding and parsing ROS2 packages while maintaining compatibility with existing tooling and workflows.
