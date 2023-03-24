use std::collections::{HashMap, HashSet, VecDeque};

use crate::automaton_multithreaded::rabin_scott_mt;
use crate::automaton_sequential::{hopcroft_algo, rabin_scott_seq};
use crate::ubig::Ubig;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AutomatonType {
    Det,
    NonDet,
}

// Structure for an automaton.
#[derive(Debug, Clone)]
pub struct Automaton {
    pub automaton_type: AutomatonType,
    pub size: usize,
    pub alphabet: usize,
    pub table: Vec<(usize, usize, usize)>,
    pub start: Vec<usize>,
    pub end: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlgorithmKind {
    /// Run command sequentially
    Sequential,
    /// Run command in multithreaded mode
    Multithreaded(usize),
}

impl Automaton {
    ////////////////////
    // Public methods //
    ////////////////////

    /// Return a new automaton
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

    /// Return an empty automaton structure.
    pub fn empty() -> Automaton {
        Automaton::new(AutomatonType::Det, 0, 0, vec![], vec![], vec![])
    }

    /// Return a determinized version of the given automata - Using Rabin-Scott's Superset Construction algorithm.
    pub fn determinized(&self, kind: AlgorithmKind) -> Automaton {
        // Return same automaton as it already is deterministic.
        let ret = match self.automaton_type {
            AutomatonType::Det => self.clone(),
            AutomatonType::NonDet => {
                let (transitions, a_size, a_start, a_end) = match kind {
                    AlgorithmKind::Sequential => rabin_scott_seq(&self),
                    AlgorithmKind::Multithreaded(n_threads) => rabin_scott_mt(&self, n_threads),
                };
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

    /// Return a minimized version of the given automata - Using Hopcroft's partition algorithm.
    pub fn minimized(&self) -> Automaton {
        if let AutomatonType::NonDet = self.automaton_type {
            return self.clone();
        } else if self.size <= 2 {
            return self.clone();
        }

        let tuple = hopcroft_algo(&self);
        let p = tuple.0;
        let len = tuple.1;

        let ret = Automaton {
            automaton_type: AutomatonType::Det,
            size: len,
            alphabet: self.alphabet,
            table: self
                .table
                .clone()
                .into_iter()
                .map(|t| {
                    if let (Some(t0), Some(t2)) = (p.get(&t.0), p.get(&t.2)) {
                        (*t0, t.1, *t2)
                    } else {
                        panic!();
                    }
                })
                .collect::<HashSet<(usize, usize, usize)>>()
                .into_iter()
                .collect::<Vec<(usize, usize, usize)>>(),
            start: Automaton::get_part_vec_from_vec(&p, &self.start),
            end: Automaton::get_part_vec_from_vec(&p, &self.end),
        };
        return ret;
    }

    /// Reverse all transitions of the automaton
    pub fn reverse_transitions(mut self) -> Self {
        self.table = self.table.drain(..).map(|(s, a, e)| (e, a, s)).collect();
        (self.start, self.end) = (self.end, self.start);
        self.automaton_type = AutomatonType::NonDet;
        self
    }

    ///////////////
    // Utilities //
    ///////////////

    /// Add a state into a set of states, adding states connected via the empty char to the set with it.
    pub fn add_state(&self, arr: &Vec<Vec<Vec<usize>>>, num: &mut Ubig, bit: usize) {
        let mut queue: VecDeque<usize> = VecDeque::from([bit]);
        while let Some(b) = queue.pop_front() {
            if !num.bit_at(&b) {
                num.set_to(&b, true);

                (&arr[0][b]).iter().for_each(|t| {
                    queue.push_front(*t);
                });
            }
        }
    }

    fn get_empty_transition_arr(&self) -> Vec<Vec<Vec<usize>>> {
        (0..self.alphabet + 1)
            .map(|_| (0..self.size + 1).map(|_| Vec::new()).collect())
            .collect()
    }

    /// Get a hashmap of leading states from a given letter and original state.
    pub fn get_transition_array(&self) -> Vec<Vec<Vec<usize>>> {
        let mut arr = self.get_empty_transition_arr();
        (&self.table)
            .into_iter()
            .for_each(|t| arr[t.1][t.0].push(t.2));
        return arr;
    }

    /// Get the array that represents all the reverse transitions of the automaton.
    pub fn get_reverse_transition_arr(&self) -> Vec<Vec<Vec<usize>>> {
        let mut arr = self.get_empty_transition_arr();
        (&self.table)
            .into_iter()
            .for_each(|t| arr[t.1][t.2].push(t.0));
        return arr;
    }

    ////////////////////
    // Static methods //
    ////////////////////

    /// Get a vector of partitions from a vector of initial states and a partition map.
    fn get_part_vec_from_vec(p: &HashMap<usize, usize>, s: &Vec<usize>) -> Vec<usize> {
        s.clone()
            .into_iter()
            .map(|s| *p.get(&s).unwrap())
            .collect::<HashSet<usize>>()
            .into_iter()
            .collect::<Vec<usize>>()
    }
}
