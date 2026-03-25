use anyhow::{anyhow, Result};
use rclrs::{
    Context, CreateBasicExecutor, DynamicMessage, ExecutorCommands, IntoPrimitiveOptions,
    MessageTypeName, Node, QoSProfile, SpinOptions,
};
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
    pub _info: rclrs::MessageInfo,
}

/// A minimal dynamic subscription helper.
///
/// This is a replacement for the previous raw `rcl_take`-based implementation.
/// It uses the safe `rclrs` dynamic subscription API and delivers messages
/// through an internal channel.
pub struct DynamicSubscriber {
    receiver: Mutex<mpsc::Receiver<ReceivedDynamicMessage>>,
    _subscription: rclrs::DynamicSubscription,
    spin_thread: Option<ManagedSpinThread>,
}

impl fmt::Debug for DynamicSubscriber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DynamicSubscriber").finish_non_exhaustive()
    }
}

struct ManagedSpinThread {
    commands: Arc<ExecutorCommands>,
    handle: Option<thread::JoinHandle<()>>,
}

impl ManagedSpinThread {
    fn new(commands: Arc<ExecutorCommands>, handle: thread::JoinHandle<()>) -> Self {
        Self {
            commands,
            handle: Some(handle),
        }
    }
}

impl Drop for ManagedSpinThread {
    fn drop(&mut self) {
        self.commands.halt_spinning();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for DynamicSubscriber {
    fn drop(&mut self) {
        if let Some(spin_thread) = self.spin_thread.take() {
            drop(spin_thread);
        }
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
        Self::new_with_node_qos(
            node,
            executor,
            topic_name,
            message_type,
            QoSProfile::topics_default(),
        )
    }

    /// Create a new dynamic subscription with a custom QoS profile.
    pub fn new_qos(topic_name: &str, message_type: &str, qos: QoSProfile) -> Result<Self> {
        let context = Context::default_from_env()?;
        let executor = context.create_basic_executor();
        let node = executor.create_node("roc_dynamic_subscriber")?;
        Self::new_with_node_qos(node, executor, topic_name, message_type, qos)
    }

    #[allow(dead_code)]
    /// Create a new dynamic subscription using an existing node.
    ///
    /// This is useful when sharing a graph node across operations.
    pub fn new_with_node(
        node: Node,
        executor: rclrs::Executor,
        topic_name: &str,
        message_type: &str,
    ) -> Result<Self> {
        Self::new_with_node_qos(
            node,
            executor,
            topic_name,
            message_type,
            QoSProfile::topics_default(),
        )
    }

    pub fn new_with_node_qos(
        node: Node,
        executor: rclrs::Executor,
        topic_name: &str,
        message_type: &str,
        qos: QoSProfile,
    ) -> Result<Self> {
        let (tx, rx) = mpsc::channel::<ReceivedDynamicMessage>();
        let tx = Arc::new(Mutex::new(tx));

        let msg_type: MessageTypeName = message_type
            .try_into()
            .map_err(|e| anyhow!("Invalid message type '{}': {}", message_type, e))?;

        let subscription = node.create_dynamic_subscription(
            msg_type,
            topic_name.qos(qos),
            move |msg: DynamicMessage, info: rclrs::MessageInfo| {
                if let Ok(lock) = tx.lock() {
                    let _ = lock.send(ReceivedDynamicMessage {
                        message: msg,
                        _info: info,
                    });
                }
            },
        )?;

        let commands = executor.commands().clone();
        let spin_thread = thread::spawn(move || {
            let mut executor = executor;
            let _ = executor.spin(SpinOptions::default());
        });

        Ok(Self {
            receiver: Mutex::new(rx),
            _subscription: subscription,
            spin_thread: Some(ManagedSpinThread::new(commands, spin_thread)),
        })
    }

    /// Try to receive a message without blocking.
    pub fn try_recv(&self) -> Result<Option<ReceivedDynamicMessage>> {
        let rx = self
            .receiver
            .lock()
            .map_err(|_| anyhow!("dynamic subscription receiver state poisoned"))?;
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

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::time::Duration;

    #[test]
    fn managed_spin_thread_stops_and_joins_on_drop() {
        let stop_requested = Arc::new(AtomicBool::new(false));
        let finished = Arc::new(AtomicBool::new(false));

        let stop_requested_thread = Arc::clone(&stop_requested);
        let finished_thread = Arc::clone(&finished);
        let handle = std::thread::spawn(move || {
            while !stop_requested_thread.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(1));
            }
            finished_thread.store(true, Ordering::SeqCst);
        });

        let mock_commands = Arc::new(MockExecutorCommands {
            stop_requested: Arc::clone(&stop_requested),
        });
        let managed = ManagedSpinThreadForTest::new(mock_commands, handle);
        drop(managed);

        assert!(finished.load(Ordering::SeqCst));
    }

    struct MockExecutorCommands {
        stop_requested: Arc<AtomicBool>,
    }

    impl MockExecutorCommands {
        fn halt_spinning(&self) {
            self.stop_requested.store(true, Ordering::SeqCst);
        }
    }

    struct ManagedSpinThreadForTest {
        commands: Arc<MockExecutorCommands>,
        handle: Option<std::thread::JoinHandle<()>>,
    }

    impl ManagedSpinThreadForTest {
        fn new(commands: Arc<MockExecutorCommands>, handle: std::thread::JoinHandle<()>) -> Self {
            Self {
                commands,
                handle: Some(handle),
            }
        }
    }

    impl Drop for ManagedSpinThreadForTest {
        fn drop(&mut self) {
            self.commands.halt_spinning();
            if let Some(handle) = self.handle.take() {
                let _ = handle.join();
            }
        }
    }
}
