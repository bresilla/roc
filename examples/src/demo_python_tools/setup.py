from setuptools import find_packages, setup

package_name = "demo_python_tools"

setup(
    name=package_name,
    version="0.1.0",
    packages=find_packages(exclude=["test"]),
    data_files=[
        ("share/ament_index/resource_index/packages", ["resource/" + package_name]),
        ("share/" + package_name, ["package.xml"]),
    ],
    install_requires=["setuptools"],
    zip_safe=True,
    maintainer="ROC Examples",
    maintainer_email="examples@example.com",
    description="Python example package with a ROS node and testable CLI helpers.",
    license="Apache-2.0",
    tests_require=["pytest"],
    entry_points={
        "console_scripts": [
            "report_node = demo_python_tools.report_node:main",
            "metrics_cli = demo_python_tools.metrics_cli:main",
        ],
    },
)
