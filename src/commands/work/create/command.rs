use clap::ArgMatches;
use std::fs;
use std::path::Path;

use crate::commands::work::create::package_templates::*;

pub fn handle(matches: ArgMatches) {
    if let Err(e) = create_package(matches) {
        eprintln!("❌ Error creating package: {}", e);
        std::process::exit(1);
    }
}

fn create_package(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let package_name = matches.get_one::<String>("PACKAGE_NAME").unwrap();

    // Parse arguments with defaults
    let package_format = matches
        .get_one::<String>("package_format")
        .map(|s| s.as_str())
        .unwrap_or("3");
    let default_description = format!("{} package", package_name);
    let description = matches
        .get_one::<String>("description")
        .map(|s| s.as_str())
        .unwrap_or(&default_description);
    let license = matches
        .get_one::<String>("license")
        .map(|s| s.as_str())
        .unwrap_or("Apache-2.0");
    let destination_directory = matches
        .get_one::<String>("destination_directory")
        .map(|s| s.as_str())
        .unwrap_or(".");
    let build_type = matches
        .get_one::<String>("build_type")
        .map(|s| s.as_str())
        .unwrap_or("ament_cmake");
    let dependencies: Vec<&str> = matches
        .get_many::<String>("dependencies")
        .map(|vals| vals.map(|s| s.as_str()).collect())
        .unwrap_or_default();
    let maintainer_email = matches
        .get_one::<String>("maintainer_email")
        .map(|s| s.as_str())
        .unwrap_or("maintainer@example.com");
    let maintainer_name = matches
        .get_one::<String>("maintainer_name")
        .map(|s| s.as_str())
        .unwrap_or("Maintainer");
    let node_name = matches.get_one::<String>("node_name");
    let library_name = matches.get_one::<String>("library_name");

    // Validate build type
    match build_type {
        "ament_cmake" | "ament_python" | "cmake" => {}
        _ => return Err(format!("Unsupported build type: {}", build_type).into()),
    }

    // Create package directory
    let package_path = Path::new(destination_directory).join(package_name);
    if package_path.exists() {
        return Err(format!(
            "Package directory '{}' already exists",
            package_path.display()
        )
        .into());
    }

    println!("Creating package '{}'...", package_name);
    fs::create_dir_all(&package_path)?;

    // Create package.xml
    let package_xml = create_package_xml(
        package_name,
        package_format,
        description,
        license,
        maintainer_name,
        maintainer_email,
        build_type,
        &dependencies,
    )?;

    let package_xml_path = package_path.join("package.xml");
    fs::write(&package_xml_path, package_xml)?;
    println!("  📝 Created package.xml");

    // Create build system files based on build type
    match build_type {
        "ament_cmake" | "cmake" => {
            create_cmake_package(&package_path, package_name, node_name, library_name)?;
        }
        "ament_python" => {
            create_python_package(
                &package_path,
                package_name,
                node_name,
                maintainer_name,
                maintainer_email,
                description,
                license,
            )?;
        }
        _ => unreachable!("Already validated build type above"),
    }

    println!("✅ Successfully created package '{}'", package_name);
    println!("   📁 Location: {}", package_path.display());

    if build_type == "ament_cmake" {
        println!(
            "   🔧 Build with: roc work build --packages-select {}",
            package_name
        );
    } else if build_type == "ament_python" {
        println!("   🐍 Python package ready for development");
    }

    Ok(())
}

fn create_cmake_package(
    package_path: &Path,
    package_name: &str,
    node_name: Option<&String>,
    library_name: Option<&String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create CMakeLists.txt
    let cmake_content = create_cmake_lists(package_name, node_name, library_name)?;
    let cmake_path = package_path.join("CMakeLists.txt");
    fs::write(&cmake_path, cmake_content)?;
    println!("  📝 Created CMakeLists.txt");

    // Create directory structure
    let src_dir = package_path.join("src");
    let include_dir = package_path.join("include").join(package_name);
    fs::create_dir_all(&src_dir)?;
    fs::create_dir_all(&include_dir)?;

    // Create node if specified
    if let Some(node_name_str) = node_name {
        let node_content = create_cpp_node_template(package_name, node_name_str);
        let node_path = src_dir.join(format!("{}.cpp", node_name_str));
        fs::write(&node_path, node_content)?;
        println!("  📝 Created C++ node: src/{}.cpp", node_name_str);
    }

    // Create library if specified
    if let Some(library_name_str) = library_name {
        let header_content = create_cpp_header_template(package_name, library_name_str);
        let source_content = create_cpp_source_template(package_name, library_name_str);

        let header_path = include_dir.join(format!("{}.hpp", library_name_str));
        let source_path = src_dir.join(format!("{}.cpp", library_name_str));

        fs::write(&header_path, header_content)?;
        fs::write(&source_path, source_content)?;
        println!(
            "  📝 Created C++ library: include/{}/{}.hpp, src/{}.cpp",
            package_name, library_name_str, library_name_str
        );
    }

    Ok(())
}

fn create_python_package(
    package_path: &Path,
    package_name: &str,
    node_name: Option<&String>,
    maintainer_name: &str,
    maintainer_email: &str,
    description: &str,
    license: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create setup.py
    let setup_content = create_setup_py(
        package_name,
        node_name,
        maintainer_name,
        maintainer_email,
        description,
        license,
    )?;
    let setup_path = package_path.join("setup.py");
    fs::write(&setup_path, setup_content)?;
    println!("  📝 Created setup.py");

    // Create setup.cfg
    let setup_cfg_content = create_setup_cfg(package_name);
    let setup_cfg_path = package_path.join("setup.cfg");
    fs::write(&setup_cfg_path, setup_cfg_content)?;
    println!("  📝 Created setup.cfg");

    // Create Python package directory
    let python_package_dir = package_path.join(package_name);
    fs::create_dir_all(&python_package_dir)?;

    // Create __init__.py
    let init_path = python_package_dir.join("__init__.py");
    fs::write(&init_path, "")?;
    println!("  📝 Created {}/__init__.py", package_name);

    // Create node if specified
    if let Some(node_name_str) = node_name {
        let node_content = create_python_node_template(package_name, node_name_str);
        let node_path = python_package_dir.join(format!("{}.py", node_name_str));
        fs::write(&node_path, node_content)?;
        println!(
            "  📝 Created Python node: {}/{}.py",
            package_name, node_name_str
        );
    }

    // Create resource directory
    let resource_dir = package_path.join("resource");
    fs::create_dir_all(&resource_dir)?;
    let resource_file = resource_dir.join(package_name);
    fs::write(&resource_file, "")?;
    println!("  📝 Created resource/{}", package_name);

    // Create test directory
    let test_dir = package_path.join("test");
    fs::create_dir_all(&test_dir)?;

    let test_files = ["test_copyright.py", "test_flake8.py", "test_pep257.py"];
    for test_file in &test_files {
        let test_content = create_python_test_template(test_file);
        let test_path = test_dir.join(test_file);
        fs::write(&test_path, test_content)?;
    }
    println!("  📝 Created test files");

    Ok(())
}
