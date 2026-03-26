from __future__ import annotations

import argparse

import rclpy
from rclpy.node import Node
from std_msgs.msg import String

from demo_python_tools.metrics_cli import build_report, demo_series_names


class ReportNode(Node):
    def __init__(self, series_name: str, topic_name: str) -> None:
        super().__init__("report_node")
        self._publisher = self.create_publisher(String, topic_name, 10)
        self._series_name = series_name
        self._cursor = 1
        self._window = 1
        self._timer = self.create_timer(1.0, self._publish_report)

    def _publish_report(self) -> None:
        report = build_report(self._series_name, self._window)
        self._window += 1
        message = String()
        message.data = report.render()
        self._publisher.publish(message)
        self.get_logger().info(f"Published report #{self._cursor}: {message.data}")
        self._cursor += 1
        if self._window > report.count:
            self._window = 1


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run the demo Python report node.")
    parser.add_argument("--series", default="steady", choices=demo_series_names())
    parser.add_argument("--topic", default="demo/python_report")
    return parser.parse_args(argv)


def main(args: list[str] | None = None) -> None:
    parsed = parse_args(args)
    rclpy.init(args=args)
    node = ReportNode(parsed.series, parsed.topic)
    try:
        rclpy.spin(node)
    finally:
        node.destroy_node()
        rclpy.shutdown()
