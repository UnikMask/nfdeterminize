use fasthash::xx::Hasher64;
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet, LinkedList, VecDeque},
    hash::BuildHasherDefault,
};

use crate::ubig::Ubig;
type HashMapXX<K, V> = HashMap<K, V, BuildHasherDefault<Hasher64>>;

// static N_THREADS: usize = 12;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AutomatonType {
    Det,
    NonDet,
}

// Structure for an automaton.
#[derive(Debug, Eq, Clone, PartialEq)]
pub struct Automaton {
    automaton_type: AutomatonType,
    size: usize,
    alphabet: usize,
    table: Vec<(usize, usize, usize)>,
    start: Vec<usize>,
    end: Vec<usize>,
}

impl Automaton {
    ///////////////////////
    // Getters & Setters //
    ///////////////////////

    pub fn set_automaton_type(&mut self, t: AutomatonType) {
        self.automaton_type = t;
    }

    pub fn set_size(&mut self, s: usize) {
        self.size = s;
    }

    pub fn get_size(&self) -> usize {
        return self.size;
    }

    pub fn get_alphabet(&self) -> usize {
        return self.alphabet;
    }

    pub fn set_alphabet(&mut self, a: usize) {
        self.alphabet = a;
    }

    pub fn set_transitions(&mut self, t: Vec<(usize, usize, usize)>) {
        self.table = t;
    }

    pub fn set_start_states(&mut self, ss: Vec<usize>) {
        self.start = ss;
    }

    pub fn set_end_states(&mut self, es: Vec<usize>) {
        self.end = es;
    }

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
    pub fn determinized(&self) -> Automaton {
        // Return same automaton as it already is deterministic.
        let ret = match self.automaton_type {
            AutomatonType::Det => self.clone(),
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

    /// Return a minimized version of the given automata - Using Hopcroft's partition algorithm.
    pub fn minimized(&self) -> Automaton {
        if let AutomatonType::NonDet = self.automaton_type {
            return self.clone();
        } else if self.size <= 2 {
            return self.clone();
        }

        let tuple = self.hopcroft_algo();
        let p = tuple.0;
        let len = tuple.1;

        let mut ret = Automaton {
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
        ret.start.sort();
        ret.end.sort();
        ret.table.sort();
        return ret;
    }

    ////////////////
    // Algorithms //
    ////////////////

    /// Rabin Scott Superset Construction Algorithm - Used for determinization of NFAs.
    /// Returns: (transitions vector, number of states, start states, end states).
    fn rabin_scott(&self) -> (Vec<(usize, usize, usize)>, usize, Vec<usize>, Vec<usize>) {
        // Rabin Scott Superset Construction Algorithm
        let mut transitions: Vec<(usize, usize, usize)> = Vec::new(); // All DFA transitions
        let mut accept_states: Vec<usize> = Vec::new(); // All accept states
        let mut num_mapper: HashMapXX<Ubig, usize> = HashMapXX::default();
        let mut frontier: VecDeque<Ubig> = VecDeque::new();

        // Select start state from all start states in the non deterministic automata.
        let transition_arr = self.get_transition_array();
        let mut start_state = Ubig::new();
        (&self.start)
            .into_iter()
            .for_each(|s| self.add_state(&transition_arr, &mut start_state, *s));
        for s in &self.end {
            if start_state.bit_at(s) {
                accept_states.push(0);
                break;
            }
        }
        num_mapper.insert(start_state.clone(), num_mapper.len());
        frontier.push_back(start_state.clone());

        // Graph exploration - Depth-first search
        while let Some(next) = frontier.pop_front() {
            (1..self.alphabet + 1).for_each(|a| {
                let mut new_s = Ubig::new();
                next.get_seq().into_iter().for_each(|s| {
                    (&transition_arr[a][s]).into_iter().for_each(|t| {
                        self.add_state(&transition_arr, &mut new_s, *t);
                    })
                });

                if !num_mapper.contains_key(&new_s) {
                    num_mapper.insert(new_s.clone(), num_mapper.len());
                    for s in &self.end {
                        if new_s.bit_at(s) {
                            accept_states.push(num_mapper.len() - 1);
                            break;
                        }
                    }
                    frontier.push_back(new_s.clone());
                }
                transitions.push((
                    *num_mapper.get(&next).unwrap(),
                    a,
                    *num_mapper.get(&new_s).unwrap(),
                ));
            });
        }
        return (transitions, num_mapper.len(), vec![0], accept_states);
    }

    /// Hopcroft algorithm for minimization of a DFA.
    /// Returns a map of what state is in which leading partition, and the number of partitions.
    fn hopcroft_algo(&self) -> (HashMap<usize, usize>, usize) {
        let finals: HashSet<usize> = self.end.clone().into_iter().collect();
        let mut p: LinkedList<Vec<usize>> = LinkedList::from_iter(vec![
            (0..self.size)
                .filter(|i| !finals.contains(i))
                .collect::<Vec<usize>>(),
            self.end.clone(),
        ]);

        let mut q = p.clone();
        let rev_arr = self.get_reverse_transition_arr();
        while let Some(set) = q.pop_front() {
            for c in 1..self.alphabet + 1 {
                let rs = Automaton::get_set_from_transitions(&rev_arr, &set, c);
                let len = p.len();
                let mut cursor = p.cursor_front_mut();
                let mut index = 0;
                while index < len {
                    let v = cursor.current().unwrap().clone();
                    let (diffs, ands) = Automaton::get_diff_ands(&v, &rs);
                    if diffs.len() > 0 && ands.len() > 0 {
                        cursor.insert_before(diffs.clone());
                        cursor.insert_before(ands.clone());
                        cursor.remove_current();
                        cursor.move_prev();

                        let mut ll = LinkedList::new();
                        ll.push_front(diffs);
                        ll.push_front(ands);
                        Automaton::replace_in_queue(&mut q, &v, ll);
                    }
                    index += 1;
                    cursor.move_next();
                }
            }
        }

        // Convert partition into map from initial state to partitioned state
        let mut ret_map: HashMap<usize, usize> = HashMap::new();
        let mut index = 0;
        p.iter().for_each(|next| {
            next.iter().for_each(|s| {
                ret_map.insert(*s, index);
            });
            index += 1;
        });
        return (ret_map, p.len());
    }

    ///////////////
    // Utilities //
    ///////////////

    /// Add a state into a set of states, adding states connected via the empty char to the set with it.
    fn add_state(&self, arr: &Vec<Vec<Vec<usize>>>, num: &mut Ubig, bit: usize) {
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

    /// Generates and empty transition array with the dimensions of self.
    fn get_empty_transition_arr(&self) -> Vec<Vec<Vec<usize>>> {
        (0..self.alphabet + 1)
            .map(|_| (0..self.size + 1).map(|_| Vec::new()).collect())
            .collect()
    }

    /// Get a hashmap of leading states from a given letter and original state.
    fn get_transition_array(&self) -> Vec<Vec<Vec<usize>>> {
        let mut arr = self.get_empty_transition_arr();
        (&self.table)
            .into_iter()
            .for_each(|t| arr[t.1][t.0].push(t.2));
        return arr;
    }

    fn get_reverse_transition_arr(&self) -> Vec<Vec<Vec<usize>>> {
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

    /// Get a list of states that are destinations of given set of states and character from
    /// a transition/reverse transition map.
    fn get_set_from_transitions(
        arr: &Vec<Vec<Vec<usize>>>,
        set: &Vec<usize>,
        c: usize,
    ) -> Vec<usize> {
        let mut bh: BinaryHeap<Reverse<usize>> = BinaryHeap::new();
        set.into_iter().for_each(|s| {
            (&arr[c][*s]).into_iter().for_each(|f| bh.push(Reverse(*f)));
        });
        let mut v = Vec::with_capacity(bh.len());
        while let Some(i) = bh.pop() {
            v.push(i.0);
        }
        return v;
    }

    fn replace_in_queue(
        q: &mut LinkedList<Vec<usize>>,
        replace: &Vec<usize>,
        mut replacement: LinkedList<Vec<usize>>,
    ) {
        let mut cursor = q.cursor_front_mut();
        while let Some(next) = cursor.current() {
            if Automaton::all_equal(replace, next) {
                replacement.iter().for_each(|r| {
                    cursor.insert_before(r.to_vec());
                });
                return;
            }
            cursor.move_next();
        }
        q.append(&mut replacement);
    }

    fn all_equal(u: &Vec<usize>, v: &Vec<usize>) -> bool {
        if u.len() != v.len() {
            false
        } else {
            let mut cursor = 0;
            while cursor < u.len() {
                if u[cursor] != v[cursor] {
                    return false;
                }
                cursor += 1;
            }
            true
        }
    }

    /// For 2 ordered sets U and V, get U n V and U \ V.
    fn get_diff_ands(u: &Vec<usize>, v: &Vec<usize>) -> (Vec<usize>, Vec<usize>) {
        let (mut cursor_u, mut cursor_v) = (0, 0);
        let (mut diffs, mut ands) = (Vec::new(), Vec::new());
        while cursor_u < u.len() && cursor_v < v.len() {
            if u[cursor_u] < v[cursor_v] {
                diffs.push(u[cursor_u]);
                cursor_u += 1;
            } else if u[cursor_u] > v[cursor_v] {
                cursor_v += 1;
            } else {
                ands.push(u[cursor_u]);
                cursor_v += 1;
                cursor_u += 1;
            }
        }
        if cursor_u < u.len() {
            u[cursor_u..u.len()].iter().for_each(|i| diffs.push(*i));
        }

        (diffs, ands)
    }
}
