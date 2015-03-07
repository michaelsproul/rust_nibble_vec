#![feature(core, collections)]

use std::fmt::{self, Debug, Formatter};

/// A data-structure for storing a sequence of 4-bit values.
///
/// Values are stored in a `Vec<u8>`, with two values per byte.
///
/// Values at even indices are stored in the most-significant half of their byte,
/// while values at odd indices are stored in the least-significant half.
///
/// Imagine a vector of MSB bytes, and you'll be right.
///
/// n = [_ _ | _ _ | _ _]
///
/// # Invariants
/// * If the length is odd, then the second half of the last byte must be 0.
pub struct NibbleVec {
    length: usize,
    data: Vec<u8>
}

impl NibbleVec {
    /// Create an empty nibble vector.
    pub fn new() -> NibbleVec {
        NibbleVec {
            length: 0,
            data: Vec::new()
        }
    }

    /// Create a nibble vector from a vector of bytes.
    ///
    /// Each byte is split into two 4-bit entries (MSB, LSB).
    pub fn from_byte_vec(vec: Vec<u8>) -> NibbleVec {
        let length = 2 * vec.len();
        NibbleVec {
            length: length,
            data: vec
        }
    }

    /// Get the number of elements stored in the vector.
    pub fn len(&self) -> usize {
        self.length
    }

    /// Fetch a single entry from the vector.
    ///
    /// Guaranteed to be a value in the interval [0, 15].
    ///
    /// **Panics** if `idx >= self.len()`.
    pub fn get(&self, idx: usize) -> u8 {
        if idx >= self.length {
            panic!("attempted access beyond vector end. len is {}, index is {}", self.length, idx);
        }
        let vec_idx = idx / 2;
        match idx % 2 {
            // If the index is even, take the first (most significant) half of the stored byte.
            0 => self.data[vec_idx] >> 4,
            // If the index is odd, take the second (least significant) half.
            _ => self.data[vec_idx] & 0x0F
        }
    }

    /// Add a single nibble to the vector.
    ///
    /// Only the 4 least-significant bits of the value are used.
    pub fn push(&mut self, val: u8) {
        if self.length % 2 == 0 {
            self.data.push(val << 4);
        } else {
            let vec_len = self.data.len();

            // Zero the second half of the last byte just to be safe.
            self.data[vec_len - 1] &= 0xF0;

            // Write the new value.
            self.data[vec_len - 1] |= val & 0x0F;
        }
        self.length += 1;
    }

    /// Split the vector into two parts.
    ///
    /// All elements at or following the given index are returned in a new `NibbleVec`,
    /// with exactly `idx` elements remaining in this vector.
    ///
    /// **Panics** if `idx > self.len()`.
    pub fn split(&mut self, idx: usize) -> NibbleVec {
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

    /// Split function for odd *indices*.
    #[inline(always)]
    fn split_odd(&mut self, idx: usize) -> NibbleVec {
        let tail_vec_size = (self.length - idx) / 2;
        let mut tail = NibbleVec::from_byte_vec(Vec::with_capacity(tail_vec_size));

        // Perform an overlap copy, copying the last nibble of the original vector only if
        // the length of the new tail is *odd*.
        let tail_length = self.length - idx;
        let take_last = tail_length % 2 == 1;
        self.overlap_copy(idx / 2, self.data.len(), &mut tail.data, &mut tail.length, take_last);

        // Remove the copied bytes, being careful to skip the idx byte.
        for _ in range(idx / 2 + 1, self.data.len()) {
            self.data.pop();
        }

        // Zero the second half of the index byte so as to maintain the last-nibble invariant.
        self.data[idx / 2] &= 0xF0;

        // Update the length of the first NibbleVec.
        self.length = idx;

        tail
    }

    /// Split function for even *indices*.
    #[inline(always)]
    fn split_even(&mut self, idx: usize) -> NibbleVec {
        // Avoid allocating a temporary vector by copying all the bytes in order, then popping them.

        // Possible to prove: l_d - ⌊i / 2⌋ = ⌊(l_v - i + 1) / 2⌋
        //  where l_d = self.data.len()
        //        l_v = self.length
        let tail_vec_size = (self.length - idx + 1) / 2;
        let mut tail = NibbleVec::from_byte_vec(Vec::with_capacity(tail_vec_size));

        // Copy the bytes.
        for i in range(idx / 2, self.data.len()) {
            tail.data.push(self.data[i]);
        }

        // Pop the same bytes.
        for _ in range(idx / 2, self.data.len()) {
            self.data.pop();
        }

        // Update lengths.
        tail.length = self.length - idx;
        self.length = idx;

        tail
    }

    /// Copy data between the second half of self.data[start] and
    /// self.data[end - 1]. The second half of the last entry is included
    /// if include_last is true.
    #[inline(always)]
    fn overlap_copy(&self, start: usize, end: usize, vec: &mut Vec<u8>, length: &mut usize, include_last: bool) {
        // Copy up to the first half of the last byte.
        for i in range(start, end - 1) {
            // The first half is the second half of the old entry.
            let first_half = self.data[i] & 0x0f;

            // The second half is the first half of the next entry.
            let second_half = self.data[i + 1] >> 4;

            vec.push((first_half << 4) | second_half);
            *length += 2;
        }

        if include_last {
            let last = self.data[end - 1] & 0x0f;
            vec.push(last << 4);
            *length += 1;
        }
    }

    /// Append another nibble vector.
    pub fn join(mut self, other: &NibbleVec) -> NibbleVec {
        // If the length is even, we can append directly.
        if self.length % 2 == 0 {
            self.length += other.length;
            self.data.push_all(other.data.as_slice());
            return self;
        }

        // If the other vector is empty, bail out.
        if other.len() == 0 {
            return self;
        }

        // If the length is odd, we have to perform an overlap copy.
        // Copy the first half of the first element, to make the vector an even length.
        self.push(other.get(0));

        // Copy the rest of the vector using an overlap copy.
        let take_last = other.len() % 2 == 0;
        other.overlap_copy(0, other.data.len(), &mut self.data, &mut self.length, take_last);

        self
    }
}

impl PartialEq<NibbleVec> for NibbleVec {
    fn eq(&self, other: &NibbleVec) -> bool {
        self.length == other.length &&
        self.data == other.data
    }
}

impl Eq for NibbleVec {}

impl PartialEq<[u8]> for NibbleVec {
    fn eq(&self, other: &[u8]) -> bool {
        if other.len() != self.len() {
            return false;
        }

        for (i, x) in other.iter().enumerate() {
            if self.get(i) != *x {
                return false;
            }
        }
        true
    }
}

impl Clone for NibbleVec {
    fn clone(&self) -> NibbleVec {
        NibbleVec {
            length: self.length,
            data: self.data.clone()
        }
    }
}

impl Debug for NibbleVec {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        try!(write!(fmt, "NibbleVec ["));

        if self.len() > 0 {
            try!(write!(fmt, "{}", self.get(0)));
        }

        for i in range(1, self.len()) {
            try!(write!(fmt, ", {}", self.get(i)));
        }
        write!(fmt, "]")
    }
}

#[cfg(test)]
mod test {
    use NibbleVec;

    fn v8_7_6_5() -> NibbleVec {
        NibbleVec::from_byte_vec(vec![8 << 4 | 7, 6 << 4 | 5])
    }

    fn v11_10_9() -> NibbleVec {
        let mut result = NibbleVec::from_byte_vec(vec![11 << 4 | 10]);
        result.push(9);
        result
    }

    #[test]
    fn get() {
        let nv = NibbleVec::from_byte_vec(vec![3 << 4 | 7]);
        assert_eq!(nv.get(0), 3u8);
        assert_eq!(nv.get(1), 7u8);
    }

    #[test]
    fn push() {
        let mut nv = NibbleVec::new();
        let data = vec![0, 1, 3, 5, 7, 9, 11, 15];
        for val in data.iter() {
            nv.push(*val);
        }

        for (i, val) in data.iter().enumerate() {
            assert_eq!(nv.get(i), *val);
        }
    }

    fn split_test(  nibble_vec: &NibbleVec,
                    idx: usize,
                    first: Vec<u8>,
                    second: Vec<u8>) {
        let mut init = nibble_vec.clone();
        let tail = init.split(idx);
        assert!(init == first[..]);
        assert!(tail == second[..]);
    }

    #[test]
    fn split_even_length() {
        let even_length = v8_7_6_5();
        split_test(&even_length, 0, vec![], vec![8, 7, 6, 5]);
        split_test(&even_length, 1, vec![8], vec![7, 6, 5]);
        split_test(&even_length, 2, vec![8, 7], vec![6, 5]);
        split_test(&even_length, 4, vec![8, 7, 6, 5], vec![]);
    }

    #[test]
    fn split_odd_length() {
        let odd_length = v11_10_9();
        split_test(&odd_length, 0, vec![], vec![11, 10, 9]);
        split_test(&odd_length, 1, vec![11], vec![10, 9]);
        split_test(&odd_length, 2, vec![11, 10], vec![9]);
        split_test(&odd_length, 3, vec![11, 10, 9], vec![]);
    }

    /// Join vec2 onto vec1 and ensure that the results matches the one expected.
    fn join_test(vec1: &NibbleVec, vec2: &NibbleVec, result: Vec<u8>) {
        let joined = vec1.clone().join(vec2);
        assert!(joined == result[..]);
    }

    #[test]
    fn join_even_length() {
        let v1 = v8_7_6_5();
        let v2 = v11_10_9();
        join_test(&v1, &v2, vec![8, 7, 6, 5, 11, 10, 9]);
        join_test(&v1, &v1, vec![8, 7, 6, 5, 8, 7, 6, 5]);
        join_test(&v1, &NibbleVec::new(), vec![8, 7, 6, 5]);
        join_test(&NibbleVec::new(), &v1, vec![8, 7, 6, 5]);
    }

    #[test]
    fn join_odd_length() {
        let v1 = v8_7_6_5();
        let v2 = v11_10_9();
        join_test(&v2, &v1, vec![11, 10, 9, 8, 7, 6, 5]);
        join_test(&v2, &v2, vec![11, 10, 9, 11, 10, 9]);
        join_test(&v2, &NibbleVec::new(), vec![11, 10, 9]);
    }

    /// Ensure that the last nibble is zeroed before reuse.
    #[test]
    fn memory_reuse() {
        let mut vec = NibbleVec::new();
        vec.push(10);
        vec.push(1);

        // Pushing.
        vec.split(1);
        vec.push(2);
        assert_eq!(vec.get(1), 2);

        // Joining.
        vec.split(1);
        vec = vec.join(&NibbleVec::from_byte_vec(vec![1 << 4 | 3, 5 << 4]));
        assert_eq!(vec.get(1), 1);
    }
}
