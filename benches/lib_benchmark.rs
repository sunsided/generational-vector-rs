use criterion::{black_box, criterion_group, criterion_main, Criterion};
use generational_vector::GenerationalVector;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("vec: push", |b| {
        let mut vec = Vec::<usize>::default();
        b.iter(|| vec.push(black_box(42)));
    });

    c.bench_function("gv: push", |b| {
        let mut vec = GenerationalVector::<usize, usize>::default();
        b.iter(|| vec.push(black_box(42)));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
