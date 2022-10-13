use std::collections::VecDeque;

use crate::trie::NodeTrie;

#[derive(Debug)]
pub enum AutomatonType {
    Det,
    NonDet,
}

// Structure for an automaton.
#[derive(Debug)]
pub struct Automaton {
    automaton_type: AutomatonType,
    size: usize,
    alphabet: usize,
    table: Vec<(usize, usize, usize)>,
    start: Vec<usize>,
    end: Vec<usize>,
}

impl Automaton {
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
    // Determinize an NFA.
    fn determinize(&self) -> Automaton {
        let mut ret = Automaton {
            automaton_type: AutomatonType::Det,
            size: self.size,
            alphabet: self.alphabet,
            table: Vec::new(),
            start: Vec::new(),
            end: Vec::new(),
        };

        // Return same automaton as it already is deterministic.
        if let AutomatonType::Det = self.automaton_type  {
            ret.table = self.table.clone();
            ret.start = self.start.clone();
            ret.end  = self.end.clone();
            return ret;
        }

        // Rabin Scott Superset Construction Algorithm
        let mut state_trie = NodeTrie::new(&self.size);
        let frontier: VecDeque<Vec<usize>> = VecDeque::new();

        if state_trie.push_seq(self.start.clone()) == true {

        }





        return ret;
    }

    // Minimize a DFA.
    fn minimize(&self) -> Automaton {
        return Automaton {
            automaton_type: AutomatonType::Det,
            size: self.size,
            alphabet: self.alphabet,
            table: Vec::new(),
            start: Vec::new(),
            end: Vec::new(),
        };
    }
}
