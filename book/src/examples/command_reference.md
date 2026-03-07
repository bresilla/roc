# Command Reference

This chapter provides a comprehensive reference for all `roc` tool commands, options, and usage patterns.

## Global Options

All `roc` commands support these global options:

| Option | Description | Default |
|--------|-------------|---------|
| `-h, --help` | Show help information | N/A |
| `-V, --version` | Show version information | N/A |

## Environment Variables

The `roc` tool respects standard ROS 2 environment variables:

| Variable | Description | Example |
|----------|-------------|---------|
| `ROS_DOMAIN_ID` | ROS 2 domain ID | `export ROS_DOMAIN_ID=0` |
| `RMW_IMPLEMENTATION` | RMW middleware implementation | `export RMW_IMPLEMENTATION=rmw_cyclone_cpp` |
| `ROS_LOCALHOST_ONLY` | Limit communication to localhost | `export ROS_LOCALHOST_ONLY=1` |

## Command Structure

```
roc <COMMAND> [SUBCOMMAND] [OPTIONS] [ARGS]
```

## Shell Completion

### `roc completion`

Generate or install shell completion scripts for `roc`.

**Syntax:**
```bash
roc completion <bash|zsh|fish> [--install|--print-path]
```

**Examples:**
```bash
# Print the completion script to stdout
roc completion bash

# Show the default installation target
roc completion zsh --print-path

# Install into the preferred shell-specific location
roc completion fish --install
```

**Notes:**
- `--install` prefers user-local locations first and falls back to common system paths.
- `--print-path` shows where `--install` will try to write first.
- Generated completions include dynamic values for topics, services, nodes, workspaces, and other graph-aware commands.
- Dynamic completions use a short-lived filesystem cache by default.
- Set `ROC_DISABLE_COMPLETION_CACHE=1` to disable that cache.
- Set `ROC_COMPLETION_CACHE_DIR=/path/to/cache` to override the cache location.

## Topic Commands

### `roc topic list`

List all available topics in the ROS 2 graph.

**Syntax:**
```bash
roc topic list
```

**Output:**
```
/chatter
/parameter_events
/rosout
```

**Exit Codes:**
- `0`: Success
- `1`: Error (no ROS 2 system found, permission issues, etc.)

**Examples:**
```bash
# Basic usage
roc topic list

# Count topics
roc topic list | wc -l

# Filter topics
roc topic list | grep "chatter"

# Store topics in variable
topics=$(roc topic list)
```

### `roc topic info`

Display detailed information about a specific topic.

**Syntax:**
```bash
roc topic info <TOPIC_NAME> [OPTIONS]
```

**Arguments:**
- `<TOPIC_NAME>`: The name of the topic to inspect (required)

**Options:**
| Option | Short | Description |
|--------|-------|-------------|
| `--verbose` | `-v` | Show detailed information including QoS profiles and endpoint data |

**Basic Output:**
```
Topic: /chatter
Type: std_msgs/msg/String
Publishers: 1
Subscribers: 0
```

**Verbose Output:**
```
Topic: /chatter
Type: std_msgs/msg/String
Publishers: 1
  Node: /talker
  Endpoint type: Publisher
  GID: 01.0f.xx.xx.xx.xx.xx.xx.xx.xx.xx.xx.xx.xx.xx.xx
  QoS Profile:
    Reliability: Reliable
    Durability: Volatile
    History: Keep last
    Depth: 10
    Deadline: Default
    Lifespan: Default
    Liveliness: Automatic
    Liveliness lease duration: Default
  Type hash: RIHS01_xxxxxxxxxxxxxxxxxxxxxxxxxxxx

Subscribers: 0
```

**Exit Codes:**
- `0`: Success
- `1`: Topic not found or error accessing topic information
- `2`: Invalid arguments

**Examples:**
```bash
# Basic topic information
roc topic info /chatter

# Detailed information with QoS profiles
roc topic info /chatter --verbose
roc topic info /chatter -v

# Check if topic exists (using exit code)
if roc topic info /my_topic > /dev/null 2>&1; then
    echo "Topic exists"
else
    echo "Topic not found"
fi

# Get only publisher count
roc topic info /chatter | grep "Publishers:" | awk '{print $2}'
```

## Output Format Details

### Topic Information Fields

| Field | Description | Example |
|-------|-------------|---------|
| **Topic** | Full topic name | `/chatter` |
| **Type** | Message type | `std_msgs/msg/String` |
| **Publishers** | Number of active publishers | `1` |
| **Subscribers** | Number of active subscribers | `0` |

### Verbose Information Fields

#### Publisher/Subscriber Details
| Field | Description | Example |
|-------|-------------|---------|
| **Node** | Node name | `/talker` |
| **Endpoint type** | Publisher or Subscriber | `Publisher` |
| **GID** | Global identifier (16 bytes, hex) | `01.0f.xx.xx...` |
| **Type hash** | Message type hash | `RIHS01_xxx...` |

#### QoS Profile Fields
| Field | Description | Possible Values |
|-------|-------------|-----------------|
| **Reliability** | Message delivery guarantee | `Reliable`, `Best effort` |
| **Durability** | Message persistence | `Volatile`, `Transient local`, `Transient`, `Persistent` |
| **History** | History policy | `Keep last`, `Keep all` |
| **Depth** | History depth (for Keep last) | `1`, `10`, `100`, etc. |
| **Deadline** | Message deadline | `Default`, time duration |
| **Lifespan** | Message lifespan | `Default`, time duration |
| **Liveliness** | Liveliness policy | `Automatic`, `Manual by node`, `Manual by topic` |
| **Liveliness lease duration** | Lease duration | `Default`, time duration |

## Error Handling

### Common Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| `No topics found` | No ROS 2 nodes running | Start ROS 2 nodes or check `ROS_DOMAIN_ID` |
| `Topic not found: /topic_name` | Specified topic doesn't exist | Verify topic name with `roc topic list` |
| `Permission denied` | Insufficient permissions | Check user permissions and ROS 2 setup |
| `Failed to create context` | ROS 2 not properly initialized | Source ROS 2 setup and check environment |
| `Timeout waiting for topic info` | Network or discovery issues | Check network connectivity and RMW configuration |

### Debugging Commands

```bash
# Check ROS 2 environment
printenv | grep ROS

# Verify RMW implementation
echo $RMW_IMPLEMENTATION

# Test basic connectivity
roc topic list

# Verbose debugging (if available)
RUST_LOG=debug roc topic info /chatter --verbose
```

## Return Codes

All `roc` commands follow standard Unix conventions:

| Code | Meaning | When Used |
|------|---------|-----------|
| `0` | Success | Command completed successfully |
| `1` | General error | Topic not found, ROS 2 system unavailable |
| `2` | Invalid arguments | Wrong number of arguments, invalid options |
| `130` | Interrupted | Command interrupted by user (Ctrl+C) |

## Performance Considerations

### Command Performance

| Command | Typical Time | Notes |
|---------|--------------|-------|
| `roc topic list` | < 100ms | Fast, caches discovery data |
| `roc topic info` | < 200ms | May be slower for first query |
| `roc topic info --verbose` | < 500ms | Additional QoS/endpoint queries |

### Optimization Tips

1. **Batch Operations**: Use `roc topic list` once, then query specific topics
2. **Caching**: Results are cached briefly to improve repeated queries
3. **Network**: Use `ROS_LOCALHOST_ONLY=1` for local-only discovery
4. **RMW Selection**: Different RMW implementations have different performance characteristics

## Comparison with ROS 2 CLI

### Feature Parity

| Feature | `ros2 topic` | `roc topic` | Notes |
|---------|--------------|-------------|-------|
| List topics | ✅ | ✅ | Full parity |
| Basic info | ✅ | ✅ | Full parity |
| Verbose info | ✅ | ✅ | Full parity with QoS details |
| Publisher count | ✅ | ✅ | Exact match |
| Subscriber count | ✅ | ✅ | Exact match |
| GID information | ✅ | ✅ | Formatted identically |
| Type hash | ✅ | ✅ | Complete hash information |

### Performance Comparison

```bash
# Benchmark both tools
time ros2 topic list
time roc topic list

time ros2 topic info /chatter --verbose
time roc topic info /chatter --verbose
```

Typical results show `roc` is 2-3x faster for most operations.

## Scripting and Automation

### Common Patterns

```bash
# Check if specific topics exist
check_topics() {
    local required_topics=("$@")
    local missing_topics=()
    
    for topic in "${required_topics[@]}"; do
        if ! roc topic info "$topic" > /dev/null 2>&1; then
            missing_topics+=("$topic")
        fi
    done
    
    if [ ${#missing_topics[@]} -eq 0 ]; then
        echo "All required topics found"
        return 0
    else
        echo "Missing topics: ${missing_topics[*]}"
        return 1
    fi
}

# Usage
check_topics "/chatter" "/rosout" "/parameter_events"
```

```bash
# Get topic statistics
get_topic_stats() {
    local topics=($(roc topic list))
    local total_pubs=0
    local total_subs=0
    
    for topic in "${topics[@]}"; do
        local info=$(roc topic info "$topic")
        local pubs=$(echo "$info" | grep "Publishers:" | awk '{print $2}')
        local subs=$(echo "$info" | grep "Subscribers:" | awk '{print $2}')
        
        total_pubs=$((total_pubs + pubs))
        total_subs=$((total_subs + subs))
    done
    
    echo "Topics: ${#topics[@]}"
    echo "Total publishers: $total_pubs"
    echo "Total subscribers: $total_subs"
}
```

### JSON Output (Future Enhancement)

While not currently supported, JSON output could be added:

```bash
# Proposed syntax (not yet implemented)
roc topic list --format json
roc topic info /chatter --format json --verbose
```

## IDL Commands

### `roc idl protobuf`

Bidirectional conversion between Protobuf (.proto) and ROS 2 (.msg) files with automatic direction detection.

**Syntax:**
```bash
roc idl protobuf [OPTIONS] <INPUT_FILES>...
```

**Arguments:**
- `<INPUT_FILES>...`: Input files to convert (.proto or .msg files)

**Options:**
| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--output <DIR>` | `-o` | Output directory for generated files | Same directory as input |
| `--package <NAME>` | `-p` | Package name for generated files | Derived from input |
| `--config <FILE>` | `-c` | Configuration file for type mappings (YAML) | None |
| `--include <DIRS>...` | `-I` | Include directories for protobuf imports | None |
| `--verbose` | `-v` | Show verbose output | False |
| `--dry-run` | `-n` | Show what would be generated without writing files | False |

**Examples:**

```bash
# Convert .proto files to .msg files (automatic detection)
roc idl protobuf robot.proto sensor_data.proto

# Convert .msg files to .proto files (automatic detection)  
roc idl protobuf RobotStatus.msg SensorData.msg

# Specify output directory
roc idl protobuf --output ./generated robot.proto

# Dry run to preview conversion
roc idl protobuf --dry-run --verbose robot.proto

# Convert with include directories for imports
roc idl protobuf -I ./proto_deps -I ./common robot.proto

# Convert with custom package name
roc idl protobuf --package my_robot_msgs robot.proto
```

**Protobuf to ROS2 (.proto → .msg):**
```bash
# Input: robot.proto
roc idl protobuf robot.proto

# Output: Robot.msg, RobotStatus.msg (based on message definitions)
```

**ROS2 to Protobuf (.msg → .proto):**
```bash
# Input: RobotStatus.msg
roc idl protobuf RobotStatus.msg  

# Output: robot_status.proto
```

**Advanced Usage:**
```bash
# Convert entire directory with verbose output
roc idl protobuf --verbose src/proto/*.proto --output msg/

# Mixed conversion with error handling
roc idl protobuf file1.proto file2.proto || echo "Conversion failed"

# Pipeline with other tools
find . -name "*.proto" -exec roc idl protobuf {} --output ./ros_msgs \;
```

**Supported Protobuf Features:**
- Proto3 syntax
- Nested messages (automatically flattened)
- Enums (converted to constants)
- Repeated fields (arrays)
- Maps (converted to key-value arrays)
- Oneof fields (converted to separate optional fields)
- Comments (preserved when possible)
- Import statements and dependencies

**Type Mappings:**
| Protobuf | ROS2 | Notes |
|----------|------|-------|
| `bool` | `bool` | Direct mapping |
| `int32` | `int32` | Direct mapping |
| `int64` | `int64` | Direct mapping |
| `uint32` | `uint32` | Direct mapping |
| `uint64` | `uint64` | Direct mapping |
| `float` | `float32` | Single precision |
| `double` | `float64` | Double precision |
| `string` | `string` | UTF-8 strings |
| `bytes` | `uint8[]` | Byte arrays |
| `repeated T` | `T[]` | Dynamic arrays |
| `map<K,V>` | `Entry[]` | Key-value pairs |

**Exit Codes:**
- `0`: Success
- `1`: Error (invalid syntax, file not found, permission issues, etc.)

**Error Examples:**
```bash
# Mixed file types (not allowed)
roc idl protobuf robot.proto RobotStatus.msg
# Error: Cannot mix .proto and .msg files in the same conversion

# Unsupported file extension
roc idl protobuf data.json
# Error: Unsupported file extension: .json

# File not found
roc idl protobuf nonexistent.proto
# Error: Input file does not exist: nonexistent.proto
```

## Troubleshooting

### Common Issues

1. **No output from `roc topic list`**
   - Check if ROS 2 nodes are running: `ros2 node list`
   - Verify ROS 2 environment: `echo $ROS_DOMAIN_ID`
   - Try different RMW: `export RMW_IMPLEMENTATION=rmw_cyclone_cpp`

2. **Permission errors**
   - Check user groups: `groups`
   - Verify ROS 2 installation permissions
   - Try running with different user

3. **Slow performance**
   - Check network configuration
   - Use `ROS_LOCALHOST_ONLY=1` for local testing
   - Consider different RMW implementation

4. **Inconsistent results**
   - Allow time for discovery: `sleep 2 && roc topic list`
   - Check for multiple ROS 2 domains
   - Verify system clock synchronization

### Debug Information

```bash
# Enable detailed logging (if built with debug support)
RUST_LOG=debug roc topic list

# Check system resources
free -h
df -h

# Network diagnostics
netstat -tuln | grep -E "(7400|7401|7411)"
```

This completes the comprehensive command reference for the `roc` tool.
