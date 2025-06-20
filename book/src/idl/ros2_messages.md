# ROS2 Message System

ROS2 uses a simple yet powerful message definition format that enables efficient communication between nodes. This chapter explains how ROC processes and converts ROS2 message definitions.

## ROS2 Message Format

ROS2 messages are defined in `.msg` files using a straightforward syntax:

### Basic Message Structure
```msg
# Comments start with hash symbols
# Field definitions: type field_name [default_value]
string robot_name
int32 robot_id
float64 battery_level
bool is_active
```

### Field Types

#### Primitive Types
ROS2 supports these built-in primitive types:

| Type | Size | Range | Description |
|------|------|-------|-------------|
| `bool` | 1 byte | true/false | Boolean value |
| `byte` | 1 byte | 0-255 | Unsigned 8-bit integer |
| `char` | 1 byte | -128 to 127 | Signed 8-bit integer |
| `int8` | 1 byte | -128 to 127 | Signed 8-bit integer |
| `uint8` | 1 byte | 0 to 255 | Unsigned 8-bit integer |
| `int16` | 2 bytes | -32,768 to 32,767 | Signed 16-bit integer |
| `uint16` | 2 bytes | 0 to 65,535 | Unsigned 16-bit integer |
| `int32` | 4 bytes | -2^31 to 2^31-1 | Signed 32-bit integer |
| `uint32` | 4 bytes | 0 to 2^32-1 | Unsigned 32-bit integer |
| `int64` | 8 bytes | -2^63 to 2^63-1 | Signed 64-bit integer |
| `uint64` | 8 bytes | 0 to 2^63-1 | Unsigned 64-bit integer |
| `float32` | 4 bytes | IEEE 754 | Single-precision float |
| `float64` | 8 bytes | IEEE 754 | Double-precision float |
| `string` | Variable | UTF-8 | Unicode string |

#### Array Types
ROS2 supports both fixed-size and dynamic arrays:

```msg
# Fixed-size arrays
int32[10] fixed_array          # Array of exactly 10 integers
float64[3] position           # 3D position vector

# Dynamic arrays (unbounded)
string[] names                # Variable number of strings
geometry_msgs/Point[] waypoints  # Array of custom message types

# Bounded arrays
int32[<=100] bounded_readings # At most 100 readings
```

#### Message Types
Messages can contain other messages as fields:

```msg
# Using standard ROS2 messages
geometry_msgs/Pose current_pose
sensor_msgs/LaserScan scan_data

# Using custom messages from same package
RobotStatus status
BatteryInfo battery

# Using messages from other packages
my_package/CustomMessage custom_field
```

### Constants and Default Values

ROS2 messages support constant definitions:

```msg
# Integer constants
int32 STATUS_OK=0
int32 STATUS_WARNING=1  
int32 STATUS_ERROR=2

# String constants
string DEFAULT_NAME="DefaultRobot"

# Float constants
float64 MAX_SPEED=10.5

# Using constants with fields
int32 current_status STATUS_OK  # Default value
string name DEFAULT_NAME
```

### Comments and Documentation
Comments provide documentation and are preserved during conversion:

```msg
# This message represents the complete state of a robot
# 
# The robot state includes position, orientation, and operational status.
# This message is published periodically by the robot state publisher.

std_msgs/Header header          # Standard ROS header with timestamp
geometry_msgs/Pose pose         # Robot position and orientation  
geometry_msgs/Twist velocity    # Current linear and angular velocity
uint8 operational_mode          # Current operational mode
```

## ROC's ROS2 Message Parser

ROC implements a comprehensive parser for ROS2 message definitions:

### Parsing Process

1. **Lexical Analysis**: Tokenize the message file into meaningful elements
2. **Syntax Parsing**: Build abstract syntax tree from tokens
3. **Type Resolution**: Resolve all message type references
4. **Validation**: Validate field names, types, and constraints
5. **Dependency Tracking**: Build dependency graph for proper ordering

### Advanced Features Supported

#### Header Information
ROC extracts and preserves:
- Package names from message paths
- Comments and documentation
- Field ordering and grouping
- Constant definitions

#### Type Analysis
ROC analyzes:
- Primitive vs. composite types
- Array bounds and constraints
- Message dependencies
- Namespace resolution

#### Error Detection
ROC validates:
- Type name correctness
- Array syntax validity
- Constant value compatibility
- Circular dependency detection

## Message to Protobuf Conversion

When converting ROS2 messages to Protobuf, ROC applies intelligent transformations:

### Type Mapping Strategy

#### Direct Mappings
```msg
# ROS2 → Protobuf
bool active          # → bool active = 1;
int32 count         # → int32 count = 2;
float64 value       # → double value = 3;
string name         # → string name = 4;
```

#### Array Conversions
```msg
# ROS2 arrays → Protobuf repeated fields
int32[] numbers              # → repeated int32 numbers = 1;
string[10] fixed_strings     # → repeated string fixed_strings = 2;
geometry_msgs/Point[] points # → repeated geometry_msgs.Point points = 3;
```

#### Message Reference Resolution
```msg
# ROS2 message reference
geometry_msgs/Pose current_pose

# Becomes Protobuf field
geometry_msgs.Pose current_pose = 1;
```

### Package and Namespace Handling

ROC converts ROS2 package structure to Protobuf packages:

```msg
# File: my_robot_msgs/msg/RobotStatus.msg
std_msgs/Header header
geometry_msgs/Pose pose
```

Becomes:
```protobuf
// robot_status.proto
syntax = "proto3";
package my_robot_msgs;

import "std_msgs/header.proto";
import "geometry_msgs/pose.proto";

message RobotStatus {
  std_msgs.Header header = 1;
  geometry_msgs.Pose pose = 2;
}
```

### Constant Handling

ROS2 constants are converted to Protobuf enums when appropriate:

```msg
# ROS2 constants
uint8 MODE_MANUAL=0
uint8 MODE_AUTO=1
uint8 MODE_EMERGENCY=2
uint8 current_mode
```

Becomes:
```protobuf
enum Mode {
  MODE_MANUAL = 0;
  MODE_AUTO = 1;
  MODE_EMERGENCY = 2;
}

message RobotControl {
  Mode current_mode = 1;
}
```

## Common ROS2 Message Patterns

### Standard Header Pattern
Many ROS2 messages include a standard header:

```msg
std_msgs/Header header
# ... other fields
```

ROC recognizes this pattern and handles the std_msgs dependency appropriately.

### Sensor Data Pattern
Sensor messages often follow this structure:

```msg
std_msgs/Header header
# Sensor-specific data fields
float64[] ranges
float64 angle_min
float64 angle_max
float64 angle_increment
```

### Status/Diagnostic Pattern
Status messages typically include:

```msg
std_msgs/Header header
uint8 level                    # Status level (OK, WARN, ERROR)
string name                    # Component name
string message                 # Human-readable status message
string hardware_id             # Hardware identifier
diagnostic_msgs/KeyValue[] values  # Additional diagnostic data
```

## Integration with ROS2 Ecosystem

### Package Dependencies
ROC understands common ROS2 message packages:

- `std_msgs`: Standard message types (Header, String, etc.)
- `geometry_msgs`: Geometric primitives (Point, Pose, Twist, etc.)
- `sensor_msgs`: Sensor data (LaserScan, Image, PointCloud, etc.)
- `nav_msgs`: Navigation messages (Path, OccupancyGrid, etc.)
- `action_msgs`: Action-related messages
- `diagnostic_msgs`: System diagnostics

### Build System Integration
ROC-generated protobuf files can be integrated into ROS2 build systems:

```cmake
# CMakeLists.txt
find_package(protobuf REQUIRED)

# Convert ROS2 messages to protobuf
execute_process(
  COMMAND roc idl protobuf ${CMAKE_CURRENT_SOURCE_DIR}/msg/*.msg
          --output ${CMAKE_CURRENT_BINARY_DIR}/proto/
)

# Add protobuf generation
protobuf_generate_cpp(PROTO_SRCS PROTO_HDRS ${PROTO_FILES})
```

## Best Practices

### Message Design
- Keep messages simple and focused
- Use descriptive field names
- Include appropriate documentation
- Follow ROS2 naming conventions

### Conversion Considerations
- Be aware of type precision differences
- Consider array bounds in target format
- Plan for constant handling strategy
- Document conversion decisions

### Performance Tips
- Use appropriate numeric types
- Minimize nested message depth
- Consider serialization efficiency
- Profile converted message performance

## Limitations and Considerations

### ROS2 to Protobuf Limitations
1. **Service Definitions**: ROC currently focuses on message definitions
2. **Action Definitions**: Action definitions require special handling
3. **Complex Constants**: Some constant expressions may not convert directly
4. **Custom Types**: Very specialized ROS2 types may need manual attention

### Protobuf to ROS2 Limitations
1. **Oneof Fields**: ROS2 doesn't have direct oneof equivalent
2. **Map Types**: Converted to key-value pair arrays
3. **Any Types**: Not directly supported in ROS2
4. **Extensions**: Protobuf extensions don't map to ROS2

Understanding these patterns and limitations helps ensure successful conversion between ROS2 message formats and Protobuf schemas.
