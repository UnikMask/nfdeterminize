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

static N_THREADS: usize = 12;

////////////////
// Algorithms //
////////////////

/// Struct of variables used for each
/// superset construction worker thread
struct RabinScottWorkerThreadMembers<'a> {
    aut: &'a Automaton,
    i: usize,
    transition_arr: Vec<Vec<Vec<usize>>>,
    end: HashSet<usize>,
    accept_states: Arc<Mutex<Vec<usize>>>,
    transitions: Arc<Mutex<Vec<(usize, usize, usize)>>>,
    stop_sig: Arc<AtomicBool>,
    num_maps: Vec<Arc<Mutex<HashMapXX<Ubig, usize>>>>,
    frontiers: Vec<Arc<Mutex<VecDeque<Ubig>>>>,
    empty_tx: Sender<(bool, usize)>,
    id_state_map: Arc<Mutex<HashMapXX<usize, usize>>>,
}
/// Multithreaded version of the Rabin-Scott/superset construction algorithm.
pub fn rabin_scott_mt(
    aut: &Automaton,
) -> (Vec<(usize, usize, usize)>, usize, Vec<usize>, Vec<usize>) {
    // Shared Memory in the algorithm
    let transitions: Arc<Mutex<Vec<(usize, usize, usize)>>> = Arc::new(Mutex::new(Vec::new()));
    let accept_states: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::new()));

    // Variables belonging to threads
    let num_maps: Vec<Arc<Mutex<HashMapXX<Ubig, usize>>>> = (0..N_THREADS)
        .map(|_| Arc::new(Mutex::new(HashMapXX::default())))
        .collect();
    let frontier_c: Vec<Arc<Mutex<VecDeque<Ubig>>>> = (0..N_THREADS)
        .map(|_| Arc::new(Mutex::new(VecDeque::new())))
        .collect();
    let (empty_tx, empty_rx): (Sender<(bool, usize)>, Receiver<(bool, usize)>) = channel();
    let id_state_map: Arc<Mutex<HashMapXX<usize, usize>>> =
        Arc::new(Mutex::new(HashMapXX::default()));

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
    let start_hash = get_hash(&start_state, N_THREADS);
    id_state_map.lock().unwrap().insert(0, 0);
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

        for i in 0..N_THREADS {
            // Generate members required for a superset construction worker thread
            let tm = RabinScottWorkerThreadMembers {
                aut: &aut,
                transition_arr: transition_arr.clone(),
                i,
                end: aut.end.iter().map(|i| *i).collect(),
                accept_states: Arc::clone(&accept_states),
                transitions: Arc::clone(&transitions),
                stop_sig: Arc::clone(&stop_sig),
                num_maps: num_maps.iter().map(|a| Arc::clone(a)).collect(),
                frontiers: frontier_c.iter().map(|a| Arc::clone(a)).collect(),
                id_state_map: Arc::clone(&id_state_map),
                empty_tx: empty_tx.clone(),
            };
            s.spawn(move || rabin_scott_worker_mt(tm));
        }

        // Main thread work
        let mut thread_status: Vec<bool> = (0..N_THREADS).map(|_| false).collect();
        let mut count: i64 = N_THREADS as i64;
        while !stop_sig.load(Ordering::Relaxed) {
            if let Ok((empty, id)) = empty_rx.recv() {
                if thread_status[id] != empty {
                    count += if empty { -1 } else { 1 };
                    thread_status[id] = empty;
                    if count == 0 {
                        stop_sig.store(true, Ordering::Relaxed);
                    }
                }
            }
        }
    });
    return (
        transitions.lock().unwrap().to_vec(),
        id_state_map.lock().unwrap().len(),
        vec![0],
        accept_states.lock().unwrap().to_vec(),
    );
}

////////////////////
// Worker Threads //
////////////////////

/// Worker thread behaviour during superset construction
fn rabin_scott_worker_mt(tm: RabinScottWorkerThreadMembers) {
    let mut local_transitions: Vec<(usize, usize, usize)> = Vec::new();
    let mut local_accepts: Vec<usize> = Vec::new();
    let mut empty = false;
    loop {
        let next: Option<Ubig>;
        let mut f = tm.frontiers[tm.i].lock().unwrap();
        next = f.pop_front();
        if let Some(next) = next {
            if empty {
                tm.empty_tx.send((false, tm.i)).unwrap();
                empty = false;
            }
            drop(f);
            for a in 1..&tm.aut.alphabet + 1 {
                let mut new_s = Ubig::new();
                for s in next.get_seq() {
                    tm.transition_arr[a][s].iter().for_each(|t| {
                        tm.aut.add_state(&tm.transition_arr, &mut new_s, *t);
                    });
                }
                // Get hashes for given state and new state
                let hash_next = get_hash(&next, N_THREADS);
                let hash_new = get_hash(&new_s, N_THREADS);

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
                    tm.frontiers[hash_new].lock().unwrap().push_back(new_s);
                }
            }
        } else {
            if !empty {
                empty = true;
                tm.empty_tx.send((true, tm.i)).unwrap();
                continue;
            } else if tm.stop_sig.load(Ordering::Relaxed) {
                tm.num_maps[tm.i]
                    .lock()
                    .unwrap()
                    .drain()
                    .for_each(|(_, id)| {
                        let mut id_sm_locked = tm.id_state_map.lock().unwrap();
                        let len = id_sm_locked.len();
                        id_sm_locked.insert(id, len);
                    });
                tm.transitions.lock().unwrap().append(
                    &mut local_transitions
                        .iter()
                        .map(|(s, a, e)| {
                            let id_sm_locked = tm.id_state_map.lock().unwrap();
                            (
                                *id_sm_locked.get(s).unwrap(),
                                *a,
                                *id_sm_locked.get(e).unwrap(),
                            )
                        })
                        .collect::<Vec<(usize, usize, usize)>>(),
                );
                tm.accept_states.lock().unwrap().append(
                    &mut local_accepts
                        .iter()
                        .map(|s| *tm.id_state_map.lock().unwrap().get(s).unwrap())
                        .collect(),
                );
                break;
            }
        }
    }
}

//////////////////////
// Helper Functions //
//////////////////////

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
