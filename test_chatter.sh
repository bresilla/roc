#!/bin/bash

echo "=== Testing improved /chatter topic handling ==="
echo

echo "1. Starting ros2 publisher for /chatter..."
ros2 topic pub /chatter std_msgs/msg/String "data: 'Hello from ROS 2!'" --rate 1 &
PUBLISHER_PID=$!

echo "2. Waiting 3 seconds for topic discovery..."
sleep 3

echo "3. Testing topic kind command (improved discovery)..."
cd /doc/code/roc && ./target/debug/roc topic kind /chatter

echo "4. Testing topic find command..."
cd /doc/code/roc && ./target/debug/roc topic find std_msgs/msg/String

echo "5. Testing topic echo --once (improved waiting)..."
cd /doc/code/roc && ./target/debug/roc topic echo /chatter --once

echo "6. Testing topic bw for 5 seconds (improved discovery)..."
cd /doc/code/roc && timeout 5 ./target/debug/roc topic bw /chatter

echo
echo "7. Cleaning up..."
kill $PUBLISHER_PID 2>/dev/null

echo "=== Test completed ==="
