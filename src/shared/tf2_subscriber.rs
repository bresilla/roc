use anyhow::Result;
use rclrs::Context;
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use crate::shared::dynamic_messages::DynamicSubscriber;

pub struct TfFrameIndex {
    frames: Arc<Mutex<BTreeSet<String>>>,
    _sub_tf: DynamicSubscriber,
    _sub_tf_static: DynamicSubscriber,
}

impl TfFrameIndex {
    pub fn new() -> Result<Self> {
        // Ensure RCL context is initialized (DynamicSubscriber also does this, but keep it explicit).
        let _ = Context::default_from_env()?;

        let frames = Arc::new(Mutex::new(BTreeSet::new()));

        // Each DynamicSubscriber spins its own executor thread internally.
        let sub_tf = Self::make_subscriber("/tf", frames.clone())?;
        let sub_tf_static = Self::make_subscriber("/tf_static", frames.clone())?;

        Ok(Self {
            frames,
            _sub_tf: sub_tf,
            _sub_tf_static: sub_tf_static,
        })
    }

    fn make_subscriber(
        topic: &str,
        frames: Arc<Mutex<BTreeSet<String>>>,
    ) -> Result<DynamicSubscriber> {
        let sub = DynamicSubscriber::new(topic, "tf2_msgs/msg/TFMessage")?;
        // Keep one handle in the struct, move the other into the drain thread.
        let drain_sub = DynamicSubscriber::new(topic, "tf2_msgs/msg/TFMessage")?;

        // Spawn a background task to drain messages.
        // This uses the same executor thread started inside DynamicSubscriber.
        let frames_clone = frames.clone();
        std::thread::spawn(move || loop {
            match drain_sub.take_message() {
                Ok(Some(msg)) => {
                    // Temporary best-effort extraction: parse Debug output.
                    // This keeps `frame list` native while we wire the full TF buffer.
                    let dbg = format!("{:?}", msg.message.view());
                    for token in dbg.split_whitespace() {
                        if let Some(rest) = token.strip_prefix("frame_id=") {
                            frames_clone
                                .lock()
                                .unwrap()
                                .insert(rest.trim_matches('"').to_string());
                        }
                        if let Some(rest) = token.strip_prefix("child_frame_id=") {
                            frames_clone
                                .lock()
                                .unwrap()
                                .insert(rest.trim_matches('"').to_string());
                        }
                    }
                }
                Ok(None) => {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                Err(_) => break,
            }
        });

        Ok(sub)
    }

    pub fn frames(&self) -> Vec<String> {
        self.frames.lock().unwrap().iter().cloned().collect()
    }
}
