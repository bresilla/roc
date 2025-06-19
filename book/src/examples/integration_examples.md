# Integration Examples

This chapter demonstrates how to integrate the `roc` tool into larger systems, automation workflows, and monitoring solutions.

## CI/CD Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/ros2-integration-test.yml
name: ROS 2 Integration Test

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  ros2-integration:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup ROS 2
      uses: ros-tooling/setup-ros@v0.6
      with:
        required-ros-distributions: jazzy
    
    - name: Build roc tool
      run: |
        source /opt/ros/jazzy/setup.bash
        cargo build --release
    
    - name: Start test nodes
      run: |
        source /opt/ros/jazzy/setup.bash
        ros2 run demo_nodes_cpp talker &
        ros2 run demo_nodes_cpp listener &
        sleep 5  # Allow nodes to start
      
    - name: Run integration tests
      run: |
        source /opt/ros/jazzy/setup.bash
        ./target/release/roc topic list
        ./target/release/roc topic info /chatter --verbose
        
        # Verify expected topics exist
        topics=$(./target/release/roc topic list)
        echo "$topics" | grep -q "/chatter" || exit 1
        echo "$topics" | grep -q "/rosout" || exit 1
        
        # Verify topic has publishers
        info=$(./target/release/roc topic info /chatter)
        echo "$info" | grep -q "Publishers: [1-9]" || exit 1
```

### GitLab CI Pipeline

```yaml
# .gitlab-ci.yml
stages:
  - build
  - test
  - integration

variables:
  ROS_DISTRO: jazzy

build:
  stage: build
  image: ros:jazzy
  script:
    - apt-get update && apt-get install -y cargo
    - cargo build --release
  artifacts:
    paths:
      - target/release/roc
    expire_in: 1 hour

integration_test:
  stage: integration
  image: ros:jazzy
  needs: ["build"]
  script:
    - source /opt/ros/jazzy/setup.bash
    - ./scripts/integration_test.sh
  artifacts:
    reports:
      junit: test_results.xml
```

## Docker Integration

### ROS 2 Development Container

```dockerfile
# Dockerfile.ros2-dev
FROM ros:jazzy

# Install Rust and build tools
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    git \
    && rm -rf /var/lib/apt/lists/*

# Copy and build roc
COPY . /workspace/roc
WORKDIR /workspace/roc
RUN cargo build --release

# Install roc tool
RUN cp target/release/roc /usr/local/bin/

# Setup entrypoint
COPY docker/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
CMD ["bash"]
```

```bash
#!/bin/bash
# docker/entrypoint.sh
source /opt/ros/jazzy/setup.bash
exec "$@"
```

### Docker Compose for Testing

```yaml
# docker-compose.yml
version: '3.8'

services:
  ros2-master:
    build:
      context: .
      dockerfile: Dockerfile.ros2-dev
    command: ros2 run demo_nodes_cpp talker
    environment:
      - ROS_DOMAIN_ID=0
    networks:
      - ros2-net

  ros2-monitor:
    build:
      context: .
      dockerfile: Dockerfile.ros2-dev
    command: |
      bash -c "
        sleep 5
        while true; do
          echo '=== Topic Monitor ==='
          roc topic list
          roc topic info /chatter --verbose
          sleep 30
        done
      "
    environment:
      - ROS_DOMAIN_ID=0
    networks:
      - ros2-net
    depends_on:
      - ros2-master

networks:
  ros2-net:
    driver: bridge
```

## Monitoring and Alerting

### Prometheus Integration

```python
#!/usr/bin/env python3
# prometheus_exporter.py - Export roc metrics to Prometheus

import subprocess
import time
import re
from prometheus_client import start_http_server, Gauge, Info
import json

# Prometheus metrics
topic_count = Gauge('ros2_topic_count', 'Number of ROS 2 topics')
topic_publishers = Gauge('ros2_topic_publishers', 'Number of publishers per topic', ['topic_name'])
topic_subscribers = Gauge('ros2_topic_subscribers', 'Number of subscribers per topic', ['topic_name'])
topic_info = Info('ros2_topic_info', 'Topic information', ['topic_name'])

def get_topic_list():
    """Get list of topics using roc tool."""
    try:
        result = subprocess.run(['roc', 'topic', 'list'], 
                              capture_output=True, text=True, timeout=10)
        if result.returncode == 0:
            return [line.strip() for line in result.stdout.strip().split('\n') if line.strip()]
        return []
    except Exception as e:
        print(f"Error getting topic list: {e}")
        return []

def get_topic_info(topic_name):
    """Get detailed topic information."""
    try:
        result = subprocess.run(['roc', 'topic', 'info', topic_name, '--verbose'], 
                              capture_output=True, text=True, timeout=10)
        if result.returncode == 0:
            return parse_topic_info(result.stdout)
        return None
    except Exception as e:
        print(f"Error getting info for {topic_name}: {e}")
        return None

def parse_topic_info(info_text):
    """Parse topic info output."""
    info = {}
    
    # Extract basic info
    type_match = re.search(r'Type: (.+)', info_text)
    if type_match:
        info['type'] = type_match.group(1)
    
    pub_match = re.search(r'Publishers: (\d+)', info_text)
    if pub_match:
        info['publishers'] = int(pub_match.group(1))
    
    sub_match = re.search(r'Subscribers: (\d+)', info_text)
    if sub_match:
        info['subscribers'] = int(sub_match.group(1))
    
    return info

def update_metrics():
    """Update Prometheus metrics."""
    topics = get_topic_list()
    topic_count.set(len(topics))
    
    for topic in topics:
        info = get_topic_info(topic)
        if info:
            topic_publishers.labels(topic_name=topic).set(info.get('publishers', 0))
            topic_subscribers.labels(topic_name=topic).set(info.get('subscribers', 0))
            topic_info.labels(topic_name=topic).info({
                'type': info.get('type', 'unknown'),
                'publishers': str(info.get('publishers', 0)),
                'subscribers': str(info.get('subscribers', 0))
            })

def main():
    # Start Prometheus metrics server
    start_http_server(8000)
    print("Prometheus exporter started on port 8000")
    
    while True:
        try:
            update_metrics()
            time.sleep(30)  # Update every 30 seconds
        except KeyboardInterrupt:
            break
        except Exception as e:
            print(f"Error updating metrics: {e}")
            time.sleep(5)

if __name__ == '__main__':
    main()
```

### Grafana Dashboard Configuration

```json
{
  "dashboard": {
    "title": "ROS 2 Topic Monitor",
    "panels": [
      {
        "title": "Topic Count",
        "type": "stat",
        "targets": [
          {
            "expr": "ros2_topic_count",
            "legendFormat": "Topics"
          }
        ]
      },
      {
        "title": "Publishers per Topic",
        "type": "graph",
        "targets": [
          {
            "expr": "ros2_topic_publishers",
            "legendFormat": "{{topic_name}}"
          }
        ]
      },
      {
        "title": "Subscribers per Topic",
        "type": "graph",
        "targets": [
          {
            "expr": "ros2_topic_subscribers",
            "legendFormat": "{{topic_name}}"
          }
        ]
      }
    ]
  }
}
```

### Alerting Rules

```yaml
# alerting_rules.yml
groups:
  - name: ros2_alerts
    rules:
      - alert: NoTopicsFound
        expr: ros2_topic_count == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "No ROS 2 topics found"
          description: "The ROS 2 system appears to be down - no topics detected"

      - alert: TopicNoPublishers
        expr: ros2_topic_publishers{topic_name!="/parameter_events"} == 0
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Topic {{ $labels.topic_name }} has no publishers"
          description: "Topic {{ $labels.topic_name }} has no active publishers"

      - alert: CriticalTopicMissing
        expr: absent(ros2_topic_publishers{topic_name="/rosout"})
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Critical topic /rosout is missing"
          description: "The /rosout topic is not available"
```

## Python Integration

### ROS 2 Node Integration

```python
#!/usr/bin/env python3
# ros2_monitor_node.py - ROS 2 node that uses roc for monitoring

import rclpy
from rclpy.node import Node
from std_msgs.msg import String
import subprocess
import json
import threading
import time

class TopicMonitorNode(Node):
    def __init__(self):
        super().__init__('topic_monitor')
        
        # Publisher for monitoring results
        self.publisher = self.create_publisher(String, '/topic_monitor/status', 10)
        
        # Timer for periodic monitoring
        self.timer = self.create_timer(10.0, self.monitor_callback)
        
        self.get_logger().info('Topic monitor node started')
    
    def get_topic_stats(self):
        """Get topic statistics using roc tool."""
        try:
            # Get topic list
            result = subprocess.run(['roc', 'topic', 'list'], 
                                  capture_output=True, text=True, timeout=5)
            if result.returncode != 0:
                return None
            
            topics = [line.strip() for line in result.stdout.strip().split('\n') if line.strip()]
            
            stats = {
                'timestamp': time.time(),
                'topic_count': len(topics),
                'topics': {}
            }
            
            # Get info for each topic
            for topic in topics[:10]:  # Limit to first 10 topics
                info_result = subprocess.run(['roc', 'topic', 'info', topic], 
                                           capture_output=True, text=True, timeout=5)
                if info_result.returncode == 0:
                    # Parse the output
                    lines = info_result.stdout.strip().split('\n')
                    topic_info = {}
                    for line in lines:
                        if line.startswith('Type:'):
                            topic_info['type'] = line.split(':', 1)[1].strip()
                        elif line.startswith('Publishers:'):
                            topic_info['publishers'] = int(line.split(':', 1)[1].strip())
                        elif line.startswith('Subscribers:'):
                            topic_info['subscribers'] = int(line.split(':', 1)[1].strip())
                    
                    stats['topics'][topic] = topic_info
            
            return stats
            
        except Exception as e:
            self.get_logger().error(f'Error getting topic stats: {e}')
            return None
    
    def monitor_callback(self):
        """Periodic monitoring callback."""
        stats = self.get_topic_stats()
        if stats:
            # Publish stats as JSON
            msg = String()
            msg.data = json.dumps(stats)
            self.publisher.publish(msg)
            
            # Log summary
            self.get_logger().info(f'Monitoring: {stats["topic_count"]} topics found')
        else:
            self.get_logger().warn('Failed to get topic statistics')

def main(args=None):
    rclpy.init(args=args)
    node = TopicMonitorNode()
    
    try:
        rclpy.spin(node)
    except KeyboardInterrupt:
        pass
    finally:
        node.destroy_node()
        rclpy.shutdown()

if __name__ == '__main__':
    main()
```

## Shell Integration

### Bash Completion

```bash
# roc_completion.bash - Bash completion for roc tool

_roc_completion() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    # Top-level commands
    local commands="topic help"
    
    # Topic subcommands
    local topic_commands="list info"
    
    case ${COMP_CWORD} in
        1)
            COMPREPLY=($(compgen -W "${commands}" -- ${cur}))
            return 0
            ;;
        2)
            case ${prev} in
                topic)
                    COMPREPLY=($(compgen -W "${topic_commands}" -- ${cur}))
                    return 0
                    ;;
            esac
            ;;
        3)
            case ${COMP_WORDS[1]} in
                topic)
                    case ${prev} in
                        info)
                            # Complete with available topics
                            local topics=$(roc topic list 2>/dev/null)
                            COMPREPLY=($(compgen -W "${topics}" -- ${cur}))
                            return 0
                            ;;
                    esac
                    ;;
            esac
            ;;
        4)
            case ${COMP_WORDS[1]} in
                topic)
                    case ${COMP_WORDS[2]} in
                        info)
                            COMPREPLY=($(compgen -W "--verbose" -- ${cur}))
                            return 0
                            ;;
                    esac
                    ;;
            esac
            ;;
    esac
    
    return 0
}

complete -F _roc_completion roc
```

### Zsh Integration

```zsh
# roc_completion.zsh - Zsh completion for roc tool

#compdef roc

_roc() {
    local context state line
    
    _arguments \
        '1: :->command' \
        '*: :->args'
    
    case $state in
        command)
            _values 'commands' \
                'topic[Topic operations]' \
                'help[Show help]'
            ;;
        args)
            case $line[1] in
                topic)
                    _roc_topic
                    ;;
            esac
            ;;
    esac
}

_roc_topic() {
    local context state line
    
    _arguments \
        '1: :->subcommand' \
        '*: :->args'
    
    case $state in
        subcommand)
            _values 'topic subcommands' \
                'list[List all topics]' \
                'info[Show topic information]'
            ;;
        args)
            case $line[1] in
                info)
                    _roc_topic_info
                    ;;
            esac
            ;;
    esac
}

_roc_topic_info() {
    local context state line
    
    _arguments \
        '1: :->topic_name' \
        '2: :->options'
    
    case $state in
        topic_name)
            # Get available topics
            local topics
            topics=(${(f)"$(roc topic list 2>/dev/null)"})
            _describe 'topics' topics
            ;;
        options)
            _values 'options' \
                '--verbose[Show detailed information]'
            ;;
    esac
}

_roc "$@"
```

## Systemd Service Integration

### Service Configuration

```ini
# /etc/systemd/system/ros2-monitor.service
[Unit]
Description=ROS 2 Topic Monitor
After=network.target
Requires=network.target

[Service]
Type=simple
User=ros
Group=ros
WorkingDirectory=/home/ros
Environment=ROS_DOMAIN_ID=0
Environment=RMW_IMPLEMENTATION=rmw_cyclone_cpp
ExecStart=/home/ros/monitoring/monitor_service.sh
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

```bash
#!/bin/bash
# /home/ros/monitoring/monitor_service.sh

source /opt/ros/jazzy/setup.bash

while true; do
    echo "$(date): Starting monitoring cycle"
    
    # Check if ROS 2 system is healthy
    if roc topic list > /dev/null 2>&1; then
        echo "$(date): ROS 2 system healthy"
        
        # Generate monitoring report
        {
            echo "=== ROS 2 System Status ==="
            echo "Timestamp: $(date)"
            echo "Topics found: $(roc topic list | wc -l)"
            echo
            
            # Check critical topics
            for topic in "/rosout" "/parameter_events"; do
                if roc topic info "$topic" > /dev/null 2>&1; then
                    echo "✓ $topic: OK"
                else
                    echo "✗ $topic: MISSING"
                fi
            done
        } > /var/log/ros2-monitor.log
        
    else
        echo "$(date): ROS 2 system appears down"
        echo "$(date): ROS 2 system down" >> /var/log/ros2-monitor.log
    fi
    
    sleep 60
done
```

This completes the integration examples. Let me now create a comprehensive command reference guide.
