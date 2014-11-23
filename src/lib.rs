/// A data-structure for storing a sequence of 4-bit values.
///
/// Values are stored in a Vec<u8>, with two values per byte.
///
/// Values at *even* indices are stored in the most-significant half of their byte,
/// while values at odd indices are stored in the least-significant half.
///
/// Imagine a vector of MSB bytes, and you'll be right.
///
/// n = [_ _ | _ _ | _ _]
pub struct NibbleVec {
    length: uint,
    data: Vec<u8>
}

impl NibbleVec {
    pub fn new() -> NibbleVec {
        NibbleVec {
            length: 0,
            data: Vec::new()
        }
    }

    pub fn from_byte_vec(vec: Vec<u8>) -> NibbleVec {
        let length = 2 * vec.len();
        NibbleVec {
            length: length,
            data: vec
        }
    }

    pub fn len(&self) -> uint {
        self.length
    }

    pub fn get(&self, idx: uint) -> Option<u8> {
        if idx >= self.length {
            return None;
        }
        let vec_idx = idx / 2;
        // If the index is even, take the first (most significant) half of the stored byte.
        // Otherwise, take the second (least significant) half.
        match idx % 2 {
            0 => Some(self.data[vec_idx] >> 4),
            _ => Some(self.data[vec_idx] & 0x0f)
        }
    }

    pub fn push(&mut self, val: u8) {
        if self.length % 2 == 0 {
            self.data.push(val << 4);
        } else {
            let vec_len = self.data.len();
            self.data[vec_len - 1] |= val & 0x0F;
        }
        self.length += 1;
    }

    /// Cut the vector into two parts.
    ///
    /// All elements at or following the given index are returned in a new `NibbleVec`.
    ///
    /// *Panics* if `idx > self.len()`.
    pub fn split(&mut self, idx: uint) -> NibbleVec {
        if idx > self.length {
            panic!("attempted to split past vector end. len is {}, index is {}", self.length, idx);
        } else if idx == self.length {
            NibbleVec::new()
        } else if idx % 2 == 0 {
            self.split_even(idx)
        } else {
            self.split_odd(idx)
        }
    }

    #[inline(always)]
    fn split_odd(&mut self, idx: uint) -> NibbleVec {
        let tail_vec_size = (self.length - idx) / 2;
        let mut tail_vec = Vec::with_capacity(tail_vec_size);

        // Copy the bytes, crossing boundaries in the original vec.
        for i in range(idx / 2, self.data.len() - 1) {
            // The first half is the second half of the old entry.
            let first_half = self.data[i] & 0x0f;

            // The second half is the first half of the next entry.
            let second_half = self.data[i + 1] >> 4;

            tail_vec.push((first_half << 4) | second_half);
        }

        // If the new length is odd, get the trailing half-byte from the old vector.
        let tail_length = self.length - idx;

        if tail_length % 2 == 1 {
            let last = self.data[self.data.len() - 1] & 0x0f;
            tail_vec.push(last << 4);
        }

        // Remove the copied bytes, being careful to skip the idx byte.
        for _ in range(idx / 2 + 1, self.data.len()) {
            self.data.pop();
        }

        // Update the length of the first NibbleVec.
        self.length = idx;

        NibbleVec {
            length: tail_length,
            data: tail_vec
        }
    }

    #[inline(always)]
    fn split_even(&mut self, idx: uint) -> NibbleVec {
        // Avoid allocating a temporary vector by copying all the bytes in order, then popping them.
        let tail_vec_size = (self.length - idx) / 2;
        let mut tail_vec = Vec::with_capacity(tail_vec_size);

        // Copy the bytes.
        for i in range(idx / 2, self.data.len()) {
            tail_vec.push(self.data[i]);
        }

        // Pop the same bytes.
        for _ in range(0, tail_vec_size) {
            self.data.pop();
        }

        let tail_length = self.length - idx;
        self.length = idx;

        NibbleVec {
            length: tail_length,
            data: tail_vec
        }
    }
}

#[cfg(test)]
mod test {
    use NibbleVec;

    #[test]
    fn get() {
        let nv = NibbleVec::from_byte_vec(vec![4 << 4 | 8]);
        assert_eq!(nv.get(0).unwrap(), 4u8);
        assert_eq!(nv.get(1).unwrap(), 8u8);
    }

    #[test]
    fn push() {
        let mut nv = NibbleVec::new();
        let data: Vec<u8> = vec![0, 1, 3, 5, 7, 9, 11, 15];
        for val in data.iter() {
            nv.push(*val);
        }

        for (i, val) in data.iter().enumerate() {
            assert_eq!(nv.get(i).unwrap(), *val);
        }
    }

    #[test]
    fn split_even_idx() {
        // Even index, even length.
        let mut nv = NibbleVec::from_byte_vec(vec![
            10 << 4 | 11,
            12 << 4 | 13
        ]);

        let tail = nv.split(2);
        assert_eq!(nv.len(), 2u);
        assert_eq!(tail.len(), 2u);
        assert_eq!(nv.get(0).unwrap(), 10u8);
        assert_eq!(nv.get(1).unwrap(), 11u8);
        assert_eq!(tail.get(0).unwrap(), 12u8);
        assert_eq!(tail.get(1).unwrap(), 13u8);

        // Even index, odd length.
        nv.push(14u8);
        let tail = nv.split(2);
        assert_eq!(nv.len(), 2u);
        assert_eq!(tail.len(), 1u);
        assert_eq!(nv.get(0).unwrap(), 10u8);
        assert_eq!(nv.get(1).unwrap(), 11u8);
        assert_eq!(tail.get(0).unwrap(), 14u8);
    }

    #[test]
    fn split_odd_idx() {
        // Odd index, even length.
        let mut nv = NibbleVec::from_byte_vec(vec![
            10 << 4 | 11,
            12 << 4 | 13
        ]);

        let tail = nv.split(3);
        assert_eq!(nv.len(), 3u);
        assert_eq!(tail.len(), 1u);
        for i in range(0, nv.len()) {
            assert_eq!(nv.get(i).unwrap(), (i + 10) as u8);
        }
        assert_eq!(tail.get(0).unwrap(), 13u8);

        // Odd index, odd length.
        let tail = nv.split(1);
        assert_eq!(nv.len(), 1u);
        assert_eq!(tail.len(), 2u);
        assert_eq!(nv.get(0).unwrap(), 10u8);
        assert_eq!(tail.get(0).unwrap(), 11u8);
        assert_eq!(tail.get(1).unwrap(), 12u8);
    }
}
