use crate::commands::cli::handle_anyhow_result;
use anyhow::{Result, anyhow};
use clap::ArgMatches;
use rclrs::IntoPrimitiveOptions;
use rclrs::{Context, CreateBasicExecutor, DynamicMessage, MessageTypeName, QoSProfile};

fn parse_array_f64(text: &str) -> Result<Vec<f64>> {
    let v: serde_yaml::Value = serde_yaml::from_str(text)
        .map_err(|e| anyhow!("Failed to parse array '{}': {}", text, e))?;
    let seq = v
        .as_sequence()
        .ok_or_else(|| anyhow!("Expected an array like [..], got: {}", text))?;
    let mut out = Vec::with_capacity(seq.len());
    for item in seq {
        let n = item
            .as_f64()
            .or_else(|| item.as_i64().map(|i| i as f64))
            .or_else(|| item.as_u64().map(|u| u as f64))
            .ok_or_else(|| anyhow!("Array element is not a number: {:?}", item))?;
        out.push(n);
    }
    Ok(out)
}

fn rpy_to_quat(roll: f64, pitch: f64, yaw: f64) -> (f64, f64, f64, f64) {
    // Standard intrinsic ZYX (yaw-pitch-roll)
    let (sr, cr) = (0.5 * roll).sin_cos();
    let (sp, cp) = (0.5 * pitch).sin_cos();
    let (sy, cy) = (0.5 * yaw).sin_cos();

    let qw = cr * cp * cy + sr * sp * sy;
    let qx = sr * cp * cy - cr * sp * sy;
    let qy = cr * sp * cy + sr * cp * sy;
    let qz = cr * cp * sy - sr * sp * cy;
    (qw, qx, qy, qz)
}

fn normalize_quat(qw: f64, qx: f64, qy: f64, qz: f64) -> (f64, f64, f64, f64) {
    let n = (qw * qw + qx * qx + qy * qy + qz * qz).sqrt();
    if n == 0.0 {
        return (1.0, 0.0, 0.0, 0.0);
    }
    (qw / n, qx / n, qy / n, qz / n)
}

fn set_f64_field(msg: &mut rclrs::DynamicMessageViewMut<'_>, field: &str, v: f64) -> Result<()> {
    let Some(rclrs::ValueMut::Simple(simple)) = msg.get_mut(field) else {
        return Err(anyhow!("Missing field '{}'", field));
    };
    match simple {
        rclrs::SimpleValueMut::Double(x) => {
            *x = v;
            Ok(())
        }
        rclrs::SimpleValueMut::Float(x) => {
            *x = v as f32;
            Ok(())
        }
        _ => Err(anyhow!("Field '{}' is not float/double", field)),
    }
}

fn set_string_field(
    msg: &mut rclrs::DynamicMessageViewMut<'_>,
    field: &str,
    v: &str,
) -> Result<()> {
    let Some(rclrs::ValueMut::Simple(simple)) = msg.get_mut(field) else {
        return Err(anyhow!("Missing field '{}'", field));
    };
    match simple {
        rclrs::SimpleValueMut::String(s) => {
            // rosidl_runtime_rs::String doesn't implement std::string methods.
            *s = v.into();
            Ok(())
        }
        _ => Err(anyhow!("Field '{}' is not string", field)),
    }
}

fn ensure_seq_len(msg: &mut DynamicMessage, field: &str, len: usize) -> Result<()> {
    let Some(rclrs::ValueMut::Sequence(seq)) = msg.get_mut(field) else {
        return Err(anyhow!("Missing sequence field '{}'", field));
    };
    match seq {
        rclrs::SequenceValueMut::MessageSequence(mut s) => {
            s.reset(len);
            Ok(())
        }
        _ => Err(anyhow!("Field '{}' is not a message sequence", field)),
    }
}

fn run_command(matches: ArgMatches) -> Result<()> {
    let frame_id = matches
        .get_one::<String>("FRAME_ID")
        .ok_or_else(|| anyhow!("FRAME_ID is required"))?;
    let child_frame_id = matches
        .get_one::<String>("CHILD_FRAME_ID")
        .ok_or_else(|| anyhow!("CHILD_FRAME_ID is required"))?;
    let translation = matches
        .get_one::<String>("TRANSLATION")
        .ok_or_else(|| anyhow!("TRANSLATION is required"))?;
    let rotation = matches
        .get_one::<String>("ROTATION")
        .ok_or_else(|| anyhow!("ROTATION is required"))?;

    let detach = matches.get_flag("detach");

    let t = parse_array_f64(translation)?;
    if t.len() != 3 {
        return Err(anyhow!("TRANSLATION must have 3 elements: [x,y,z]"));
    }

    let r = parse_array_f64(rotation)?;
    let (qw, qx, qy, qz) = match r.len() {
        4 => (r[0], r[1], r[2], r[3]),
        3 => rpy_to_quat(r[0], r[1], r[2]),
        _ => {
            return Err(anyhow!(
                "ROTATION must be [qw,qx,qy,qz] or [roll,pitch,yaw]"
            ));
        }
    };
    let (qw, qx, qy, qz) = normalize_quat(qw, qx, qy, qz);

    let context = Context::default_from_env()?;
    let executor = context.create_basic_executor();
    let node = executor.create_node("roc_frame_pub")?;

    let msg_type: MessageTypeName = "tf2_msgs/msg/TFMessage".try_into()?;

    // Match tf_static QoS (transient local + reliable).
    let qos = QoSProfile::services_default().reliable().transient_local();
    let publisher = node.create_dynamic_publisher(msg_type, "/tf_static".qos(qos))?;

    // Build TFMessage with one TransformStamped.
    let mut tfmsg = DynamicMessage::new("tf2_msgs/msg/TFMessage".try_into()?)?;
    ensure_seq_len(&mut tfmsg, "transforms", 1)?;

    // transforms[0]
    let mut tfmsg_view = tfmsg.view_mut();
    let Some(rclrs::ValueMut::Sequence(rclrs::SequenceValueMut::MessageSequence(mut transforms))) =
        tfmsg_view.get_mut("transforms")
    else {
        return Err(anyhow!("TFMessage.transforms missing"));
    };
    let ts = transforms
        .as_mut_slice()
        .get_mut(0)
        .ok_or_else(|| anyhow!("TFMessage.transforms[0] missing"))?;

    // header.frame_id
    let Some(rclrs::ValueMut::Simple(rclrs::SimpleValueMut::Message(mut header))) =
        ts.get_mut("header")
    else {
        return Err(anyhow!("TransformStamped.header missing"));
    };
    set_string_field(&mut header, "frame_id", frame_id)?;

    // child_frame_id
    set_string_field(ts, "child_frame_id", child_frame_id)?;

    // transform.translation + rotation
    let Some(rclrs::ValueMut::Simple(rclrs::SimpleValueMut::Message(mut transform))) =
        ts.get_mut("transform")
    else {
        return Err(anyhow!("TransformStamped.transform missing"));
    };

    let Some(rclrs::ValueMut::Simple(rclrs::SimpleValueMut::Message(mut translation_msg))) =
        transform.get_mut("translation")
    else {
        return Err(anyhow!("Transform.translation missing"));
    };
    set_f64_field(&mut translation_msg, "x", t[0])?;
    set_f64_field(&mut translation_msg, "y", t[1])?;
    set_f64_field(&mut translation_msg, "z", t[2])?;

    let Some(rclrs::ValueMut::Simple(rclrs::SimpleValueMut::Message(mut rotation_msg))) =
        transform.get_mut("rotation")
    else {
        return Err(anyhow!("Transform.rotation missing"));
    };
    set_f64_field(&mut rotation_msg, "x", qx)?;
    set_f64_field(&mut rotation_msg, "y", qy)?;
    set_f64_field(&mut rotation_msg, "z", qz)?;
    set_f64_field(&mut rotation_msg, "w", qw)?;

    publisher.publish(tfmsg)?;

    // Default behavior matches `static_transform_publisher`: keep the node alive so the
    // TRANSIENT_LOCAL sample is reliably available to late joiners.
    if !detach {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3600));
        }
    }
    Ok(())
}

pub fn handle(matches: ArgMatches) {
    handle_anyhow_result(run_command(matches));
}
