cmake_minimum_required(VERSION 3.8)
project(demo_math_cpp)

if(CMAKE_COMPILER_IS_GNUCXX OR CMAKE_CXX_COMPILER_ID MATCHES "Clang")
  add_compile_options(-Wall -Wextra -Wpedantic)
endif()

find_package(ament_cmake REQUIRED)
find_package(rclcpp REQUIRED)
find_package(std_msgs REQUIRED)

add_library(demo_math_pipeline
  src/math_pipeline.cpp
  src/stream_catalog.cpp
)
target_include_directories(demo_math_pipeline
  PUBLIC
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
    $<INSTALL_INTERFACE:include>
)
ament_target_dependencies(demo_math_pipeline rclcpp std_msgs)

add_executable(stats_node src/stats_node.cpp)
target_link_libraries(stats_node demo_math_pipeline)
ament_target_dependencies(stats_node rclcpp std_msgs)

add_executable(catalog_report_node src/catalog_report_node.cpp)
target_link_libraries(catalog_report_node demo_math_pipeline)
ament_target_dependencies(catalog_report_node rclcpp std_msgs)

install(
  TARGETS demo_math_pipeline stats_node catalog_report_node
  EXPORT demo_math_cpp_targets
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
ament_export_libraries(demo_math_pipeline)
ament_export_targets(demo_math_cpp_targets HAS_LIBRARY_TARGET)
ament_export_dependencies(rclcpp std_msgs)

ament_package()
