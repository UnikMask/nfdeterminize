mod automaton;
mod automaton_encoder;
mod ubig;

use std::{env, fs};

use automaton::Automaton;

#[macro_use]
extern crate pest_derive;

/// Main function of the program. Takes arguments:
/// + Only 1 argument is allowed - the finite state machine file.
/// + If there are more/less arguments than 1, the program will fail.
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Number of arguments must be 1!");
    }

    let contents =
        fs::read_to_string(&args[1]).expect("Given file path does not exist or is not a file!");

    let new_dfa = Automaton::from(&contents.to_string());
    println!("Hello, {:?}", new_dfa.determinized().minimized());
}
