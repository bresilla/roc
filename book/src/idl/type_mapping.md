# Type Mapping

Effective conversion between Protobuf and ROS2 message formats requires careful consideration of type system differences. This chapter provides comprehensive information about how ROC maps types between these systems.

## Type System Comparison

### Protobuf Type System
Protobuf uses a rich type system designed for cross-language compatibility:

- **Primitive Types**: Integers of various sizes, floating-point, boolean, string, bytes
- **Composite Types**: Messages (structs), enums, oneofs (unions)
- **Container Types**: Repeated fields (arrays), maps
- **Special Types**: Well-known types (Timestamp, Duration, Any, etc.)
- **Advanced Features**: Optional fields, default values, extensions

### ROS2 Type System
ROS2 uses a simpler, more constrained type system:

- **Primitive Types**: Fixed-size integers, floating-point, boolean, string
- **Composite Types**: Messages (structs), constants
- **Container Types**: Fixed and dynamic arrays
- **Special Types**: Standard message types (Header, etc.)
- **Constraints**: Bounded arrays, default values

## Comprehensive Type Mapping Tables

### Protobuf to ROS2 Mapping

#### Numeric Types

| Protobuf Type | ROS2 Type | Size | Signed | Notes |
|---------------|-----------|------|--------|-------|
| `bool` | `bool` | 1 byte | N/A | Direct mapping |
| `int32` | `int32` | 4 bytes | Yes | Direct mapping |
| `int64` | `int64` | 8 bytes | Yes | Direct mapping |
| `uint32` | `uint32` | 4 bytes | No | Direct mapping |
| `uint64` | `uint64` | 8 bytes | No | Direct mapping |
| `sint32` | `int32` | 4 bytes | Yes | ZigZag encoded in protobuf |
| `sint64` | `int64` | 8 bytes | Yes | ZigZag encoded in protobuf |
| `fixed32` | `uint32` | 4 bytes | No | Fixed-width encoding |
| `fixed64` | `uint64` | 8 bytes | No | Fixed-width encoding |
| `sfixed32` | `int32` | 4 bytes | Yes | Fixed-width signed |
| `sfixed64` | `int64` | 8 bytes | Yes | Fixed-width signed |
| `float` | `float32` | 4 bytes | Yes | IEEE 754 single precision |
| `double` | `float64` | 8 bytes | Yes | IEEE 754 double precision |

#### String and Binary Types

| Protobuf Type | ROS2 Type | Notes |
|---------------|-----------|-------|
| `string` | `string` | UTF-8 encoded strings |
| `bytes` | `uint8[]` | Binary data as byte array |

#### Container Types

| Protobuf Type | ROS2 Type | Example |
|---------------|-----------|---------|
| `repeated T` | `T[]` | `repeated int32 values` → `int32[] values` |
| `map<K,V>` | `MapEntry[]` | `map<string,int32> data` → `DataEntry[] data` |

### ROS2 to Protobuf Mapping

#### Numeric Types

| ROS2 Type | Protobuf Type | Rationale |
|-----------|---------------|-----------|
| `bool` | `bool` | Direct mapping |
| `byte` | `uint32` | ROS2 byte is unsigned 8-bit |
| `char` | `int32` | ROS2 char is signed 8-bit |
| `int8` | `int32` | Protobuf doesn't have 8-bit integers |
| `uint8` | `uint32` | Protobuf doesn't have 8-bit integers |
| `int16` | `int32` | Protobuf doesn't have 16-bit integers |
| `uint16` | `uint32` | Protobuf doesn't have 16-bit integers |
| `int32` | `int32` | Direct mapping |
| `uint32` | `uint32` | Direct mapping |
| `int64` | `int64` | Direct mapping |
| `uint64` | `uint64` | Direct mapping |
| `float32` | `float` | Direct mapping |
| `float64` | `double` | Direct mapping |
| `string` | `string` | Direct mapping |

#### Array Types

| ROS2 Type | Protobuf Type | Notes |
|-----------|---------------|-------|
| `T[]` | `repeated T` | Dynamic arrays |
| `T[N]` | `repeated T` | Fixed-size arrays (size constraint lost) |
| `T[<=N]` | `repeated T` | Bounded arrays (bound constraint lost) |

## Special Type Conversions

### Protobuf Oneof to ROS2
Protobuf oneof fields don't have a direct equivalent in ROS2. ROC handles this by creating separate optional fields:

```protobuf
// Protobuf
message Command {
  oneof command_type {
    string text_command = 1;
    int32 numeric_command = 2;
    bool flag_command = 3;
  }
}
```

Converts to:
```msg
# ROS2 - all fields are optional, only one should be set
string text_command
int32 numeric_command
bool flag_command
```

### Protobuf Maps to ROS2
Maps are converted to arrays of key-value pair messages:

```protobuf
// Protobuf
message Configuration {
  map<string, double> parameters = 1;
}
```

Converts to:
```msg
# Configuration.msg
ConfigurationParametersEntry[] parameters

# ConfigurationParametersEntry.msg (auto-generated)
string key
float64 value
```

### ROS2 Constants to Protobuf
ROS2 constants are converted to enum values when they represent a set of related values:

```msg
# ROS2
uint8 STATE_IDLE=0
uint8 STATE_MOVING=1
uint8 STATE_ERROR=2
uint8 current_state
```

Converts to:
```protobuf
// Protobuf
enum State {
  STATE_IDLE = 0;
  STATE_MOVING = 1;
  STATE_ERROR = 2;
}

message RobotStatus {
  State current_state = 1;
}
```

## Well-Known Type Mappings

### Protobuf Well-Known Types
ROC provides special handling for common Protobuf well-known types:

| Protobuf Type | ROS2 Equivalent | Notes |
|---------------|-----------------|-------|
| `google.protobuf.Timestamp` | `builtin_interfaces/Time` | Nanosecond precision |
| `google.protobuf.Duration` | `builtin_interfaces/Duration` | Nanosecond precision |
| `google.protobuf.Empty` | Empty message | No fields |
| `google.protobuf.StringValue` | `string` | Wrapper type flattened |
| `google.protobuf.Int32Value` | `int32` | Wrapper type flattened |
| `google.protobuf.BoolValue` | `bool` | Wrapper type flattened |

### Standard ROS2 Types
Common ROS2 types have conventional Protobuf mappings:

| ROS2 Type | Protobuf Equivalent | Notes |
|-----------|---------------------|-------|
| `std_msgs/Header` | Custom message | Timestamp + frame_id |
| `geometry_msgs/Point` | Custom message | x, y, z coordinates |
| `geometry_msgs/Quaternion` | Custom message | x, y, z, w components |
| `geometry_msgs/Pose` | Custom message | Position + orientation |
| `geometry_msgs/Twist` | Custom message | Linear + angular velocity |

## Type Conversion Edge Cases

### Precision and Range Considerations

#### Integer Overflow Scenarios
```msg
# ROS2 uint8 field
uint8 small_value 255  # Maximum value for uint8
```

When converted to Protobuf `uint32`, the range increases significantly. ROC preserves the original constraint information in comments:

```protobuf
// Protobuf
message Example {
  uint32 small_value = 1;  // Originally uint8, max value 255
}
```

#### Floating-Point Precision
```msg
# ROS2 float32
float32 precise_value 3.14159265359
```

Converting to Protobuf maintains the precision level:
```protobuf
float precise_value = 1;  // 32-bit precision maintained
```

### Array Bound Handling

#### Fixed-Size Arrays
```msg
# ROS2 fixed-size array
float64[3] position
```

Protobuf doesn't support fixed-size arrays, so this becomes:
```protobuf
repeated double position = 1;  // Size constraint documented separately
```

#### Bounded Arrays
```msg
# ROS2 bounded array
int32[<=100] readings
```

The bound constraint is preserved in documentation:
```protobuf
repeated int32 readings = 1;  // Maximum 100 elements
```

## Advanced Mapping Strategies

### Nested Message Flattening
ROC flattens nested Protobuf messages for ROS2 compatibility:

```protobuf
// Protobuf nested messages
message Robot {
  message Status {
    bool active = 1;
    string state = 2;
  }
  Status current_status = 1;
  string robot_id = 2;
}
```

Becomes:
```msg
# Robot.msg
RobotStatus current_status
string robot_id

# RobotStatus.msg (flattened)
bool active
string state
```

### Package and Namespace Translation

#### Protobuf Package to ROS2 Package
```protobuf
// Protobuf
syntax = "proto3";
package robotics.sensors;

message LaserData { ... }
```

Becomes:
```msg
# File: robotics_sensors_msgs/msg/LaserData.msg
# Content of the message...
```

#### ROS2 Package to Protobuf Package
```msg
# File: my_robot_msgs/msg/Status.msg
# Message content...
```

Becomes:
```protobuf
syntax = "proto3";
package my_robot_msgs;

message Status { ... }
```

## Configuration and Customization

### Custom Type Mappings
ROC supports configuration files for custom type mappings:

```yaml
# type_mappings.yaml
protobuf_to_ros2:
  "my.custom.Timestamp": "builtin_interfaces/Time"
  "my.custom.Position": "geometry_msgs/Point"

ros2_to_protobuf:
  "my_msgs/CustomType": "my.package.CustomMessage"
```

Usage:
```bash
roc idl protobuf --config type_mappings.yaml input_files...
```

### Mapping Validation
ROC validates type mappings and warns about potential issues:

```
Warning: Converting uint64 to int64 may cause overflow for large values
Warning: Map type conversion may affect lookup performance
Warning: Oneof semantics lost in ROS2 conversion
```

### Performance Implications

#### Serialization Efficiency
Different type choices affect serialization performance:

- **Protobuf varint encoding**: Smaller integers encode more efficiently
- **Fixed-width types**: Predictable size but potentially wasteful
- **String vs bytes**: UTF-8 validation overhead for strings

#### Memory Usage
Type conversions can affect memory usage:

- **Array bounds**: ROS2 bounded arrays vs Protobuf repeated fields
- **Message size**: Nested vs flattened message structures
- **Field ordering**: Affects struct packing and cache efficiency

## Best Practices for Type Mapping

### Design Considerations
1. **Choose appropriate numeric types**: Don't use int64 when int32 suffices
2. **Consider array bounds**: Use bounded arrays in ROS2 when possible
3. **Document constraints**: Preserve semantic meaning across conversions
4. **Plan for evolution**: Design messages that can evolve over time

### Conversion Guidelines
1. **Test thoroughly**: Validate converted messages with real data
2. **Preserve semantics**: Maintain the original meaning of fields
3. **Document decisions**: Record rationale for non-obvious mappings
4. **Monitor performance**: Profile converted message performance

### Maintenance Strategies
1. **Version control**: Track message schema changes
2. **Backward compatibility**: Plan for schema evolution
3. **Testing automation**: Automated conversion validation
4. **Documentation updates**: Keep mapping documentation current

Understanding these type mapping strategies ensures successful and maintainable conversions between Protobuf and ROS2 message formats.
