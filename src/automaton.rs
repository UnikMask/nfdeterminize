use fasthash::xx::Hasher64;
use std::{
    collections::{HashMap, HashSet, VecDeque},
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
        let mut bookmark = 0;
        let mut frontier: VecDeque<Ubig> = VecDeque::new();

        //let mut frontier_c: Vec<VecDeque<Ubig>> = Vec::with_capacity(N_THREADS + 1);
        //for i in 0..N_THREADS + 1 {
        //    frontier_c[i] = VecDeque::new();
        //}

        // Select start state from all start states in the non deterministic automata.
        let transition_arr = self.get_transition_array();
        let mut start_state = Ubig::new();
        let mut cache: Vec<Option<Ubig>> = (0..self.size).map(|_| None).collect();
        (&self.start)
            .into_iter()
            .for_each(|s| self.add_state(&transition_arr, &mut start_state, *s, &mut cache));
        for s in &self.end {
            if start_state.bit_at(s) {
                accept_states.push(0);
                break;
            }
        }
        num_mapper.insert(start_state.clone(), bookmark);
        frontier.push_back(start_state.clone());
        bookmark += 1;

        // Graph exploration - Depth-first search
        while let Some(next) = frontier.pop_front() {
            // Explore all new states for each alphabet letter.
            for a in 1..self.alphabet + 1 {
                let mut new_s = Ubig::new();
                for s in next.get_seq() {
                    let s_trs = &transition_arr[a][s];

                    s_trs.into_iter().for_each(|t| {
                        self.add_state(&transition_arr, &mut new_s, *t, &mut cache);
                    });
                }

                if !num_mapper.contains_key(&new_s) {
                    let num = bookmark;
                    num_mapper.insert(new_s.clone(), num);

                    bookmark += 1;
                    for s in &self.end {
                        if new_s.bit_at(s) {
                            accept_states.push(num);
                            break;
                        }
                    }
                    frontier.push_back(new_s.clone());
                }
                let num = match num_mapper.get(&new_s) {
                    Some(n) => n,
                    None => panic!("New state was not successfully added to mapper!"),
                };
                let s_n = match num_mapper.get(&next) {
                    Some(s) => s,
                    None => panic!("State in queue was not succesfully added to mapper!"),
                };
                transitions.push((*s_n, a, *num));
            }
        }
        return (transitions, num_mapper.len(), vec![0], accept_states);
    }

    /// Hopcroft algorithm for minimization of a DFA.
    /// Returns a map of what state is in which leading partition, and the number of partitions.
    fn hopcroft_algo(&self) -> (HashMap<usize, usize>, usize) {
        let finals: HashSet<usize> = self.end.clone().into_iter().collect();

        // Get sets of all non-finals and finals.
        let mut p: VecDeque<HashSet<usize>> = VecDeque::from_iter(vec![
            (0..self.size)
                .filter(|i| !finals.contains(i))
                .collect::<HashSet<usize>>(),
            finals.clone(),
        ]);
        let mut p_frontier: VecDeque<HashSet<usize>> = p.clone();
        let rev_map = self.get_reverse_transition_map();

        // Iterate until the partition frontier is empty
        while let Some(set) = p_frontier.pop_front() {
            // Iterate over each input symbol
            for c in 1..self.alphabet + 1 {
                let rs = Automaton::get_set_transitions(&rev_map, &set, c);

                // Loop through all sets in P.
                for _ in 0..p.len() {
                    let r = match p.pop_front() {
                        Some(set) => set,
                        None => break,
                    };

                    if r.intersection(&rs).next() != None && r.difference(&rs).next() != None {
                        // Get sets of intersections and differences
                        let r0 = r.intersection(&rs).cloned().collect::<HashSet<usize>>();
                        let r1 = r.difference(&rs).cloned().collect::<HashSet<usize>>();
                        let replacements = vec![r1.clone(), r0.clone()];

                        p.push_back(r1.clone());
                        p.push_back(r0.clone());

                        // Replace r with r0 and r1 if r is in frontier
                        if !Automaton::replace_in_partition(&mut p_frontier, &r, replacements) {
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
        return (ret_map, p.len());
    }

    ///////////////
    // Utilities //
    ///////////////

    /// Add a state into a set of states, adding states connected via the empty char to the set with it.
    fn add_state(
        &self,
        arr: &Vec<Vec<HashSet<usize>>>,
        num: &mut Ubig,
        bit: usize,
        cache: &mut Vec<Option<Ubig>>,
    ) {
        //cache_option = None;
        let mut extras = Ubig::new();
        if let Some(cache_state) = &cache[bit] {
            println!("Cache found for {bit:?}!");
            extras = cache_state.clone();
        } else {
            let mut queue: VecDeque<usize> = VecDeque::from([bit]);
            while let Some(b) = queue.pop_front() {
                if !extras.bit_at(&b) {
                    extras.set_to(&b, true);

                    if let Some(cache_state) = &cache[b] {
                        println!("Cached bit found - {b:?}");
                        for c_b in cache_state.get_seq() {
                            extras.set_to(&c_b, true);
                        }
                    } else {
                        println!("Cache not found for {b:?}!");
                        (&arr[0][b]).into_iter().for_each(|t| {
                            queue.push_back(*t);
                        });
                    }
                }
            }
            cache[bit] = Some(extras.clone());
        }
        for b in extras.get_seq() {
            num.set_to(&b, true);
        }
    }

    /// Get a hashmap of leading states from a given letter and original state.
    fn get_transition_array(&self) -> Vec<Vec<HashSet<usize>>> {
        //let mut map: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
        let mut arr: Vec<Vec<HashSet<usize>>> = Vec::new();
        for _ in 0..(self.alphabet + 1) {
            let mut alphabet_arr: Vec<HashSet<usize>> = Vec::new();
            for _ in 0..self.size + 1 {
                alphabet_arr.push(HashSet::new());
            }
            arr.push(alphabet_arr);
        }

        for transition in &self.table {
            arr[transition.1][transition.0].insert(transition.2);
        }

        return arr;
    }

    /// Get a map of all reverse transitions in the DFA.
    fn get_reverse_transition_map(&self) -> HashMap<(usize, usize), Vec<usize>> {
        let mut map: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
        for transition in &self.table {
            match map.get_mut(&(transition.2, transition.1)) {
                Some(v) => v.push(transition.0),
                None => {
                    map.insert((transition.2, transition.1), vec![transition.0]);
                }
            }
        }
        return map;
    }

    ////////////////////
    // Static methods //
    ////////////////////

    /// Get a vector of partitions from a vector of initial states and a partition map.
    fn get_part_vec_from_vec(p: &HashMap<usize, usize>, s: &Vec<usize>) -> Vec<usize> {
        s.clone()
            .into_iter()
            .map(|s| match p.get(&s) {
                Some(t) => *t,
                None => panic!(),
            })
            .collect::<HashSet<usize>>()
            .into_iter()
            .collect::<Vec<usize>>()
    }

    /// Get a list of states that are destinations of given set of states and character from
    /// a transition/reverse transition map.
    fn get_set_transitions(
        map: &HashMap<(usize, usize), Vec<usize>>,
        set: &HashSet<usize>,
        c: usize,
    ) -> HashSet<usize> {
        let mut ret: HashSet<usize> = HashSet::new();
        set.into_iter().for_each(|s| {
            if let Some(arr) = map.get(&(*s, c)) {
                arr.into_iter().for_each(|v| {
                    ret.insert(*v);
                });
            }
        });
        ret
    }

    /// Replace a set with other sets from a set of sets (a partition).
    fn replace_in_partition(
        p: &mut VecDeque<HashSet<usize>>,
        replaced: &HashSet<usize>,
        replacements: Vec<HashSet<usize>>,
    ) -> bool {
        for _ in 0..p.len() {
            let next = match p.pop_front() {
                Some(next) => next,
                None => break,
            };
            if *replaced == next {
                for replacement in replacements {
                    p.push_back(replacement.clone());
                }
                return true;
            } else {
                p.push_back(next);
            }
        }
        return false;
    }
}
