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
                // Get contents of automaton from automaton -> core -> inner
                let mut contents = pairs
                    .next()
                    .unwrap()
                    .into_inner()
                    .next()
                    .unwrap()
                    .into_inner();
                let mut ret = Automaton::empty();

                // Get the pairs for all the properties of the automaton.
                ret.set_automaton_type(match contents.next().unwrap().as_str() {
                    "det" => AutomatonType::Det,
                    "nondet" => AutomatonType::NonDet,
                    "epsilon" => AutomatonType::NonDet,
                    _ => panic!("Unreachable code!"),
                });

                // Set size and alphabet.
                ret.set_size(str::parse(contents.next().unwrap().as_str()).unwrap());
                let alphabet_parse = contents.next().unwrap();
                match alphabet_parse.as_rule() {
                    Rule::LETTER_STR => {
                        ret.set_alphabet(alphabet_parse.as_str().len());
                    }
                    Rule::NUM => {
                        ret.set_alphabet(str::parse(alphabet_parse.as_str()).unwrap());
                    }
                    _ => {
                        panic!("Unreachable code!");
                    }
                }

                // Set transitions
                let mut tuple_table: Vec<(usize, usize, usize)> = Vec::new();
                for (i_a, a) in contents
                    .next()
                    .unwrap()
                    .into_inner()
                    .into_iter()
                    .enumerate()
                {
                    let i_with_eps = match alphabet_parse.as_rule() {
                        Rule::LETTER_STR => match alphabet_parse.as_str().chars().nth(i_a) {
                            Some(letter) => match letter {
                                '@' => 0,
                                _ => i_a + 1,
                            },
                            None => i_a + 1,
                        },
                        _ => i_a + 1,
                    };
                    for (i_s, s_in) in a.into_inner().into_iter().enumerate() {
                        for s_out in s_in.into_inner() {
                            tuple_table.push((
                                i_s,
                                i_with_eps,
                                str::parse(s_out.as_str().trim()).unwrap(),
                            ));
                        }
                    }
                }
                ret.set_transitions(tuple_table);

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
