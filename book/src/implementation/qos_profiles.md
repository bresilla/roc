# QoS Profile Handling

Quality of Service (QoS) profiles are critical for ROS 2 communication behavior. The `roc` tool provides detailed QoS information that helps developers understand and debug communication patterns, compatibility issues, and performance characteristics.

## QoS Overview

QoS profiles define the communication behavior between publishers and subscribers. They consist of several policies that must be compatible between endpoints for successful communication.

### QoS Policies in `roc`

`roc` displays the following QoS policies:

1. **Reliability** - Message delivery guarantees
2. **History** - Message queue behavior  
3. **Durability** - Message persistence
4. **Deadline** - Maximum time between messages
5. **Lifespan** - How long messages remain valid
6. **Liveliness** - Endpoint aliveness checking

## Data Structure Implementation

### Rust QoS Representation
```rust
#[derive(Debug, Clone)]
pub struct QosProfile {
    pub history: QosHistoryPolicy,
    pub depth: usize,
    pub reliability: QosReliabilityPolicy,
    pub durability: QosDurabilityPolicy,
    pub deadline_sec: u64,
    pub deadline_nsec: u64,
    pub lifespan_sec: u64,
    pub lifespan_nsec: u64,
    pub liveliness: QosLivelinessPolicy,
    pub liveliness_lease_duration_sec: u64,
    pub liveliness_lease_duration_nsec: u64,
    pub avoid_ros_namespace_conventions: bool,
}
```

### Policy Enumerations
```rust
#[derive(Debug, Clone)]
pub enum QosReliabilityPolicy {
    SystemDefault,    // Use DDS implementation default
    Reliable,         // Guarantee delivery
    BestEffort,       // Best effort delivery
    Unknown,          // Unrecognized value
    BestAvailable,    // Match majority of endpoints
}

#[derive(Debug, Clone)]
pub enum QosHistoryPolicy {
    SystemDefault,    // Use DDS implementation default
    KeepLast,         // Keep last N messages
    KeepAll,          // Keep all messages
    Unknown,          // Unrecognized value
}

#[derive(Debug, Clone)]
pub enum QosDurabilityPolicy {
    SystemDefault,    // Use DDS implementation default
    TransientLocal,   // Persist for late joiners
    Volatile,         // Don't persist
    Unknown,          // Unrecognized value
    BestAvailable,    // Match majority of endpoints
}

#[derive(Debug, Clone)]
pub enum QosLivelinessPolicy {
    SystemDefault,    // Use DDS implementation default
    Automatic,        // DDS manages liveliness
    ManualByNode,     // Application asserts per node (deprecated)
    ManualByTopic,    // Application asserts per topic
    Unknown,          // Unrecognized value
    BestAvailable,    // Match majority of endpoints
}
```

## Conversion from RMW Types

### From C Enums to Rust Enums
```rust
impl QosReliabilityPolicy {
    fn from_rmw(reliability: rmw_qos_reliability_policy_e) -> Self {
        match reliability {
            rmw_qos_reliability_policy_e_RMW_QOS_POLICY_RELIABILITY_SYSTEM_DEFAULT 
                => QosReliabilityPolicy::SystemDefault,
            rmw_qos_reliability_policy_e_RMW_QOS_POLICY_RELIABILITY_RELIABLE 
                => QosReliabilityPolicy::Reliable,
            rmw_qos_reliability_policy_e_RMW_QOS_POLICY_RELIABILITY_BEST_EFFORT 
                => QosReliabilityPolicy::BestEffort,
            rmw_qos_reliability_policy_e_RMW_QOS_POLICY_RELIABILITY_BEST_AVAILABLE 
                => QosReliabilityPolicy::BestAvailable,
            _ => QosReliabilityPolicy::Unknown,
        }
    }
}
```

### Complete QoS Profile Conversion
```rust
impl QosProfile {
    fn from_rmw(qos: &rmw_qos_profile_t) -> Self {
        QosProfile {
            history: QosHistoryPolicy::from_rmw(qos.history),
            depth: qos.depth,
            reliability: QosReliabilityPolicy::from_rmw(qos.reliability),
            durability: QosDurabilityPolicy::from_rmw(qos.durability),
            deadline_sec: qos.deadline.sec,
            deadline_nsec: qos.deadline.nsec,
            lifespan_sec: qos.lifespan.sec,
            lifespan_nsec: qos.lifespan.nsec,
            liveliness: QosLivelinessPolicy::from_rmw(qos.liveliness),
            liveliness_lease_duration_sec: qos.liveliness_lease_duration.sec,
            liveliness_lease_duration_nsec: qos.liveliness_lease_duration.nsec,
            avoid_ros_namespace_conventions: qos.avoid_ros_namespace_conventions,
        }
    }
}
```

## Display Formatting

### Duration Formatting
Duration values require special formatting because they can represent:
- **Infinite duration**: `0x7FFFFFFFFFFFFFFF` seconds and nanoseconds
- **Unspecified duration**: `0` seconds and nanoseconds  
- **Specific duration**: Actual time values

```rust
impl QosProfile {
    pub fn format_duration(&self, sec: u64, nsec: u64) -> String {
        if sec == 0x7FFFFFFFFFFFFFFF && nsec == 0x7FFFFFFFFFFFFFFF {
            "Infinite".to_string()
        } else if sec == 0 && nsec == 0 {
            "0 nanoseconds".to_string()
        } else {
            format!("{} nanoseconds", sec * 1_000_000_000 + nsec)
        }
    }
}
```

### Policy Display
```rust
impl QosReliabilityPolicy {
    pub fn to_string(&self) -> &'static str {
        match self {
            QosReliabilityPolicy::SystemDefault => "SYSTEM_DEFAULT",
            QosReliabilityPolicy::Reliable => "RELIABLE",
            QosReliabilityPolicy::BestEffort => "BEST_EFFORT",
            QosReliabilityPolicy::Unknown => "UNKNOWN",
            QosReliabilityPolicy::BestAvailable => "BEST_AVAILABLE",
        }
    }
}
```

## QoS Policy Details

### Reliability Policy

**RELIABLE**
- Guarantees message delivery
- Uses acknowledgments and retransmissions
- Higher bandwidth and latency overhead
- Suitable for critical data

**BEST_EFFORT**  
- Attempts delivery without guarantees
- No acknowledgments or retransmissions
- Lower bandwidth and latency
- Suitable for high-frequency sensor data

Example output:
```
Reliability: RELIABLE
```

### History Policy

**KEEP_LAST**
- Maintains a queue of the last N messages
- Depth field indicates queue size
- Older messages are discarded when queue is full
- Most common for real-time systems

**KEEP_ALL**
- Attempts to deliver all messages
- Subject to resource limits
- Can cause memory growth under high load
- Suitable when no data loss is acceptable

Example output:
```
History (KEEP_LAST): 10
```

### Durability Policy

**VOLATILE**
- Messages exist only while publisher is active
- Late-joining subscribers miss earlier messages
- Default for most applications

**TRANSIENT_LOCAL**
- Messages are stored for late-joining subscribers
- Publisher maintains message history
- Useful for configuration or status topics

Example output:
```
Durability: TRANSIENT_LOCAL
```

### Deadline Policy

Specifies the maximum expected time between consecutive messages.

**Infinite** (default)
- No deadline constraint
- Publisher can send at any rate

**Finite deadline**
- Publisher commits to sending within deadline
- Subscriber can detect missed deadlines
- Useful for real-time systems

Example output:
```
Deadline: Infinite
Deadline: 100000000 nanoseconds  # 100ms
```

### Lifespan Policy

Defines how long messages remain valid after publication.

**Infinite** (default)
- Messages never expire
- Suitable for persistent data

**Finite lifespan**
- Messages expire after specified time
- Useful for time-sensitive data

Example output:
```
Lifespan: 5000000000 nanoseconds  # 5 seconds
```

### Liveliness Policy

Determines how endpoint "aliveness" is maintained and monitored.

**AUTOMATIC**
- DDS automatically maintains liveliness
- Most common and recommended setting

**MANUAL_BY_TOPIC**
- Application must explicitly assert liveliness
- Provides fine-grained control
- Used in safety-critical systems

Example output:
```
Liveliness: AUTOMATIC
Liveliness lease duration: Infinite
```

## QoS Compatibility

### Compatibility Rules

For successful communication, QoS policies must be compatible:

| Policy | Publisher | Subscriber | Compatible? |
|--------|-----------|------------|-------------|
| Reliability | RELIABLE | RELIABLE | ✅ |
| Reliability | RELIABLE | BEST_EFFORT | ✅ |
| Reliability | BEST_EFFORT | RELIABLE | ❌ |
| Reliability | BEST_EFFORT | BEST_EFFORT | ✅ |

| Policy | Publisher | Subscriber | Compatible? |
|--------|-----------|------------|-------------|
| Durability | TRANSIENT_LOCAL | TRANSIENT_LOCAL | ✅ |
| Durability | TRANSIENT_LOCAL | VOLATILE | ✅ |
| Durability | VOLATILE | TRANSIENT_LOCAL | ❌ |
| Durability | VOLATILE | VOLATILE | ✅ |

### Common QoS Profiles

**Sensor Data Profile**
```
Reliability: BEST_EFFORT
History (KEEP_LAST): 5
Durability: VOLATILE
Deadline: Infinite
Lifespan: Infinite
Liveliness: AUTOMATIC
```

**Parameter Profile**  
```
Reliability: RELIABLE
History (KEEP_LAST): 1000
Durability: VOLATILE
Deadline: Infinite
Lifespan: Infinite
Liveliness: AUTOMATIC
```

**Services Profile**
```
Reliability: RELIABLE
History (KEEP_LAST): 10
Durability: VOLATILE
Deadline: Infinite
Lifespan: Infinite
Liveliness: AUTOMATIC
```

## Debugging QoS Issues

### Common Problems

**No Communication**
- Check reliability compatibility
- Verify durability compatibility
- Ensure deadline constraints are met

**High Latency**
- RELIABLE policy adds overhead
- Large history depth increases processing
- Network congestion from retransmissions

**Memory Usage**
- KEEP_ALL history can grow unbounded
- TRANSIENT_LOCAL stores message history
- Large depth values consume memory

### Using `roc` for QoS Debugging

1. **Check endpoint QoS**:
   ```bash
   roc topic info /my_topic --verbose
   ```

2. **Compare publisher and subscriber QoS**:
   Look for compatibility issues in the output

3. **Monitor over time**:
   Run repeatedly to see if QoS settings change

4. **Verify against expectations**:
   Compare displayed QoS with application configuration

## Performance Impact

### Policy Performance Characteristics

| Policy | Bandwidth | Latency | Memory | CPU |
|--------|-----------|---------|--------|-----|
| RELIABLE | High | Higher | Medium | Higher |
| BEST_EFFORT | Low | Lower | Low | Lower |
| KEEP_ALL | - | - | High | Medium |
| KEEP_LAST | - | - | Low | Low |
| TRANSIENT_LOCAL | Medium | - | High | Medium |

### Optimization Guidelines

1. **Use BEST_EFFORT** for high-frequency sensor data
2. **Use RELIABLE** for commands and critical data
3. **Keep history depth small** for real-time performance
4. **Use VOLATILE durability** unless persistence is needed
5. **Set realistic deadlines** to detect communication issues

The QoS system in `roc` provides essential visibility into ROS 2 communication behavior, enabling developers to optimize performance and debug connectivity issues.
