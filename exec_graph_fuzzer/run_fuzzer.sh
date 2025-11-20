#!/bin/bash

# Script to build and run the execution graph fuzzer

set -e

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Build the target
echo "Building target program..."
cd "$SCRIPT_DIR/../exec_graph_fuzzer_target"
make

# Build the fuzzer
echo "Building fuzzer..."
cd "$SCRIPT_DIR"
cargo build --release

# Run the fuzzer
echo "Starting fuzzer..."
echo "Press Ctrl+C to stop"
./target/release/exec_graph_fuzzer
