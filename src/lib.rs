/// Data-structure for storing a sequence of 4-bit values.
pub struct NibbleVec {
    offset: bool,
    length: uint,
    data: Vec<u8>
}

impl NibbleVec {
    pub fn new() -> NibbleVec {
        NibbleVec {
            offset: false,
            length: 0,
            data: Vec::new()
        }
    }

    pub fn from_byte_vec(vec: Vec<u8>) -> NibbleVec {
        let length = 2 * vec.len();
        NibbleVec {
            offset: false,
            length: length,
            data: vec
        }
    }

    pub fn get(&self, idx: uint) -> Option<u8> {
        if idx >= self.length {
            return None;
        }
        let vec_idx = idx / 2;
        match (self.offset, idx % 2) {
            // Take the most significant bits.
            (false, 0) | (true, 1) => Some(self.data[vec_idx] >> 4),
            // Take the least significant bits.
            _ => Some(self.data[vec_idx] & 0x0f)
        }
    }
}

#[cfg(test)]
mod test {
    use NibbleVec;

    #[test]
    fn get() {
        let nv = NibbleVec::from_byte_vec(vec![(1 << 6) + (1 << 3)]);
        assert_eq!(nv.get(0).unwrap(), 4u8);
        assert_eq!(nv.get(1).unwrap(), 8u8);
    }
}
