# Protobuf Integration

ROC provides comprehensive support for Protocol Buffers (Protobuf), enabling seamless conversion between `.proto` and ROS2 `.msg` formats. This chapter details the technical implementation and capabilities of ROC's Protobuf integration.

## Protobuf Parser Implementation

ROC implements a pure Rust Protobuf parser that handles the complete proto3 specification without external dependencies. The parser is built with performance and accuracy in mind.

### Supported Protobuf Features

#### Basic Syntax Elements
- **Syntax Declaration**: `syntax = "proto3";`
- **Package Declaration**: `package com.example.robotics;`
- **Import Statements**: `import "google/protobuf/timestamp.proto";`
- **Comments**: Single-line (`//`) and multi-line (`/* */`) comments

#### Message Definitions
```protobuf
message RobotStatus {
  // Basic field definition
  string name = 1;
  int32 id = 2;
  bool active = 3;
  
  // Repeated fields (arrays)
  repeated double joint_positions = 4;
  repeated string error_messages = 5;
}
```

#### Nested Messages
```protobuf
message Robot {
  message Status {
    bool online = 1;
    string last_error = 2;
  }
  
  Status current_status = 1;
  string robot_id = 2;
}
```

**Flattening Behavior**: ROC automatically flattens nested message names:
- `Robot.Status` becomes `RobotStatus.msg`
- `Sensor.Camera.Configuration` becomes `SensorCameraConfiguration.msg`

#### Enumerations
```protobuf
enum RobotState {
  ROBOT_STATE_UNKNOWN = 0;
  ROBOT_STATE_IDLE = 1;
  ROBOT_STATE_MOVING = 2;
  ROBOT_STATE_ERROR = 3;
}

message RobotCommand {
  RobotState desired_state = 1;
}
```

ROC converts enums to ROS2 constants within messages:
```msg
# RobotCommand.msg
uint8 ROBOT_STATE_UNKNOWN=0
uint8 ROBOT_STATE_IDLE=1
uint8 ROBOT_STATE_MOVING=2
uint8 ROBOT_STATE_ERROR=3
uint8 desired_state
```

#### Oneof Fields
```protobuf
message Command {
  oneof command_type {
    string text_command = 1;
    int32 numeric_command = 2;
    bool boolean_command = 3;
  }
}
```

Oneof fields are converted to separate optional fields in ROS2:
```msg
# Command.msg
string text_command
int32 numeric_command  
bool boolean_command
```

#### Map Types
```protobuf
message Configuration {
  map<string, string> parameters = 1;
  map<int32, double> sensor_readings = 2;
}
```

Maps are converted to arrays of key-value pairs:
```msg
# Configuration.msg
# Generated from map<string, string> parameters
ConfigurationParametersEntry[] parameters
# Generated from map<int32, double> sensor_readings  
ConfigurationSensorReadingsEntry[] sensor_readings

# ConfigurationParametersEntry.msg
string key
string value

# ConfigurationSensorReadingsEntry.msg
int32 key
float64 value
```

## Type Conversion System

ROC implements intelligent type mapping between Protobuf and ROS2 type systems:

### Primitive Types

| Protobuf Type | ROS2 Type | Notes |
|---------------|-----------|-------|
| `bool` | `bool` | Direct mapping |
| `int32` | `int32` | Direct mapping |
| `int64` | `int64` | Direct mapping |
| `uint32` | `uint32` | Direct mapping |
| `uint64` | `uint64` | Direct mapping |
| `sint32` | `int32` | Signed integer |
| `sint64` | `int64` | Signed integer |
| `fixed32` | `uint32` | Fixed-width unsigned |
| `fixed64` | `uint64` | Fixed-width unsigned |
| `sfixed32` | `int32` | Fixed-width signed |
| `sfixed64` | `int64` | Fixed-width signed |
| `float` | `float32` | Single precision |
| `double` | `float64` | Double precision |
| `string` | `string` | UTF-8 strings |
| `bytes` | `uint8[]` | Byte arrays |

### Repeated Fields
Protobuf repeated fields map directly to ROS2 arrays:
```protobuf
repeated double values = 1;        // → float64[] values
repeated string names = 2;         // → string[] names
repeated RobotStatus robots = 3;   // → RobotStatus[] robots
```

### Well-Known Types
ROC provides mappings for common Protobuf well-known types:

```protobuf
import "google/protobuf/timestamp.proto";
import "google/protobuf/duration.proto";

message TimedData {
  google.protobuf.Timestamp timestamp = 1;  // → builtin_interfaces/Time
  google.protobuf.Duration timeout = 2;     // → builtin_interfaces/Duration
}
```

## Conversion Process

### Proto to Msg Conversion

1. **Parsing**: Parse `.proto` file into abstract syntax tree
2. **Validation**: Validate proto3 syntax and semantic rules
3. **Dependency Analysis**: Build dependency graph of message types
4. **Type Resolution**: Resolve all type references and nested definitions
5. **Flattening**: Flatten nested messages into separate files
6. **Generation**: Generate `.msg` files in dependency order

### Msg to Proto Conversion

1. **Parsing**: Parse `.msg` files and extract field definitions
2. **Type Mapping**: Convert ROS2 types to Protobuf equivalents
3. **Packaging**: Organize messages into appropriate proto packages
4. **Generation**: Generate `.proto` files with proper syntax

## Advanced Features

### Comment Preservation
ROC preserves comments during conversion when possible:

```protobuf
// This is a robot status message
message RobotStatus {
  // The robot's unique identifier
  string id = 1;
  
  // Whether the robot is currently active
  bool active = 2;
}
```

Becomes:
```msg
# This is a robot status message
# The robot's unique identifier
string id
# Whether the robot is currently active
bool active
```

### Package Handling
ROC intelligently handles package declarations:

- **Proto to Msg**: Uses package name as prefix for generated message names
- **Msg to Proto**: Groups related messages into logical packages
- **Namespace Mapping**: Converts between proto packages and ROS2 namespaces

### Import Resolution
For proto files with imports, ROC:
1. Tracks imported dependencies
2. Generates corresponding ROS2 message files
3. Updates field references to use correct message types
4. Maintains dependency order in output

## Error Handling and Validation

ROC provides comprehensive error reporting:

### Syntax Errors
```
Error parsing robot.proto:5:10
  |
5 | message Robot {
  |          ^^^^^ Expected message name
```

### Semantic Errors
```
Error: Undefined message type 'UnknownStatus' referenced in field 'status'
  --> robot.proto:15:3
```

### Conversion Warnings
```
Warning: Oneof field 'command_type' converted to separate optional fields
Note: ROS2 messages don't support oneof semantics
```

## Performance Characteristics

ROC's Protobuf implementation is optimized for:

- **Speed**: Pure Rust implementation with zero-copy parsing where possible
- **Memory**: Minimal memory allocations during parsing
- **Scalability**: Handles large proto files and complex dependency graphs
- **Reliability**: Comprehensive error handling and validation

## Usage Examples

### Basic Conversion
```bash
# Convert proto to msg
roc idl protobuf robot_api.proto sensor_data.proto

# Convert msg to proto
roc idl protobuf RobotStatus.msg SensorReading.msg
```

### Advanced Options
```bash
# Specify output directory
roc idl protobuf --output ./generated *.proto

# Dry run to preview output
roc idl protobuf --dry-run complex_robot.proto

# Verbose output for debugging
roc idl protobuf --verbose robot_messages/*.proto
```

### Integration with Build Systems
```bash
# Generate messages as part of build process
roc idl protobuf src/proto/*.proto --output msg/
colcon build --packages-select my_robot_interfaces
```
