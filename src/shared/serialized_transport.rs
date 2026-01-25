use anyhow::{anyhow, Result};
use rclrs::{
    Context, CreateBasicExecutor, IntoPrimitiveOptions, MessageTypeName, QoSProfile,
    SerializedMessage,
};
use std::time::Duration;

pub struct SerializedReceiver {
    _context: Context,
    sub: rclrs::SerializedSubscription,
    buf: SerializedMessage,
}

impl SerializedReceiver {
    pub fn new(topic: &str, message_type: &str) -> Result<Self> {
        let context = Context::default_from_env()?;
        let executor = context.create_basic_executor();
        let node = executor.create_node("roc_bag_receiver")?;

        let ty: MessageTypeName = message_type.try_into()?;
        let qos = QoSProfile::topics_default().reliable();
        let sub = node.create_serialized_subscription(ty, topic.qos(qos))?;

        // Start with 1MB buffer.
        let buf = SerializedMessage::new(1024 * 1024)?;

        Ok(Self {
            _context: context,
            sub,
            buf,
        })
    }

    pub fn take(&mut self) -> Result<Option<Vec<u8>>> {
        self.buf.clear();
        let info = self.sub.take(&mut self.buf)?;
        if info.is_none() {
            return Ok(None);
        }
        Ok(Some(self.buf.as_bytes().to_vec()))
    }
}

pub struct SerializedSender {
    _context: Context,
    pub_: rclrs::SerializedPublisher,
    buf: SerializedMessage,
}

impl SerializedSender {
    pub fn new(topic: &str, message_type: &str) -> Result<Self> {
        let context = Context::default_from_env()?;
        let executor = context.create_basic_executor();
        let node = executor.create_node("roc_bag_sender")?;

        let ty: MessageTypeName = message_type.try_into()?;
        let qos = QoSProfile::topics_default().reliable();
        let pub_ = node.create_serialized_publisher(ty, topic.qos(qos))?;

        let buf = SerializedMessage::new(1024 * 1024)?;

        Ok(Self {
            _context: context,
            pub_,
            buf,
        })
    }

    pub fn publish(&mut self, data: &[u8]) -> Result<()> {
        self.buf
            .set_bytes(data)
            .map_err(|e| anyhow!("Failed to set bytes: {e}"))?;
        self.pub_
            .publish(&self.buf)
            .map_err(|e| anyhow!("Publish failed: {e}"))?;
        Ok(())
    }
}

pub fn sleep_short() {
    std::thread::sleep(Duration::from_millis(10));
}
