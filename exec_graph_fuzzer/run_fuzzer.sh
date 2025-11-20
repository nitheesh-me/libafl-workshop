#!/bin/bash

# Script to build and run the execution graph fuzzer

set -e

# Build the target
echo "Building target program..."
cd ../exec_graph_fuzzer_target
make

# Build the fuzzer
echo "Building fuzzer..."
cd ../exec_graph_fuzzer
cargo build --release

# Run the fuzzer
echo "Starting fuzzer..."
echo "Press Ctrl+C to stop"
./target/release/exec_graph_fuzzer
