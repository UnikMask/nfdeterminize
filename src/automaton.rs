use fasthash::xx::{self, Hasher64};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::{BuildHasherDefault, Hasher},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use crate::ubig::Ubig;
type HashMapXX<K, V> = HashMap<K, V, BuildHasherDefault<Hasher64>>;

static N_THREADS: usize = 6;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AutomatonType {
    Det,
    NonDet,
}

// Structure for an automaton.
#[derive(Debug, Eq, Clone, PartialEq)]
pub struct Automaton {
    pub automaton_type: AutomatonType,
    pub size: usize,
    pub alphabet: usize,
    pub table: Vec<(usize, usize, usize)>,
    pub start: Vec<usize>,
    pub end: Vec<usize>,
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

    ////////////////
    // Algorithms //
    ////////////////

    /// Rabin Scott Superset Construction Algorithm - Used for determinization of NFAs.
    /// Returns: (transitions vector, number of states, start states, end states).
    fn rabin_scott(&self) -> (Vec<(usize, usize, usize)>, usize, Vec<usize>, Vec<usize>) {
        // Shared Memory in the algorithm
        let transitions: Arc<Mutex<Vec<(usize, usize, usize)>>> = Arc::new(Mutex::new(Vec::new()));
        let accept_states: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::new()));
        let num_mapper: Arc<Mutex<HashMapXX<Ubig, usize>>> =
            Arc::new(Mutex::new(HashMapXX::default()));

        // Variables belonging to threads
        let frontier_c: Vec<Arc<Mutex<VecDeque<Ubig>>>> = (0..N_THREADS)
            .map(|_| Arc::new(Mutex::new(VecDeque::new())))
            .collect();
        let (empty_tx, empty_rx): (Sender<(bool, usize)>, Receiver<(bool, usize)>) = channel();

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
        thread::scope(|s| {
            let stop_sig: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

            for i in 0..N_THREADS {
                // Copied automaton properties
                let transition_arr = transition_arr.clone();

                // Mutex clones
                let end: HashSet<usize> = self.end.iter().map(|i| *i).collect();
                let accept_states = Arc::clone(&accept_states);
                let transitions = Arc::clone(&transitions);
                let stop_sig = Arc::clone(&stop_sig);
                let num_mapper = Arc::clone(&num_mapper);
                let frontiers: Vec<Arc<Mutex<VecDeque<Ubig>>>> =
                    frontier_c.iter().map(|a| Arc::clone(a)).collect();

                let empty_tx = empty_tx.clone();
                let mut local_transitions: Vec<(usize, usize, usize)> = Vec::new();
                let mut local_accepts: Vec<usize> = Vec::new();
                let mut empty = false;

                s.spawn(move || loop {
                    let next: Option<Ubig>;
                    let mut f = frontiers[i].lock().unwrap();
                    next = f.pop_front();
                    if let None = next {
                        if !empty {
                            empty = true;
                            empty_tx.send((true, i)).unwrap();
                            continue;
                        } else if stop_sig.load(Ordering::Relaxed) {
                            transitions.lock().unwrap().append(&mut local_transitions);
                            accept_states.lock().unwrap().append(&mut local_accepts);
                            break;
                        }
                    } else if let Some(next) = next {
                        if empty {
                            empty_tx.send((false, i)).unwrap();
                            empty = false;
                        }
                        drop(f);
                        for a in 1..&self.alphabet + 1 {
                            let mut new_s = Ubig::new();
                            for s in next.get_seq() {
                                transition_arr[a][s].iter().for_each(|t| {
                                    Automaton::add_state(&transition_arr, &mut new_s, *t);
                                });
                            }

                            // Get shared num mapper HashMap and perform ops on shared memory.
                            //println!("Thread {i:?} waits to access num mapper...");
                            let mut num_mapper_l = num_mapper.lock().unwrap();
                            //println!("Thread {i:?} is using num mapper...");
                            let num = num_mapper_l.len();
                            let is_new = !num_mapper_l.contains_key(&new_s);
                            if is_new {
                                num_mapper_l.insert(new_s.clone(), num);
                            }
                            local_transitions.push((
                                *num_mapper_l.get(&next).unwrap(),
                                a,
                                *num_mapper_l.get(&new_s).unwrap(),
                            ));
                            drop(num_mapper_l);
                            //println!("Thread {i:?} has dropped num mapper!");

                            if is_new {
                                let mut hasher = xx::Hasher64::default();
                                hasher.write(&new_s.num);
                                let hash = (hasher.finish() as usize) % N_THREADS;
                                for s in new_s.get_seq().iter() {
                                    if end.contains(s) {
                                        local_accepts.push(num);
                                        break;
                                    }
                                }
                                //println!("Thread {i:?} tries to access frontier {hash:?}...");
                                frontiers[hash].lock().unwrap().push_back(new_s);
                                //println!("New state added to thread {hash:?} frontier");
                            }
                        }
                    }
                });
            }
            // Keep track
            let mut thread_status: Vec<bool> = (0..N_THREADS).map(|_| false).collect();
            let mut count: i64 = N_THREADS as i64;
            while !stop_sig.load(Ordering::Relaxed) {
                if let Ok((empty, id)) = empty_rx.recv() {
                    if thread_status[id] != empty {
                        count += if empty { -1 } else { 1 };
                        thread_status[id] = empty;
                        if count == 0 {
                            //println!("Kill signal sent!");
                            stop_sig.store(true, Ordering::Relaxed);
                        }
                    }
                }
            }
        });

        // Graph exploration - Depth-first search
        return (
            transitions.lock().unwrap().to_vec(),
            num_mapper.lock().unwrap().len(),
            vec![0],
            accept_states.lock().unwrap().to_vec(),
        );
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
        let rev_arr = self.get_reverse_transition_arr();

        // Iterate until the partition frontier is empty
        while let Some(set) = p_frontier.pop_front() {
            // Iterate over each input symbol
            for c in 1..self.alphabet + 1 {
                let rs = Automaton::get_set_transitions(&rev_arr, &set, c);

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
    fn get_set_transitions(
        arr: &Vec<Vec<Vec<usize>>>,
        set: &HashSet<usize>,
        c: usize,
    ) -> HashSet<usize> {
        let mut ret: HashSet<usize> = HashSet::new();
        set.iter().for_each(|s| {
            (&arr[c][*s]).into_iter().for_each(|f| {
                ret.insert(*f);
            });
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
                replacements
                    .into_iter()
                    .for_each(|r| p.push_back(r.clone()));
                return true;
            } else {
                p.push_back(next);
            }
        }
        false
    }
}
