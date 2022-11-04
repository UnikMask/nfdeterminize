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

        let p = self.hopcroft_algo();

        return Automaton {
            automaton_type: AutomatonType::Det,
            size: p.len(),
            alphabet: self.alphabet,
            table: self
                .table
                .into_iter()
                .map(|t| (get_set(p, t.0), t.1, get_set(p, t.2)))
                .collect::<HashSet<&(usize, usize, usize)>>()
                .into_iter()
                .cloned()
                .collect::<Vec<(usize, usize, usize)>>(),
            start: self
                .start
                .into_iter()
                .map(|s| get_set(p, s))
                .collect::<HashSet<&usize>>()
                .into_iter()
                .cloned()
                .collect::<Vec<usize>>(),
            start: self
                .end
                .into_iter()
                .map(|s| get_set(p, s))
                .collect::<HashSet<&usize>>()
                .into_iter()
                .cloned()
                .collect::<Vec<usize>>(),
        };
    }

    fn hopcroft_algo(&self) -> VecDeque<HashSet<usize>> {
        let finals: HashSet<usize> = self.end.clone().into_iter().collect();

        // Get sets of all non-finals and finals.
        let mut p: VecDeque<HashSet<usize>> = VecDeque::from_iter(vec![
            (0..self.size)
                .collect::<HashSet<usize>>()
                .difference(&finals)
                .cloned()
                .collect::<HashSet<usize>>(),
            finals.clone(),
        ]);

        let mut p_frontier: VecDeque<HashSet<usize>> = VecDeque::from_iter(vec![
            (0..self.size)
                .collect::<HashSet<usize>>()
                .difference(&finals)
                .cloned()
                .collect::<HashSet<usize>>(),
            finals.clone(),
        ]);

        // Get transition map for efficiency
        let map = self.get_transition_map();

        // Iterate until the partition frontier is empty
        loop {
            // Get a new set from the frontier and break if the frontier's empty
            let set = match p_frontier.pop_front() {
                Some(s) => s,
                None => break,
            };

            // Iterate over each input symbol
            for c in 0..self.alphabet {
                let rs = self.reverse_transition(&set, c);

                // Loop through all sets in P.
                let len = p.len();
                for _ in 0..len {
                    let r = match p.pop_front() {
                        Some(set) => set,
                        None => break,
                    };

                    if r.intersection(&rs).next() != None && r.difference(&rs).next() != None {
                        // Get sets of intersections and differences
                        let r0 = r.intersection(&rs).cloned().collect::<HashSet<usize>>();
                        let r1 = r.difference(&rs).cloned().collect::<HashSet<usize>>();

                        p.push_back(r0.clone());
                        p.push_back(r1.clone());

                        // Replace r with r0 and r1 if r is in frontier
                        if !partition_replace(
                            &mut p_frontier,
                            &r,
                            &mut vec![r1.clone(), r0.clone()],
                        ) {
                            // Add to frontier whichever of r0 or r1 is the smallest
                            if r0.len() <= r1.len() {
                                p_frontier.push_back(r0);
                            } else {
                                p_frontier.push_back(r1);
                            }
                        }
                    } else {
                        p.push_back(r);
                    }
                }
            }
        }

        return p;
    }

    fn get_transition_map(&self) -> HashMap<(usize, usize), usize> {
        let mut map: HashMap<(usize, usize), usize> = HashMap::new();

        for transition in &self.table {
            map.insert((transition.0, transition.1), transition.2);
        }

        return map;
    }
    fn reverse_transition(&self, set: &HashSet<usize>, c: usize) -> HashSet<usize> {
        return (0..self.size)
            .filter(|i| set.contains(&self.get_transition_map()[&(*i, c)]))
            .collect();
    }
}

// Check if a set is contained in a set in a partition.
fn partition_replace(
    p: &mut VecDeque<HashSet<usize>>,
    replaced: &HashSet<usize>,
    replacements: &mut Vec<HashSet<usize>>,
) -> bool {
    let p_len = p.len();
    for _ in 0..p_len {
        let next = match p.pop_front() {
            Some(next) => next,
            None => break,
        };
        if replaced.difference(&next).next() == None {
            loop {
                p.push_back(match replacements.pop() {
                    Some(r) => r,
                    None => break,
                });
                return true;
            }
        } else {
            p.push_back(next);
        }
    }
    return false;
}
