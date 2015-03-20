NibbleVec
====

[![Build Status](https://travis-ci.org/michaelsproul/rust_nibble_vec.svg)](https://travis-ci.org/michaelsproul/rust_nibble_vec)

Data-structure for storing a sequence of half-bytes.

Wraps a `Vec<u8>`, providing safe and memory-efficient storage of 4-bit values.

In terms of supported operations, the structure behaves kind of like a fixed length array, in that insertions into the middle of the vector are difficult (and unimplemented at present).

# License

MIT License. Copyright (c) Michael Sproul 2015.
