#!/usr/bin/env sh

# Wait for 5 seconds for simulate normal viewer boot time
sleep 5
# Get file path from first argument
file_path=$1
# Cat file to stdout
cat "$file_path"
