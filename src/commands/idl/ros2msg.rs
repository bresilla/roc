use anyhow::{anyhow, Result};
use clap::ArgMatches;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct Ros2MsgConversionOptions {
    msg_files: Vec<PathBuf>,
    output_dir: Option<PathBuf>, // None means inplace, Some means explicit output directory
    package_name: Option<String>,
    verbose: bool,
    dry_run: bool,
}

impl Ros2MsgConversionOptions {
    fn from_matches(matches: &ArgMatches) -> Result<Self> {
        let msg_files: Vec<PathBuf> = matches
            .get_many::<String>("msg_files")
            .ok_or_else(|| anyhow!("Message files are required"))?
            .map(PathBuf::from)
            .collect();

        let output_dir = matches
            .get_one::<String>("output_dir")
            .filter(|s| *s != ".") // If it's "." (default), treat as None for inplace
            .map(PathBuf::from);

        let package_name = matches.get_one::<String>("package_name").cloned();
        let verbose = matches.get_flag("verbose");
        let dry_run = matches.get_flag("dry_run");

        Ok(Ros2MsgConversionOptions {
            msg_files,
            output_dir,
            package_name,
            verbose,
            dry_run,
        })
    }
}

pub fn handle(matches: ArgMatches) {
    let options = match Ros2MsgConversionOptions::from_matches(&matches) {
        Ok(opts) => opts,
        Err(e) => {
            eprintln!("Error parsing arguments: {}", e);
            return;
        }
    };

    if options.verbose {
        println!("🚀 Starting ROS 2 message to Protobuf conversion...");
        println!("   Message files: {:?}", options.msg_files);
        match &options.output_dir {
            Some(dir) => println!("   Output directory: {}", dir.display()),
            None => println!("   Output mode: inplace (same directory as .msg files)"),
        }
        println!("   Package name: {:?}", options.package_name);
    }

    if let Err(e) = convert_ros2msg_to_protobuf(&options) {
        eprintln!("Error during conversion: {}", e);
        std::process::exit(1);
    }

    if options.verbose {
        println!("✅ Conversion completed successfully!");
    }
}

fn convert_ros2msg_to_protobuf(options: &Ros2MsgConversionOptions) -> Result<()> {
    // Validate input files
    for msg_file in &options.msg_files {
        if !msg_file.exists() {
            return Err(anyhow!("Message file does not exist: {:?}", msg_file));
        }
        if msg_file.extension().and_then(|s| s.to_str()) != Some("msg") {
            return Err(anyhow!("File is not a .msg file: {:?}", msg_file));
        }
    }

    // Create output directory if it's explicitly specified and doesn't exist
    if let Some(output_dir) = &options.output_dir {
        if !options.dry_run {
            fs::create_dir_all(output_dir)
                .map_err(|e| anyhow!("Failed to create output directory: {}", e))?;
        }
    }

    // Convert each message file
    for msg_file in &options.msg_files {
        if options.verbose {
            println!("🔄 Converting {}...", msg_file.display());
        }

        let msg_content = fs::read_to_string(msg_file)
            .map_err(|e| anyhow!("Failed to read message file: {}", e))?;

        let proto_content = parse_msg_to_proto(&msg_content, msg_file, &options.package_name)?;

        // Determine output directory and file name
        let output_dir = match &options.output_dir {
            Some(dir) => dir.clone(),
            None => msg_file.parent().unwrap_or(Path::new(".")).to_path_buf(),
        };

        let msg_name = msg_file
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Invalid message file name: {:?}", msg_file))?;

        let output_file = output_dir.join(format!("{}.proto", msg_name));

        if options.verbose {
            println!("   Generated: {}", output_file.display());
        }

        if options.dry_run {
            println!("Would write to: {}", output_file.display());
            println!("Content:\n{}", proto_content);
            println!("---");
        } else {
            // Create the directory if it doesn't exist (for inplace mode)
            if let Some(parent) = output_file.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    anyhow!("Failed to create directory {}: {}", parent.display(), e)
                })?;
            }

            fs::write(&output_file, proto_content)
                .map_err(|e| anyhow!("Failed to write proto file: {}", e))?;
        }
    }

    Ok(())
}

fn parse_msg_to_proto(
    msg_content: &str,
    msg_file: &Path,
    package_name: &Option<String>,
) -> Result<String> {
    let msg_name = msg_file
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("Invalid message file name: {:?}", msg_file))?;

    let mut proto_content = String::new();

    // Add proto syntax
    proto_content.push_str("syntax = \"proto3\";\n\n");

    // Add package declaration
    if let Some(pkg) = package_name {
        proto_content.push_str(&format!("package {};\n\n", pkg));
    } else {
        // Derive package name from file path or use default
        let derived_package = derive_package_name_from_path(msg_file);
        proto_content.push_str(&format!("package {};\n\n", derived_package));
    }

    // Parse message fields
    let fields = parse_ros2_message_fields(msg_content)?;

    // Generate protobuf message
    proto_content.push_str(&format!("message {} {{\n", msg_name));

    for (i, field) in fields.iter().enumerate() {
        let field_number = i + 1;
        let proto_field = convert_ros2_field_to_proto(field, field_number)?;
        proto_content.push_str(&format!("  {}\n", proto_field));
    }

    proto_content.push_str("}\n");

    Ok(proto_content)
}

#[derive(Debug, Clone)]
struct Ros2Field {
    field_type: String,
    field_name: String,
    is_array: bool,
    is_optional: bool,
    comment: Option<String>,
}

fn parse_ros2_message_fields(content: &str) -> Result<Vec<Ros2Field>> {
    let mut fields = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse field: "type name" or "type[] name" with optional comment
        let (field_part, comment) = if let Some(comment_pos) = line.find('#') {
            (
                line[..comment_pos].trim(),
                Some(line[comment_pos + 1..].trim().to_string()),
            )
        } else {
            (line, None)
        };

        let parts: Vec<&str> = field_part.split_whitespace().collect();
        if parts.len() != 2 {
            continue; // Skip malformed lines
        }

        let field_type = parts[0];
        let field_name = parts[1];

        // Check if it's an array type
        let (base_type, is_array) = if field_type.ends_with("[]") {
            (field_type.strip_suffix("[]").unwrap(), true)
        } else {
            (field_type, false)
        };

        // Check if it's optional (from comment)
        let is_optional = comment
            .as_ref()
            .map(|c| c.contains("optional"))
            .unwrap_or(false);

        fields.push(Ros2Field {
            field_type: base_type.to_string(),
            field_name: field_name.to_string(),
            is_array,
            is_optional,
            comment,
        });
    }

    Ok(fields)
}

fn convert_ros2_field_to_proto(field: &Ros2Field, field_number: usize) -> Result<String> {
    let proto_type = convert_ros2_type_to_proto(&field.field_type);

    let mut proto_field = String::new();

    // Add repeated if it's an array
    if field.is_array {
        proto_field.push_str("repeated ");
    } else if field.is_optional {
        proto_field.push_str("optional ");
    }

    proto_field.push_str(&format!(
        "{} {} = {}",
        proto_type, field.field_name, field_number
    ));

    // Add comment if present
    if let Some(comment) = &field.comment {
        proto_field.push_str(&format!(";  // {}", comment));
    } else {
        proto_field.push(';');
    }

    Ok(proto_field)
}

fn convert_ros2_type_to_proto(ros2_type: &str) -> String {
    match ros2_type {
        "bool" => "bool".to_string(),
        "int8" => "int32".to_string(),    // Proto3 doesn't have int8
        "uint8" => "uint32".to_string(),  // Proto3 doesn't have uint8
        "int16" => "int32".to_string(),   // Proto3 doesn't have int16
        "uint16" => "uint32".to_string(), // Proto3 doesn't have uint16
        "int32" => "int32".to_string(),
        "uint32" => "uint32".to_string(),
        "int64" => "int64".to_string(),
        "uint64" => "uint64".to_string(),
        "float32" => "float".to_string(),
        "float64" => "double".to_string(),
        "string" => "string".to_string(),
        "wstring" => "string".to_string(), // Convert wide string to regular string
        "time" => "google.protobuf.Timestamp".to_string(),
        "duration" => "google.protobuf.Duration".to_string(),
        _ => {
            // Custom message type - convert from ROS2 naming to proto naming
            // Handle flattened nested types like "RobotStatus" -> "Robot.Status"
            // For now, keep as-is, but in a full implementation we'd need type mapping
            ros2_type.to_string()
        }
    }
}

fn derive_package_name_from_path(msg_file: &Path) -> String {
    // Try to derive a reasonable package name from the file path
    if let Some(parent) = msg_file.parent() {
        if let Some(dir_name) = parent.file_name().and_then(|s| s.to_str()) {
            // Convert directory name to valid protobuf package name
            return dir_name.replace('-', "_").replace(' ', "_").to_lowercase();
        }
    }

    // Default package name
    "generated_messages".to_string()
}
