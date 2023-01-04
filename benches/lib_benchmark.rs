use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use generational_vector::{vector, GenerationalVector};
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
        let mut vec = vector::GenerationalVector::<_, usize>::new();
        let value = 42usize;
        b.iter(|| vec.push(black_box(value)));
    });

    c.bench_function("gv: push(NonZeroUsize)", |b| {
        let mut vec = GenerationalVector::default();
        let value = NonZeroUsize::new(42).unwrap();
        b.iter(|| vec.push(black_box(value)));
    });

    let mut group = c.benchmark_group("gv: remove/push");
    for size in [1, 2, 15, 16, 128].iter() {
        let mut vec = GenerationalVector::default();
        let value = 42usize;

        let mut idxs = Vec::default();
        for _ in 0..1000 {
            idxs.push(vec.push(value));
        }

        group.throughput(Throughput::Elements(*size as _));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                for _ in 0..size {
                    let idx = idxs.pop().unwrap();
                    vec.remove(&idx);
                }

                for _ in 0..size {
                    idxs.push(vec.push(black_box(value)));
                }
            });
        });
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
