use criterion::{black_box, criterion_group, criterion_main, Criterion};
use generational_vector::GenerationalVector;
use std::num::NonZeroUsize;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("vec: push", |b| {
        let mut vec = Vec::<NonZeroUsize>::default();
        let value = NonZeroUsize::new(42).unwrap();
        b.iter(|| vec.push(black_box(value)));
    });

    c.bench_function("gv: push", |b| {
        let mut vec = GenerationalVector::default();
        let value = 42usize;
        b.iter(|| vec.push(black_box(value)));
    });

    c.bench_function("gv: push(usize)", |b| {
        let mut vec = GenerationalVector::<_, usize>::new();
        let value = 42usize;
        b.iter(|| vec.push(black_box(value)));
    });

    c.bench_function("gv: push(NonZeroUsize)", |b| {
        let mut vec = GenerationalVector::<NonZeroUsize>::new();
        let value = NonZeroUsize::new(42).unwrap();
        b.iter(|| vec.push(black_box(value)));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
