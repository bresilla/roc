use anyhow::Result;
use rclrs::Context;
use rclrs::QoSProfile;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use crate::shared::dynamic_messages::DynamicSubscriber;

#[derive(Debug, Clone, Copy)]
pub struct TfEdgeTransform {
    pub tx: f64,
    pub ty: f64,
    pub tz: f64,
    pub qx: f64,
    pub qy: f64,
    pub qz: f64,
    pub qw: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TfEdgeKind {
    Static,
    Dynamic,
}

fn extract_string(v: rclrs::Value<'_>) -> Option<String> {
    match v {
        rclrs::Value::Simple(rclrs::SimpleValue::String(s)) => Some(s.to_string()),
        rclrs::Value::Simple(rclrs::SimpleValue::BoundedString(s)) => Some(s.to_string()),
        rclrs::Value::Simple(rclrs::SimpleValue::WString(s)) => Some(s.to_string()),
        rclrs::Value::Simple(rclrs::SimpleValue::BoundedWString(s)) => Some(s.to_string()),
        _ => None,
    }
}

fn extract_f64(v: rclrs::Value<'_>) -> Option<f64> {
    match v {
        rclrs::Value::Simple(rclrs::SimpleValue::Double(x)) => Some(*x),
        rclrs::Value::Simple(rclrs::SimpleValue::Float(x)) => Some(*x as f64),
        _ => None,
    }
}

fn get_msg_field<'a>(
    msg: &'a rclrs::DynamicMessageView<'a>,
    name: &str,
) -> Option<rclrs::Value<'a>> {
    msg.get(name)
}

fn parse_transform_stamped(
    transform: &rclrs::DynamicMessageView<'_>,
) -> Option<(String, String, TfEdgeTransform)> {
    let header = get_msg_field(transform, "header")?;
    let rclrs::Value::Simple(rclrs::SimpleValue::Message(header)) = header else {
        return None;
    };
    let frame_id = header.get("frame_id").and_then(extract_string)?;

    let child_frame_id = get_msg_field(transform, "child_frame_id").and_then(extract_string)?;

    let tf = get_msg_field(transform, "transform")?;
    let rclrs::Value::Simple(rclrs::SimpleValue::Message(tf)) = tf else {
        return None;
    };
    let translation = tf.get("translation")?;
    let rclrs::Value::Simple(rclrs::SimpleValue::Message(translation)) = translation else {
        return None;
    };
    let rotation = tf.get("rotation")?;
    let rclrs::Value::Simple(rclrs::SimpleValue::Message(rotation)) = rotation else {
        return None;
    };

    let tx = translation.get("x").and_then(extract_f64)?;
    let ty = translation.get("y").and_then(extract_f64)?;
    let tz = translation.get("z").and_then(extract_f64)?;
    let qx = rotation.get("x").and_then(extract_f64)?;
    let qy = rotation.get("y").and_then(extract_f64)?;
    let qz = rotation.get("z").and_then(extract_f64)?;
    let qw = rotation.get("w").and_then(extract_f64)?;

    Some((
        frame_id,
        child_frame_id,
        TfEdgeTransform {
            tx,
            ty,
            tz,
            qx,
            qy,
            qz,
            qw,
        },
    ))
}

fn add_tfmessage_edges(
    frames: &Arc<Mutex<BTreeSet<String>>>,
    edges: &Arc<Mutex<BTreeMap<(String, String), TfEdgeTransform>>>,
    msg: &rclrs::DynamicMessage,
) {
    let view = msg.view();
    let Some(rclrs::Value::Sequence(rclrs::SequenceValue::MessageSequence(seq))) =
        view.get("transforms")
    else {
        return;
    };

    for transform in seq.iter() {
        let Some((parent, child, tf)) = parse_transform_stamped(transform) else {
            continue;
        };

        let Ok(mut frames_guard) = frames.lock() else {
            return;
        };
        let Ok(mut edges_guard) = edges.lock() else {
            return;
        };

        if !parent.is_empty() {
            frames_guard.insert(parent.clone());
        }
        if !child.is_empty() {
            frames_guard.insert(child.clone());
        }
        if !parent.is_empty() && !child.is_empty() {
            edges_guard.insert((parent, child), tf);
        }
    }
}

pub struct TfFrameIndex {
    #[allow(dead_code)]
    frames: Arc<Mutex<BTreeSet<String>>>,
    edges_dynamic: Arc<Mutex<BTreeMap<(String, String), TfEdgeTransform>>>,
    edges_static: Arc<Mutex<BTreeMap<(String, String), TfEdgeTransform>>>,
    _sub_tf: Arc<DynamicSubscriber>,
    _sub_tf_static: Arc<DynamicSubscriber>,
}

impl TfFrameIndex {
    pub fn new() -> Result<Self> {
        let _ = Context::default_from_env()?;

        let frames = Arc::new(Mutex::new(BTreeSet::new()));
        let edges_dynamic = Arc::new(Mutex::new(BTreeMap::new()));
        let edges_static = Arc::new(Mutex::new(BTreeMap::new()));

        let sub_tf = Arc::new(DynamicSubscriber::new("/tf", "tf2_msgs/msg/TFMessage")?);

        // /tf_static is published with TRANSIENT_LOCAL durability.
        let tf_static_qos = QoSProfile::services_default().reliable().transient_local();
        let sub_tf_static = Arc::new(DynamicSubscriber::new_qos(
            "/tf_static",
            "tf2_msgs/msg/TFMessage",
            tf_static_qos,
        )?);

        Self::start_drain_thread(
            Arc::clone(&sub_tf),
            Arc::clone(&frames),
            Arc::clone(&edges_dynamic),
        );
        Self::start_drain_thread(
            Arc::clone(&sub_tf_static),
            Arc::clone(&frames),
            Arc::clone(&edges_static),
        );

        Ok(Self {
            frames,
            edges_dynamic,
            edges_static,
            _sub_tf: sub_tf,
            _sub_tf_static: sub_tf_static,
        })
    }

    pub fn has_any_data(&self) -> bool {
        self.edges_dynamic
            .lock()
            .map(|edges| !edges.is_empty())
            .unwrap_or(false)
            || self
                .edges_static
                .lock()
                .map(|edges| !edges.is_empty())
                .unwrap_or(false)
    }

    fn start_drain_thread(
        sub: Arc<DynamicSubscriber>,
        frames: Arc<Mutex<BTreeSet<String>>>,
        edges: Arc<Mutex<BTreeMap<(String, String), TfEdgeTransform>>>,
    ) {
        std::thread::spawn(move || {
            loop {
                match sub.take_message() {
                    Ok(Some(msg)) => {
                        add_tfmessage_edges(&frames, &edges, &msg.message);
                    }
                    Ok(None) => {
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                    Err(_) => break,
                }
            }
        });
    }

    #[allow(dead_code)]
    pub fn frames(&self) -> Vec<String> {
        self.frames
            .lock()
            .map(|frames| frames.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn edges(&self) -> Vec<((String, String), TfEdgeTransform, TfEdgeKind)> {
        let Ok(dyn_edges) = self.edges_dynamic.lock() else {
            return Vec::new();
        };
        let Ok(stat_edges) = self.edges_static.lock() else {
            return Vec::new();
        };

        let mut out: BTreeMap<(String, String), (TfEdgeTransform, TfEdgeKind)> = BTreeMap::new();
        for (k, v) in stat_edges.iter() {
            out.insert(k.clone(), (*v, TfEdgeKind::Static));
        }
        for (k, v) in dyn_edges.iter() {
            out.insert(k.clone(), (*v, TfEdgeKind::Dynamic));
        }

        out.into_iter().map(|(k, (v, t))| (k, v, t)).collect()
    }
}
