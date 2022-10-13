use std::collections::HashMap;



pub struct NodeTrie {
    size: usize,
    children: HashMap<usize, NodeTrie>,
}

impl NodeTrie {
    pub fn new(n: &usize) -> NodeTrie {
        NodeTrie {
            size: *n,
            children: HashMap::new(),
        }
    }

    pub fn push(&mut self, node: usize) -> bool {
        if !self.children.contains_key(&node) {
            self.children.insert(node, NodeTrie::new(&self.size));
            true
        } else {
            false
        }
    }

    pub fn push_seq (mut self, nodes: Vec<usize>) -> bool {
        let mut is_new = false;
        let mut trie = &self;
        let mut node_ptr = 0;
        return is_new;
    }

    pub fn contains(&self, seq: &Vec<usize>) -> bool {
       if seq.len() == 0 {
            return true;
       } else {
            match &self.children.get(&seq[0]) {
                Some(mut trie) => {
                    return trie.contains(&seq[1..].to_vec());
                },
                None => return false,
            }
       } 
    }
}
