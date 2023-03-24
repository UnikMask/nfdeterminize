#![feature(scoped_threads)]
mod automaton;
pub mod automaton_encoder;
mod automaton_multithreaded;
mod automaton_sequential;
mod automaton_test;
mod transition_graphs;
mod ubig;

use std::{
    fmt::Debug,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use automaton::{AlgorithmKind, Automaton};
use clap::{Parser, ValueEnum};
use transition_graphs::{get_buffer_and_stack_aut, get_two_stack_aut};

static N_THREADS: usize = 12;

#[derive(clap::Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct ProgramArguments {
    #[clap(subcommand)]
    action: Action,

    /// Print verbose output
    #[clap(long)]
    verbose: bool,

    /// Time the minimization/determinization process
    #[clap(short, long)]
    timed: bool,

    #[clap(short, long)]
    n_threads: Option<usize>,

    /// Specify whether the system should be run sequentially, multithreaded, or multiprocessed
    #[clap(short, long, value_enum)]
    mode: Option<AlgorithmAction>,

    #[clap(short, long)]
    /// File to print the automaton to
    file: Option<PathBuf>,
}

impl ProgramArguments {
    pub fn print_verbose(&self, msg: &str) {
        if self.verbose {
            print!("{}", msg);
        }
    }

    pub fn get_automaton(&self) -> Automaton {
        let format: &AutomatonFormat = match &self.action {
            Action::Run { format, .. } => format,
            Action::Minimize { format } => format,
            Action::Determinize { format } => format,
        };
        match format {
            AutomatonFormat::File { fp } => match fs::read_to_string(&fp) {
                Ok(aut) => {
                    self.print_verbose("Parsing automaton from file...");
                    Automaton::from(&aut.to_string())
                }
                Err(_) => {
                    eprintln!(
                        "File {} is a directory or does not exist!",
                        fp.to_str().unwrap()
                    );
                    Automaton::empty()
                }
            },
            AutomatonFormat::Bns { b, s } => {
                self.print_verbose("Generating Buffer and Stack automata...");
                get_buffer_and_stack_aut(*b, *s)
            }
            AutomatonFormat::TwoStack { n1, n2 } => {
                self.print_verbose("Generating two-stack automata...");
                get_two_stack_aut(*n1, *n2)
            }
        }
    }
}

#[derive(Debug, Copy, Clone, ValueEnum)]
enum AlgorithmAction {
    Sequential,
    Multithreaded,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
enum MinimizationMethod {
    PartitionRefine,
    Brzozowski,
}

#[derive(clap::Subcommand, Debug)]
enum Action {
    /// Run Determinization then minimization.
    Run {
        #[clap(subcommand)]
        format: AutomatonFormat,

        #[clap(short, long, value_enum)]
        method: Option<MinimizationMethod>,
    },
    /// Run minimization.
    Minimize {
        #[clap(subcommand)]
        format: AutomatonFormat,
    },

    /// Run determinization.
    Determinize {
        #[clap(subcommand)]
        format: AutomatonFormat,
    },
}

#[derive(clap::Subcommand, Debug)]
enum AutomatonFormat {
    /// Get automaton from a file.
    File { fp: std::path::PathBuf },
    /// Use a generated Buffer and Stack TPN automaton.
    Bns { b: usize, s: usize },
    /// Use a generated 2-stack TPN automaton.
    TwoStack { n1: usize, n2: usize },
}

/// Main function of the program. Takes arguments:
/// + Only 1 argument is allowed - the finite state machine file.
/// + If there are more/less arguments than 1, the program will fail.
fn main() {
    let clap_args = ProgramArguments::parse();

    let automaton = clap_args.get_automaton();
    if clap_args.verbose {
        println!("Automaton size: {:?}", automaton.size);
    }

    // Set the mode of the program - sequential or multithreaded, with number of threads
    let mode = match clap_args.mode {
        None => AlgorithmAction::Multithreaded,
        Some(k) => k,
    };
    let n_threads = match clap_args.n_threads {
        None => N_THREADS,
        Some(n) => n,
    };
    let mode = match mode {
        AlgorithmAction::Sequential => AlgorithmKind::Sequential,
        AlgorithmAction::Multithreaded => AlgorithmKind::Multithreaded(n_threads),
    };

    let start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("")
        .as_millis();

    let final_dfa = match clap_args.action {
        Action::Run { method, .. } => {
            let method = match method {
                None => MinimizationMethod::PartitionRefine,
                Some(m) => m,
            };
            match method {
                MinimizationMethod::PartitionRefine => {
                    clap_args.print_verbose("Determinizing automata... ");
                    let new_dfa = automaton.determinized(mode);
                    if clap_args.verbose {
                        println!("Intermediate Automaton Size: {:?}", new_dfa.size);
                    }
                    clap_args.print_verbose("Minimizing automata... ");
                    new_dfa.minimized()
                }
                MinimizationMethod::Brzozowski => {
                    clap_args.print_verbose("Determinizing automata... ");
                    let new_dfa = automaton.reverse_transitions().determinized(mode);
                    if clap_args.verbose {
                        println!("Intermediate Automaton Size: {:?}", new_dfa.size);
                    }
                    clap_args.print_verbose("Redeterminizing automata... ");
                    new_dfa.reverse_transitions().determinized(mode)
                }
            }
        }
        Action::Minimize { .. } => {
            clap_args.print_verbose("Minimizing automata... ");
            automaton.minimized()
        }
        Action::Determinize { .. } => {
            clap_args.print_verbose("Determinizing automata... ");
            automaton.determinized(mode)
        }
    };

    // Print final dfa to file/stdout
    clap_args.print_verbose(&format!("Final Automaton size: {:?}\n", final_dfa.size));
    if let Some(fp) = clap_args.file {
        if let Ok(mut f) = File::create(fp.clone()) {
            if let Err(_) = f.write_all(format!("{final_dfa:?}").as_bytes()) {
                eprintln!("Writing to file failed!");
            }
        } else {
            eprintln!("File {:?} already exists!", fp);
        }
    } else {
        println!("{:?}", final_dfa);
    }

    if clap_args.timed {
        let end = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("")
            .as_millis();
        println!(
            "Time taken: {:?} seconds.",
            f64::from((end as usize - start as usize) as i32) / f64::from(1000)
        );
    }
}
