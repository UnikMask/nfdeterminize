extern crate pest;

use pest::Parser;

use crate::automaton::{Automaton, AutomatonType};

#[derive(Parser)]
#[grammar = "automaton.pest"]
struct AutomatonParser;

impl From<&String> for Automaton {
    fn from(s: &String) -> Self {
        return match AutomatonParser::parse(Rule::automaton, s) {
            Ok(mut pairs) => {
                let mut contents = pairs.next().unwrap().into_inner();
                let mut ret = Automaton::empty();

                // Get the pairs for all the properties of the automaton.
                ret.set_automaton_type(match contents.next().unwrap().as_str() {
                    "det" => AutomatonType::Det,
                    "nondet" => AutomatonType::NonDet,
                    _ => panic!("Unreachable code!"),
                });

                // Set size and alphabet.
                ret.set_size(str::parse(contents.next().unwrap().as_str()).unwrap());
                ret.set_alphabet(str::parse(contents.next().unwrap().as_str()).unwrap());

                // Set transitions
                let mut table: Vec<(usize, usize, usize)> = Vec::new();
                for t in contents.next().unwrap().into_inner() {
                    let mut t_inner = t.into_inner();
                    table.push((
                        str::parse(t_inner.next().unwrap().as_str()).unwrap(),
                        str::parse(t_inner.next().unwrap().as_str()).unwrap(),
                        str::parse(t_inner.next().unwrap().as_str()).unwrap(),
                    ));
                }
                ret.set_transitions(table);

                // Set start states.
                let mut start: Vec<usize> = Vec::new();
                for num in contents.next().unwrap().into_inner() {
                    start.push(str::parse(num.as_str()).unwrap());
                }
                ret.set_start_states(start);

                // Set ends states.
                let mut end: Vec<usize> = Vec::new();
                for num in contents.next().unwrap().into_inner() {
                    end.push(str::parse(num.as_str()).unwrap());
                }
                ret.set_end_states(end);
                ret
            }
            Err(error) => {
                println!("{:?}", error.to_string());
                Automaton::empty()
            }
        };
    }
}
