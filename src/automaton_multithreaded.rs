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
use uuid::Uuid;

use crate::{automaton::Automaton, ubig::Ubig};

type HashMapXX<K, V> = HashMap<K, V, BuildHasherDefault<Hasher64>>;
type Transition = (usize, usize, usize);

////////////////
// Algorithms //
////////////////

/// Struct of variables used for each
/// superset construction worker thread
struct RabinScottWorkerThreadMembers<'a> {
    aut: &'a Automaton,
    i: usize,
    n_threads: usize,
    transition_arr: Vec<Vec<Vec<usize>>>,
    end: HashSet<usize>,
    stop_sig: Arc<AtomicBool>,
    num_maps: Vec<Arc<Mutex<HashMapXX<Ubig, usize>>>>,
    frontiers: Vec<Arc<Mutex<VecDeque<Ubig>>>>,
    frontier_empty_tx: Sender<(bool, usize)>,
    reduce_tx: Sender<usize>,
    transition_tx: Sender<Transition>,
    accept_tx: Sender<usize>,
}

/// Multithreaded version of the Rabin-Scott/superset construction algorithm.
pub fn rabin_scott_mt(
    aut: &Automaton,
    n_threads: usize,
) -> (Vec<Transition>, usize, Vec<usize>, Vec<usize>) {
    // Shared Memory in the algorithm
    let mut transitions: Vec<Transition> = Vec::new();
    let mut accept_states: Vec<usize> = Vec::new();
    let mut id_state_map: HashMapXX<usize, usize> = HashMapXX::default();

    // Variables belonging to threads
    let num_maps: Vec<Arc<Mutex<HashMapXX<Ubig, usize>>>> = (0..n_threads)
        .map(|_| Arc::new(Mutex::new(HashMapXX::default())))
        .collect();
    let frontier_c: Vec<Arc<Mutex<VecDeque<Ubig>>>> = (0..n_threads)
        .map(|_| Arc::new(Mutex::new(VecDeque::new())))
        .collect();
    let (frontier_empty_tx, frontier_empty_rx): (Sender<(bool, usize)>, Receiver<(bool, usize)>) =
        channel();
    let (reduce_tx, reduce_rx): (Sender<usize>, Receiver<usize>) = channel();
    let (transition_tx, transition_rx): (Sender<Transition>, Receiver<Transition>) = channel();
    let (accept_tx, accept_rx): (Sender<usize>, Receiver<usize>) = channel();

    // Select start state from all start states in the non deterministic automata.
    let transition_arr = aut.get_transition_array();
    let mut start_state = Ubig::new();
    (&aut.start)
        .into_iter()
        .for_each(|s| aut.add_state(&transition_arr, &mut start_state, *s));
    for s in &aut.end {
        if start_state.bit_at(s) {
            accept_states.push(0);
            break;
        }
    }
    let start_hash = get_hash(&start_state, n_threads);
    id_state_map.insert(0, 0);
    num_maps[start_hash]
        .lock()
        .unwrap()
        .insert(start_state.clone(), 0);
    frontier_c[start_hash]
        .lock()
        .unwrap()
        .push_back(start_state.clone());

    thread::scope(|s| {
        let stop_sig: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

        // Initialise worker thread vars and spawn worker threads
        for i in 0..n_threads {
            let tm = RabinScottWorkerThreadMembers {
                aut: &aut,
                transition_arr: transition_arr.clone(),
                i,
                n_threads,
                end: aut.end.iter().map(|i| *i).collect(),
                stop_sig: Arc::clone(&stop_sig),
                num_maps: num_maps.iter().map(|a| Arc::clone(a)).collect(),
                frontiers: frontier_c.iter().map(|a| Arc::clone(a)).collect(),
                transition_tx: transition_tx.clone(),
                reduce_tx: reduce_tx.clone(),
                accept_tx: accept_tx.clone(),
                frontier_empty_tx: frontier_empty_tx.clone(),
            };
            s.spawn(move || rabin_scott_worker_mt(tm));
        }

        // Main thread work
        let mut thread_status: Vec<bool> = (0..n_threads).map(|_| false).collect();
        let mut count = n_threads as i64;
        while !stop_sig.load(Ordering::Relaxed) {
            if let Ok((empty, id)) = frontier_empty_rx.recv() {
                if thread_status[id] != empty {
                    count += if empty { -1 } else { 1 };
                    thread_status[id] = empty;
                    if count == 0 {
                        stop_sig.store(true, Ordering::Relaxed);
                    }
                }
            }
        }
        count = n_threads as i64;
        let mut threads_reduces = (0..n_threads).map(|_| false).collect::<Vec<bool>>();
        while count > 0 {
            if let Ok(tr) = transition_rx.try_recv() {
                add_transition(tr, &mut transitions, &mut id_state_map);
            }
            if let Ok(s) = accept_rx.try_recv() {
                add_accept(s, &mut accept_states, &mut id_state_map);
            }
            if let Ok(thread) = reduce_rx.try_recv() {
                if !threads_reduces[thread] {
                    count -= 1;
                    threads_reduces[thread] = true;
                }
            }
        }
        transition_rx
            .try_iter()
            .for_each(|tr| add_transition(tr, &mut transitions, &mut id_state_map));
        accept_rx
            .try_iter()
            .for_each(|s| add_accept(s, &mut accept_states, &mut id_state_map));
    });
    return (transitions, id_state_map.len(), vec![0], accept_states);
}

////////////////////
// Worker Threads //
////////////////////

/// Worker thread behaviour during superset construction
fn rabin_scott_worker_mt(tm: RabinScottWorkerThreadMembers) {
    let mut local_transitions: Vec<Transition> = Vec::new();
    let mut local_accepts: Vec<usize> = Vec::new();
    let mut frontier_empty = false;
    loop {
        let next: Option<Ubig>;
        let mut f = tm.frontiers[tm.i].lock().unwrap();
        next = f.pop_front();
        if let Some(next) = next {
            if frontier_empty {
                tm.frontier_empty_tx.send((false, tm.i)).unwrap();
                frontier_empty = false;
            }
            drop(f);
            rabin_scott_worker_mt_explore_loop(
                &tm,
                next,
                &mut local_transitions,
                &mut local_accepts,
            );
        } else if !frontier_empty {
            frontier_empty = true;
            tm.frontier_empty_tx.send((true, tm.i)).unwrap();
            continue;
        } else if tm.stop_sig.load(Ordering::Relaxed) {
            local_transitions
                .drain(..)
                .for_each(|(s, a, e)| tm.transition_tx.send((s, a, e)).unwrap());
            local_accepts
                .drain(..)
                .for_each(|s| tm.accept_tx.send(s).unwrap());
            tm.reduce_tx.send(tm.i).unwrap();
            break;
        }
    }
}

/// Explore-state loop of a superset construction worker thread -
/// Main component of superset construction.
fn rabin_scott_worker_mt_explore_loop(
    tm: &RabinScottWorkerThreadMembers,
    next: Ubig,
    local_transitions: &mut Vec<Transition>,
    local_accepts: &mut Vec<usize>,
) {
    for a in 1..&tm.aut.alphabet + 1 {
        let mut new_s = Ubig::new();
        for s in next.get_seq() {
            tm.transition_arr[a][s].iter().for_each(|t| {
                tm.aut.add_state(&tm.transition_arr, &mut new_s, *t);
            });
        }
        // Get hashes for given state and new state
        let hash_next = get_hash(&next, tm.n_threads);
        let hash_new = get_hash(&new_s, tm.n_threads);

        // Get shared num mapper HashMap and perform ops on shared memory.
        let mut num_map_new = tm.num_maps[hash_new].lock().unwrap();
        let is_new = !num_map_new.contains_key(&new_s);
        if is_new {
            num_map_new.insert(new_s.clone(), get_new_id());
        }
        let id_new = *num_map_new.get(&new_s).unwrap();
        drop(num_map_new);

        let id_next = *tm.num_maps[hash_next].lock().unwrap().get(&next).unwrap();
        local_transitions.push((id_next, a, id_new));
        if is_new {
            for s in new_s.get_seq().iter() {
                if tm.end.contains(s) {
                    local_accepts.push(id_new);
                    break;
                }
            }
            let mut new_frontier = tm.frontiers[hash_new].lock().unwrap();
            if new_frontier.len() == 0 {
                tm.frontier_empty_tx.send((false, hash_new)).unwrap();
            }
            new_frontier.push_back(new_s);
        }
    }
}

//////////////////////
// Helper Functions //
//////////////////////

fn add_transition(
    transition: Transition,
    transitions: &mut Vec<Transition>,
    id_state_map: &mut HashMapXX<usize, usize>,
) {
    let (s, a, e) = transition;
    let new_states = vec![s.clone(), e.clone()];
    new_states.iter().for_each(|ns| {
        if !id_state_map.contains_key(ns) {
            id_state_map.insert(*ns, id_state_map.len());
        }
    });
    transitions.push((
        *id_state_map.get(&s).unwrap(),
        a,
        *id_state_map.get(&e).unwrap(),
    ));
}

fn add_accept(s: usize, accepts: &mut Vec<usize>, id_state_map: &mut HashMapXX<usize, usize>) {
    if !id_state_map.contains_key(&s) {
        id_state_map.insert(s.clone(), id_state_map.len());
    }
    accepts.push(*id_state_map.get(&s).unwrap());
}

/// Get the hash of a Ubig
fn get_hash(u: &Ubig, n: usize) -> usize {
    let mut hasher = xx::Hasher64::default();
    hasher.write(&u.num);
    (hasher.finish() as usize) % n
}

/// Get a random new ID to assign to a state
fn get_new_id() -> usize {
    Uuid::new_v4().as_u128() as usize
}
