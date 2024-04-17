#!/usr/bin/env bash

set -e

# Get output file path from first argument
output=$1
# Get content from secend argument
if [ -z "$2" ]; then
	content=""
else
	content=$1
	output=$2
fi

# Write content to output file
echo -n "$content" >>"$output"
