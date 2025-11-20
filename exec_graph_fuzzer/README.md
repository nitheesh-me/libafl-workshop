# Execution Graph Fuzzer

This fuzzer demonstrates tracking execution graphs (verified axioms) during fuzzing and reporting crashes with their execution path context.

## Overview

The execution graph fuzzer:
- **Tracks execution graphs** as verified axioms - each unique path through the program is considered a verified axiom
- **Reports crashes** with detailed context showing the exact execution path taken to reach the crash
- **Uses custom feedback** to identify interesting new execution paths
- **Monitors stderr** to capture execution graph information from the target program

## Key Components

### 1. Execution Graph Tracking

The fuzzer uses a custom `ExecutionGraphFeedback` that:
- Parses execution graphs from the target's stderr output
- Maintains a set of all verified execution graphs (axioms)
- Identifies new, previously unexplored execution paths
- Reports each new axiom discovered during fuzzing

### 2. Enhanced Crash Reporting

The `ExecutionGraphCrashFeedback` extends standard crash detection to:
- Capture the execution path that led to the crash
- Display the complete execution graph at the time of crash
- Include crash messages from the target program
- Save crashes with full context to the solutions corpus

### 3. Target Program

The target program (`../exec_graph_fuzzer_target/target.c`):
- Emits execution node information to stderr as it runs
- Prints the complete execution graph on exit
- Contains multiple branching paths forming a complex execution graph
- Has hidden crash conditions deep in specific execution paths

## Building

```bash
# Build the target program
cd ../exec_graph_fuzzer_target
make

# Build the fuzzer
cd ../exec_graph_fuzzer
cargo build --release
```

## Running

```bash
cargo run --release
```

The fuzzer will:
1. Load initial corpus seeds
2. Start exploring the input space
3. Report each new execution graph (axiom) discovered
4. Report any crashes with their execution path
5. Save crashing inputs to the `./solutions/` directory

## Example Output

When discovering new execution paths:
```
[+] New execution graph (verified axiom): 0->2->3->4->5->6->7->8
    Input: "FUZZ!X" (len: 6)
    Total verified axioms: 10
```

When finding a crash:
```
[!] CRASH DETECTED!
[!] Execution path to crash: 0->2->3->4->5->6->7->8
[!] CRASH: Found the secret path!
[!] Crashing input: "FUZZ!X" (len: 6)
[!] Crash saved to solutions corpus
```

## Understanding Execution Graphs

An execution graph represents the sequence of program states (nodes) traversed during execution:
- Each node represents a decision point or program state
- Edges represent transitions between states
- The complete graph shows the path taken from start to finish
- Each unique graph is a "verified axiom" - a proven path through the program

For example, the graph `0->2->3->4->5->6->7->8` means:
- Started at node 0 (entry point)
- Took the path through nodes 2, 3, 4, 5, 6, 7
- Reached node 8 (the crash point)

## Implementation Details

### Custom Feedback Implementation

The `ExecutionGraphFeedback` implements LibAFL's `Feedback` trait to:
- Extract execution graph data from the `StdErrObserver`
- Parse the graph format (`GRAPH:node->node->node`)
- Compare against previously seen graphs
- Return `true` when a new graph is discovered

### Custom Crash Feedback

The `ExecutionGraphCrashFeedback` wraps LibAFL's standard `CrashFeedback` and adds:
- Execution graph context extraction
- Enhanced crash reporting with path information
- Integration with the stderr observer for context

## Files

- `src/main.rs` - Main fuzzer implementation with custom feedback
- `corpus/` - Initial seed inputs for fuzzing
- `solutions/` - Discovered crashing inputs (created during fuzzing)
- `../exec_graph_fuzzer_target/` - Target program that demonstrates execution graphs

## Notes

- The fuzzer runs in release mode for better performance
- Execution graphs are tracked via stderr output from the target
- Each unique execution path is considered a separate verified axiom
- Crashes are reported with full execution context for easier debugging
