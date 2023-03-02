use std::collections::{HashMap, VecDeque};

use crate::automaton::{Automaton, AutomatonType};

pub fn get_buffer_and_stack_aut(b: usize, n: usize) -> Automaton {
    #[derive(Debug, PartialEq, Eq, Hash, Clone)]
    struct BnSState {
        buffer: Vec<usize>,
        stack: VecDeque<usize>,
    }

    let start_state = BnSState {
        buffer: vec![],
        stack: VecDeque::new(),
    };
    let mut count = 1;
    let mut states_set: HashMap<BnSState, usize> = HashMap::from([(start_state.clone(), 0)]);
    let mut transitions: Vec<(usize, usize, usize)> = Vec::new();
    let mut q: VecDeque<BnSState> = VecDeque::from([start_state]);

    // Closure that adds given transition and adds new state to queue if not in states set.
    let mut resolve_new_state =
        |new_state: BnSState, l: usize, old_state: &BnSState, queue: &mut VecDeque<BnSState>| {
            if !states_set.contains_key(&new_state) {
                states_set.insert(new_state.clone(), count);
                queue.push_back(new_state.clone());
                count += 1;
            }
            transitions.push((
                *states_set.get(old_state).unwrap(),
                l,
                *states_set.get(&new_state).unwrap(),
            ));
        };

    while let Some(s) = q.pop_front() {
        // Move token from stack into output stream
        if s.stack.len() != 0 {
            let a = *s.stack.front().unwrap();
            let mut new_state = BnSState {
                buffer: s.buffer.iter().map(|l| decrease_ranks(*l, a)).collect(),
                stack: s.stack.iter().map(|l| decrease_ranks(*l, a)).collect(),
            };
            new_state.stack.pop_front();
            resolve_new_state(new_state, a, &s, &mut q);
        }

        // Move token from buffer into stack
        if s.buffer.len() != 0 && s.stack.len() < n {
            s.buffer.iter().for_each(|l| {
                let mut new_state = BnSState {
                    buffer: s
                        .buffer
                        .iter()
                        .filter_map(|i| if i != l { Some(*i) } else { None })
                        .collect(),
                    stack: s.stack.clone(),
                };
                new_state.stack.push_front(*l);
                resolve_new_state(new_state, 0, &s, &mut q);
            });
        }

        // Move token from input stream into buffer.
        if s.buffer.len() < b {
            let mut new_state = s.clone();
            new_state.buffer.push(s.buffer.len() + s.stack.len() + 1);
            resolve_new_state(new_state, 0, &s, &mut q);
        }
    }
    Automaton::new(
        AutomatonType::NonDet,
        count,
        b + n - 1,
        transitions,
        Vec::from([0]),
        Vec::from([0]),
    )
}

fn decrease_ranks(l: usize, a: usize) -> usize {
    if l > a {
        l - 1
    } else {
        l
    }
}
