#![allow(dead_code)]

use anyhow::{Result, anyhow};
use libloading::Library;
use std::ffi::c_void;
use std::path::PathBuf;

use crate::shared::ros_tf_msgs_sys as sys;

#[repr(C)]
pub struct RosidlMessageTypeSupport {
    pub typesupport_identifier: *const i8,
    pub data: *const c_void,
    pub func: *const c_void,
}

type GetMsgTsFn = unsafe extern "C" fn() -> *const RosidlMessageTypeSupport;

pub struct LoadedTypeSupport {
    _lib: Library,
    pub handle: *const RosidlMessageTypeSupport,
}

fn ros_lib_dir() -> PathBuf {
    let prefix =
        std::env::var("AMENT_PREFIX_PATH").unwrap_or_else(|_| "/opt/ros/jazzy".to_string());
    PathBuf::from(prefix).join("lib")
}

fn load_symbol(lib: &Library, symbol: &str) -> Result<*const RosidlMessageTypeSupport> {
    unsafe {
        let sym: libloading::Symbol<GetMsgTsFn> = lib
            .get(symbol.as_bytes())
            .map_err(|e| anyhow!("Missing symbol {}: {}", symbol, e))?;
        Ok(sym())
    }
}

fn load_introspection_c_lib(pkg: &str) -> Result<Library> {
    let libname = format!("lib{}__rosidl_typesupport_introspection_c.so", pkg);
    let path = ros_lib_dir().join(libname);
    unsafe { Library::new(&path).map_err(|e| anyhow!("Failed to load {}: {}", path.display(), e)) }
}

fn load_typesupport_c_lib(pkg: &str) -> Result<Library> {
    let libname = format!("lib{}__rosidl_typesupport_c.so", pkg);
    let path = ros_lib_dir().join(libname);
    unsafe { Library::new(&path).map_err(|e| anyhow!("Failed to load {}: {}", path.display(), e)) }
}

pub struct LoadedMessageTypeSupport {
    pub _introspection_lib: Library,
    pub _typesupport_c_lib: Library,
    pub msg_type_support: *const sys::rosidl_message_type_support_t,
}

type GetMsgTsC = unsafe extern "C" fn() -> *const sys::rosidl_message_type_support_t;

unsafe fn load_ts_c_symbol(
    lib: &Library,
    symbol: &str,
) -> Result<*const sys::rosidl_message_type_support_t> {
    let sym: libloading::Symbol<GetMsgTsC> = lib
        .get(symbol.as_bytes())
        .map_err(|e| anyhow!("Missing symbol {}: {}", symbol, e))?;
    Ok(sym())
}

pub fn load_tfmessage_support() -> Result<LoadedMessageTypeSupport> {
    let intro = load_introspection_c_lib("tf2_msgs")?;
    let ts_c = load_typesupport_c_lib("tf2_msgs")?;
    let symbol = "rosidl_typesupport_c__get_message_type_support_handle__tf2_msgs__msg__TFMessage";
    let msg_type_support = unsafe { load_ts_c_symbol(&ts_c, symbol)? };
    Ok(LoadedMessageTypeSupport {
        _introspection_lib: intro,
        _typesupport_c_lib: ts_c,
        msg_type_support,
    })
}

pub fn tfmessage_introspection() -> Result<LoadedTypeSupport> {
    let lib = load_introspection_c_lib("tf2_msgs")?;
    let handle = load_symbol(
        &lib,
        "rosidl_typesupport_introspection_c__get_message_type_support_handle__tf2_msgs__msg__TFMessage",
    )?;
    Ok(LoadedTypeSupport { _lib: lib, handle })
}

pub fn transform_stamped_typesupport() -> Result<LoadedTypeSupport> {
    let lib = load_introspection_c_lib("geometry_msgs")?;
    let handle = load_symbol(
        &lib,
        "rosidl_typesupport_introspection_c__get_message_type_support_handle__geometry_msgs__msg__TransformStamped",
    )?;
    Ok(LoadedTypeSupport { _lib: lib, handle })
}

pub fn header_typesupport() -> Result<LoadedTypeSupport> {
    let lib = load_introspection_c_lib("std_msgs")?;
    let handle = load_symbol(
        &lib,
        "rosidl_typesupport_introspection_c__get_message_type_support_handle__std_msgs__msg__Header",
    )?;
    Ok(LoadedTypeSupport { _lib: lib, handle })
}

pub fn time_typesupport() -> Result<LoadedTypeSupport> {
    let lib = load_introspection_c_lib("builtin_interfaces")?;
    let handle = load_symbol(
        &lib,
        "rosidl_typesupport_introspection_c__get_message_type_support_handle__builtin_interfaces__msg__Time",
    )?;
    Ok(LoadedTypeSupport { _lib: lib, handle })
}

pub fn init_tfmessage(msg: &mut sys::tf2_msgs__msg__TFMessage) {
    unsafe {
        sys::tf2_msgs__msg__TFMessage__init(msg);
    }
}

pub fn fini_tfmessage(msg: &mut sys::tf2_msgs__msg__TFMessage) {
    unsafe {
        sys::tf2_msgs__msg__TFMessage__fini(msg);
    }
}
