# A generational indexing-based Vector

This crates provides a vector type that uses generational indices to access
its elements. The addition of a generation counter to an index allows for invalidation
of stale references to previously deleted vector entries.

The vector itself is backed by a _free_ list to keep track of reusable holes
after element removal.

```rust
use generational_vector::{GenerationalVector, DeletionResult};

fn example() {
    let mut v = GenerationalVector::default();

    // Adding elements.
    let a = v.push("first");
    let b = v.push("second");
    assert_eq!(v.get(&a).unwrap(), &"first");
    assert_eq!(v.get(&b).unwrap(), &"second");

    // Removing elements.
    v.remove(&b);
    assert!(v.get(&b).is_none());

    // Overwriting a previously freed slot.
    let c = v.push("third");
    assert_eq!(v.get(&c).unwrap(), &"third");

    // The previous index 'b' internally points to the
    // same address as c. It uses an older generation however,
    // so is considered "not found":
    assert_eq!(v.get(&b), None);

    // Values can be enumerated.
    // Note that the ordering depends on insertions and deletions.
    for value in v {
        println!("{}", value);
    }
}
```

The above script is taken from [examples/example.rs](examples/example.rs) and can be run using

```shell
cargo run --example example
```

You can find more usage examples in [tests/tests.rs](tests/tests.rs). 

## Crate features

- `smallvec`: Enables the use of `SmallVec<T>` for the free list.
- `tinyvec`: Enables the use of `TinyVec<T>` for the free list.

## Benchmarks

This project uses Criterion for benchmarking. To execute the benchmarks, run

```shell
cargo criterion
```

## Material and sources

- [RustConf 2018 - Closing Keynote - Using Rust For Game Development by Catherine West](https://www.youtube.com/watch?v=aKLntZcp27M)
- [Generational indices guide](https://lucassardois.medium.com/generational-indices-guide-8e3c5f7fd594) by Lucas S.
