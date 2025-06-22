use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, anyhow};

/// Represents a ROS2 package discovered in the workspace
#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub path: PathBuf,
    pub build_type: BuildType,
    pub version: String,
    pub description: String,
    pub maintainers: Vec<String>,
    pub build_deps: Vec<String>,
    pub buildtool_deps: Vec<String>,
    pub exec_deps: Vec<String>,
    pub test_deps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuildType {
    AmentCmake,
    AmentPython,
    Cmake,
    Other(String),
}

impl From<&str> for BuildType {
    fn from(s: &str) -> Self {
        match s {
            "ament_cmake" => BuildType::AmentCmake,
            "ament_python" => BuildType::AmentPython,
            "cmake" => BuildType::Cmake,
            other => BuildType::Other(other.to_string()),
        }
    }
}

/// Configuration for package discovery
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Base paths to search for packages
    pub base_paths: Vec<PathBuf>,
    /// Whether to include hidden packages
    pub include_hidden: bool,
    /// Maximum depth for directory traversal
    pub max_depth: Option<usize>,
    /// Patterns to exclude from search
    pub exclude_patterns: Vec<String>,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            base_paths: vec![PathBuf::from(".")],
            include_hidden: false,
            max_depth: None,
            exclude_patterns: vec![
                "build".to_string(),
                "install".to_string(),
                "log".to_string(),
                "target".to_string(),
                ".git".to_string(),
                ".svn".to_string(),
                ".hg".to_string(),
                "node_modules".to_string(),
            ],
        }
    }
}

/// Discovers ROS2 packages in the specified paths
pub fn discover_packages(config: &DiscoveryConfig) -> Result<Vec<Package>> {
    let mut packages = Vec::new();
    
    for base_path in &config.base_paths {
        if !base_path.exists() {
            eprintln!("Warning: Base path {} does not exist", base_path.display());
            continue;
        }
        
        if !base_path.is_dir() {
            eprintln!("Warning: Base path {} is not a directory", base_path.display());
            continue;
        }
        
        discover_packages_in_path(base_path, &mut packages, config)?;
    }
    
    Ok(packages)
}

fn discover_packages_in_path(path: &Path, packages: &mut Vec<Package>, config: &DiscoveryConfig) -> Result<()> {
    // Use walkdir with proper filtering to avoid excluded directories entirely
    for entry in walkdir::WalkDir::new(path)
        .max_depth(config.max_depth.unwrap_or(100)) // Reasonable default
        .into_iter()
        .filter_entry(|e| !should_exclude_path(e.path(), &config.exclude_patterns))
        .filter_map(|e| e.ok()) 
    {
        let entry_path = entry.path();
        
        // Skip if COLCON_IGNORE exists (standard ROS2 convention)
        if entry_path.join("COLCON_IGNORE").exists() {
            continue;
        }
        
        // Skip if AMENT_IGNORE exists (ament-specific ignore)
        if entry_path.join("AMENT_IGNORE").exists() {
            continue;
        }
        
        // Look for package.xml - this is the key check that must work
        let package_xml = entry_path.join("package.xml");
        if package_xml.is_file() {
            match parse_package_xml(&package_xml) {
                Ok(package) => {
                    if config.include_hidden || !is_hidden_package(&package) {
                        // Check for duplicates by name and prefer source packages
                        if let Some(existing_idx) = packages.iter().position(|p| p.name == package.name) {
                            // If we found a duplicate, prefer the source package over installed package
                            let existing_package = &packages[existing_idx];
                            let current_is_source = is_source_package(&package);
                            let existing_is_source = is_source_package(existing_package);
                            
                            // Replace if this is a source package and existing is not
                            if current_is_source && !existing_is_source {
                                packages[existing_idx] = package;
                            }
                            // If both are source or both are installed, keep the first one found
                        } else {
                            packages.push(package);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse {}: {}", package_xml.display(), e);
                }
            }
        }
    }
    
    Ok(())
}

fn should_exclude_path(path: &Path, exclude_patterns: &[String]) -> bool {
    // Check if any component of the path matches exclusion patterns
    for component in path.components() {
        if let Some(component_str) = component.as_os_str().to_str() {
            for pattern in exclude_patterns {
                if component_str == pattern {
                    return true;
                }
                // Support glob-like patterns in the future
                if pattern.contains('*') || pattern.contains('?') {
                    // TODO: Implement glob pattern matching
                    continue;
                }
            }
        }
    }
    
    // Also check the final filename/directory name
    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        for pattern in exclude_patterns {
            if file_name == pattern {
                return true;
            }
        }
    }
    
    false
}

fn is_hidden_package(package: &Package) -> bool {
    // Consider a package hidden if its name starts with underscore
    // or if it's in a hidden directory
    package.name.starts_with('_') || 
    package.path.iter().any(|component| {
        component.to_str().map_or(false, |s| s.starts_with('.'))
    })
}

fn is_source_package(package: &Package) -> bool {
    // A source package is one that's likely in a source directory (src/)
    // rather than an install directory (install/, build/, etc.)
    let path_str = package.path.to_string_lossy().to_lowercase();
    
    // Prefer packages in src/ directories
    if path_str.contains("/src/") || path_str.contains("\\src\\") {
        return true;
    }
    
    // Avoid packages in install/, build/, log/ directories
    if path_str.contains("/install/") || path_str.contains("\\install\\") ||
       path_str.contains("/build/") || path_str.contains("\\build\\") ||
       path_str.contains("/log/") || path_str.contains("\\log\\") {
        return false;
    }
    
    // For packages not in specific directories, prefer ones with source indicators
    package.path.join("CMakeLists.txt").exists() || 
    package.path.join("setup.py").exists() ||
    package.path.join("setup.cfg").exists() ||
    package.path.join("src").exists() ||
    package.path.join("include").exists()
}

fn parse_package_xml(package_xml_path: &Path) -> Result<Package> {
    let xml_content = fs::read_to_string(package_xml_path)
        .map_err(|e| anyhow!("Failed to read {}: {}", package_xml_path.display(), e))?;
    
    let doc = roxmltree::Document::parse(&xml_content)
        .map_err(|e| anyhow!("Failed to parse XML: {}", e))?;
    
    let root = doc.root_element();
    
    // Extract package name
    let name = root
        .descendants()
        .find(|n| n.has_tag_name("name"))
        .and_then(|n| n.text())
        .ok_or_else(|| anyhow!("Missing package name in {}", package_xml_path.display()))?
        .to_string();
    
    // Extract version
    let version = root
        .descendants()
        .find(|n| n.has_tag_name("version"))
        .and_then(|n| n.text())
        .unwrap_or("0.0.0")
        .to_string();
    
    // Extract description
    let description = root
        .descendants()
        .find(|n| n.has_tag_name("description"))
        .and_then(|n| n.text())
        .unwrap_or("")
        .to_string();
    
    // Extract maintainers
    let maintainers: Vec<String> = root
        .descendants()
        .filter(|n| n.has_tag_name("maintainer"))
        .filter_map(|n| n.text())
        .map(|s| s.to_string())
        .collect();
    
    // Extract dependencies
    let build_deps: Vec<String> = root
        .descendants()
        .filter(|n| n.has_tag_name("build_depend"))
        .filter_map(|n| n.text())
        .map(|s| s.to_string())
        .collect();
    
    let buildtool_deps: Vec<String> = root
        .descendants()
        .filter(|n| n.has_tag_name("buildtool_depend"))
        .filter_map(|n| n.text())
        .map(|s| s.to_string())
        .collect();
    
    let exec_deps: Vec<String> = root
        .descendants()
        .filter(|n| n.has_tag_name("exec_depend") || n.has_tag_name("run_depend"))
        .filter_map(|n| n.text())
        .map(|s| s.to_string())
        .collect();
    
    let test_deps: Vec<String> = root
        .descendants()
        .filter(|n| n.has_tag_name("test_depend"))
        .filter_map(|n| n.text())
        .map(|s| s.to_string())
        .collect();
    
    // Extract build type
    let build_type = root
        .descendants()
        .find(|n| n.has_tag_name("export"))
        .and_then(|export| {
            export
                .descendants()
                .find(|n| n.has_tag_name("build_type"))
        })
        .and_then(|n| n.text())
        .map(BuildType::from)
        .unwrap_or_else(|| infer_build_type(package_xml_path.parent().unwrap()));
    
    Ok(Package {
        name,
        path: package_xml_path.parent().unwrap().to_path_buf(),
        build_type,
        version,
        description,
        maintainers,
        build_deps,
        buildtool_deps,
        exec_deps,
        test_deps,
    })
}

fn infer_build_type(package_path: &Path) -> BuildType {
    if package_path.join("CMakeLists.txt").exists() {
        BuildType::AmentCmake
    } else if package_path.join("setup.py").exists() || package_path.join("setup.cfg").exists() {
        BuildType::AmentPython
    } else {
        BuildType::AmentCmake // Default
    }
}

/// Find proto and msg files in the given packages
#[derive(Debug, Clone)]
pub struct IdlFiles {
    pub proto_files: Vec<PathBuf>,
    pub msg_files: Vec<PathBuf>,
}

pub fn find_idl_files_in_packages(packages: &[Package]) -> Result<IdlFiles> {
    let mut proto_files = Vec::new();
    let mut msg_files = Vec::new();
    
    for package in packages {
        // Look for proto files
        let proto_dir = package.path.join("proto");
        if proto_dir.exists() {
            find_files_with_extension(&proto_dir, "proto", &mut proto_files)?;
        }
        
        // Look for msg files
        let msg_dir = package.path.join("msg");
        if msg_dir.exists() {
            find_files_with_extension(&msg_dir, "msg", &mut msg_files)?;
        }
        
        // Also look in the root directory for proto files
        find_files_with_extension(&package.path, "proto", &mut proto_files)?;
    }
    
    Ok(IdlFiles { proto_files, msg_files })
}

fn find_files_with_extension(dir: &Path, extension: &str, files: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == extension {
                    files.push(path);
                }
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_package_discovery() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path();
        
        // Create a test package
        let package_dir = workspace_path.join("test_package");
        fs::create_dir_all(&package_dir).unwrap();
        
        let package_xml = r#"<?xml version="1.0"?>
<package format="3">
  <name>test_package</name>
  <version>1.0.0</version>
  <description>Test package</description>
  <maintainer email="test@example.com">Test User</maintainer>
  <license>MIT</license>
  <buildtool_depend>ament_cmake</buildtool_depend>
  <build_depend>rclcpp</build_depend>
  <exec_depend>rclcpp</exec_depend>
</package>"#;
        
        fs::write(package_dir.join("package.xml"), package_xml).unwrap();
        fs::write(package_dir.join("CMakeLists.txt"), "# Test CMakeLists.txt").unwrap();
        
        let config = DiscoveryConfig {
            base_paths: vec![workspace_path.to_path_buf()],
            ..Default::default()
        };
        
        let packages = discover_packages(&config).unwrap();
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].name, "test_package");
        assert_eq!(packages[0].build_type, BuildType::AmentCmake);
    }
}
