use anyhow::{anyhow, Result};
use rclrs::{Context, CreateBasicExecutor, DynamicMessage, MessageTypeName, Node, SpinOptions};
use std::{
    fmt,
    sync::{mpsc, Arc, Mutex},
    thread,
};

/// A received dynamic message with its ROS metadata.
///
/// Note: `rclrs::DynamicMessage` does not implement `Debug`, so this type does
/// not derive it.
pub struct ReceivedDynamicMessage {
    pub message: DynamicMessage,
    pub info: rclrs::MessageInfo,
}

/// A minimal dynamic subscription helper.
///
/// This is a replacement for the previous raw `rcl_take`-based implementation.
/// It uses the safe `rclrs` dynamic subscription API and delivers messages
/// through an internal channel.
pub struct DynamicSubscriber {
    receiver: Mutex<mpsc::Receiver<ReceivedDynamicMessage>>,
    #[allow(dead_code)]
    _subscription: rclrs::DynamicSubscription,
    #[allow(dead_code)]
    _spin_thread: thread::JoinHandle<()>,
}

impl fmt::Debug for DynamicSubscriber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DynamicSubscriber").finish_non_exhaustive()
    }
}

impl DynamicSubscriber {
    /// Create a new dynamic subscription.
    ///
    /// Important: This will spin a dedicated executor thread to drive callbacks.
    pub fn new(topic_name: &str, message_type: &str) -> Result<Self> {
        let context = Context::default_from_env()?;
        let executor = context.create_basic_executor();
        let node = executor.create_node("roc_dynamic_subscriber")?;
        Self::new_with_node(node, executor, topic_name, message_type)
    }

    /// Create a new dynamic subscription using an existing node.
    ///
    /// This is useful when sharing a graph node across operations.
    pub fn new_with_node(
        node: Node,
        executor: rclrs::Executor,
        topic_name: &str,
        message_type: &str,
    ) -> Result<Self> {
        let (tx, rx) = mpsc::channel::<ReceivedDynamicMessage>();
        let tx = Arc::new(Mutex::new(tx));

        let msg_type: MessageTypeName = message_type
            .try_into()
            .map_err(|e| anyhow!("Invalid message type '{}': {}", message_type, e))?;

        let subscription = node.create_dynamic_subscription(
            msg_type,
            topic_name,
            move |msg: DynamicMessage, info: rclrs::MessageInfo| {
                if let Ok(lock) = tx.lock() {
                    let _ = lock.send(ReceivedDynamicMessage { message: msg, info });
                }
            },
        )?;

        let spin_thread = thread::spawn(move || {
            // Spin forever. The thread will end when process exits.
            // This tool is primarily for CLI usage.
            let mut executor = executor;
            let _ = executor.spin(SpinOptions::default());
        });

        Ok(Self {
            receiver: Mutex::new(rx),
            _subscription: subscription,
            _spin_thread: spin_thread,
        })
    }

    /// Try to receive a message without blocking.
    pub fn try_recv(&self) -> Result<Option<ReceivedDynamicMessage>> {
        let rx = self.receiver.lock().unwrap();
        match rx.try_recv() {
            Ok(msg) => Ok(Some(msg)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => {
                Err(anyhow!("dynamic subscription channel disconnected"))
            }
        }
    }

    /// Compatibility helper used by existing topic commands.
    ///
    /// This replaces the former `take_message()` method from the raw `rcl_take` implementation.
    pub fn take_message(&self) -> Result<Option<ReceivedDynamicMessage>> {
        self.try_recv()
    }
}
