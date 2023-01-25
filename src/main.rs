mod automaton;
mod automaton_encoder;
mod ubig;

use automaton::Automaton;
use automaton::AutomatonType;

#[macro_use]
extern crate pest_derive;

fn main() {
    let new_dfa =
        Automaton::from("{det, 2, 2, [(0, 0, 1), (0, 1, 0), (1, 0, 1), (1, 1, 0)], [0], [1]}");
    println!("Hello, {:?}", new_dfa.determinized().minimized());
}
