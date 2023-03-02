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

pub fn get_two_stack_aut(n1: usize, n2: usize) -> Automaton {
    #[derive(Debug, PartialEq, Eq, Hash, Clone)]
    struct SnSState {
        stack1: VecDeque<usize>,
        stack2: VecDeque<usize>,
    }

    let start_state = SnSState {
        stack1: VecDeque::new(),
        stack2: VecDeque::new(),
    };
    let mut count = 1;
    let mut states_set: HashMap<SnSState, usize> = HashMap::from([(start_state.clone(), 0)]);
    let mut transitions: Vec<(usize, usize, usize)> = Vec::new();
    let mut q: VecDeque<SnSState> = VecDeque::from([start_state]);

    // Closure that adds given transition and adds new state to queue if not in states set.
    let mut resolve_new_state =
        |new_state: SnSState, l: usize, old_state: &SnSState, queue: &mut VecDeque<SnSState>| {
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
        if s.stack2.len() > 0 {
            let a = *s.stack2.front().unwrap();
            let mut new_state = SnSState {
                stack1: s.stack1.iter().map(|l| decrease_ranks(*l, a)).collect(),
                stack2: s.stack2.iter().map(|l| decrease_ranks(*l, a)).collect(),
            };
            new_state.stack2.pop_front();
            resolve_new_state(new_state, a, &s, &mut q);
        }

        if s.stack2.len() < n2 && s.stack1.len() > 0 {
            let mut new_state = s.clone();
            new_state
                .stack2
                .push_front(new_state.stack1.pop_front().unwrap());
            resolve_new_state(new_state, 0, &s, &mut q);
        }

        if s.stack1.len() < n1 {
            let mut new_state = s.clone();
            new_state
                .stack1
                .push_front(s.stack1.len() + s.stack2.len() + 1);
            resolve_new_state(new_state, 0, &s, &mut q);
        }
    }

    Automaton::new(
        AutomatonType::NonDet,
        count,
        n1 + n2 - 1,
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
