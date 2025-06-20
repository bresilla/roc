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

// Basic RMW headers
#include "rmw/rmw/allocators.h"
#include "rmw/rmw/event_callback_type.h"
#include "rmw/rmw/init.h"
#include "rmw/rmw/init_options.h"
#include "rmw/rmw/ret_types.h"
#include "rmw/rmw/rmw.h"
#include "rmw/rmw/types.h"

#endif // WRAPPER_H