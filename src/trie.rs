use std::collections::{HashMap, HashSet, VecDeque};

pub enum Seq {
    Cons(usize, Box<Seq>),
    Nil,
}

impl From<&Vec<usize>> for Seq {
    fn from(v: &Vec<usize>) -> Self {
        let mut head = Seq::Nil;

        for &n in v {
            head = Seq::Cons(n, Box::new(head));
        }
        return head;
    }
}

impl From<&HashSet<usize>> for Seq {
    fn from(s: &HashSet<usize>) -> Self {
        let mut v = Vec::from_iter(s);
        v.sort();

        let mut head = Seq::Nil;
        for n in v {
            head = Seq::Cons(*n, Box::new(head));
        }
        return head;
    }
}

pub struct NodeTrie {
    children: HashMap<usize, NodeTrie>,
    state_num: Option<usize>,
}

pub struct FrontierController {
    pub frontier: VecDeque<Seq>,
    counter: usize,
}

impl FrontierController {
    pub fn new() -> Self {
        Self {
            frontier: VecDeque::new(),
            counter: 0,
        }
    }

    pub fn size(&self) -> usize {
        return self.counter;
    }
}

impl NodeTrie {
    pub fn new(controller: &mut FrontierController, assign: bool) -> Self {
        controller.counter += if assign { 1 } else { 0 };
        Self {
            children: HashMap::new(),
            state_num: if assign {
                Some(controller.counter.clone() - 1)
            } else {
                None
            },
        }
    }

    pub fn push(&mut self, sequence: &Seq, controller: &mut FrontierController) -> bool {
        match sequence {
            Seq::Cons(next, rest) => match self.children.get_mut(&next) {
                Some(c) => c.push(&rest, controller),
                None => {
                    // Set new counter number
                    let mut new_trie =
                        NodeTrie::new(controller, if let Seq::Nil = **rest { true } else { false });

                    new_trie.push(&rest, controller);
                    self.children.insert(*next, new_trie);
                    return false;
                }
            },
            Seq::Nil => {
                return true;
            }
        }
    }

    pub fn get(&mut self, seq: &Seq) -> Option<&mut NodeTrie> {
        match seq {
            Seq::Nil => {
                return Some(self);
            }
            Seq::Cons(next, rest) => match self.children.get_mut(&next) {
                Some(child) => {
                    return child.get(&rest);
                }
                None => {
                    return None;
                }
            },
        }
    }

    pub fn get_addr(&mut self, seq: &Seq, controller: &mut FrontierController) -> Option<usize> {
        match seq {
            Seq::Nil => {
                let address: usize = match self.state_num {
                    Some(address) => address,
                    None => {
                        let address = controller.counter - 1;
                        self.state_num = Some(address);
                        controller.counter += 1;
                        address
                    }
                };
                return Some(address);
            }
            Seq::Cons(next, rest) => match self.children.get_mut(&next) {
                Some(child) => {
                    return child.get_addr(&rest, controller);
                }
                None => {
                    return None;
                }
            },
        }
    }

    pub fn contains(&self, seq: &Seq) -> bool {
        match seq {
            Seq::Nil => {
                return true;
            }
            Seq::Cons(next, rest) => match self.children.get(&next) {
                Some(child) => {
                    return child.contains(&rest);
                }
                None => {
                    return false;
                }
            },
        }
    }
}
