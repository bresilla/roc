use crate::commands::cli::{install_ctrlc_flag, print_error_and_exit};
use crate::ui::{
    blocks,
    output::{self, OutputMode},
};
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use rclrs::IntoPrimitiveOptions;
use rclrs::{Context, CreateBasicExecutor, DynamicMessage, MessageTypeName, QoSProfile};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

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

fn print_frame_publish_header(
    output_mode: OutputMode,
    frame_id: &str,
    child_frame_id: &str,
    translation: &str,
    rotation: &str,
    detach: bool,
) {
    match output_mode {
        OutputMode::Human => {
            blocks::print_section("Frame Publish");
            blocks::print_field("Parent", frame_id);
            blocks::print_field("Child", child_frame_id);
            blocks::print_field("Translation", translation);
            blocks::print_field("Rotation", rotation);
            blocks::print_field("Topic", "/tf_static");
            blocks::print_field("Mode", if detach { "detach" } else { "latched" });
            println!();
            if !detach {
                blocks::print_note("Press Ctrl+C to stop");
            }
        }
        OutputMode::Plain => {
            output::print_plain_section("frame-publish");
            output::print_plain_field("parent", frame_id);
            output::print_plain_field("child", child_frame_id);
            output::print_plain_field("translation", translation);
            output::print_plain_field("rotation", rotation);
            output::print_plain_field("topic", "/tf_static");
            output::print_plain_field("mode", if detach { "detach" } else { "latched" });
        }
        OutputMode::Json => {}
    }
}

fn print_frame_publish_status(output_mode: OutputMode, frame_id: &str, child_frame_id: &str) {
    match output_mode {
        OutputMode::Human => blocks::print_status(
            "PUB",
            &[
                ("parent", frame_id.to_string()),
                ("child", child_frame_id.to_string()),
            ],
        ),
        OutputMode::Plain => output::print_plain_status(
            "pub",
            &[
                ("parent", frame_id.to_string()),
                ("child", child_frame_id.to_string()),
            ],
        ),
        OutputMode::Json => {}
    }
}

fn print_frame_publish_summary(
    output_mode: OutputMode,
    frame_id: &str,
    child_frame_id: &str,
    translation: &str,
    rotation: &str,
    detach: bool,
    elapsed_secs: f64,
    interrupted: bool,
) -> Result<()> {
    match output_mode {
        OutputMode::Human => {
            println!();
            blocks::print_section("Frame Summary");
            blocks::print_field("Parent", frame_id);
            blocks::print_field("Child", child_frame_id);
            blocks::print_field("Elapsed", format!("{elapsed_secs:.2}s"));
            blocks::print_success("Static transform publisher stopped");
        }
        OutputMode::Plain => {
            output::print_plain_section("frame-summary");
            output::print_plain_field("parent", frame_id);
            output::print_plain_field("child", child_frame_id);
            output::print_plain_field("translation", translation);
            output::print_plain_field("rotation", rotation);
            output::print_plain_field("mode", if detach { "detach" } else { "latched" });
            output::print_plain_field("elapsed_secs", format!("{elapsed_secs:.3}"));
            output::print_plain_field("interrupted", interrupted);
            output::print_plain_field("status", "ok");
        }
        OutputMode::Json => {
            output::print_json(&json!({
                "command": "frame pub",
                "parent": frame_id,
                "child": child_frame_id,
                "translation": translation,
                "rotation": rotation,
                "mode": if detach { "detach" } else { "latched" },
                "elapsed_secs": elapsed_secs,
                "interrupted": interrupted,
                "status": "ok"
            }))?;
        }
    }
    Ok(())
}

fn print_frame_publish_error(
    output_mode: OutputMode,
    frame_id: Option<&str>,
    child_frame_id: Option<&str>,
    error: &str,
) {
    match output_mode {
        OutputMode::Human => print_error_and_exit(error),
        OutputMode::Plain => {
            output::print_plain_section("frame-publish-error");
            if let Some(frame_id) = frame_id {
                output::print_plain_field("parent", frame_id);
            }
            if let Some(child_frame_id) = child_frame_id {
                output::print_plain_field("child", child_frame_id);
            }
            output::print_plain_field("status", "error");
            output::print_plain_field("error", error);
            std::process::exit(1);
        }
        OutputMode::Json => {
            let _ = output::print_json(&json!({
                "command": "frame pub",
                "parent": frame_id,
                "child": child_frame_id,
                "status": "error",
                "error": error
            }));
            std::process::exit(1);
        }
    }
}

fn run_command(matches: ArgMatches) -> Result<()> {
    let output_mode = OutputMode::from_matches(&matches);
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

    print_frame_publish_header(
        output_mode,
        frame_id,
        child_frame_id,
        translation,
        rotation,
        detach,
    );

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

    let started_at = Instant::now();
    publisher.publish(tfmsg)?;
    print_frame_publish_status(output_mode, frame_id, child_frame_id);

    // Default behavior matches `static_transform_publisher`: keep the node alive so the
    // TRANSIENT_LOCAL sample is reliably available to late joiners.
    let mut interrupted = false;
    if !detach {
        let running = Arc::new(AtomicBool::new(true));
        install_ctrlc_flag(Arc::clone(&running))?;
        while running.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(100));
        }
        interrupted = true;
    }

    print_frame_publish_summary(
        output_mode,
        frame_id,
        child_frame_id,
        translation,
        rotation,
        detach,
        started_at.elapsed().as_secs_f64(),
        interrupted,
    )?;
    Ok(())
}

pub fn handle(matches: ArgMatches) {
    let output_mode = OutputMode::from_matches(&matches);
    let frame_id = matches.get_one::<String>("FRAME_ID").cloned();
    let child_frame_id = matches.get_one::<String>("CHILD_FRAME_ID").cloned();

    if let Err(error) = run_command(matches) {
        print_frame_publish_error(
            output_mode,
            frame_id.as_deref(),
            child_frame_id.as_deref(),
            &error.to_string(),
        );
    }
}
