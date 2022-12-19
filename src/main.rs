mod automaton;
mod trie;
mod ubig;

use automaton::Automaton;
use automaton::AutomatonType;

fn main() {
    let dfa = Automaton::new(
        AutomatonType::Det,
        2,
        2,
        vec![(0, 0, 1), (0, 1, 0), (1, 0, 1), (1, 1, 0)],
        vec![0],
        vec![1],
    );
    println!("Hello, {:?}", dfa);
}
