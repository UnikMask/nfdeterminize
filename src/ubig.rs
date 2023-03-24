use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Eq)]
pub struct Ubig {
    pub num: Vec<u8>,
}

#[derive(Clone)]
pub struct CompressedUbig {
    pub cnum: Vec<u8>,
}

impl Hash for CompressedUbig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.cnum.hash(state);
    }
}

impl Hash for Ubig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.num.hash(state);
    }
}

impl PartialEq for Ubig {
    fn eq(&self, other: &Ubig) -> bool {
        let mut a = self;
        let mut b = other;

        if a.num.len() < b.num.len() {
            let t = a;
            a = b;
            b = t;
        }
        for bit in 0..a.num.len() {
            if bit >= b.num.len() {
                if a.num[bit] == 1 {
                    return false;
                }
            } else if a.num[bit] != b.num[bit] {
                return false;
            }
        }
        return true;
    }
}

impl CompressedUbig {
    fn decompress(self) -> Ubig {
        Ubig {
            num: decompress_size_prepended(self.cnum.as_slice().clone()).unwrap(),
        }
    }
}

impl Ubig {
    /// Create a new Ubig.
    pub fn new() -> Ubig {
        Ubig { num: Vec::new() }
    }

    /// Get a Ubig's bit sequence.
    pub fn get_seq(&self) -> Vec<usize> {
        let mut ret: Vec<usize> = Vec::new();
        for byte in 0..self.num.len() {
            for bit in 0..8 {
                if (self.num[byte] >> bit) & 1 == 1 {
                    ret.push(byte * 8 + bit);
                }
            }
        }
        return ret;
    }

    pub fn bit_at(&self, pos: &usize) -> bool {
        if *pos < self.num.len() * 8 {
            return (self.num[pos / 8] >> (pos % 8)) & 1 == 1;
        } else {
            return false;
        }
    }

    // Flip a bit on given array position.
    pub fn flip(&mut self, bit: &usize) {
        if *bit < self.num.len() * 8 {
            let pos = bit / 8;
            self.num[pos] = self.num[pos] ^ (1 << bit % 8);
        } else {
            self.extend(bit);
            self.flip(bit);
        }
    }

    pub fn set_to(&mut self, bit: &usize, val: bool) {
        if *bit < self.num.len() * 8 {
            let pos = bit / 8;
            if val {
                self.num[pos] = self.num[pos] | (1 << bit % 8);
            } else {
                self.num[pos] = self.num[pos] & (0xFF ^ (1 << bit % 8));
            }
        } else {
            self.extend(bit);
            self.flip(bit);
        }
    }

    // Extend the vector of bits of the array to required size.
    fn extend(&mut self, new_size: &usize) {
        let mut size_incr = new_size - self.num.len();
        while size_incr > 8 {
            self.num.push(0);
            size_incr -= 8;
        }
        self.num.push(0);
    }

    fn compress(self) -> CompressedUbig {
        return CompressedUbig {
            cnum: compress_prepend_size(self.num.as_slice().clone()),
        };
    }
}

#[cfg(test)]
mod ubig_tests {

    use std::hash::Hasher;

    use super::{CompressedUbig, Ubig};
    use fasthash::xx::Hasher64;

    fn get_hash(u: &CompressedUbig, n: usize) -> usize {
        let mut hasher = Hasher64::default();
        hasher.write(&u.cnum);
        (hasher.finish() as usize) % n
    }

    // Helpher methods for testing - generates ubigs from a single bit or from sequences of bits.
    impl Ubig {
        fn from_bit(bit: &usize) -> Ubig {
            let mut ret = Ubig::new();
            ret.flip(bit);
            return ret;
        }
        fn from_seq(bit_list: &Vec<usize>) -> Ubig {
            let mut ret = Ubig::new();
            for bit in bit_list {
                ret.set_to(&bit, true);
            }
            return ret;
        }
    }

    #[test]
    fn test_clone() {
        let num0 = Ubig::from_bit(&0);
        assert_eq!(num0, num0.clone());

        let num1 = Ubig::from_bit(&1);
        assert_eq!(num1, num1.clone());

        let num4 = Ubig::from_bit(&4);
        assert_eq!(num4, num4.clone());
    }

    #[test]
    fn test_from_bit() {
        // Test num with first bit switched.
        let num0 = Ubig::from_bit(&0);
        assert_eq!(num0.bit_at(&0), true);

        // Test num with first bit switched.
        let num4 = Ubig::from_bit(&4);
        assert_eq!(num4.bit_at(&4), true);

        // Test number with bigger extensions.
        let num8 = Ubig::from_bit(&8);
        assert_eq!(num8.bit_at(&8), true);
    }

    #[test]
    fn test_seqs() {
        let empty_seq = vec![];
        assert_eq!(Ubig::from_seq(&empty_seq).get_seq(), empty_seq);
        let simple_seq = vec![0];
        assert_eq!(Ubig::from_seq(&simple_seq).get_seq(), simple_seq);
        let no_ext_seq = vec![1, 3, 7];
        assert_eq!(Ubig::from_seq(&no_ext_seq).get_seq(), no_ext_seq);
        let with_ext_seq = vec![0, 8, 24];
        assert_eq!(Ubig::from_seq(&with_ext_seq).get_seq(), with_ext_seq);
    }

    #[test]
    fn test_set_to() {
        let mut test_ubig = Ubig::new();
        test_ubig.set_to(&0, true);
        assert_eq!(test_ubig.bit_at(&0), true);
        test_ubig.set_to(&0, true);
        assert_eq!(test_ubig.bit_at(&0), true);
        test_ubig.set_to(&0, false);
        assert_eq!(test_ubig.bit_at(&0), false);

        test_ubig.set_to(&11, true);
        assert_eq!(test_ubig.bit_at(&11), true);
        test_ubig.set_to(&11, true);
        assert_eq!(test_ubig.bit_at(&11), true);
        test_ubig.set_to(&11, false);
        assert_eq!(test_ubig.bit_at(&11), false);
    }

    #[test]
    fn test_compress_decompress() {
        let test_seq = vec![1, 8, 24, 32, 121, 12389, 120321];
        let u = Ubig::from_seq(&test_seq);
        let uc = u.clone().compress();

        assert_eq!(uc.decompress(), u);
        let un = Ubig::from_seq(&test_seq);
        assert_eq!(get_hash(&u.compress(), 8), get_hash(&un.compress(), 8));
    }
}
