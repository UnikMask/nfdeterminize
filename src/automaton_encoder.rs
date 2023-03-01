extern crate pest;

use pest::Parser;

use crate::automaton::{Automaton, AutomatonType};

#[derive(pest_derive::Parser)]
#[grammar = "automaton.pest"]
struct AutomatonParser;

impl From<&String> for Automaton {
    fn from(s: &String) -> Self {
        println!("Parsing...");
        return match AutomatonParser::parse(Rule::automaton, s) {
            Ok(mut pairs) => {
                // Get contents of automaton from automaton -> core -> inner
                println!("Accessing core...");
                let mut contents = pairs
                    .next()
                    .unwrap()
                    .into_inner()
                    .next()
                    .unwrap()
                    .into_inner();
                let mut ret = Automaton::empty();

                // Get the pairs for all the properties of the automaton.
                println!("Determining type...");
                ret.set_automaton_type(match contents.next().unwrap().as_str() {
                    "det" => AutomatonType::Det,
                    "nondet" => AutomatonType::NonDet,
                    "epsilon" => AutomatonType::NonDet,
                    _ => panic!("Unreachable code!"),
                });

                // Set size and alphabet.
                println!("Determining size and alphabet");
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
                println!("Creating tuple table...");
                let mut tuple_table: Vec<(usize, usize, usize)> = Vec::new();
                let mut epsilon_increment = 1;
                for (i_a, a) in contents
                    .next()
                    .unwrap()
                    .into_inner()
                    .into_iter()
                    .enumerate()
                {
                    // Set the alphabet type
                    let i_with_eps = match alphabet_parse.as_rule() {
                        Rule::LETTER_STR => match alphabet_parse.as_str().chars().nth(i_a) {
                            Some(letter) => match letter {
                                '@' => {
                                    epsilon_increment = 0;
                                    ret.set_alphabet(ret.get_alphabet() - 1);
                                    0
                                }
                                _ => i_a + epsilon_increment,
                            },
                            None => i_a + epsilon_increment,
                        },
                        _ => i_a + epsilon_increment,
                    };

                    // Use barebones array parsing here as it is faster than pest's parsing speeds for arrays.
                    for (i_s, s_in) in a.into_inner().into_iter().enumerate() {
                        for (_, s_out) in s_in
                            .as_str()
                            .trim_matches(|c| {
                                c == '[' || c == ']' || c == '\n' || c == ' ' || c == '\t'
                            })
                            .split(',')
                            .enumerate()
                        {
                            if s_out.len() > 0 {
                                tuple_table.push((
                                    i_s + 1,
                                    i_with_eps,
                                    s_out.trim().parse::<usize>().unwrap(),
                                ));
                            }
                        }
                    }
                }
                ret.set_transitions(tuple_table);

                // Set start states.
                println!("Getting start states...");
                let mut start: Vec<usize> = Vec::new();
                for num in contents.next().unwrap().into_inner() {
                    start.push(str::parse(num.as_str().trim()).unwrap());
                }
                ret.set_start_states(start);

                // Set ends states.
                println!("Creating end states...");
                let mut end: Vec<usize> = Vec::new();
                for num in contents.next().unwrap().into_inner() {
                    end.push(str::parse(num.as_str().trim()).unwrap());
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
