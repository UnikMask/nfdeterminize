#[derive(Debug, Hash, Eq)]
pub struct Ubig {
    num: Vec<u8>,
}

impl PartialEq for Ubig {
    fn eq(&self, other: &Ubig) -> bool {
        if self.num.len() != other.num.len() {
            return false;
        } else {
            for b in 0..self.num.len() {
                if self.num[b] & other.num[b] != 0 {
                    return false;
                }
            }
            return true;
        }
    }
}

impl Clone for Ubig {
    fn clone(&self) -> Self {
        let mut ret = Ubig {
            num: Vec::with_capacity(self.num.len()),
        };
        for b in 0..self.num.len() {
            ret.num[b] = self.num[b];
        }
        ret
    }
}

impl Ubig {
    pub fn new() -> Ubig {
        Ubig { num: Vec::new() }
    }

    pub fn from_seq(bit_list: Vec<usize>) -> Ubig {
        let mut ret = Ubig::new();
        ret.flip_seq(bit_list);
        return ret;
    }

    pub fn get_seq(&self) -> Vec<usize> {
        let mut ret: Vec<usize> = Vec::new();
        for byte in 0..self.num.len() {
            for bit in 0..8 {
                if (self.num[byte] & (1 << bit)) == 1 {
                    ret.push(byte * 8 + bit);
                }
            }
        }
        return ret;
    }

    pub fn bit_at(&self, pos: usize) -> bool {
        if pos < self.num.len() {
            return self.num[pos / 8] & (1 << pos % 8) == 1;
        } else {
            return false;
        }
    }

    pub fn from_bit(bit: usize) -> Ubig {
        let mut ret = Ubig::new();
        ret.flip(bit);
        return ret;
    }

    // Flip bits given from a sequence.
    pub fn flip_seq(&mut self, bit_list: Vec<usize>) {
        for bit in bit_list {
            self.flip(bit);
        }
    }

    // Flip a bit on given array position.
    pub fn flip(&mut self, bit: usize) {
        if bit < self.num.len() * 8 {
            let pos = bit / 8;
            self.num[pos] = self.num[pos] ^ (1 << bit % 8);
        } else {
            self.extend(bit);
            self.flip(bit);
        }
    }

    // Extend the vector of bits of the array to required size.
    fn extend(&mut self, new_size: usize) {
        let mut size_incr = new_size;
        while size_incr > 0 {
            self.num.push(0);
            size_incr -= 8;
        }
    }
}
