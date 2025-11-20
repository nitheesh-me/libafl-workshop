use std::{
    path::PathBuf,
    collections::HashSet,
};

use libafl_bolts::{
    Named,
    tuples::{
        tuple_list,
    },
    current_nanos,
    rands::StdRand,
};

use libafl::{
    Error,
    corpus::{
        inmemory::InMemoryCorpus,
        ondisk::OnDiskCorpus,
    },
    fuzzer::{
        StdFuzzer,
        Fuzzer,
    },
    feedbacks::{
        CrashFeedback,
        Feedback,
    },
    inputs::{
        BytesInput,
        UsesInput,
        HasBytesVec,
    },
    observers::{
        stdio::StdErrObserver,
        ObserversTuple,
    },
    executors::{
        command::CommandExecutor,
        ExitKind,
    },
    monitors::SimpleMonitor,
    mutators::scheduled::{
        StdScheduledMutator,
        havoc_mutations,
    },
    stages::mutational::StdMutationalStage,
    state::{
        StdState,
        State,
    },
    events::{
        simple::SimpleEventManager,
        EventFirer,
    },
    schedulers::QueueScheduler,
};

/// Custom Feedback that tracks execution graphs (verified axioms)
/// An execution graph represents the path taken through the program
/// Each unique path is considered a verified axiom that we've explored
#[derive(Clone, Debug)]
struct ExecutionGraphFeedback {
    /// Set of all verified execution graphs (axioms)
    verified_graphs: HashSet<String>,
    name: String,
    observer_name: String,
}

impl ExecutionGraphFeedback {
    fn new(name: &str, observer_name: &str) -> Self {
        Self {
            verified_graphs: HashSet::new(),
            name: name.to_string(),
            observer_name: observer_name.to_string(),
        }
    }

    /// Parse execution graph from stderr output
    /// Format: "GRAPH:0->2->3->4"
    fn parse_graph(stderr: &[u8]) -> Option<String> {
        let stderr_str = String::from_utf8_lossy(stderr);
        
        // Find the GRAPH: line
        for line in stderr_str.lines() {
            if line.starts_with("GRAPH:") {
                return Some(line[6..].to_string());
            }
        }
        
        None
    }
}

impl<S> Feedback<S> for ExecutionGraphFeedback
where
    S: State + UsesInput,
    S::Input: HasBytesVec,
{
    fn is_interesting<EM, OT>(
        &mut self,
        _state: &mut S,
        _manager: &mut EM,
        input: &S::Input,
        observers: &OT,
        _exit_kind: &ExitKind
    ) -> Result<bool, Error>
       where EM: EventFirer<State = S>,
             OT: ObserversTuple<S>
    {
        let observer = observers.match_name::<StdErrObserver>(&self.observer_name)
            .expect("ExecutionGraphFeedback needs a StdErrObserver");
    
        // Check if we have stderr output
        if let Some(stderr) = &observer.stderr {
            // Parse the execution graph from stderr
            if let Some(graph) = Self::parse_graph(stderr) {
                // Check if this is a new execution graph (axiom)
                if !self.verified_graphs.contains(&graph) {
                    // New axiom verified!
                    self.verified_graphs.insert(graph.clone());
                    
                    let input_str = String::from_utf8_lossy(input.bytes());
                    println!("[+] New execution graph (verified axiom): {}", graph);
                    println!("    Input: {:?} (len: {})", input_str, input.bytes().len());
                    println!("    Total verified axioms: {}", self.verified_graphs.len());
                    
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
}

impl Named for ExecutionGraphFeedback {
    fn name(&self) -> &str {
        &self.name
    }
}

/// Enhanced crash feedback that reports the execution graph when a crash occurs
#[derive(Clone, Debug)]
struct ExecutionGraphCrashFeedback {
    crash_feedback: CrashFeedback,
    observer_name: String,
}

impl ExecutionGraphCrashFeedback {
    fn new(observer_name: &str) -> Self {
        Self {
            crash_feedback: CrashFeedback::new(),
            observer_name: observer_name.to_string(),
        }
    }
}

impl<S> Feedback<S> for ExecutionGraphCrashFeedback
where
    S: State + UsesInput,
    S::Input: HasBytesVec,
{
    fn is_interesting<EM, OT>(
        &mut self,
        state: &mut S,
        manager: &mut EM,
        input: &S::Input,
        observers: &OT,
        exit_kind: &ExitKind
    ) -> Result<bool, Error>
       where EM: EventFirer<State = S>,
             OT: ObserversTuple<S>
    {
        // First check if it's a crash using the standard crash feedback
        let is_crash = self.crash_feedback.is_interesting(state, manager, input, observers, exit_kind)?;
        
        if is_crash {
            // Report the crash with execution graph context
            println!("\n[!] CRASH DETECTED!");
            
            let observer = observers.match_name::<StdErrObserver>(&self.observer_name)
                .expect("ExecutionGraphCrashFeedback needs a StdErrObserver");
            
            if let Some(stderr) = &observer.stderr {
                let stderr_str = String::from_utf8_lossy(stderr);
                
                // Extract and display the execution graph
                if let Some(graph) = ExecutionGraphFeedback::parse_graph(stderr) {
                    println!("[!] Execution path to crash: {}", graph);
                }
                
                // Display crash message if present
                for line in stderr_str.lines() {
                    if line.starts_with("CRASH:") {
                        println!("[!] {}", line);
                    }
                }
            }
            
            let input_str = String::from_utf8_lossy(input.bytes());
            println!("[!] Crashing input: {:?} (len: {})", input_str, input.bytes().len());
            println!("[!] Crash saved to solutions corpus\n");
        }
        
        Ok(is_crash)
    }
}

impl Named for ExecutionGraphCrashFeedback {
    fn name(&self) -> &str {
        self.crash_feedback.name()
    }
}

fn main() {
    env_logger::init();

    println!("=== Execution Graph Fuzzer ===");
    println!("This fuzzer tracks execution graphs as verified axioms");
    println!("and reports crashes with their execution path context.\n");

    // Observer to capture stderr where execution graph is printed
    let observer = StdErrObserver::new("stderr_observer".to_string());
    
    // Our custom feedback tracks execution graphs (verified axioms)
    let mut feedback = ExecutionGraphFeedback::new(
        "exec_graph_feedback",
        observer.name(),
    );

    // Enhanced crash feedback that reports execution graph context
    let mut objective = ExecutionGraphCrashFeedback::new(observer.name());

    let monitor = SimpleMonitor::new( |s| println!("{s}") );
    let mut mgr = SimpleEventManager::new(monitor);

    let mut executor = CommandExecutor::builder()
        .program("../exec_graph_fuzzer_target/target")
        .build(tuple_list!(observer))
        .unwrap();

    // our state
    let mut state = StdState::new(
        StdRand::with_seed(current_nanos()),
        InMemoryCorpus::<BytesInput>::new(),
        OnDiskCorpus::new(PathBuf::from("./solutions")).unwrap(),
        &mut feedback,
        &mut objective,
    ).unwrap();

    // Standard mutations to explore the input space
    let mutator = StdScheduledMutator::with_max_stack_pow(
        havoc_mutations(),
        9,
    );

    let mut stages = tuple_list!(StdMutationalStage::new(mutator));

    let scheduler = QueueScheduler::new();
    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

    // load the initial corpus
    state.load_initial_inputs(&mut fuzzer, &mut executor, &mut mgr, &[PathBuf::from("./corpus/")]).unwrap();

    println!("Starting fuzzer loop...\n");

    // fuzz
    fuzzer.fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr).expect("Error in fuzz loop");
}
