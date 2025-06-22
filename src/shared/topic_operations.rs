use rclrs::*;
use std::ptr;
use std::ffi::CString;
use anyhow::{Result, anyhow};

use crate::shared::graph_context::RclGraphContext;
use crate::shared::graph_types::{TopicInfo, TopicEndpointInfo, EndpointType};
use crate::shared::qos_profile::QosProfile;
use crate::shared::graph_utils::format_topic_type_hash;

/// Get all topics in the ROS graph using direct RCL API calls
pub fn get_topic_names(context: &RclGraphContext) -> Result<Vec<String>> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }
    
    unsafe {
        let mut allocator = rcutils_get_default_allocator();
        let mut topic_names_and_types = rcl_names_and_types_t { 
            names: rcutils_get_zero_initialized_string_array(),
            types: ptr::null_mut(),
        };
        
        let ret = rcl_get_topic_names_and_types(
            context.node(),
            &mut allocator as *mut _,
            false, // no_demangle
            &mut topic_names_and_types,
        );
        
        if ret != 0 {
            RclGraphContext::reset_error_state();
            return Err(anyhow!("Failed to get topic names: {}", ret));
        }
        
        // Convert the topic names to Vec<String>
        let mut result = Vec::new();
        for i in 0..topic_names_and_types.names.size {
            if !topic_names_and_types.names.data.add(i).is_null() {
                let name_ptr = *topic_names_and_types.names.data.add(i);
                if !name_ptr.is_null() {
                    let name_cstr = std::ffi::CStr::from_ptr(name_ptr);
                    if let Ok(name_str) = name_cstr.to_str() {
                        result.push(name_str.to_string());
                    }
                }
            }
        }
        
        // Clean up
        rcl_names_and_types_fini(&mut topic_names_and_types);
        
        Ok(result)
    }
}

/// Get all topics in the ROS graph with their type information using direct RCL API calls
pub fn get_topics_with_types(context: &RclGraphContext) -> Result<Vec<TopicInfo>> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }
    
    unsafe {
        let mut allocator = rcutils_get_default_allocator();
        let mut topic_names_and_types = rcl_names_and_types_t { 
            names: rcutils_get_zero_initialized_string_array(),
            types: ptr::null_mut(),
        };
        
        let ret = rcl_get_topic_names_and_types(
            context.node(),
            &mut allocator,
            false, // no_demangle
            &mut topic_names_and_types,
        );
        
        if ret != 0 {
            RclGraphContext::reset_error_state();
            return Err(anyhow!("Failed to get topic names and types: {}", ret));
        }
        
        // Convert the topic names and types to Vec<TopicInfo>
        let mut result = Vec::new();
        for i in 0..topic_names_and_types.names.size {
            if !topic_names_and_types.names.data.add(i).is_null() {
                let name_ptr = *topic_names_and_types.names.data.add(i);
                if !name_ptr.is_null() {
                    let name_cstr = std::ffi::CStr::from_ptr(name_ptr);
                    if let Ok(name_str) = name_cstr.to_str() {
                        // Get the types for this topic
                        let mut topic_types = Vec::new();
                        if !topic_names_and_types.types.is_null() {
                            let types_array = topic_names_and_types.types.add(i);
                            if !types_array.is_null() {
                                let types_for_topic = &*types_array;
                                for j in 0..types_for_topic.size {
                                    if !types_for_topic.data.add(j).is_null() {
                                        let type_ptr = *types_for_topic.data.add(j);
                                        if !type_ptr.is_null() {
                                            let type_cstr = std::ffi::CStr::from_ptr(type_ptr);
                                            if let Ok(type_str) = type_cstr.to_str() {
                                                topic_types.push(type_str.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        
                        result.push(TopicInfo {
                            name: name_str.to_string(),
                            types: topic_types,
                        });
                    }
                }
            }
        }
        
        // Clean up
        rcl_names_and_types_fini(&mut topic_names_and_types);
        
        Ok(result)
    }
}

/// Get all topics and their types as tuples
pub fn get_topic_names_and_types(context: &RclGraphContext) -> Result<Vec<(String, String)>> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }
    
    unsafe {
        let mut allocator = rcutils_get_default_allocator();
        let mut topic_names_and_types = rcl_names_and_types_t { 
            names: rcutils_get_zero_initialized_string_array(),
            types: ptr::null_mut(),
        };
        
        let ret = rcl_get_topic_names_and_types(
            context.node(),
            &mut allocator as *mut _,
            false, // no_demangle
            &mut topic_names_and_types,
        );
        
        if ret != 0 {
            RclGraphContext::reset_error_state();
            return Err(anyhow!("Failed to get topic names and types: {}", ret));
        }
        
        // Convert the topics and types to Vec<(String, String)>
        let mut result = Vec::new();
        for i in 0..topic_names_and_types.names.size {
            if !topic_names_and_types.names.data.add(i).is_null() {
                let name_ptr = *topic_names_and_types.names.data.add(i);
                if !name_ptr.is_null() {
                    let name_cstr = std::ffi::CStr::from_ptr(name_ptr);
                    if let Ok(name_str) = name_cstr.to_str() {
                        // Get the corresponding type(s) - there may be multiple types per topic
                        if !topic_names_and_types.types.add(i).is_null() {
                            let types_array = &*topic_names_and_types.types.add(i);
                            for j in 0..types_array.size {
                                if !types_array.data.add(j).is_null() {
                                    let type_ptr = *types_array.data.add(j);
                                    if !type_ptr.is_null() {
                                        let type_cstr = std::ffi::CStr::from_ptr(type_ptr);
                                        if let Ok(type_str) = type_cstr.to_str() {
                                            result.push((name_str.to_string(), type_str.to_string()));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Clean up
        rcl_names_and_types_fini(&mut topic_names_and_types);
        
        Ok(result)
    }
}

/// Count the number of publishers for a given topic
pub fn count_publishers(context: &RclGraphContext, topic_name: &str) -> Result<usize> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }
    
    let topic_name_c = CString::new(topic_name).map_err(|e| anyhow!("Invalid topic name: {}", e))?;
    
    unsafe {
        let mut count: usize = 0;
        let ret = rcl_count_publishers(
            context.node(),
            topic_name_c.as_ptr(),
            &mut count,
        );
        
        if ret != 0 {
            RclGraphContext::reset_error_state();
            return Err(anyhow!("Failed to count publishers for topic '{}': {}", topic_name, ret));
        }
        
        Ok(count)
    }
}

/// Count the number of subscribers for a given topic
pub fn count_subscribers(context: &RclGraphContext, topic_name: &str) -> Result<usize> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }
    
    let topic_name_c = CString::new(topic_name).map_err(|e| anyhow!("Invalid topic name: {}", e))?;
    
    unsafe {
        let mut count: usize = 0;
        let ret = rcl_count_subscribers(
            context.node(),
            topic_name_c.as_ptr(),
            &mut count,
        );
        
        if ret != 0 {
            RclGraphContext::reset_error_state();
            return Err(anyhow!("Failed to count subscribers for topic '{}': {}", topic_name, ret));
        }
        
        Ok(count)
    }
}

/// Get detailed information about all publishers to a topic
pub fn get_publishers_info(context: &RclGraphContext, topic_name: &str) -> Result<Vec<TopicEndpointInfo>> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }
    
    let topic_name_c = CString::new(topic_name).map_err(|e| anyhow!("Invalid topic name: {}", e))?;
    
    unsafe {
        let mut allocator = rcutils_get_default_allocator();
        
        // Initialize the array properly
        let mut publishers_info: rcl_topic_endpoint_info_array_t = std::mem::zeroed();
        publishers_info.size = 0;
        publishers_info.info_array = std::ptr::null_mut();
        
        let ret = rcl_get_publishers_info_by_topic(
            context.node(),
            &mut allocator,
            topic_name_c.as_ptr(),
            false, // no_mangle
            &mut publishers_info,
        );
        
        if ret != 0 {
            RclGraphContext::reset_error_state();
            return Err(anyhow!("Failed to get publishers info for topic '{}': {}", topic_name, ret));
        }
        
        // Convert to our Rust struct
        let mut result = Vec::new();
        for i in 0..publishers_info.size {
            let info = &*(publishers_info.info_array.add(i));
            
            // Extract strings safely
            let node_name = if info.node_name.is_null() {
                "unknown".to_string()
            } else {
                std::ffi::CStr::from_ptr(info.node_name).to_string_lossy().to_string()
            };
            
            let node_namespace = if info.node_namespace.is_null() {
                "/".to_string()
            } else {
                std::ffi::CStr::from_ptr(info.node_namespace).to_string_lossy().to_string()
            };
            
            let topic_type = if info.topic_type.is_null() {
                "unknown".to_string()
            } else {
                std::ffi::CStr::from_ptr(info.topic_type).to_string_lossy().to_string()
            };
            
            // Extract topic type hash
            let topic_type_hash = format_topic_type_hash(&info.topic_type_hash);
            
            // Extract endpoint type
            let endpoint_type = EndpointType::from_rmw(info.endpoint_type);
            
            // Extract GID (Global ID) - it's a fixed-size array in RMW
            let gid = std::slice::from_raw_parts(info.endpoint_gid.as_ptr(), info.endpoint_gid.len()).to_vec();
            
            // Extract QoS profile
            let qos_profile = QosProfile::from_rmw(&info.qos_profile);
            
            result.push(TopicEndpointInfo {
                node_name,
                node_namespace,
                topic_type,
                topic_type_hash,
                endpoint_type,
                gid,
                qos_profile,
            });
        }
        
        // Clean up the array
        rmw_topic_endpoint_info_array_fini(&mut publishers_info, &mut allocator);
        
        Ok(result)
    }
}

/// Get detailed information about all subscribers to a topic
pub fn get_subscribers_info(context: &RclGraphContext, topic_name: &str) -> Result<Vec<TopicEndpointInfo>> {
    if !context.is_valid() {
        return Err(anyhow!("RCL context is not valid"));
    }
    
    let topic_name_c = CString::new(topic_name).map_err(|e| anyhow!("Invalid topic name: {}", e))?;
    
    unsafe {
        let mut allocator = rcutils_get_default_allocator();
        
        // Initialize the array properly
        let mut subscribers_info: rcl_topic_endpoint_info_array_t = std::mem::zeroed();
        subscribers_info.size = 0;
        subscribers_info.info_array = std::ptr::null_mut();
        
        let ret = rcl_get_subscriptions_info_by_topic(
            context.node(),
            &mut allocator,
            topic_name_c.as_ptr(),
            false, // no_mangle
            &mut subscribers_info,
        );
        
        if ret != 0 {
            RclGraphContext::reset_error_state();
            return Err(anyhow!("Failed to get subscribers info for topic '{}': {}", topic_name, ret));
        }
        
        // Convert to our Rust struct
        let mut result = Vec::new();
        for i in 0..subscribers_info.size {
            let info = &*(subscribers_info.info_array.add(i));
            
            // Extract strings safely
            let node_name = if info.node_name.is_null() {
                "unknown".to_string()
            } else {
                std::ffi::CStr::from_ptr(info.node_name).to_string_lossy().to_string()
            };
            
            let node_namespace = if info.node_namespace.is_null() {
                "/".to_string()
            } else {
                std::ffi::CStr::from_ptr(info.node_namespace).to_string_lossy().to_string()
            };
            
            let topic_type = if info.topic_type.is_null() {
                "unknown".to_string()
            } else {
                std::ffi::CStr::from_ptr(info.topic_type).to_string_lossy().to_string()
            };
            
            // Extract topic type hash
            let topic_type_hash = format_topic_type_hash(&info.topic_type_hash);
            
            // Extract endpoint type
            let endpoint_type = EndpointType::from_rmw(info.endpoint_type);
            
            // Extract GID (Global ID)
            let gid = std::slice::from_raw_parts(info.endpoint_gid.as_ptr(), info.endpoint_gid.len()).to_vec();
            
            // Extract QoS profile
            let qos_profile = QosProfile::from_rmw(&info.qos_profile);
            
            result.push(TopicEndpointInfo {
                node_name,
                node_namespace,
                topic_type,
                topic_type_hash,
                endpoint_type,
                gid,
                qos_profile,
            });
        }

        // Clean up the array
        rmw_topic_endpoint_info_array_fini(&mut subscribers_info, &mut allocator);
        
        Ok(result)
    }
}