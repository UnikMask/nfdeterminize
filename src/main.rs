mod automaton;
mod automaton_encoder;
mod ubig;

use automaton::Automaton;
use automaton::AutomatonType;

#[macro_use]
extern crate pest_derive;

fn main() {
    let dfa = Automaton::new(
        AutomatonType::NonDet,
        2,
        2,
        vec![(0, 0, 1), (0, 1, 0), (1, 0, 1), (1, 1, 0)],
        vec![0],
        vec![1],
    );
    println!("Hello, {:?}", dfa.determinized().minimized());
}
