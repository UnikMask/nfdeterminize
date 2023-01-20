extern crate pest;

use pest::Parser;

use crate::automaton::Automaton;

#[derive(Parser)]
#[grammar = "automaton.pest"]
struct AutomatonParser;

impl From<&str> for Automaton {
    fn from(s: &str) -> Self {
        let parse = AutomatonParser::parse(Rule::field, s);
        Automaton::empty()
    }
}
