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

use crate::{automaton::Automaton, ubig::Ubig};

type HashMapXX<K, V> = HashMap<K, V, BuildHasherDefault<Hasher64>>;

static N_THREADS: usize = 12;

pub fn rabin_scott_mt(
    aut: &Automaton,
) -> (Vec<(usize, usize, usize)>, usize, Vec<usize>, Vec<usize>) {
    // Shared Memory in the algorithm
    let transitions: Arc<Mutex<Vec<(usize, usize, usize)>>> = Arc::new(Mutex::new(Vec::new()));
    let accept_states: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::new()));
    let num_mapper: Arc<Mutex<HashMapXX<Ubig, usize>>> = Arc::new(Mutex::new(HashMapXX::default()));

    // Variables belonging to threads
    let frontier_c: Vec<Arc<Mutex<VecDeque<Ubig>>>> = (0..N_THREADS)
        .map(|_| Arc::new(Mutex::new(VecDeque::new())))
        .collect();
    let (empty_tx, empty_rx): (Sender<(bool, usize)>, Receiver<(bool, usize)>) = channel();

    // Select start state from all start states in the non deterministic automata.
    let transition_arr = aut.get_transition_array();
    let mut start_state = Ubig::new();
    (&aut.start)
        .into_iter()
        .for_each(|s| aut.add_state(&transition_arr, &mut start_state, *s));
    for s in &aut.end {
        if start_state.bit_at(s) {
            accept_states.lock().unwrap().push(0);
            break;
        }
    }
    num_mapper.lock().unwrap().insert(start_state.clone(), 0);
    frontier_c[0].lock().unwrap().push_back(start_state.clone());

    thread::scope(|s| {
        let stop_sig: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

        for i in 0..N_THREADS {
            // Copied automaton properties
            let transition_arr = transition_arr.clone();

            // Mutex clones
            let end: HashSet<usize> = aut.end.iter().map(|i| *i).collect();
            let accept_states = Arc::clone(&accept_states);
            let transitions = Arc::clone(&transitions);
            let stop_sig = Arc::clone(&stop_sig);
            let num_mapper = Arc::clone(&num_mapper);
            let frontiers: Vec<Arc<Mutex<VecDeque<Ubig>>>> =
                frontier_c.iter().map(|a| Arc::clone(a)).collect();

            let empty_tx = empty_tx.clone();
            let local_transitions: Vec<(usize, usize, usize)> = Vec::new();
            let local_accepts: Vec<usize> = Vec::new();
            s.spawn(move || {
                rabin_scott_worker_mt(
                    aut,
                    i,
                    transition_arr,
                    end,
                    accept_states,
                    transitions,
                    stop_sig,
                    num_mapper,
                    frontiers,
                    empty_tx,
                    local_transitions,
                    local_accepts,
                )
            });
        }
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
    return (
        transitions.lock().unwrap().to_vec(),
        num_mapper.lock().unwrap().len(),
        vec![0],
        accept_states.lock().unwrap().to_vec(),
    );
}

fn rabin_scott_worker_mt(
    aut: &Automaton,
    i: usize,
    transition_arr: Vec<Vec<Vec<usize>>>,
    end: HashSet<usize>,
    accept_states: Arc<Mutex<Vec<usize>>>,
    transitions: Arc<Mutex<Vec<(usize, usize, usize)>>>,
    stop_sig: Arc<AtomicBool>,
    num_mapper: Arc<Mutex<HashMapXX<Ubig, usize>>>,
    frontiers: Vec<Arc<Mutex<VecDeque<Ubig>>>>,
    empty_tx: Sender<(bool, usize)>,
    mut local_transitions: Vec<(usize, usize, usize)>,
    mut local_accepts: Vec<usize>,
) {
    let mut empty = false;
    loop {
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
            for a in 1..&aut.alphabet + 1 {
                let mut new_s = Ubig::new();
                for s in next.get_seq() {
                    transition_arr[a][s].iter().for_each(|t| {
                        aut.add_state(&transition_arr, &mut new_s, *t);
                    });
                }

                // Get shared num mapper HashMap and perform ops on shared memory.
                let mut num_mapper_l = num_mapper.lock().unwrap();
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
                    frontiers[hash].lock().unwrap().push_back(new_s);
                }
            }
        }
    }
}
