# ROS2 Topic Delay Implementation Analysis

## Problem Statement
Implement a `roc topic delay` command that can actually delay/buffer messages flowing through a ROS2 topic, not just pause execution.

## Key Architectural Insights

### The Challenge
- **We can't delay messages we didn't publish** - If another node is publishing to `/chatter`, we can't intercept those messages at the application level
- **DDS is the transport layer** - All ROS2 messages flow through DDS (Fast-DDS, Cyclone DX, etc.)
- **RMW is the abstraction** - RMW layer talks to DDS implementations
- **True interception requires DDS-level work** - Beyond the scope of a CLI tool

## Potential Implementation Approaches

### Option 1: DDS-level Interception ⭐ (Ideal but Complex)
- Hook into the DDS middleware directly  
- Intercept messages at the DDS participant/reader/writer level
- Buffer and delay them before forwarding
- **Pros**: True message interception, transparent to applications
- **Cons**: Requires deep DDS integration, middleware modification

### Option 2: RMW Middleware Plugin 
- Create a custom RMW implementation that adds delay functionality
- Act as a proxy between the standard RMW and the actual DDS implementation  
- **Pros**: Cleaner abstraction at RMW level
- **Cons**: Complex RMW plugin development

### Option 3: DDS QoS-based Approach
- Use DDS QoS policies like `DEADLINE` or custom middleware services
- Some DDS implementations support middleware plugins
- **Pros**: Leverages existing DDS features  
- **Cons**: Limited by DDS implementation capabilities

### Option 4: Transparent Proxy Node (Realistic Alternative)
- Subscribe to the original topic
- Buffer messages with timestamps  
- Re-publish on delayed topic or remap the original topic
- **Pros**: Implementable with current tools
- **Cons**: Not truly transparent, requires topic remapping

## Technical Requirements for True Implementation

### DDS-Level Integration Needs:
- Deep integration with DDS implementation (Fast-DDS, Cyclone DX)
- Custom middleware or plugin development  
- Potentially modifying the RMW layer
- Understanding of DDS participant/reader/writer lifecycle

### RCL/RMW Integration Points:
- `rmw_create_subscription()` - Intercept subscription creation
- `rmw_take()` - Intercept message taking  
- `rmw_publish()` - Intercept message publishing
- Buffer management at RMW level

## Current Status
- ✅ **Successfully expanded RCL/RMW bindings** - Added subscription/publisher functions
- ✅ **RMW callback interception available** - `rmw_subscription_set_on_new_message_callback()`
- ✅ **Direct RMW access** - Can use `rmw_take()`, `rmw_publish()` functions
- ✅ **Avoided DDS dependency** - Working purely at RMW level

## Available RMW Interception Points

### 1. Message Callbacks ⭐ (Most Promising)
```c
rmw_ret_t rmw_subscription_set_on_new_message_callback(
  rmw_subscription_t * subscription,
  rmw_event_callback_t callback,
  const void * user_data);
```
- **Hook**: Intercept when new messages arrive
- **Buffer**: Store messages with timestamps  
- **Delay**: Re-publish after specified delay

### 2. Direct RMW Take/Publish
```c 
rmw_ret_t rmw_take(rmw_subscription_t * subscription, void * ros_message, bool * taken, rmw_subscription_allocation_t * allocation);
rmw_ret_t rmw_publish(const rmw_publisher_t * publisher, const void * ros_message, rmw_publisher_allocation_t * allocation);
```
- **Intercept**: Override `rmw_take()` to buffer messages
- **Publish**: Use `rmw_publish()` to send delayed messages

### 3. RCL Level Interception
```c
rcl_ret_t rcl_take(const rcl_subscription_t * subscription, void * ros_message, rmw_message_info_t * message_info, rcl_subscription_allocation_t * allocation);
rcl_ret_t rcl_publish(const rcl_publisher_t * publisher, const void * ros_message, rmw_publisher_allocation_t * allocation);
```
- **Higher level**: Work at RCL rather than RMW
- **Easier**: More abstracted, less DDS-specific

## Implementation Strategy (Without DDS)

### Approach: RMW Proxy/Interceptor
1. **Create shadow subscription** to target topic
2. **Register callback** using `rmw_subscription_set_on_new_message_callback()`  
3. **Buffer messages** with timestamps in delay queue
4. **Create shadow publisher** for the same topic (or remapped topic)
5. **Timer-based dispatch** to publish buffered messages after delay

### Code Structure
```rust
struct TopicDelayInterceptor {
    source_subscription: rmw_subscription_t,
    target_publisher: rmw_publisher_t, 
    delay_duration: Duration,
    message_buffer: DelayQueue<(Vec<u8>, Instant)>,
    callback_handle: rmw_event_callback_t,
}

impl TopicDelayInterceptor {
    fn on_message_callback(&mut self, user_data: *const c_void, num_events: usize) {
        // Take message from RMW
        // Add to delay buffer with timestamp
        // Schedule for delayed publishing
    }
    
    fn process_delayed_messages(&mut self) {
        // Check delay buffer for ready messages
        // Publish messages that have waited long enough
    }
}
```

## Next Steps (Today/Tomorrow)
1. ✅ **Expand RCL bindings** - Added subscription/publisher functions to wrapper.h
2. ✅ **Investigate RMW callbacks** - Found `rmw_subscription_set_on_new_message_callback()`
3. 🔄 **Prototype RMW interceptor** - Implement shadow subscription + delayed publisher
4. 🔄 **Test message buffering** - Verify we can intercept and re-publish messages
5. 🔄 **Handle topic remapping** - Allow delayed messages on same or different topic

## Key Insight: RMW-Level Interception is Possible! 🎯
- No need for DDS-level work
- Can intercept at RMW callback level
- Buffer and re-publish with time delays
- Implement as transparent proxy between topics

## Technical Feasibility: HIGH ✅
This is **definitely implementable** without going to DDS level. The RMW layer provides sufficient hooks for message interception and delayed re-publishing.

## References
- [ROS2 RMW Implementation](https://docs.ros.org/en/rolling/Concepts/About-ROS-Middleware-Implementations.html)
- [DDS Specification](https://www.omg.org/spec/DDS/)
- [Fast-DDS User Manual](https://fast-dds.docs.eprosima.com/)
- [Cyclone DX Documentation](https://cyclonedx.org/)

## Code Location
Currently in: `/doc/code/roc/src/commands/topic/delay.rs`
Status: Placeholder implementation (simple sleep)
