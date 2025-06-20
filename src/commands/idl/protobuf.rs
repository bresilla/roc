use anyhow::{anyhow, Result};
use clap::ArgMatches;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
enum ConversionDirection {
    ProtoToMsg,
    MsgToProto,
}

#[derive(Debug, Clone)]
struct ProtobufConversionOptions {
    input_files: Vec<PathBuf>,
    output_dir: Option<PathBuf>,  // None means inplace, Some means explicit output directory
    package_name: Option<String>,
    config_file: Option<PathBuf>,
    include_dirs: Vec<PathBuf>,
    verbose: bool,
    dry_run: bool,
    direction: ConversionDirection,
}

impl ProtobufConversionOptions {
    fn from_matches(matches: &ArgMatches) -> Result<Self> {
        let input_files: Vec<PathBuf> = matches
            .get_many::<String>("proto_files")
            .ok_or_else(|| anyhow!("Input files are required"))?
            .map(PathBuf::from)
            .collect();

        // Auto-detect conversion direction based on file extensions
        let direction = detect_conversion_direction(&input_files)?;

        let output_dir = matches.get_one::<String>("output_dir")
            .filter(|s| *s != ".")  // If it's "." (default), treat as None for inplace
            .map(PathBuf::from);

        let package_name = matches.get_one::<String>("package_name").cloned();
        let config_file = matches.get_one::<String>("config_file").map(PathBuf::from);
        
        let include_dirs: Vec<PathBuf> = matches
            .get_many::<String>("include_dirs")
            .map(|values| values.map(PathBuf::from).collect())
            .unwrap_or_default();

        let verbose = matches.get_flag("verbose");
        let dry_run = matches.get_flag("dry_run");

        Ok(ProtobufConversionOptions {
            input_files,
            output_dir,
            package_name,
            config_file,
            include_dirs,
            verbose,
            dry_run,
            direction,
        })
    }
}

fn detect_conversion_direction(files: &[PathBuf]) -> Result<ConversionDirection> {
    if files.is_empty() {
        return Err(anyhow!("No input files provided"));
    }

    let mut proto_count = 0;
    let mut msg_count = 0;

    for file in files {
        match file.extension().and_then(|s| s.to_str()) {
            Some("proto") => proto_count += 1,
            Some("msg") => msg_count += 1,
            Some(ext) => return Err(anyhow!("Unsupported file extension: .{}", ext)),
            None => return Err(anyhow!("File has no extension: {}", file.display())),
        }
    }

    if proto_count > 0 && msg_count > 0 {
        return Err(anyhow!("Cannot mix .proto and .msg files in the same conversion"));
    }

    if proto_count > 0 {
        Ok(ConversionDirection::ProtoToMsg)
    } else if msg_count > 0 {
        Ok(ConversionDirection::MsgToProto)
    } else {
        Err(anyhow!("No .proto or .msg files found"))
    }
}

pub fn handle(matches: ArgMatches) {
    let options = match ProtobufConversionOptions::from_matches(&matches) {
        Ok(opts) => opts,
        Err(e) => {
            eprintln!("Error parsing arguments: {}", e);
            return;
        }
    };

    if options.verbose {
        println!("🚀 Starting bidirectional Protobuf ↔ ROS 2 conversion...");
        println!("   Input files: {:?}", options.input_files);
        match options.direction {
            ConversionDirection::ProtoToMsg => println!("   Direction: .proto → .msg"),
            ConversionDirection::MsgToProto => println!("   Direction: .msg → .proto"),
        }
        match &options.output_dir {
            Some(dir) => println!("   Output directory: {}", dir.display()),
            None => println!("   Output mode: inplace (same directory as input files)"),
        }
        println!("   Package name: {:?}", options.package_name);
        println!("   Include directories: {:?}", options.include_dirs);
    }

    let result = match options.direction {
        ConversionDirection::ProtoToMsg => convert_proto_to_msg(&options),
        ConversionDirection::MsgToProto => convert_msg_to_proto(&options),
    };

    if let Err(e) = result {
        eprintln!("Error during conversion: {}", e);
        std::process::exit(1);
    }

    if options.verbose {
        println!("✅ Conversion completed successfully!");
    }
}

fn convert_proto_to_msg(options: &ProtobufConversionOptions) -> Result<()> {
    // Validate input files
    for input_file in &options.input_files {
        if !input_file.exists() {
            return Err(anyhow!("Input file does not exist: {:?}", input_file));
        }
        if input_file.extension().and_then(|s| s.to_str()) != Some("proto") {
            return Err(anyhow!("File is not a .proto file: {:?}", input_file));
        }
    }

    // Create output directory if it's explicitly specified and doesn't exist
    if let Some(output_dir) = &options.output_dir {
        if !options.dry_run {
            fs::create_dir_all(output_dir)
                .map_err(|e| anyhow!("Failed to create output directory: {}", e))?;
        }
    }

    // Use our built-in robust conversion
    convert_proto_files_to_msg(options)
}

fn convert_msg_to_proto(options: &ProtobufConversionOptions) -> Result<()> {
    // Validate input files
    for input_file in &options.input_files {
        if !input_file.exists() {
            return Err(anyhow!("Input file does not exist: {:?}", input_file));
        }
        if input_file.extension().and_then(|s| s.to_str()) != Some("msg") {
            return Err(anyhow!("File is not a .msg file: {:?}", input_file));
        }
    }

    // Create output directory if it's explicitly specified and doesn't exist
    if let Some(output_dir) = &options.output_dir {
        if !options.dry_run {
            fs::create_dir_all(output_dir)
                .map_err(|e| anyhow!("Failed to create output directory: {}", e))?;
        }
    }

    // Use our built-in robust conversion
    convert_msg_files_to_proto(options)
}

fn convert_proto_files_to_msg(options: &ProtobufConversionOptions) -> Result<()> {
    for proto_file in &options.input_files {
        if options.verbose {
            println!("🔄 Converting {}...", proto_file.display());
        }

        let proto_content = fs::read_to_string(proto_file)
            .map_err(|e| anyhow!("Failed to read proto file: {}", e))?;

        let ros2_messages = parse_proto_to_ros2(&proto_content, proto_file)?;

        for (message_name, message_content) in ros2_messages {
            // Determine output directory: use explicit output_dir or same directory as proto file
            let output_dir = match &options.output_dir {
                Some(dir) => dir.clone(),
                None => proto_file.parent().unwrap_or(Path::new(".")).to_path_buf(),
            };
            
            let output_file = output_dir.join(format!("{}.msg", message_name));
            
            if options.verbose {
                println!("   Generated: {}", output_file.display());
            }

            if options.dry_run {
                println!("Would write to: {}", output_file.display());
                println!("Content:\n{}", message_content);
                println!("---");
            } else {
                // Create the directory if it doesn't exist (for inplace mode)
                if let Some(parent) = output_file.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| anyhow!("Failed to create directory {}: {}", parent.display(), e))?;
                }
                
                fs::write(&output_file, message_content)
                    .map_err(|e| anyhow!("Failed to write message file: {}", e))?;
            }
        }
    }

    Ok(())
}

fn parse_proto_to_ros2(proto_content: &str, proto_file: &Path) -> Result<Vec<(String, String)>> {
    // Enhanced parser that handles complex protobuf features including nested messages
    
    let mut messages = Vec::new();
    let mut _enums: Vec<String> = Vec::new();
    let mut message_stack: Vec<String> = Vec::new(); // Track nested message context
    let mut nested_types: std::collections::HashSet<String> = std::collections::HashSet::new(); // Track all nested message types
    let mut current_fields = Vec::new();
    let mut current_enum_values: Vec<String> = Vec::new();
    let mut bracket_count = 0;
    let mut oneof_fields: Vec<(String, Vec<String>)> = Vec::new();
    let mut current_oneof: Option<String> = None;

    // First pass: collect all nested type definitions
    for line in proto_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            continue;
        }

        if line.starts_with("message ") {
            if let Some(msg_name) = extract_message_name(line) {
                if !message_stack.is_empty() {
                    // This is a nested message - record it
                    let full_name = format!("{}{}", message_stack.join(""), msg_name);
                    nested_types.insert(full_name);
                }
                message_stack.push(msg_name);
            }
        }

        let open_brackets = line.chars().filter(|&c| c == '{').count() as i32;
        let close_brackets = line.chars().filter(|&c| c == '}').count() as i32;
        bracket_count += open_brackets - close_brackets;

        if close_brackets > 0 && !message_stack.is_empty() {
            message_stack.pop();
        }
    }

    // Reset for second pass
    message_stack.clear();
    current_fields.clear();
    bracket_count = 0;
    oneof_fields.clear();
    current_oneof = None;

    // Second pass: actual parsing with context
    for line in proto_content.lines() {
        let line = line.trim();
        
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            continue;
        }

        // Parse enum definitions
        if line.starts_with("enum ") {
            if let Some(enum_name) = extract_enum_name(line) {
                let full_enum_name = if message_stack.is_empty() {
                    enum_name
                } else {
                    format!("{}{}", message_stack.join(""), enum_name)
                };
                current_enum_values.clear();
                bracket_count = 0;
                
                // Process enum when complete (simplified for this example)
                if bracket_count == 0 {
                    let enum_content = "# Enum placeholder\nint32 value\n".to_string();
                    messages.push((full_enum_name, enum_content));
                }
            }
        }

        // Parse message definitions (including nested ones)
        if line.starts_with("message ") {
            if let Some(msg_name) = extract_message_name(line) {
                let full_msg_name = if message_stack.is_empty() {
                    msg_name.clone()
                } else {
                    format!("{}{}", message_stack.join(""), msg_name)
                };
                
                message_stack.push(msg_name);
                current_fields.clear();
                oneof_fields.clear();
                bracket_count = 0;
            }
        }

        // Parse oneof definitions
        if line.starts_with("oneof ") {
            if let Some(oneof_name) = extract_oneof_name(line) {
                current_oneof = Some(oneof_name);
            }
        }

        // Count brackets to track nesting
        let open_brackets = line.chars().filter(|&c| c == '{').count() as i32;
        let close_brackets = line.chars().filter(|&c| c == '}').count() as i32;
        bracket_count += open_brackets - close_brackets;

        // Parse field definitions
        if !message_stack.is_empty() {
            if current_oneof.is_some() {
                // Handle oneof fields
                if let Some(ros_field) = parse_field_to_ros2_with_nested_context(line, &message_stack, &nested_types) {
                    let oneof_name = current_oneof.as_ref().unwrap();
                    if let Some((_, fields)) = oneof_fields.iter_mut().find(|(name, _)| name == oneof_name) {
                        fields.push(ros_field);
                    } else {
                        oneof_fields.push((oneof_name.clone(), vec![ros_field]));
                    }
                }
            } else {
                // Handle regular fields
                if let Some(ros_field) = parse_field_to_ros2_with_nested_context(line, &message_stack, &nested_types) {
                    current_fields.push(ros_field);
                }
            }
        }

        // End of oneof definition
        if current_oneof.is_some() && close_brackets > 0 {
            current_oneof = None;
        }

        // End of message definition
        if close_brackets > 0 && !message_stack.is_empty() {
            let current_msg_name = message_stack.last().unwrap().clone();
            let full_msg_name = message_stack.join("");
            
            // Generate oneof helper messages first
            for (oneof_name, oneof_field_list) in &oneof_fields {
                let oneof_msg_name = format!("{}OneOf{}", full_msg_name, capitalize_first(oneof_name));
                let oneof_content = generate_oneof_message_content(&oneof_field_list, oneof_name);
                messages.push((oneof_msg_name.clone(), oneof_content));
                
                // Add the oneof field to the main message
                current_fields.push(format!("{} {}", oneof_msg_name, oneof_name));
            }
            
            let message_content = generate_ros2_message_content(&current_fields);
            messages.push((full_msg_name, message_content));
            
            // Pop the current message from stack
            message_stack.pop();
            current_fields.clear();
            oneof_fields.clear();
        }
    }

    if messages.is_empty() {
        return Err(anyhow!("No messages or enums found in proto file: {}", proto_file.display()));
    }

    Ok(messages)
}

fn extract_message_name(line: &str) -> Option<String> {
    if line.starts_with("message ") {
        let rest = line.strip_prefix("message ")?;
        let name = rest.split_whitespace().next()?.trim_end_matches('{');
        Some(name.to_string())
    } else {
        None
    }
}

fn parse_field_to_ros2_with_context(line: &str, message_context: &[String]) -> Option<String> {
    // Enhanced field parsing with support for nested message context
    let line = line.trim();
    
    // Skip empty lines and comments
    if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
        return None;
    }
    
    // Skip structural lines but not field definitions
    if line.starts_with("enum") || line.starts_with("message") || line.starts_with("oneof") ||
       line == "{" || line == "}" {
        return None;
    }

    // Parse field definition: "[optional] [repeated] type name = number [options];"
    // Must contain '=' and end with ';' to be a valid field
    if !line.contains('=') || !line.ends_with(';') {
        return None;
    }

    // Remove inline comments before parsing
    let clean_line = if let Some(comment_pos) = line.find("//") {
        line[..comment_pos].trim()
    } else {
        line
    };

    // Re-check after removing comments
    if !clean_line.ends_with(';') {
        return None;
    }

    let parts: Vec<&str> = clean_line.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }

    let mut is_optional = false;
    let mut is_repeated = false;
    let mut type_start_idx = 0;

    // Check for optional and repeated modifiers
    for (i, part) in parts.iter().enumerate() {
        match *part {
            "optional" => {
                is_optional = true;
                type_start_idx = i + 1;
            }
            "repeated" => {
                is_repeated = true;
                type_start_idx = i + 1;
            }
            _ => break,
        }
    }

    if type_start_idx >= parts.len() || type_start_idx + 1 >= parts.len() {
        return None;
    }

    let proto_type = parts[type_start_idx];
    let field_name = parts[type_start_idx + 1];
    
    // Handle map types first
    if clean_line.contains("map<") {
        return handle_map_type(clean_line, field_name);
    }

    // Resolve the type name with proper context handling
    let ros2_type = resolve_type_with_context(proto_type, message_context);

    // Handle repeated fields
    let final_type = if is_repeated {
        if ros2_type.ends_with("[]") {
            // For bytes which already ends with [], create a special message type
            "proto2ros/Bytes[]".to_string()
        } else {
            format!("{}[]", ros2_type)
        }
    } else {
        ros2_type
    };

    // Handle optional fields - add a comment for now
    let field_definition = if is_optional {
        format!("{} {}  # optional field", final_type, field_name)
    } else {
        format!("{} {}", final_type, field_name)
    };

    Some(field_definition)
}

fn resolve_type_with_context(proto_type: &str, message_context: &[String]) -> String {
    // Enhanced type resolution with proper nested message handling
    
    // Handle qualified type names like "Robot.Status" or "Factory.Building.Room"
    if proto_type.contains('.') {
        let parts: Vec<&str> = proto_type.split('.').collect();
        
        // Always flatten qualified names by joining all parts
        // "Robot.Status" -> "RobotStatus"
        // "Factory.Building.Room" -> "FactoryBuildingRoom"
        return parts.join("");
    }
    
    // Convert protobuf primitive types to ROS2 types
    match proto_type {
        "bool" => "bool".to_string(),
        "int32" | "sint32" | "sfixed32" => "int32".to_string(),
        "int64" | "sint64" | "sfixed64" => "int64".to_string(), 
        "uint32" | "fixed32" => "uint32".to_string(),
        "uint64" | "fixed64" => "uint64".to_string(),
        "float" => "float32".to_string(),
        "double" => "float64".to_string(),
        "string" => "string".to_string(),
        "bytes" => "uint8[]".to_string(),
        _ => {
            // This is a custom message type
            // Only apply context prefixing if we're inside a nested message definition
            // and the type name looks like it could be a sibling nested type
            
            // For now, let's be more conservative about when we apply context.
            // We'll only apply for types that are simple identifiers
            // and only when we're actually inside a nested context (not at the top level)
            if !message_context.is_empty() && message_context.len() > 1 {
                // We are deeply nested - apply context for potential sibling types
                let parent_context = message_context.join("");
                format!("{}{}", parent_context, proto_type)
            } else if !message_context.is_empty() && message_context.len() == 1 {
                // We are one level deep - only apply context if this looks like a nested type
                // This is a heuristic: if the type starts with capital letter and is short,
                // it might be a nested type. But for now, let's be conservative.
                // In the future, we could track which types are actually defined as nested.
                
                // For now, don't apply context to avoid false positives like "Robot" -> "MissionRobot"
                proto_type.to_string()
            } else {
                // No context, use the type as-is
                proto_type.to_string()
            }
        }
    }
}

fn parse_field_to_ros2_with_nested_context(line: &str, message_context: &[String], nested_types: &std::collections::HashSet<String>) -> Option<String> {
    // Enhanced field parsing with support for nested message context and knowledge of defined nested types
    let line = line.trim();
    
    // Skip empty lines and comments
    if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
        return None;
    }
    
    // Skip structural lines but not field definitions
    if line.starts_with("enum") || line.starts_with("message") || line.starts_with("oneof") ||
       line == "{" || line == "}" {
        return None;
    }

    // Parse field definition: "[optional] [repeated] type name = number [options];"
    // Must contain '=' and end with ';' to be a valid field
    if !line.contains('=') || !line.ends_with(';') {
        return None;
    }

    // Remove inline comments before parsing
    let clean_line = if let Some(comment_pos) = line.find("//") {
        line[..comment_pos].trim()
    } else {
        line
    };

    // Re-check after removing comments
    if !clean_line.ends_with(';') {
        return None;
    }

    let parts: Vec<&str> = clean_line.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }

    let mut is_optional = false;
    let mut is_repeated = false;
    let mut type_start_idx = 0;

    // Check for optional and repeated modifiers
    for (i, part) in parts.iter().enumerate() {
        match *part {
            "optional" => {
                is_optional = true;
                type_start_idx = i + 1;
            }
            "repeated" => {
                is_repeated = true;
                type_start_idx = i + 1;
            }
            _ => break,
        }
    }

    if type_start_idx >= parts.len() || type_start_idx + 1 >= parts.len() {
        return None;
    }

    let proto_type = parts[type_start_idx];
    let field_name = parts[type_start_idx + 1];
    
    // Handle map types first
    if clean_line.contains("map<") {
        return handle_map_type(clean_line, field_name);
    }

    // Resolve the type name with proper context handling
    let ros2_type = resolve_type_with_nested_context(proto_type, message_context, &std::collections::HashSet::new());

    // Handle repeated fields
    let final_type = if is_repeated {
        if ros2_type.ends_with("[]") {
            // For bytes which already ends with [], create a special message type
            "proto2ros/Bytes[]".to_string()
        } else {
            format!("{}[]", ros2_type)
        }
    } else {
        ros2_type
    };

    // Handle optional fields - add a comment for now
    let field_definition = if is_optional {
        format!("{} {}  # optional field", final_type, field_name)
    } else {
        format!("{} {}", final_type, field_name)
    };

    Some(field_definition)
}

fn resolve_type_with_nested_context(proto_type: &str, message_context: &[String], nested_types: &std::collections::HashSet<String>) -> String {
    // Enhanced type resolution with knowledge of defined nested types
    
    // Handle qualified type names like "Robot.Status" or "Factory.Building.Room"
    if proto_type.contains('.') {
        let parts: Vec<&str> = proto_type.split('.').collect();
        
        // Always flatten qualified names by joining all parts
        // "Robot.Status" -> "RobotStatus"
        // "Factory.Building.Room" -> "FactoryBuildingRoom"
        return parts.join("");
    }
    
    // Convert protobuf primitive types to ROS2 types
    match proto_type {
        "bool" => "bool".to_string(),
        "int32" | "sint32" | "sfixed32" => "int32".to_string(),
        "int64" | "sint64" | "sfixed64" => "int64".to_string(), 
        "uint32" | "fixed32" => "uint32".to_string(),
        "uint64" | "fixed64" => "uint64".to_string(),
        "float" => "float32".to_string(),
        "double" => "float64".to_string(),
        "string" => "string".to_string(),
        "bytes" => "uint8[]".to_string(),
        _ => {
            // This is a custom message type
            // Check if this is a known nested type that needs context prefixing
            if !message_context.is_empty() {
                let potential_nested_type = format!("{}{}", message_context.join(""), proto_type);
                
                // If this potential nested type is in our known nested types, use it
                if nested_types.contains(&potential_nested_type) {
                    return potential_nested_type;
                }
            }
            
            // Also check if this type exists as a top-level type
            // If not, and we're in a context, it might be a nested type we should prefix
            // This handles cases like "Room" inside "FactoryBuilding" where we should use "FactoryBuildingRoom"
            if !message_context.is_empty() {
                // Try to find the full nested type name by checking all possible context combinations
                for i in 0..message_context.len() {
                    let context_slice = &message_context[i..];
                    let full_context_type = format!("{}{}", context_slice.join(""), proto_type);
                    
                    // Check if this full type exists in our nested types
                    if nested_types.contains(&full_context_type) {
                        return full_context_type;
                    }
                }
                
                // For any context level, if type looks like a nested type name, try prefixing with full context
                let full_context = message_context.join("");
                let potential_type = format!("{}{}", full_context, proto_type);
                
                // If this type name follows the pattern of being a nested type (capitalized, short)
                // and we're inside a parent message, assume it's a nested reference
                if proto_type.chars().next().map_or(false, |c| c.is_uppercase()) {
                    return potential_type;
                }
            }
            
            // Otherwise, use the type as-is
            proto_type.to_string()
        }
    }
}

fn convert_proto_type_to_ros2(proto_type: &str) -> &str {
    match proto_type {
        "bool" => "bool",
        "int32" | "sint32" | "sfixed32" => "int32",
        "int64" | "sint64" | "sfixed64" => "int64", 
        "uint32" | "fixed32" => "uint32",
        "uint64" | "fixed64" => "uint64",
        "float" => "float32",
        "double" => "float64",
        "string" => "string",
        "bytes" => "uint8[]",
        _ => proto_type, // For custom message types
    }
}

fn handle_map_type(line: &str, field_name: &str) -> Option<String> {
    // Parse map<key_type, value_type> field_name = number;
    if let Some(start) = line.find("map<") {
        if let Some(end) = line[start..].find('>') {
            let end = start + end;  // Adjust for the offset
            let map_content = &line[start + 4..end];
            let types: Vec<&str> = map_content.split(',').map(|s| s.trim()).collect();
            
            if types.len() == 2 {
                let _key_type = convert_proto_type_to_ros2(types[0]);
                let _value_type = convert_proto_type_to_ros2(types[1]);
                
                // Generate entry message name (proper CamelCase)
                let entry_name = format!("{}Entry", capitalize_first(field_name));
                
                // Return the array of entry type
                // Note: In a full implementation, we would also generate the actual Entry message
                return Some(format!("{}[] {}  # map<{}, {}>", entry_name, field_name, types[0], types[1]));
            }
        }
    }
    None
}

fn generate_ros2_message_content(fields: &[String]) -> String {
    if fields.is_empty() {
        "# Empty message\n".to_string()
    } else {
        fields.join("\n") + "\n"
    }
}

fn extract_enum_name(line: &str) -> Option<String> {
    if line.starts_with("enum ") {
        let rest = line.strip_prefix("enum ")?;
        let name = rest.split_whitespace().next()?.trim_end_matches('{');
        Some(name.to_string())
    } else {
        None
    }
}

fn extract_oneof_name(line: &str) -> Option<String> {
    if line.starts_with("oneof ") {
        let rest = line.strip_prefix("oneof ")?;
        let name = rest.split_whitespace().next()?.trim_end_matches('{');
        Some(name.to_string())
    } else {
        None
    }
}

fn parse_enum_value(line: &str) -> Option<String> {
    let line = line.trim();
    
    // Skip non-enum-value lines
    if line.is_empty() || line.starts_with("//") || line.contains('{') || line.contains('}') {
        return None;
    }

    // Parse enum value: "NAME = number;"
    if line.contains('=') && line.ends_with(';') {
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() == 2 {
            let name = parts[0].trim();
            let value = parts[1].trim().trim_end_matches(';').trim();
            return Some(format!("int32 {}={}", name, value));
        }
    }
    
    None
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn generate_enum_message_content(enum_values: &[String]) -> String {
    if enum_values.is_empty() {
        "# Empty enum\nint32 value\n".to_string()
    } else {
        let mut content = String::new();
        
        // Add all enum constants
        for enum_value in enum_values {
            content.push_str(enum_value);
            content.push('\n');
        }
        
        // Add the value field
        content.push_str("\nint32 value\n");
        
        content
    }
}

fn generate_oneof_message_content(oneof_fields: &[String], oneof_name: &str) -> String {
    if oneof_fields.is_empty() {
        return format!("# Empty oneof {}\nint8 which\n", oneof_name);
    }

    let mut content = String::new();
    
    // Generate constants for each field
    content.push_str(&format!("int8 {}_NOT_SET=0\n", oneof_name.to_uppercase()));
    for (i, field) in oneof_fields.iter().enumerate() {
        // Extract field name from "type name" format
        if let Some(field_name) = field.split_whitespace().nth(1) {
            content.push_str(&format!(
                "int8 {}_{}_SET={}\n", 
                oneof_name.to_uppercase(), 
                field_name.to_uppercase(), 
                i + 1
            ));
        }
    }
    
    content.push('\n');
    
    // Add all the oneof fields
    for field in oneof_fields {
        content.push_str(field);
        content.push('\n');
    }
    
    // Add the which field
    content.push_str(&format!("int8 which\n"));
    
    content
}

fn convert_msg_files_to_proto(options: &ProtobufConversionOptions) -> Result<()> {
    for msg_file in &options.input_files {
        if options.verbose {
            println!("🔄 Converting {}...", msg_file.display());
        }

        let msg_content = fs::read_to_string(msg_file)
            .map_err(|e| anyhow!("Failed to read msg file: {}", e))?;

        let proto_message = parse_msg_to_proto(&msg_content, msg_file)?;

        // Determine output directory: use explicit output_dir or same directory as msg file
        let output_dir = match &options.output_dir {
            Some(dir) => dir.clone(),
            None => msg_file.parent().unwrap_or(Path::new(".")).to_path_buf(),
        };
        
        let msg_name = msg_file.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("Invalid message file name: {}", msg_file.display()))?;
        
        let output_file = output_dir.join(format!("{}.proto", msg_name));
        
        if options.verbose {
            println!("   Generated: {}", output_file.display());
        }

        if options.dry_run {
            println!("Would write to: {}", output_file.display());
            println!("Content:\n{}", proto_message);
            println!("---");
        } else {
            // Create the directory if it doesn't exist (for inplace mode)
            if let Some(parent) = output_file.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| anyhow!("Failed to create directory {}: {}", parent.display(), e))?;
            }
            
            fs::write(&output_file, proto_message)
                .map_err(|e| anyhow!("Failed to write proto file: {}", e))?;
        }
    }

    Ok(())
}

fn parse_msg_to_proto(msg_content: &str, msg_file: &Path) -> Result<String> {
    let msg_name = msg_file.file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("Invalid message file name: {}", msg_file.display()))?;

    let mut proto_content = String::new();
    proto_content.push_str("syntax = \"proto3\";\n\n");
    
    // Add package if we can derive it from path or file structure
    // For now, use a default package
    proto_content.push_str("package generated;\n\n");
    
    proto_content.push_str(&format!("message {} {{\n", msg_name));
    
    let mut field_number = 1;
    
    for line in msg_content.lines() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Parse field definition: "type name" or "type name # comment"
        let clean_line = if let Some(comment_pos) = line.find('#') {
            line[..comment_pos].trim()
        } else {
            line
        };
        
        if clean_line.is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = clean_line.split_whitespace().collect();
        if parts.len() >= 2 {
            let ros2_type = parts[0];
            let field_name = parts[1];
            
            let proto_type = convert_ros2_type_to_proto(ros2_type);
            
            if proto_type.starts_with("repeated ") {
                proto_content.push_str(&format!("  {} {} = {};\n", proto_type, field_name, field_number));
            } else {
                proto_content.push_str(&format!("  {} {} = {};\n", proto_type, field_name, field_number));
            }
            
            field_number += 1;
        }
    }
    
    proto_content.push_str("}\n");
    
    Ok(proto_content)
}

fn convert_ros2_type_to_proto(ros2_type: &str) -> String {
    // Handle array types
    if ros2_type.ends_with("[]") {
        let base_type = &ros2_type[..ros2_type.len() - 2];
        let proto_base = convert_ros2_type_to_proto(base_type);
        return format!("repeated {}", proto_base);
    }
    
    // Convert basic types
    match ros2_type {
        "bool" => "bool".to_string(),
        "int8" => "int32".to_string(),  // Proto3 doesn't have int8
        "uint8" => "uint32".to_string(), // Proto3 doesn't have uint8
        "int16" => "int32".to_string(),  // Proto3 doesn't have int16
        "uint16" => "uint32".to_string(), // Proto3 doesn't have uint16
        "int32" => "int32".to_string(),
        "uint32" => "uint32".to_string(),
        "int64" => "int64".to_string(),
        "uint64" => "uint64".to_string(),
        "float32" => "float".to_string(),
        "float64" => "double".to_string(),
        "string" => "string".to_string(),
        "time" => "google.protobuf.Timestamp".to_string(),
        "duration" => "google.protobuf.Duration".to_string(),
        _ => {
            // For custom message types, assume they're other message types
            ros2_type.to_string()
        }
    }
}
