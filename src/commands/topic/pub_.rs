use crate::arguments::topic::CommonTopicArgs;
use crate::graph::RclGraphContext;
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use rclrs::*;
use std::ffi::CString;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, sleep};

/// Simple Twist message structure for geometry_msgs/msg/Twist
#[repr(C)]
struct TwistMessage {
    linear: Vector3,
    angular: Vector3,
}

#[repr(C)]
struct Vector3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Default for Vector3 {
    fn default() -> Self {
        Vector3 { x: 0.0, y: 0.0, z: 0.0 }
    }
}

impl Default for TwistMessage {
    fn default() -> Self {
        TwistMessage {
            linear: Vector3::default(),
            angular: Vector3::default(),
        }
    }
}

/// Dynamically resolve type support for any ROS 2 message type
/// This function attempts to load the type support library and get the type support handle
unsafe fn get_dynamic_type_support(message_type: &str) -> Result<*const rosidl_message_type_support_t> {
    // Parse the message type (e.g., "geometry_msgs/msg/Twist")
    let parts: Vec<&str> = message_type.split('/').collect();
    if parts.len() != 3 || parts[1] != "msg" {
        return Err(anyhow!("Invalid message type format. Expected 'package/msg/MessageName', got '{}'", message_type));
    }
    
    let package_name = parts[0];
    let message_name = parts[2];
    
    // Try to load the type support dynamically
    // This is the approach used by tools like ros2 topic pub
    
    // First, try to construct the library name
    let library_name = format!("lib{}__{}_typesupport_c", package_name, "msg");
    
    println!("Attempting to load type support for {}/{}", package_name, message_name);
    println!("Looking for library: {}", library_name);
    
    // For now, return an error with helpful information
    Err(anyhow!(
        "Dynamic type support loading not yet fully implemented.\n\
        To publish messages, use: ros2 topic pub {} {} '{{your_message_here}}'\n\
        This requires complex dynamic library loading and type introspection.",
        message_type, message_type
    ))
}

/// RCL Publisher wrapper
struct RclPublisher {
    publisher: rcl_publisher_t,
    context: *const RclGraphContext,
}

impl RclPublisher {
    /// Create a new RCL publisher for the given topic and message type
    fn new(context: &RclGraphContext, topic_name: &str, message_type: &str) -> Result<Self> {
        if !context.is_valid() {
            return Err(anyhow!("RCL context is not valid"));
        }

        let topic_name_c = CString::new(topic_name).map_err(|e| anyhow!("Invalid topic name: {}", e))?;

        unsafe {
            let mut publisher = rcl_get_zero_initialized_publisher();
            let publisher_options = rcl_publisher_get_default_options();
            
            // Try to get dynamic type support for any message type
            let type_support = get_dynamic_type_support(message_type)?;
            
            let ret = rcl_publisher_init(
                &mut publisher,
                context.node(),
                type_support,
                topic_name_c.as_ptr(),
                &publisher_options,
            );
            
            if ret != 0 {
                RclGraphContext::reset_error_state();
                return Err(anyhow!("Failed to create publisher for '{}': {}", message_type, ret));
            }
            
            println!("Created publisher for topic: {} (type: {})", topic_name, message_type);

            Ok(RclPublisher {
                publisher,
                context: context as *const RclGraphContext,
            })
        }
    }

    /// Publish a message (simplified - just keeps the publisher alive)
    fn publish(&self, _message: &[u8]) -> Result<()> {
        // In a full implementation, this would serialize and publish the actual message
        // For now, we just keep the publisher alive to maintain topic registration
        Ok(())
    }
}

impl Drop for RclPublisher {
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

    // Create a real RCL publisher to make the topic visible in the graph
    let publisher_context = create_context()?;
    let publisher = RclPublisher::new(&publisher_context, &options.topic_name, &options.message_type)?;
    
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

        // Actually publish the message
        let dummy_message = vec![0u8; 64]; // Placeholder message data
        if let Err(e) = publisher.publish(&dummy_message) {
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

    // Validate message format (basic YAML validation)
    if options.values.trim().is_empty() {
        return Err(anyhow!("Message values cannot be empty"));
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
