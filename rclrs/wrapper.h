#ifndef WRAPPER_H
#define WRAPPER_H

// Very minimal wrapper - just include the basic RCL types
#include "rcl/rcl/allocator.h"
#include "rcl/rcl/context.h"
#include "rcl/rcl/graph.h"
#include "rcl/rcl/init.h"
#include "rcl/rcl/init_options.h"
#include "rcl/rcl/node.h"
#include "rcl/rcl/publisher.h"
#include "rcl/rcl/subscription.h"

// Serialization support (fixes dependency issues)
#include "rosidl_dynamic_typesupport/api/serialization_support.h"
#include "rosidl_runtime_c/message_type_support_struct.h"

// Dynamic type support headers (C only - avoiding C++ headers for now)
#include "rosidl_typesupport_c/type_support_map.h"
#include "rosidl_typesupport_c/identifier.h"
#include "rosidl_typesupport_introspection_c/message_introspection.h"
#include "rosidl_typesupport_introspection_c/identifier.h"

// Message serialization headers
#include "rmw/rmw/serialized_message.h"

// Type description and hash headers
#include "rosidl_runtime_c/type_description_utils.h"
#include "rosidl_runtime_c/type_hash.h"

// YAML parameter parsing headers  
#include "rcl_yaml_param_parser/parser.h"
#include "rcutils/rcutils/types/string_array.h"

// Basic RMW headers
#include "rmw/rmw/allocators.h"
#include "rmw/rmw/event_callback_type.h"
#include "rmw/rmw/init.h"
#include "rmw/rmw/init_options.h"
#include "rmw/rmw/ret_types.h"
#include "rmw/rmw/rmw.h"
#include "rmw/rmw/types.h"

#endif // WRAPPER_H