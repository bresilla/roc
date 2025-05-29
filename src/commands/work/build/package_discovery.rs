use std::path::{Path, PathBuf};
use std::fs;
use crate::commands::work::build::{PackageMeta, BuildType};

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

fn discover_packages_in_path(path: &Path, packages: &mut Vec<PackageMeta>) -> Result<(), Box<dyn std::error::Error>> {
    for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        
        // Skip if COLCON_IGNORE exists
        if path.join("COLCON_IGNORE").exists() {
            continue;
        }
        
        // Look for package.xml
        let package_xml = path.join("package.xml");
        if package_xml.is_file() {
            match parse_package_xml(&package_xml) {
                Ok(package_meta) => {
                    packages.push(package_meta);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse {}: {}", package_xml.display(), e);
                }
            }
        }
    }
    
    Ok(())
}

fn parse_package_xml(package_xml_path: &Path) -> Result<PackageMeta, Box<dyn std::error::Error>> {
    let xml_content = fs::read_to_string(package_xml_path)?;
    let doc = roxmltree::Document::parse(&xml_content)?;
    let root = doc.root_element();
    
    // Extract package name
    let name = root
        .descendants()
        .find(|n| n.has_tag_name("name"))
        .and_then(|n| n.text())
        .ok_or("Missing package name")?
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
    
    Ok(PackageMeta {
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
    } else if package_path.join("setup.py").exists() {
        BuildType::AmentPython
    } else {
        BuildType::AmentCmake // Default
    }
}
