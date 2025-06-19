# Advanced Usage Examples

This chapter covers advanced usage patterns and complex scenarios for the `roc` tool.

## Advanced Topic Analysis

### QoS Profile Comparison

When debugging communication issues, compare QoS profiles between publishers and subscribers:

```bash
#!/bin/bash
# qos_compare.sh - Compare QoS profiles for a topic

TOPIC="$1"
if [ -z "$TOPIC" ]; then
    echo "Usage: $0 <topic_name>"
    exit 1
fi

echo "=== QoS Analysis for $TOPIC ==="
roc topic info "$TOPIC" --verbose | grep -A 10 "QoS Profile:"
```

### Multi-Domain Discovery

Working across multiple ROS domains:

```bash
#!/bin/bash
# multi_domain_scan.sh - Scan topics across multiple domains

for domain in {0..10}; do
    export ROS_DOMAIN_ID=$domain
    echo "=== Domain $domain ==="
    topics=$(roc topic list 2>/dev/null)
    if [ -n "$topics" ]; then
        echo "$topics"
        echo "Topic count: $(echo "$topics" | wc -l)"
    else
        echo "No topics found"
    fi
    echo
done
```

## Performance Monitoring

### Topic Discovery Timing

Measure topic discovery performance:

```bash
#!/bin/bash
# discovery_benchmark.sh - Benchmark topic discovery

echo "Benchmarking topic discovery..."

echo "roc topic list:"
time roc topic list > /dev/null

echo "ros2 topic list:"
time ros2 topic list > /dev/null

echo "roc topic info (verbose):"
TOPIC=$(roc topic list | head -1)
if [ -n "$TOPIC" ]; then
    time roc topic info "$TOPIC" --verbose > /dev/null
fi
```

### Memory Usage Analysis

Monitor memory usage during large-scale discovery:

```bash
#!/bin/bash
# memory_profile.sh - Profile memory usage

echo "Memory usage during topic discovery:"

# Get baseline memory
baseline=$(ps -o rss= -p $$)
echo "Baseline memory: ${baseline}KB"

# Run topic discovery and monitor memory
(
    while true; do
        ps -o rss= -p $$ 2>/dev/null || break
        sleep 0.1
    done
) &
monitor_pid=$!

# Perform discovery operations
roc topic list > /dev/null
roc topic info /chatter --verbose > /dev/null 2>&1

kill $monitor_pid 2>/dev/null
```

## Integration Patterns

### Continuous Monitoring

Monitor topic health continuously:

```bash
#!/bin/bash
# topic_monitor.sh - Continuous topic monitoring

TOPIC="$1"
INTERVAL="${2:-5}"

if [ -z "$TOPIC" ]; then
    echo "Usage: $0 <topic_name> [interval_seconds]"
    exit 1
fi

echo "Monitoring $TOPIC every ${INTERVAL}s (Ctrl+C to stop)"

while true; do
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "=== $timestamp ==="
    
    # Get current topic info
    info=$(roc topic info "$TOPIC" 2>/dev/null)
    if [ $? -eq 0 ]; then
        echo "$info"
        
        # Extract publisher/subscriber counts
        pub_count=$(echo "$info" | grep "Publishers:" | awk '{print $2}')
        sub_count=$(echo "$info" | grep "Subscribers:" | awk '{print $2}')
        
        echo "Status: $pub_count publishers, $sub_count subscribers"
    else
        echo "Topic not found or error occurred"
    fi
    
    echo
    sleep "$INTERVAL"
done
```

### Automated Health Checks

Create health check scripts for ROS 2 systems:

```bash
#!/bin/bash
# ros2_health_check.sh - Comprehensive ROS 2 system health check

echo "=== ROS 2 System Health Check ==="
echo "Timestamp: $(date)"
echo

# Check critical topics
critical_topics=("/rosout" "/parameter_events")
for topic in "${critical_topics[@]}"; do
    echo "Checking $topic..."
    info=$(roc topic info "$topic" 2>/dev/null)
    if [ $? -eq 0 ]; then
        echo "✓ $topic: OK"
        echo "$info" | grep -E "(Publishers|Subscribers):"
    else
        echo "✗ $topic: MISSING"
    fi
    echo
done

# Check for common issues
echo "=== Potential Issues ==="

# Find topics with no publishers or subscribers
all_topics=$(roc topic list 2>/dev/null)
if [ -n "$all_topics" ]; then
    while IFS= read -r topic; do
        info=$(roc topic info "$topic" 2>/dev/null)
        if echo "$info" | grep -q "Publishers: 0"; then
            echo "⚠ $topic: No publishers"
        fi
        if echo "$info" | grep -q "Subscribers: 0"; then
            echo "⚠ $topic: No subscribers"
        fi
    done <<< "$all_topics"
else
    echo "✗ No topics found - ROS 2 system may be down"
fi
```

## Data Export and Analysis

### JSON Export

Export topic information in structured format:

```bash
#!/bin/bash
# export_topics.sh - Export topic information to JSON

output_file="topics_$(date +%Y%m%d_%H%M%S).json"

echo "Exporting topic information to $output_file..."

echo "{" > "$output_file"
echo '  "timestamp": "'$(date -Iseconds)'",' >> "$output_file"
echo '  "topics": [' >> "$output_file"

topics=$(roc topic list 2>/dev/null)
if [ -n "$topics" ]; then
    first=true
    while IFS= read -r topic; do
        if [ "$first" = true ]; then
            first=false
        else
            echo "    ," >> "$output_file"
        fi
        
        echo "    {" >> "$output_file"
        echo '      "name": "'$topic'",' >> "$output_file"
        
        # Get topic info and parse it
        info=$(roc topic info "$topic" --verbose 2>/dev/null)
        if [ $? -eq 0 ]; then
            type=$(echo "$info" | grep "Type:" | cut -d' ' -f2-)
            pub_count=$(echo "$info" | grep "Publishers:" | awk '{print $2}')
            sub_count=$(echo "$info" | grep "Subscribers:" | awk '{print $2}')
            
            echo '      "type": "'$type'",' >> "$output_file"
            echo '      "publishers": '$pub_count',' >> "$output_file"
            echo '      "subscribers": '$sub_count >> "$output_file"
        else
            echo '      "error": "Failed to get topic info"' >> "$output_file"
        fi
        
        echo -n "    }" >> "$output_file"
    done <<< "$topics"
    echo >> "$output_file"
fi

echo "  ]" >> "$output_file"
echo "}" >> "$output_file"

echo "Export complete: $output_file"
```

### CSV Export for Analysis

```bash
#!/bin/bash
# export_csv.sh - Export topic data to CSV for analysis

output_file="topics_$(date +%Y%m%d_%H%M%S).csv"

echo "Exporting topic information to $output_file..."

# CSV header
echo "Timestamp,Topic,Type,Publishers,Subscribers" > "$output_file"

topics=$(roc topic list 2>/dev/null)
if [ -n "$topics" ]; then
    while IFS= read -r topic; do
        timestamp=$(date -Iseconds)
        info=$(roc topic info "$topic" 2>/dev/null)
        
        if [ $? -eq 0 ]; then
            type=$(echo "$info" | grep "Type:" | cut -d' ' -f2- | tr ',' '_')
            pub_count=$(echo "$info" | grep "Publishers:" | awk '{print $2}')
            sub_count=$(echo "$info" | grep "Subscribers:" | awk '{print $2}')
            
            echo "$timestamp,$topic,$type,$pub_count,$sub_count" >> "$output_file"
        else
            echo "$timestamp,$topic,ERROR,0,0" >> "$output_file"
        fi
    done <<< "$topics"
fi

echo "Export complete: $output_file"
echo "Analyze with: python3 -c \"import pandas as pd; df=pd.read_csv('$output_file'); print(df.describe())\""
```

## Custom RMW Configuration

### Testing Different RMW Implementations

```bash
#!/bin/bash
# rmw_comparison.sh - Compare performance across RMW implementations

rmw_implementations=(
    "rmw_cyclone_cpp"
    "rmw_fastrtps_cpp"
    "rmw_connext_cpp"
)

for rmw in "${rmw_implementations[@]}"; do
    echo "=== Testing with $rmw ==="
    export RMW_IMPLEMENTATION="$rmw"
    
    # Test basic discovery
    echo "Topic discovery test:"
    time roc topic list > /dev/null 2>&1
    
    if [ $? -eq 0 ]; then
        topic_count=$(roc topic list 2>/dev/null | wc -l)
        echo "Success: Found $topic_count topics"
        
        # Test detailed info
        first_topic=$(roc topic list 2>/dev/null | head -1)
        if [ -n "$first_topic" ]; then
            echo "Detailed info test:"
            time roc topic info "$first_topic" --verbose > /dev/null 2>&1
        fi
    else
        echo "Failed: $rmw not available or error occurred"
    fi
    echo
done
```

## Error Handling and Debugging

### Verbose Debugging

Enable detailed debugging for troubleshooting:

```bash
#!/bin/bash
# debug_roc.sh - Debug roc tool issues

echo "=== ROS 2 Environment ==="
printenv | grep ROS | sort

echo -e "\n=== RMW Implementation ==="
echo "RMW_IMPLEMENTATION: ${RMW_IMPLEMENTATION:-default}"

echo -e "\n=== System Info ==="
echo "OS: $(uname -a)"
echo "User: $(whoami)"
echo "Groups: $(groups)"

echo -e "\n=== ROS 2 Process Check ==="
ps aux | grep -E "(ros|dds)" | grep -v grep

echo -e "\n=== Network Interfaces ==="
ip addr show | grep -E "(inet|UP|DOWN)"

echo -e "\n=== ROC Tool Test ==="
echo "Testing roc topic list..."
if roc topic list; then
    echo "✓ Basic functionality works"
    
    echo -e "\nTesting verbose info..."
    first_topic=$(roc topic list | head -1)
    if [ -n "$first_topic" ]; then
        echo "Testing with topic: $first_topic"
        roc topic info "$first_topic" --verbose
    fi
else
    echo "✗ Basic functionality failed"
    echo "Exit code: $?"
fi
```

## Performance Optimization

### Batch Operations

Optimize for scenarios with many topics:

```bash
#!/bin/bash
# batch_optimize.sh - Optimized batch topic analysis

# Get all topics once
topics=($(roc topic list 2>/dev/null))
topic_count=${#topics[@]}

echo "Found $topic_count topics"

if [ $topic_count -eq 0 ]; then
    echo "No topics found"
    exit 1
fi

# Process in batches to avoid overwhelming the system
batch_size=10
batch_count=$(( (topic_count + batch_size - 1) / batch_size ))

echo "Processing in $batch_count batches of $batch_size..."

for ((batch=0; batch<batch_count; batch++)); do
    start=$((batch * batch_size))
    end=$((start + batch_size))
    
    echo "Batch $((batch+1))/$batch_count (topics $start-$((end-1)))"
    
    for ((i=start; i<end && i<topic_count; i++)); do
        topic="${topics[i]}"
        echo "  Processing: $topic"
        roc topic info "$topic" > /dev/null 2>&1
    done
    
    # Small delay between batches
    sleep 0.1
done

echo "Batch processing complete"
```

This completes the advanced usage examples. Next, let me create a command reference guide.
