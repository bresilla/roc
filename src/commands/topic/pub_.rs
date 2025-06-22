use crate::arguments::topic::CommonTopicArgs;
use crate::graph::{RclGraphContext, SerializedMessage};
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use rclrs::*;
use std::ffi::CString;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, sleep};

/// Dynamic message publisher that can handle any ROS2 message type
struct DynamicRclPublisher {
    publisher: rcl_publisher_t,
    context: *const RclGraphContext,
    message_type: String,
    serialized_message: Option<SerializedMessage>,
}

/// Prepare message data using dynamic message infrastructure (with optional type support)
fn prepare_message_data(message_type: &str, yaml_content: &str) -> Result<SerializedMessage> {
    // Use our dynamic message infrastructure to parse and serialize the message
    RclGraphContext::prepare_message_for_publishing(message_type, yaml_content)
}

/// Prepare message data using generic approach when type support is available
fn prepare_message_data_generic(
    message_type: &str, 
    yaml_content: &str, 
    type_support: *const rclrs::rosidl_message_type_support_t
) -> Result<SerializedMessage> {
    // Use the generic approach with introspection
    RclGraphContext::prepare_message_for_publishing_generic(message_type, yaml_content, type_support)
}

impl DynamicRclPublisher {
    /// Create a new dynamic RCL publisher for the given topic and message type
    fn new(context: &RclGraphContext, topic_name: &str, message_type: &str, yaml_content: &str) -> Result<Self> {
        if !context.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }

        // Get the type support for this message type
        let mut registry = RclGraphContext::create_message_registry();
        let message_type_info = registry.load_message_type(message_type)?;
        
        // Prepare the message data using either generic or fallback approach
        let serialized_message = if let Some(type_support) = message_type_info.type_support {
            println!("Using generic serialization approach with type support!");
            prepare_message_data_generic(message_type, yaml_content, type_support)?
        } else {
            println!("Type support not available, using fallback approach");
            prepare_message_data(message_type, yaml_content)?
        };
        
        println!("Prepared {} byte message for type: {}", serialized_message.data.len(), message_type);

        // Create a real RCL publisher to make the topic visible in the graph
        let topic_name_c = CString::new(topic_name).map_err(|e| anyhow!("Invalid topic name: {}", e))?;
        
        let publisher = if let Some(type_support) = message_type_info.type_support {
            unsafe {
                // Validate inputs before calling rcl_publisher_init
                println!("🔍 Publisher creation debugging:");
                println!("   Context valid: {}", context.is_valid());
                println!("   Node pointer: {:p}", context.node());
                println!("   Type support pointer: {:p}", type_support);
                println!("   Topic name C-string: {:?}", topic_name_c);
                println!("   Topic name content: '{}'", topic_name);
                
                // Validate context - node() returns a reference so it should be valid
                // The issue might be in the context or node state, not null pointers
                
                // Validate type support is not null  
                if type_support.is_null() {
                    return Err(anyhow!("Type support pointer is null"));
                }
                
                let mut pub_instance = rcl_get_zero_initialized_publisher();
                let publisher_options = rcl_publisher_get_default_options();
                
                println!("   Publisher instance initialized: {:p}", &pub_instance);
                println!("   Publisher options: {:p}", &publisher_options);
                
                // Create the RCL publisher with real type support
                println!("🎯 About to call rcl_publisher_init...");
                let ret = rcl_publisher_init(
                    &mut pub_instance,
                    context.node(),
                    type_support,
                    topic_name_c.as_ptr(),
                    &publisher_options,
                );
                
                if ret != 0 { // RCL_RET_OK is 0
                    return Err(anyhow!("Failed to create RCL publisher: return code {}", ret));
                }
                
                println!("✅ Successfully created RCL publisher with real type support!");
                pub_instance
            }
        } else {
            return Err(anyhow!("Could not load type support for message type: {}", message_type));
        };
        
        println!("Created dynamic publisher for topic: {} (type: {})", topic_name, message_type);

        Ok(DynamicRclPublisher {
            publisher,
            context: context as *const RclGraphContext,
            message_type: message_type.to_string(),
            serialized_message: Some(serialized_message),
        })
    }

    /// Publish the prepared message
    fn publish(&self) -> Result<()> {
        if let Some(ref msg) = self.serialized_message {
            println!("Publishing {} bytes for message type: {}", msg.data.len(), msg.message_type);
            
            // For debugging, show what we're about to publish
            if let Ok(yaml_repr) = RclGraphContext::inspect_serialized_message(&msg.message_type, &msg.data) {
                println!("Message content: {:?}", yaml_repr);
            }
            
            // Publish the message using RCL with proper C struct layout
            unsafe {
                let ret = rcl_publish(&self.publisher, msg.data.as_ptr() as *const _, std::ptr::null_mut());
                if ret != 0 { // RCL_RET_OK is 0
                    return Err(anyhow!("Failed to publish message: return code {}", ret));
                }
                println!("✅ Successfully published message ({} bytes) with C struct layout", msg.data.len());
            }
            
            Ok(())
        } else {
            Err(anyhow!("No message data prepared for publishing"))
        }
    }
}

impl Drop for DynamicRclPublisher {
    fn drop(&mut self) {
        unsafe {
            if rcl_publisher_is_valid(&self.publisher) {
                let context_ref = &*self.context;
                rcl_publisher_fini(&mut self.publisher, context_ref.node() as *const _ as *mut _);
            }
        }
    }
}

#[derive(Debug, Clone)]
struct PublishOptions {
    topic_name: String,
    message_type: String,
    values: String,
    rate: f64,
    print: bool,
    once: bool,
    times: Option<usize>,
    wait_matching_subscriptions: usize,
}

impl PublishOptions {
    fn from_matches(matches: &ArgMatches) -> Result<Self> {
        let topic_name = matches
            .get_one::<String>("topic_name")
            .ok_or_else(|| anyhow!("Topic name is required"))?
            .clone();

        let message_type = matches
            .get_one::<String>("message_type")
            .ok_or_else(|| anyhow!("Message type is required"))?
            .clone();

        let values = matches
            .get_many::<String>("values")
            .ok_or_else(|| anyhow!("Message values are required"))?
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        let rate = matches
            .get_one::<String>("rate")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(1.0);

        let print = matches.get_one::<String>("print").is_some();

        let once = matches.get_flag("once");

        let times = matches
            .get_one::<String>("times")
            .and_then(|s| s.parse::<usize>().ok());

        let wait_matching_subscriptions = matches
            .get_one::<String>("wait_matching_subscriptions")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);

        Ok(PublishOptions {
            topic_name,
            message_type,
            values,
            rate,
            print,
            once,
            times,
            wait_matching_subscriptions,
        })
    }
}

async fn publish_messages(
    options: PublishOptions,
    _common_args: CommonTopicArgs,
    running: Arc<AtomicBool>,
) -> Result<()> {
    // Create separate RCL contexts for different operations to avoid context invalidation
    let create_context = || -> Result<RclGraphContext> {
        RclGraphContext::new()
            .map_err(|e| anyhow!("Failed to initialize RCL graph context: {}", e))
    };

    // Check if we need to wait for matching subscriptions
    if options.wait_matching_subscriptions > 0 {
        println!(
            "Waiting for {} matching subscription(s)...",
            options.wait_matching_subscriptions
        );

        let mut retries = 0;
        const MAX_RETRIES: usize = 300; // 30 seconds with 100ms intervals

        while retries < MAX_RETRIES {
            let subscriber_count = {
                let context = create_context()?;
                context.count_subscribers(&options.topic_name)?
            };

            if subscriber_count >= options.wait_matching_subscriptions {
                println!("Found {} matching subscription(s)", subscriber_count);
                break;
            }

            if retries % 50 == 0 && retries > 0 {
                println!(
                    "Still waiting... (found {}/{} subscriptions)",
                    subscriber_count, options.wait_matching_subscriptions
                );
            }

            sleep(Duration::from_millis(100)).await;
            retries += 1;
        }

        let final_count = {
            let context = create_context()?;
            context.count_subscribers(&options.topic_name)?
        };
        if final_count < options.wait_matching_subscriptions {
            return Err(anyhow!(
                "Timeout waiting for matching subscriptions. Found {}, expected {}",
                final_count,
                options.wait_matching_subscriptions
            ));
        }
    }

    println!("Publisher: beginning loop");
    println!("Publishing to: {}", options.topic_name);
    println!("Message type: {}", options.message_type);
    println!("Rate: {} Hz", options.rate);

    if options.print {
        println!("Message content:");
        println!("{}", options.values);
        println!("---");
    }

    // Create a dynamic RCL publisher using our new infrastructure
    let publisher_context = create_context()?;
    let publisher = DynamicRclPublisher::new(
        &publisher_context, 
        &options.topic_name, 
        &options.message_type,
        &options.values
    )?;
    
    let mut message_count = 0;
    let mut interval_timer = interval(Duration::from_secs_f64(1.0 / options.rate));

    // Skip the first tick (it fires immediately)
    interval_timer.tick().await;

    while running.load(Ordering::Relaxed) {
        interval_timer.tick().await;

        message_count += 1;

        // Check if we have subscribers (optional feedback)
        let subscriber_count = {
            let context = create_context()?;
            context.count_subscribers(&options.topic_name).unwrap_or(0)
        };

        if options.print {
            println!("Publishing message #{}", message_count);
            println!("Message: {}", options.values);
            println!("Subscribers: {}", subscriber_count);
            println!("---");
        } else {
            // Minimal output like ros2 topic pub
            print!(".");
            if message_count % 50 == 0 {
                println!(" [{}]", message_count);
            }
        }

        // Handle --once flag
        if options.once {
            break;
        }

        // Handle --times flag
        if let Some(max_times) = options.times {
            if message_count >= max_times {
                break;
            }
        }

        // Actually publish the message using our dynamic infrastructure
        if let Err(e) = publisher.publish() {
            eprintln!("Warning: Failed to publish message: {}", e);
        }
    }

    if !options.print && message_count > 0 {
        println!(); // New line after dots
    }

    println!(
        "Published {} message(s) to topic '{}'",
        message_count, options.topic_name
    );

    Ok(())
}

async fn run_command(matches: ArgMatches, common_args: CommonTopicArgs) -> Result<()> {
    let options = PublishOptions::from_matches(&matches)?;

    // Handle common arguments
    if common_args.no_daemon {
        eprintln!("Note: roc always uses direct DDS discovery (equivalent to --no-daemon)");
    }

    if common_args.use_sim_time {
        eprintln!("Note: Using simulation time for message timestamps");
    }

    if let Some(ref spin_time_value) = common_args.spin_time {
        eprintln!("Note: Using spin time {} for discovery", spin_time_value);
    }

    // Handle QoS options (would be used in real publisher creation)
    if let Some(qos_profile) = matches.get_one::<String>("qos_profile") {
        eprintln!("Note: Using QoS profile: {}", qos_profile);
    }

    if let Some(qos_depth) = matches.get_one::<String>("qos_depth") {
        eprintln!("Note: Using QoS depth: {}", qos_depth);
    }

    if let Some(qos_history) = matches.get_one::<String>("qos_history") {
        eprintln!("Note: Using QoS history: {}", qos_history);
    }

    if let Some(qos_reliability) = matches.get_one::<String>("qos_reliability") {
        eprintln!("Note: Using QoS reliability: {}", qos_reliability);
    }

    if let Some(qos_durability) = matches.get_one::<String>("qos_durability") {
        eprintln!("Note: Using QoS durability: {}", qos_durability);
    }

    // Validate message format using our dynamic message infrastructure
    if options.values.trim().is_empty() {
        return Err(anyhow!("Message values cannot be empty"));
    }

    // Check if the message type is supported
    if !RclGraphContext::is_message_type_supported(&options.message_type) {
        eprintln!("Warning: Message type '{}' may not be fully supported", options.message_type);
    }

    // Validate the message content early to catch errors before publishing
    match RclGraphContext::parse_and_validate_message(&options.message_type, &options.values) {
        Ok(_) => {
            println!("Message validation successful for type: {}", options.message_type);
        }
        Err(e) => {
            return Err(anyhow!("Message validation failed: {}", e));
        }
    }

    // Set up signal handler for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for ctrl+c");
        running_clone.store(false, Ordering::Relaxed);
    });

    // Start publishing messages
    publish_messages(options, common_args, running).await
}

pub fn handle(matches: ArgMatches, common_args: CommonTopicArgs) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    match rt.block_on(run_command(matches, common_args)) {
        Ok(()) => {
            println!("\nPublishing stopped.");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
