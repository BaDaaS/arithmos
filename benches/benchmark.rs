use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use string_algo_prime::{PrimeSignature, find_anagram_windows};

fn bench_signature_creation(c: &mut Criterion) {
    let input = b"the quick brown fox jumps over the lazy dog";
    c.bench_function("signature_new", |b| {
        b.iter(|| PrimeSignature::new(black_box(input)));
    });
}

fn bench_anagram_check(c: &mut Criterion) {
    let s1 = PrimeSignature::new(b"listen");
    let s2 = PrimeSignature::new(b"silent");
    c.bench_function("is_anagram_of", |b| {
        b.iter(|| black_box(&s1).is_anagram_of(black_box(&s2)));
    });
}

fn bench_contains_letters(c: &mut Criterion) {
    let haystack = PrimeSignature::new(b"abcdefghij");
    let needle = PrimeSignature::new(b"bdf");
    c.bench_function("contains_letters_of", |b| {
        b.iter(|| black_box(&haystack).contains_letters_of(black_box(&needle)));
    });
}

fn bench_find_anagram_windows(c: &mut Criterion) {
    let text = b"abcbacbadabcabc";
    let needle = PrimeSignature::new(b"abc");
    c.bench_function("find_anagram_windows", |b| {
        b.iter(|| find_anagram_windows(black_box(text), black_box(&needle)));
    });
}

criterion_group!(
    benches,
    bench_signature_creation,
    bench_anagram_check,
    bench_contains_letters,
    bench_find_anagram_windows,
);
criterion_main!(benches);
