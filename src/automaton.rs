use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

use crate::ubig::Ubig;

#[derive(Debug, PartialEq, Eq)]
pub enum AutomatonType {
    Det,
    NonDet,
}

// Structure for an automaton.
#[derive(Debug, Eq)]
pub struct Automaton {
    automaton_type: AutomatonType,
    size: usize,
    alphabet: usize,
    table: Vec<(usize, usize, usize)>,
    start: Vec<usize>,
    end: Vec<usize>,
}

impl PartialEq for Automaton {
    fn eq(&self, b: &Automaton) -> bool {
        if self.automaton_type != b.automaton_type {
            return false;
        }
        if self.size != b.size {
            return false;
        }
        if self.alphabet != b.alphabet {
            return false;
        }
        if self.table.len() != b.table.len() {
            return false;
        }
        for i in 0..self.table.len() {
            if self.table[i] != b.table[i] {
                return false;
            }
        }
        if self.start.len() != b.start.len() {
            return false;
        }
        for i in 0..self.start.len() {
            if self.start[i] != b.start[i] {
                return false;
            }
        }
        if self.end.len() != b.end.len() {
            return false;
        }
        for i in 0..self.end.len() {
            if self.end[i] != b.end[i] {
                return false;
            }
        }
        return true;
    }
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
        let mut transitions: Vec<(usize, usize, usize)> = Vec::new(); // All DFA transitions
        let mut accept_states: Vec<usize> = Vec::new(); // All accept states
        let mut num_mapper: HashMap<Ubig, usize> = HashMap::new();
        let mut bookmark = 0;
        let start_states = self
            .start
            .iter()
            .map(|s| (Ubig::from_bit(s)))
            .collect::<Vec<Ubig>>();
        let mut frontier: VecDeque<Ubig> = VecDeque::new();

        for s in start_states {
            frontier.push_back(s);
        }
        // Graph exploration - Depth-first search
        while let Some(next) = frontier.pop_front() {
            let nd_transitions = self.get_transition_map();

            // Explore all new states for each alphabet letter.
            for a in 0..self.alphabet {
                let mut new_s = Ubig::new();
                for s in next.get_seq() {
                    if let Some(s_trs) = nd_transitions.get(&(s, a)) {
                        for t in s_trs {
                            new_s.flip(t);
                        }
                    }
                }

                if !num_mapper.contains_key(&new_s) {
                    let num = bookmark;
                    num_mapper.insert(new_s.clone(), num);

                    bookmark += 1;
                    frontier.push_back(new_s.clone());
                    for s in &self.end {
                        if new_s.bit_at(s) {
                            accept_states.push(num);
                            break;
                        }
                    }
                }
                let num = match num_mapper.get(&new_s) {
                    Some(n) => n,
                    None => panic!(),
                };
                let s_n = match num_mapper.get(&next) {
                    Some(s) => s,
                    None => panic!(),
                };
                transitions.push((*s_n, a, *num));
            }
        }
        return (Vec::new(), 0, Vec::new(), Vec::new());
    }

    // Determinize an NFA.
    fn determinized(self) -> Automaton {
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
                .map(|t| {
                    let t0 = match p.get(&t.0) {
                        Some(t) => t,
                        None => panic!(),
                    };
                    let t2 = match p.get(&t.2) {
                        Some(t) => t,
                        None => panic!(),
                    };
                    (*t0, t.1, *t2)
                })
                .collect::<HashSet<(usize, usize, usize)>>()
                .into_iter()
                .collect::<Vec<(usize, usize, usize)>>(),
            start: self
                .start
                .into_iter()
                .map(|s| match p.get(&s) {
                    Some(t) => t,
                    None => panic!(),
                })
                .collect::<HashSet<&usize>>()
                .into_iter()
                .cloned()
                .collect::<Vec<usize>>(),
            end: self
                .end
                .into_iter()
                .map(|s| match p.get(&s) {
                    Some(t) => t,
                    None => panic!(),
                })
                .collect::<HashSet<&usize>>()
                .into_iter()
                .cloned()
                .collect::<Vec<usize>>(),
        };
    }

    fn hopcroft_algo(&self) -> HashMap<usize, usize> {
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

        // Convert partition into map from initial state to partitioned state
        let mut ret_map: HashMap<usize, usize> = HashMap::new();
        for i in 0..p.len() {
            for s in p[i].iter() {
                ret_map.insert(*s, i);
            }
        }
        return ret_map;
    }

    fn get_transition_map(&self) -> HashMap<(usize, usize), Vec<usize>> {
        let mut map: HashMap<(usize, usize), Vec<usize>> = HashMap::new();

        for transition in &self.table {
            match map.get_mut(&(transition.0, transition.1)) {
                Some(v) => v.push(transition.2),
                None => {
                    map.insert((transition.0, transition.1), vec![transition.2]);
                }
            }
        }

        return map;
    }
    fn reverse_transition(&self, set: &HashSet<usize>, c: usize) -> HashSet<usize> {
        return (0..self.size)
            .filter(|i| set.contains(&self.get_transition_map()[&(*i, c)][0]))
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

#[cfg(test)]
mod tests {
    use super::Automaton;
    use super::AutomatonType;

    #[test]
    fn test_determinization_redundant() {
        let redundant_nd = Automaton {
            automaton_type: AutomatonType::NonDet,
            size: 1,
            alphabet: 2,
            table: vec![(1, 1, 1), (1, 2, 1)],
            start: vec![1],
            end: vec![1],
        };
        let redundant_d = Automaton {
            automaton_type: AutomatonType::Det,
            size: 1,
            alphabet: 2,
            table: vec![(1, 1, 1), (1, 2, 1)],
            start: vec![1],
            end: vec![1],
        };
        assert_eq!(redundant_nd.determinized(), redundant_d);
    }
}
