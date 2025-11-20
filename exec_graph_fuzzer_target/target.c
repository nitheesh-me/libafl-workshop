#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>

// This target demonstrates execution graph exploration
// It has multiple branches that form a graph of execution paths
// The fuzzer should track which paths (axioms) have been verified

// Global state to track execution path
int execution_path[32];
int path_index = 0;

void record_node(int node_id) {
    if (path_index < 32) {
        execution_path[path_index++] = node_id;
        fprintf(stderr, "NODE:%d\n", node_id);
    }
}

void print_execution_graph() {
    fprintf(stderr, "GRAPH:");
    for (int i = 0; i < path_index; i++) {
        fprintf(stderr, "%d", execution_path[i]);
        if (i < path_index - 1) {
            fprintf(stderr, "->");
        }
    }
    fprintf(stderr, "\n");
}

int main(int argc, char **argv) {
    char buffer[256];
    size_t len;
    
    // Read input from stdin
    len = fread(buffer, 1, sizeof(buffer) - 1, stdin);
    buffer[len] = '\0';
    
    // Reset execution path
    path_index = 0;
    
    // Start node
    record_node(0);
    
    // Need at least 4 bytes to explore deep paths
    if (len < 4) {
        record_node(1);
        print_execution_graph();
        return 0;
    }
    
    // Branch based on first byte
    record_node(2);
    if (buffer[0] == 'F') {
        record_node(3);
        
        // Branch based on second byte
        if (buffer[1] == 'U') {
            record_node(4);
            
            // Branch based on third byte
            if (buffer[2] == 'Z') {
                record_node(5);
                
                // Branch based on fourth byte
                if (buffer[3] == 'Z') {
                    record_node(6);
                    
                    // Deep path - check fifth byte
                    if (len >= 5 && buffer[4] == '!') {
                        record_node(7);
                        
                        // Victory condition - but also a crash!
                        if (len >= 6 && buffer[5] == 'X') {
                            record_node(8);
                            print_execution_graph();
                            fprintf(stderr, "CRASH: Found the secret path!\n");
                            // Intentional crash to demonstrate crash reporting
                            abort();
                        } else {
                            record_node(9);
                        }
                    } else {
                        record_node(10);
                    }
                } else {
                    record_node(11);
                }
            } else {
                record_node(12);
            }
        } else {
            record_node(13);
        }
    } else if (buffer[0] == 'A') {
        record_node(14);
        
        if (buffer[1] == 'B') {
            record_node(15);
            
            if (buffer[2] == 'C') {
                record_node(16);
                
                if (buffer[3] == 'D') {
                    record_node(17);
                    
                    // Another deep path
                    if (len >= 5 && buffer[4] == 'E') {
                        record_node(18);
                        
                        if (len >= 6 && buffer[5] == 'F') {
                            record_node(19);
                            print_execution_graph();
                            fprintf(stderr, "CRASH: Found alternative secret path!\n");
                            // Another crash path
                            abort();
                        }
                    } else {
                        record_node(20);
                    }
                }
            }
        }
    } else {
        record_node(21);
    }
    
    print_execution_graph();
    return 0;
}
