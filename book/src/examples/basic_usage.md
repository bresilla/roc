# Basic Usage Examples

This chapter provides practical examples of using the `roc` tool for common ROS 2 introspection tasks.

## Prerequisites

Before running these examples, ensure:
- ROS 2 is installed and sourced
- At least one ROS 2 node is running (e.g., `ros2 run demo_nodes_cpp talker`)
- The `roc` tool is built and available in your PATH

## Getting Started

### 1. List All Topics

The most basic operation is listing all available topics in the ROS 2 graph:

```bash
roc topic list
```

Expected output:
```
/chatter
/parameter_events
/rosout
```

### 2. Get Basic Topic Information

To get basic information about a specific topic:

```bash
roc topic info /chatter
```

Output:
```
Topic: /chatter
Type: std_msgs/msg/String
Publishers: 1
Subscribers: 0
```

### 3. Get Detailed Topic Information

For comprehensive topic details including QoS profiles and endpoint information:

```bash
roc topic info /chatter --verbose
```

Expected verbose output:
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

## Common Use Cases

### Debugging Communication Issues

When nodes aren't communicating properly, use verbose topic info to check QoS compatibility:

```bash
# Check publisher QoS
roc topic info /my_topic --verbose

# Compare with subscriber expectations
# Look for QoS mismatches in reliability, durability, etc.
```

### Monitoring System Health

Check critical system topics:

```bash
# Monitor rosout for system messages
roc topic info /rosout --verbose

# Check parameter events
roc topic info /parameter_events --verbose
```

### Network Diagnostics

Use GID information to identify nodes across the network:

```bash
# Get detailed endpoint information
roc topic info /my_topic --verbose | grep "GID"
```

## Working with Multiple Topics

### Batch Information Gathering

```bash
# Get info for all topics
for topic in $(roc topic list); do
    echo "=== $topic ==="
    roc topic info "$topic"
    echo
done
```

### Filtering by Node

```bash
# Find topics published by a specific node
roc topic info /chatter --verbose | grep "Node:"
```

## Integration with ROS 2 Tools

The `roc` tool complements existing ROS 2 CLI tools:

```bash
# Compare outputs
ros2 topic info /chatter --verbose
roc topic info /chatter --verbose

# Use roc for faster queries
time roc topic list
time ros2 topic list
```

## Troubleshooting

### No Topics Found

If `roc topic list` returns empty:

1. Check if ROS 2 nodes are running:
   ```bash
   ros2 node list
   ```

2. Verify ROS 2 environment:
   ```bash
   echo $ROS_DOMAIN_ID
   printenv | grep ROS
   ```

3. Test with a simple publisher:
   ```bash
   ros2 run demo_nodes_cpp talker
   ```

### Permission Issues

If you encounter permission errors:

```bash
# Check RMW implementation
echo $RMW_IMPLEMENTATION

# Try with different RMW
export RMW_IMPLEMENTATION=rmw_cyclone_cpp
roc topic list
```

### Performance Considerations

For systems with many topics:

```bash
# Use targeted queries instead of listing all topics
roc topic info /specific_topic --verbose
```

## Next Steps

- See [Advanced Usage](advanced_usage.md) for complex scenarios
- Check [Command Reference](command_reference.md) for all available options
- Read [Integration Examples](integration_examples.md) for using roc in scripts and automation
