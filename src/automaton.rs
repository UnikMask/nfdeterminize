use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

use crate::trie::FrontierController;
use crate::trie::NodeTrie;
use crate::trie::Seq;

#[derive(Debug)]
pub enum AutomatonType {
    Det,
    NonDet,
}

// Structure for an automaton.
#[derive(Debug)]
pub struct Automaton {
    automaton_type: AutomatonType,
    size: usize,
    alphabet: usize,
    table: Vec<(usize, usize, usize)>,
    start: Vec<usize>,
    end: Vec<usize>,
}

impl Automaton {
    pub fn new(
        automaton_type: AutomatonType,
        size: usize,
        alphabet: usize,
        table: Vec<(usize, usize, usize)>,
        start: Vec<usize>,
        end: Vec<usize>,
    ) -> Automaton {
        Automaton {
            automaton_type,
            size,
            alphabet,
            table,
            start,
            end,
        }
    }

    // Rabin Scott Superset Construction Algorithm
    fn rabin_scott(&self) -> (Vec<usize>, Vec<usize>, Vec<(usize, usize, usize)>) {
        // Rabin Scott Superset Construction Algorithm
        let mut controller = FrontierController::new();
        let mut state_trie = NodeTrie::new(&mut controller, true); // Empty sequence/set is a state.
        let mut transitions: Vec<(usize, usize, usize)> = Vec::new();

        // Insert start state
        controller.frontier.push_front(Seq::from(&self.start));

        // Loop add to trie until frontier is empty
        'main: loop {
            let mut state_set: Seq = match controller.frontier.pop_front() {
                Some(mut set) => set,
                None => break 'main,
            };
            // Populate sets for each alphabet symbols
            let mut symbol_sets: Vec<HashSet<usize>> = Vec::new();
            for _ in 0..self.alphabet {
                symbol_sets.push(HashSet::new());
            }

            let start_state = match state_trie.get_addr(&state_set, &mut controller) {
                Some(address) => address,
                None => panic!("Start state does not have an address!"),
            };

            // Populate symbols set to get each set of states for each symbol
            while let Seq::Cons(next, rest) = state_set {
                for transition in &self.table {
                    if transition.0 == next {
                        symbol_sets[transition.1].insert(transition.2);
                    }
                }
                state_set = *rest;
            }

            // For each sets of states, populate the state trie, get a number, and populate
            // the transition map.
            'symbolRec: for i in 0..self.alphabet {
                let current_set = match symbol_sets.get(i) {
                    Some(set) => set,
                    None => break 'symbolRec,
                };
                let seq = Seq::from(current_set);

                // Check if the new sequence is new. If it is, add it to map and frontier.
                if let None = state_trie.get(&seq) {
                    state_trie.push(&seq, &mut controller);
                }
                // Get the sequence's address and the new transition to the transitions map.
                let end_state = match state_trie.get_addr(&seq, &mut controller) {
                    Some(address) => address,
                    None => panic!("Node address should have been assigned!"),
                };
                transitions.push((start_state, i, end_state));
            }
        }
        return (self.start, self.end, transitions);
    }

    // Determinize an NFA.
    fn determinize(&self) -> Automaton {
        let mut ret = Automaton {
            automaton_type: AutomatonType::Det,
            size: self.size,
            alphabet: self.alphabet,
            table: Vec::new(),
            start: Vec::new(),
            end: Vec::new(),
        };

        // Return same automaton as it already is deterministic.
        let ret = match self.automaton_type {
            AutomatonType::Det => Automaton {
                automaton_type: AutomatonType::Det,
                size: self.size ,
                alphabet: self.alphabet,
                table: self.table.clone(),
                start: self.start.clone(),
                end: self.end.clone()
            },
            AutomatonType::NonDet => {
                let (transitions, start, end) = self.rabin_scott();
                return Automaton {
                    automaton_type: AutomatonType::Det,
                    size: transitions.size(),
                    alphabel: self.alphabet,
                    table: transitions,
                    start: start,
                    end: end
                };
            }
        }
        if let AutomatonType::Det = self.automaton_type {
            ret.table = self.table.clone();
            ret.start = self.start.clone();
            ret.end = self.end.clone();
            return ret;
        }
        return ret;
    }

    // Minimize a DFA.
    fn minimize(&self) -> Automaton {
        return Automaton {
            automaton_type: AutomatonType::Det,
            size: self.size,
            alphabet: self.alphabet,
            table: Vec::new(),
            start: Vec::new(),
            end: Vec::new(),
        };
    }
}
