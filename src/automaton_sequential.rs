use fasthash::xx::Hasher64;
use std::{
    cmp::{Ordering, Reverse},
    collections::{BinaryHeap, HashMap, HashSet, VecDeque},
    hash::BuildHasherDefault,
};

use crate::{
    automaton::Automaton,
    ubig::{CompressedUbig, Ubig},
};

type HashMapXX<K, V> = HashMap<K, V, BuildHasherDefault<Hasher64>>;

impl Automaton {
    /// Replace an element in a queue with a new element, and append the rest to the queue.
    fn replace_in_queue(
        q: &mut VecDeque<Vec<usize>>,
        replace: &Vec<usize>,
        mut replacement: VecDeque<Vec<usize>>,
    ) {
        let mut found = false;
        let mut iter = q.iter_mut();
        while let Some(next) = iter.next() {
            if Automaton::all_equal(replace, &next) {
                found = true;
                *next = replacement.pop_front().unwrap();
                break;
            }
        }
        if found {
            replacement.drain(..).for_each(|r| {
                q.push_back(r);
            });
        } else {
            q.push_back(
                replacement
                    .iter()
                    .min_by(|x, y| {
                        if x.len() < y.len() {
                            Ordering::Less
                        } else {
                            Ordering::Greater
                        }
                    })
                    .unwrap()
                    .clone(),
            );
        }
    }

    /// Compare 2 ordered vectors to check if they are equal.
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
}

/// Rabin Scott Superset Construction Algorithm - Used for determinization of NFAs.
/// Returns: (transitions vector, number of states, start states, end states).
pub fn rabin_scott_seq(
    aut: &Automaton,
) -> (Vec<(usize, usize, usize)>, usize, Vec<usize>, Vec<usize>) {
    // Rabin Scott Superset Construction Algorithm
    let mut transitions: Vec<(usize, usize, usize)> = Vec::new(); // All DFA transitions
    let mut accept_states: Vec<usize> = Vec::new(); // All accept states
    let mut num_mapper: HashMapXX<CompressedUbig, usize> = HashMapXX::default();
    let mut frontier: VecDeque<Ubig> = VecDeque::new();

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
    num_mapper.insert(start_state.clone().compress(), num_mapper.len());
    frontier.push_back(start_state.clone());

    // Graph exploration - Depth-first search
    while let Some(next) = frontier.pop_front() {
        (1..aut.alphabet + 1).for_each(|a| {
            let mut new_s = Ubig::new();
            next.get_seq().into_iter().for_each(|s| {
                (&transition_arr[a][s]).into_iter().for_each(|t| {
                    aut.add_state(&transition_arr, &mut new_s, *t);
                })
            });
            let compressed_new_s = new_s.clone().compress();

            if !num_mapper.contains_key(&compressed_new_s) {
                num_mapper.insert(compressed_new_s.clone(), num_mapper.len());
                for s in &aut.end {
                    if new_s.bit_at(s) {
                        accept_states.push(num_mapper.len() - 1);
                        break;
                    }
                }
                frontier.push_back(new_s.clone());
            }
            let next_compressed = next.clone().compress();
            transitions.push((
                *num_mapper.get(&next_compressed).unwrap(),
                a,
                *num_mapper.get(&compressed_new_s).unwrap(),
            ));
        });
    }
    return (transitions, num_mapper.len(), vec![0], accept_states);
}

/// Hopcroft algorithm for minimization of a DFA.
/// Returns a map of what state is in which leading partition, and the number of partitions.
pub fn hopcroft_algo(aut: &Automaton) -> (HashMap<usize, usize>, usize) {
    let finals: HashSet<usize> = aut.end.clone().into_iter().collect();
    let mut p: Vec<Vec<usize>> = Vec::from_iter(vec![
        (0..aut.size)
            .filter(|i| !finals.contains(i))
            .collect::<Vec<usize>>(),
        aut.end.clone(),
    ]);
    let mut q = VecDeque::from(p.clone());
    let mut state_partition_map = (0..aut.size)
        .map(|i| if !finals.contains(&i) { 0 } else { 1 })
        .collect::<Vec<usize>>();

    let rev_arr = aut.get_reverse_transition_arr();
    while let Some(set) = q.pop_front() {
        for c in 1..aut.alphabet + 1 {
            let rs = Automaton::get_set_from_transitions(&rev_arr, &set, c);
            let potential_partitions: HashSet<usize> = (&rs)
                .into_iter()
                .map(|i| state_partition_map.get(*i).unwrap().clone())
                .collect();
            potential_partitions.into_iter().for_each(|i| {
                let v = p.get(i).unwrap();
                let (diffs, ands) = Automaton::get_diff_ands(&v, &rs);
                if diffs.len() > 0 && ands.len() > 0 {
                    *p.get_mut(i).unwrap() = diffs.clone();
                    (&ands).into_iter().for_each(|j| {
                        *state_partition_map.get_mut(*j).unwrap() = p.len();
                    });
                    p.push(ands.clone());

                    Automaton::replace_in_queue(
                        &mut q,
                        p.get(i).unwrap(),
                        VecDeque::from(vec![diffs, ands]),
                    );
                }
            });
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
