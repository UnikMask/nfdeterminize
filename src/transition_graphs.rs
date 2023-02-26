use std::collections::{HashSet, VecDeque};

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
    let mut states_set: HashSet<BnSState> = HashSet::from([start_state.clone()]);
    let mut q: VecDeque<BnSState> = VecDeque::from([start_state]);

    while let Some(s) = q.pop_front() {
        if s.stack.len() != 0 {
            let a = s.stack[0];
            let mut new_state = BnSState {
                buffer: s.buffer.iter().map(|l| decrease_ranks(*l, a)).collect(),
                stack: s.stack.iter().map(|l| decrease_ranks(*l, a)).collect(),
            };
            new_state.stack.pop_front();
            if !states_set.contains(&new_state) {
                states_set.insert(new_state.clone());
                q.push_back(new_state);
            }
        }

        if s.buffer.len() != 0 && s.stack.len() < n {
            s.buffer.iter().for_each(|l| {
                let new_state = BnSState {
                    buffer: s
                        .buffer
                        .iter()
                        .filter_map(|i| if i != l { Some(*l) } else { None })
                        .collect(),
                    stack: s.stack.clone(),
                };
                if !states_set.contains(&new_state) {
                    states_set.insert(new_state.clone());
                    q.push_back(new_state);
                }
            });
        }

        if s.buffer.len() < b {
            s.buffer.iter().for_each(|l| {
                let mut new_state = s.clone();
                new_state.buffer.push(s.buffer.len() + s.stack.len() + 1);
                if !states_set.contains(&new_state) {
                    states_set.insert(new_state.clone());
                    q.push_back(new_state);
                }
            });
        }
    }

    return Automaton::empty();
}

fn decrease_ranks(l: usize, a: usize) -> usize {
    if l > a {
        l - 1
    } else {
        l
    }
}
