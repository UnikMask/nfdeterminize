use fasthash::xx::Hasher64;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::BuildHasherDefault,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use crate::ubig::Ubig;
type HashMapXX<K, V> = HashMap<K, V, BuildHasherDefault<Hasher64>>;

static N_THREADS: usize = 12;

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
        // Shared Memory in the algorithm
        let mut transitions: Arc<Mutex<Vec<(usize, usize, usize)>>> =
            Arc::new(Mutex::new(Vec::new()));
        let mut accept_states: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::new()));
        let mut num_mapper: Arc<Mutex<HashMapXX<Ubig, usize>>> =
            Arc::new(Mutex::new(HashMapXX::default()));

        // Variables belonging to threads
        let mut frontier_c: Vec<Arc<Mutex<VecDeque<Ubig>>>> = (0..N_THREADS)
            .map(|_| Arc::new(Mutex::new(VecDeque::new())))
            .collect();
        let empty_chan: Vec<(Sender<(bool, i32)>, Receiver<(bool, i32)>)> =
            (0..N_THREADS).map(|_| channel()).collect();

        // Select start state from all start states in the non deterministic automata.
        let transition_arr = self.get_transition_array();
        let mut start_state = Ubig::new();
        (&self.start)
            .into_iter()
            .for_each(|s| Automaton::add_state(&transition_arr, &mut start_state, *s));
        for s in &self.end {
            if start_state.bit_at(s) {
                accept_states.lock().unwrap().push(0);
                break;
            }
        }
        num_mapper.lock().unwrap().insert(start_state.clone(), 0);
        frontier_c[0].lock().unwrap().push_back(start_state.clone());

        // Multi-threaded code
        let mut handles = vec![];
        for i in 0..N_THREADS {
            let (alphabet, start, end) = (self.alphabet, self.start.clone(), self.end.clone());
            let local_arr = transition_arr.clone();
            let num_mapper = Arc::clone(&num_mapper);
            let frontiers: Vec<Arc<Mutex<VecDeque<Ubig>>>> =
                frontier_c.iter().map(|a| Arc::clone(a)).collect();

            handles.push(thread::spawn(move || loop {
                if let Some(next) = frontiers[i].lock().unwrap().pop_front() {
                    for a in 1..alphabet + 1 {
                        let mut new_s = Ubig::new();
                        for s in next.get_seq() {
                            &local_arr[a][s].iter().for_each(|t| {
                                Automaton::add_state(&transition_arr, &mut new_s, *t)
                            });
                        }
                        let mut num_mapper_l = num_mapper.lock().unwrap();
                        let num = num_mapper_l.len();
                        if !num_mapper_l.contains_key(&new_s) {
                            num_mapper_l.insert(new_s.clone(), num);
                        }
                        transitions.lock().unwrap().push((
                            *num_mapper_l.get(&new_s).unwrap(),
                            a,
                            *num_mapper_l.get(&next).unwrap(),
                        ));
                        end.iter().for_each(|s| {
                            if new_s.bit_at(s) {
                                accept_states.lock().unwrap().push(num)
                            }
                        });
                    }
                } else {
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }

        // Graph exploration - Depth-first search
        while let Some(next) = frontier.pop_front() {}
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
    fn add_state(arr: &Vec<Vec<Vec<usize>>>, num: &mut Ubig, bit: usize) {
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

    /// Get a hashmap of leading states from a given letter and original state.
    fn get_transition_array(&self) -> Vec<Vec<Vec<usize>>> {
        //let mut map: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
        let mut arr: Vec<Vec<Vec<usize>>> = Vec::new();
        for _ in 0..(self.alphabet + 1) {
            let mut alphabet_arr: Vec<Vec<usize>> = Vec::new();
            for _ in 0..self.size + 1 {
                alphabet_arr.push(Vec::new());
            }
            arr.push(alphabet_arr);
        }

        for transition in &self.table {
            arr[transition.1][transition.0].push(transition.2);
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
