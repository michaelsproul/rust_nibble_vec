use criterion::{criterion_group, criterion_main, Criterion};
use nibble_vec::NibbleVec;

fn v8_7_6_5() -> NibbleVec {
    NibbleVec::from_byte_vec(vec![8 << 4 | 7, 6 << 4 | 5])
}

fn v11_10_9() -> NibbleVec {
    let mut result = NibbleVec::from_byte_vec(vec![11 << 4 | 10]);
    result.push(9);
    result
}

fn split_test(nibble_vec: &NibbleVec, idx: usize, first: Vec<u8>, second: Vec<u8>) {
    let mut init = nibble_vec.clone();
    let tail = init.split(idx);
    assert!(init == first[..]);
    assert!(tail == second[..]);
}

fn nib_split_even(b: &mut Criterion) {
    let even_length = v8_7_6_5();
    b.bench_function("make nib vec", |b| b.iter(|| {
        split_test(&even_length, 1, vec![8], vec![7, 6, 5]);
        split_test(&even_length, 2, vec![8, 7], vec![6, 5]);
    }));
}

fn nib_make_split(b: &mut Criterion) {
    b.bench_function("nib_make_split", |b| {
        b.iter(|| {
            let odd_length = v11_10_9();
            split_test(&odd_length, 0, vec![], vec![11, 10, 9]);
            split_test(&odd_length, 1, vec![11], vec![10, 9]);
        })
    });
}

fn join_test(vec1: &NibbleVec, vec2: &NibbleVec, result: Vec<u8>) {
    let joined = vec1.clone().join(vec2);
    assert!(joined == result[..]);
}

fn nib_join_test(b: &mut Criterion) {
    b.bench_function("trie remove", |b| {
        b.iter(|| {
            let v1 = v8_7_6_5();
            let v2 = v11_10_9();
            join_test(&v1, &v2, vec![8, 7, 6, 5, 11, 10, 9]);
            join_test(&v1, &v1, vec![8, 7, 6, 5, 8, 7, 6, 5]);
        });
    });
}

criterion_group!(benches, nib_split_even, nib_make_split, nib_join_test);
criterion_main!(benches);
