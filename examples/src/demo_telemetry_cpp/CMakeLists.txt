cmake_minimum_required(VERSION 3.8)
project(demo_telemetry_cpp)

if(CMAKE_COMPILER_IS_GNUCXX OR CMAKE_CXX_COMPILER_ID MATCHES "Clang")
  add_compile_options(-Wall -Wextra -Wpedantic)
endif()

find_package(ament_cmake REQUIRED)
find_package(demo_math_cpp REQUIRED)
find_package(rclcpp REQUIRED)
find_package(std_msgs REQUIRED)

add_library(demo_telemetry_support
  src/telemetry_formatter.cpp
)
target_include_directories(demo_telemetry_support
  PUBLIC
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:include>
    ${demo_math_cpp_INCLUDE_DIRS}
)
target_link_libraries(demo_telemetry_support ${demo_math_cpp_LIBRARIES})
ament_target_dependencies(demo_telemetry_support rclcpp std_msgs)

add_executable(telemetry_node src/telemetry_node.cpp)
target_link_libraries(telemetry_node demo_telemetry_support)
ament_target_dependencies(telemetry_node rclcpp std_msgs)

add_executable(telemetry_monitor_node src/telemetry_monitor_node.cpp)
target_link_libraries(telemetry_monitor_node demo_telemetry_support)
ament_target_dependencies(telemetry_monitor_node rclcpp std_msgs)

install(
  TARGETS demo_telemetry_support telemetry_node telemetry_monitor_node
  LIBRARY DESTINATION lib
  ARCHIVE DESTINATION lib
  RUNTIME DESTINATION lib/${PROJECT_NAME}
)

install(
  DIRECTORY include/
  DESTINATION include
)

install(
  DIRECTORY config/
  DESTINATION share/${PROJECT_NAME}/config
)

ament_export_include_directories(include)
ament_export_dependencies(demo_math_cpp rclcpp std_msgs)

ament_package()
