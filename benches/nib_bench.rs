use criterion::{criterion_group, criterion_main, Criterion};
use nibble_vec::NibbleVec;

fn even_8to5() -> NibbleVec {
    NibbleVec::from_byte_vec(vec![8 << 4 | 7, 6 << 4 | 5])
}

fn odd_11to9() -> NibbleVec {
    let mut result = NibbleVec::from_byte_vec(vec![11 << 4 | 10]);
    result.push(9);
    result
}

fn split_test(nibble_vec: &NibbleVec, idx: usize) {
    let mut init = nibble_vec.clone();
    let _tail = init.split(idx);
}

fn nib_split_even(b: &mut Criterion) {
    let even_length = even_8to5();
    b.bench_function("make nib vec", |b| b.iter(|| {
        split_test(&even_length, 1);
        split_test(&even_length, 2);
    }));
}

fn nib_make_split(b: &mut Criterion) {
    b.bench_function("nib_make_split", |b| {
        b.iter(|| {
            let odd_length = odd_11to9();
            split_test(&odd_length, 0);
            split_test(&odd_length, 1);
        })
    });
}

fn nib_get(b: &mut Criterion) {
    b.bench_function("nib_make_split", |b| {
        let v = vec![243, 2, 3, 251, 5, 6, 7, 8, 255];
        let nv = NibbleVec::from(v.clone());
        b.iter(|| {
            for (i, _) in v.iter().enumerate() {
                nv.get(i);
            }
        })
    });
}

fn join_test(vec1: &NibbleVec, vec2: &NibbleVec) {
    let _joined = vec1.clone().join(vec2);
}

fn nib_join_test(b: &mut Criterion) {
    b.bench_function("trie remove", |b| {
        b.iter(|| {
            let v1 = even_8to5();
            let v2 = odd_11to9();
            join_test(&v1, &v2);
            join_test(&v1, &v1);
        });
    });
}

criterion_group!(benches, nib_split_even, nib_make_split, nib_join_test, nib_get);
criterion_main!(benches);
