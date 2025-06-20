use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use crate::shared::package_discovery::{discover_packages, find_idl_files_in_packages, DiscoveryConfig, Package};

#[derive(Debug, Clone)]
pub struct ProjectDiscoveryOptions {
    pub search_root: PathBuf,
    pub include_hidden: bool,
    pub max_depth: Option<usize>,
    pub verbose: bool,
}

impl Default for ProjectDiscoveryOptions {
    fn default() -> Self {
        Self {
            search_root: PathBuf::from("."),
            include_hidden: false,
            max_depth: Some(10), // Reasonable default to avoid infinite recursion
            verbose: false,
        }
    }
}

/// Discovers IDL projects in the workspace
pub struct ProjectDiscovery {
    config: DiscoveryConfig,
}

impl ProjectDiscovery {
    pub fn new(config: DiscoveryConfig) -> Self {
        Self { config }
    }

    /// Discover all IDL projects in the workspace
    pub fn discover_projects(&self) -> Result<Vec<IdlProject>> {
        if self.config.verbose {
            println!("🔍 Discovering IDL projects in: {}", self.config.search_root.display());
        }

        let mut projects = Vec::new();
        let mut visited_dirs = HashSet::new();

        self.discover_recursive(&self.config.search_root, 0, &mut projects, &mut visited_dirs)?;

        if self.config.verbose {
            println!("📋 Found {} IDL projects", projects.len());
        }

        Ok(projects)
    }

    fn discover_recursive(
        &self,
        dir: &Path,
        depth: usize,
        projects: &mut Vec<IdlProject>,
        visited_dirs: &mut HashSet<PathBuf>,
    ) -> Result<()> {
        if depth >= self.config.max_depth {
            return Ok(());
        }

        let canonical_dir = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
        if visited_dirs.contains(&canonical_dir) {
            return Ok(());
        }
        visited_dirs.insert(canonical_dir);

        if self.should_exclude_directory(dir) {
            if self.config.verbose {
                println!("⚠️  Skipping excluded directory: {}", dir.display());
            }
            return Ok(());
        }

        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(()), // Skip directories we can't read
        };

        let mut proto_files = Vec::new();
        let mut msg_files = Vec::new();
        let mut subdirs = Vec::new();

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                    match extension {
                        "proto" => proto_files.push(path),
                        "msg" => msg_files.push(path),
                        _ => {}
                    }
                }
            } else if path.is_dir() {
                subdirs.push(path);
            }
        }

        // If we found IDL files in this directory, create a project
        if !proto_files.is_empty() || !msg_files.is_empty() {
            let package_name = self.detect_package_name(dir);
            
            let project = IdlProject {
                root_path: dir.to_path_buf(),
                proto_files,
                msg_files,
                package_name,
            };

            if self.config.verbose {
                println!("📦 Found IDL project: {}", dir.display());
                if !project.proto_files.is_empty() {
                    println!("   Proto files: {}", project.proto_files.len());
                }
                if !project.msg_files.is_empty() {
                    println!("   Msg files: {}", project.msg_files.len());
                }
                if let Some(ref pkg) = project.package_name {
                    println!("   Package: {}", pkg);
                }
            }

            projects.push(project);
        }

        // Recursively search subdirectories
        for subdir in subdirs {
            self.discover_recursive(&subdir, depth + 1, projects, visited_dirs)?;
        }

        Ok(())
    }

    fn should_exclude_directory(&self, dir: &Path) -> bool {
        let dir_str = dir.to_string_lossy();
        
        for pattern in &self.config.exclude_patterns {
            if self.matches_pattern(&dir_str, pattern) {
                return true;
            }
        }

        false
    }

    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        // Simple pattern matching - could be enhanced with proper glob support
        if pattern.ends_with("/*") {
            let prefix = &pattern[..pattern.len() - 2];
            path.contains(prefix)
        } else if pattern.contains('*') {
            // Basic wildcard support
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                path.starts_with(parts[0]) && path.ends_with(parts[1])
            } else {
                false
            }
        } else {
            path.contains(pattern)
        }
    }

    fn detect_package_name(&self, dir: &Path) -> Option<String> {
        // Try to detect package name from various sources
        
        // 1. Look for package.xml (ROS package)
        if let Some(name) = self.detect_ros_package_name(dir) {
            return Some(name);
        }

        // 2. Look for Cargo.toml (Rust package)
        if let Some(name) = self.detect_cargo_package_name(dir) {
            return Some(name);
        }

        // 3. Look for CMakeLists.txt (CMake project)
        if let Some(name) = self.detect_cmake_package_name(dir) {
            return Some(name);
        }

        // 4. Use directory name as fallback
        dir.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
    }

    fn detect_ros_package_name(&self, dir: &Path) -> Option<String> {
        let package_xml = dir.join("package.xml");
        if !package_xml.exists() {
            return None;
        }

        match fs::read_to_string(&package_xml) {
            Ok(content) => {
                // Simple XML parsing to extract package name
                if let Some(start) = content.find("<name>") {
                    if let Some(end) = content[start + 6..].find("</name>") {
                        let name = content[start + 6..start + 6 + end].trim();
                        return Some(name.to_string());
                    }
                }
            }
            Err(_) => {}
        }

        None
    }

    fn detect_cargo_package_name(&self, dir: &Path) -> Option<String> {
        let cargo_toml = dir.join("Cargo.toml");
        if !cargo_toml.exists() {
            return None;
        }

        match fs::read_to_string(&cargo_toml) {
            Ok(content) => {
                // Simple TOML parsing to extract package name
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("name") && line.contains('=') {
                        if let Some(equals_pos) = line.find('=') {
                            let name_part = line[equals_pos + 1..].trim();
                            let name = name_part.trim_matches('"').trim_matches('\'');
                            return Some(name.to_string());
                        }
                    }
                }
            }
            Err(_) => {}
        }

        None
    }

    fn detect_cmake_package_name(&self, dir: &Path) -> Option<String> {
        let cmake_file = dir.join("CMakeLists.txt");
        if !cmake_file.exists() {
            return None;
        }

        match fs::read_to_string(&cmake_file) {
            Ok(content) => {
                // Look for project() declaration
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("project(") {
                        if let Some(start) = line.find('(') {
                            if let Some(end) = line[start..].find(')') {
                                let project_args = &line[start + 1..start + end];
                                let name = project_args.split_whitespace().next()?;
                                return Some(name.to_string());
                            }
                        }
                    }
                }
            }
            Err(_) => {}
        }

        None
    }
}

/// Statistics about discovered projects
#[derive(Debug, Default)]
pub struct DiscoveryStats {
    pub total_projects: usize,
    pub proto_projects: usize,
    pub msg_projects: usize,
    pub mixed_projects: usize,
    pub total_proto_files: usize,
    pub total_msg_files: usize,
}

impl DiscoveryStats {
    pub fn from_projects(projects: &[IdlProject]) -> Self {
        let mut stats = DiscoveryStats::default();
        
        stats.total_projects = projects.len();
        
        for project in projects {
            stats.total_proto_files += project.proto_files.len();
            stats.total_msg_files += project.msg_files.len();
            
            let has_proto = !project.proto_files.is_empty();
            let has_msg = !project.msg_files.is_empty();
            
            match (has_proto, has_msg) {
                (true, true) => stats.mixed_projects += 1,
                (true, false) => stats.proto_projects += 1,
                (false, true) => stats.msg_projects += 1,
                (false, false) => {}, // Shouldn't happen
            }
        }
        
        stats
    }
    
    pub fn print_summary(&self) {
        println!("📊 Discovery Summary:");
        println!("   Total projects: {}", self.total_projects);
        println!("   Proto-only projects: {}", self.proto_projects);
        println!("   Msg-only projects: {}", self.msg_projects);
        println!("   Mixed projects: {}", self.mixed_projects);
        println!("   Total proto files: {}", self.total_proto_files);
        println!("   Total msg files: {}", self.total_msg_files);
    }
}
