use crate::automaton::Automaton;

use std::collections::{HashSet, VecDeque};

pub fn get_buffer_and_stack_automaton(b: usize, n: usize) -> Automaton {
    #[derive(PartialEq, Eq, Hash, Clone)]
    struct BnsState {
        buffer: Vec<usize>,
        stack: VecDeque<usize>,
    }

    let empty_state = BnsState {
        buffer: Vec::new(),
        stack: VecDeque::new(),
    };
    let mut states_set: HashSet<BnsState> = HashSet::from([empty_state]);

    // Initialise queue with start state - empty buffer and stack.
    let mut q: VecDeque<BnsState> = VecDeque::from([empty_state]);

    // Queue in new states in automata until there is none left.
    while let Some(s) = q.pop_front() {
        // Search for transitions which push a token to the output.
        if s.stack.len() != 0 {
            let a = s.stack[s.stack.len()];
            let mut new_state = BnsState {
                buffer: s
                    .stack
                    .iter()
                    .map(|l| if *l > a { *l - 1 } else { *l })
                    .collect::<Vec<usize>>(),
                stack: s
                    .stack
                    .iter()
                    .map(|l| if *l > a { *l - 1 } else { *l })
                    .collect::<VecDeque<usize>>(),
            };
            new_state.stack.pop_front();

            if !states_set.contains(&new_state) {
                q.push_back(new_state.clone());
                states_set.insert(new_state);
            }
        }

        // Look at epsilon-transitions from transferring a buffer token to the stack.
        if s.buffer.len() != 0 && s.stack.len() < n {
            for l in s.buffer {
                // TODO buffer-to-stack transitions
            }
        }

        // TODO Input to buffer transitions
    }
    return Automaton::empty();
}
