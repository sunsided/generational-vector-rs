use generational_vector::GenerationalVector;

fn main() {
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
