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
    fn rabin_scott(&self) -> (Vec<(usize, usize, usize)>, usize, Vec<usize>, Vec<usize>) {
        // Rabin Scott Superset Construction Algorithm
        let mut controller = FrontierController::new();
        let mut state_trie = NodeTrie::new(&mut controller, true); // Empty sequence/set is a state.
        let mut transitions: Vec<(usize, usize, usize)> = Vec::new(); // All DFA transitions
        let mut accept_states: Vec<usize> = Vec::new(); // All accept states

        // Insert start state in trie and frontier
        let s0 = Seq::from(&self.start);
        state_trie.push(&s0, &mut controller);
        let s0_addr = match state_trie.get_addr(&s0, &mut controller) {
            Some(address) => address,
            None => panic!("Start state is supposed to have an address!"),
        };
        controller.frontier.push_front(s0);

        // Loop add to trie until frontier is empty
        'main: loop {
            let mut state_set: Seq = match controller.frontier.pop_front() {
                Some(set) => set,
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

                // Iterate through sequence to find if it is an accept state
                let mut seq_iter = &seq;
                let mut is_accept_state = false;
                'accept_check: while let Seq::Cons(next, rest) = seq_iter {
                    for accept_state in &self.end {
                        if next == accept_state {
                            is_accept_state = true;
                            break 'accept_check;
                        }
                    }
                    seq_iter = &rest;
                }
                if is_accept_state {
                    accept_states.push(end_state);
                }
                transitions.push((start_state, i, end_state));
            }
        }
        return (transitions, controller.size(), vec![s0_addr], accept_states);
    }

    // Determinize an NFA.
    fn determinize(self) -> Automaton {
        // Return same automaton as it already is deterministic.
        let ret = match self.automaton_type {
            AutomatonType::Det => Automaton {
                automaton_type: AutomatonType::Det,
                size: self.size,
                alphabet: self.alphabet,
                table: self.table.clone(),
                start: self.start.clone(),
                end: self.end.clone(),
            },
            AutomatonType::NonDet => {
                let (transitions, a_size, a_start, a_end) = self.rabin_scott();
                return Automaton {
                    automaton_type: AutomatonType::Det,
                    size: a_size,
                    alphabet: self.alphabet,
                    table: transitions,
                    start: a_start,
                    end: a_end,
                };
            }
        };
        return ret;
    }

    // Minimize a DFA.
    fn minimize(self) -> Automaton {
        if let AutomatonType::Det = self.automaton_type {
            return self;
        }
        // Get vector with all accept states removed.
        let mut pk: Vec<Vec<usize>> = Vec::new();
        let mut last = 0;
        pk.push(Vec::new());
        for end in &self.end {
            pk[0].append(&mut (last..*end).collect());
            last = end + 1;
        }
        pk[0].append(&mut (last..self.size).collect());
        pk.push(self.end.clone());

        return Automaton {
            automaton_type: AutomatonType::Det,
            size: self.size,
            alphabet: self.alphabet,
            table: Vec::new(),
            start: Vec::new(),
            end: Vec::new(),
        };
    }

    fn hopcroft_step(&self, p: &Vec<Vec<usize>>) -> Vec<Vec<usize>> {
        let mut p_next: Vec<Vec<usize>> = Vec::new();

        // Get transition map for efficiency
        let map = self.get_transition_map();

        // Iterate for each set
        for i in 0..p.len() {
            let set = match p.get(i) {
                Some(s) => s,
                None => break,
            };

            // Iterate over each pair of the set
            for j in 0..set.len() - 1 {
                for k in j + 1..set.len() {
                    let mut distinguish = false;
                    for s in 0..self.alphabet {
                        if get_set(map[&(set[j], s)], p) != get_set(map[&(set[k], s)], p) {
                            distinguish = true;
                            break;
                        }
                    }

                    if distinguish {
                        // Let j and k into 2 different sets
                    }
                }
            }
        }

        return p_next;
    }

    fn get_transition_map(&self) -> HashMap<(usize, usize), usize> {
        let mut map: HashMap<(usize, usize), usize> = HashMap::new();

        for transition in &self.table {
            map.insert((transition.0, transition.1), transition.2);
        }

        return map;
    }
}

fn get_set(state: usize, p: &Vec<Vec<usize>>) -> Option<usize> {
    // Iterate for each set
    for i in 0..p.len() {
        for s in match p.get(i) {
            Some(s) => s,
            None => break,
        } {
            if state == *s {
                return Some(i);
            }
        }
    }
    return None;
}
