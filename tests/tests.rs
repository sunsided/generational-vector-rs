use generational_vector::GenerationalVector;

#[test]
fn default() {
    let gv: GenerationalVector<&str> = Default::default();
    assert_eq!(gv.len(), 0);
    assert!(gv.is_empty());
    assert_eq!(gv.count_num_free(), 0);
}

#[test]
fn new() {
    let gv: GenerationalVector<&str> = GenerationalVector::new();
    assert_eq!(gv.len(), 0);
    assert!(gv.is_empty());
    assert_eq!(gv.count_num_free(), 0);
}

#[test]
fn insert() {
    let mut gv = GenerationalVector::default();

    let a = gv.push("a");
    let b = gv.push("b");
    let c = gv.push("c");
    assert_eq!(gv.get(&a), Some(&"a"));
    assert_eq!(gv.get(&b), Some(&"b"));
    assert_eq!(gv.get(&c), Some(&"c"));
    assert_eq!(gv.len(), 3);
    assert!(!gv.is_empty());
    assert_eq!(gv.count_num_free(), 0);
}

#[test]
fn new_from_vec() {
    let gv: GenerationalVector<_> = vec!["a", "b", "c"].into();
    assert_eq!(gv.len(), 3);
}

#[test]
fn new_from_iter() {
    let vec = ["a", "b", "c"];
    let gv: GenerationalVector<_> = GenerationalVector::new_from_iter(vec);
    assert_eq!(gv.len(), 3);
}

#[test]
fn into_iter() {
    let gv: GenerationalVector<_> = vec!["a", "b", "c"].into();
    let vec: Vec<_> = gv.into_iter().collect();
    assert_eq!(vec.len(), 3);
    assert!(vec.contains(&"a"));
    assert!(vec.contains(&"b"));
    assert!(vec.contains(&"c"));
}

#[test]
fn for_loop() {
    let gv: GenerationalVector<_> = vec!["a", "b", "c"].into();

    let mut vec = Vec::default();
    for entry in gv {
        vec.push(entry);
    }

    assert_eq!(vec.len(), 3);
    assert!(vec.contains(&"a"));
    assert!(vec.contains(&"b"));
    assert!(vec.contains(&"c"));
}

#[test]
fn for_loop_with_ref() {
    let gv: GenerationalVector<_> = vec!["a", "b", "c"].into();
    let gv_ref = &gv;

    let mut vec = Vec::default();
    for &entry in gv_ref.into_iter() {
        vec.push(entry);
    }

    assert_eq!(vec.len(), 3);
    assert!(vec.contains(&"a"));
    assert!(vec.contains(&"b"));
    assert!(vec.contains(&"c"));
}

#[test]
fn for_loop_with_mut_ref() {
    let mut gv: GenerationalVector<_> = vec![10, 20, 30].into();
    let gv_ref = &mut gv;

    let mut vec = Vec::default();
    for entry in gv_ref.into_iter() {
        *entry = *entry + 1;
    }

    let gv_ref = &gv;
    for &entry in gv_ref.into_iter() {
        vec.push(entry);
    }

    assert_eq!(vec.len(), 3);
    assert!(vec.contains(&11));
    assert!(vec.contains(&21));
    assert!(vec.contains(&31));
}

#[test]
fn remove() {
    let mut gv = GenerationalVector::default();

    let a = gv.push("a");
    let _ = gv.push("b");
    let _ = gv.push("c");

    gv.remove(&a);

    assert_eq!(gv.get(&a), None);
    assert_eq!(gv.len(), 2);
    assert!(!gv.is_empty());

    // Since one element was deleted, there is exactly one free slot.
    assert_eq!(gv.count_num_free(), 1);

    // The internal vector stays expanded.
    assert_eq!(gv.capacity(), 4);
}

#[test]
fn insert_after_delete() {
    let mut gv = GenerationalVector::default();

    let a = gv.push("a");
    let _ = gv.push("b");
    let _ = gv.push("c");

    gv.remove(&a);
    let d = gv.push("d");

    // The index of element "a" was re-assigned to "d",
    // however the generation differs.
    assert_ne!(a, d);

    // The vector still has three elements however.
    assert_eq!(gv.len(), 3);
    assert!(!gv.is_empty());

    // No free slots.
    assert_eq!(gv.count_num_free(), 0);

    // The internal vector was expanded.
    assert_eq!(gv.capacity(), 4);
}

#[test]
fn insert_after_delete_twice() {
    let mut gv = GenerationalVector::default();

    let a = gv.push("a");
    let _ = gv.push("b");
    let _ = gv.push("c");

    gv.remove(&a);
    let d = gv.push("d");

    gv.remove(&d);
    let e = gv.push("e");

    // The index of element "a" was re-assigned to "e",
    // however the generation was incremented twice.
    assert_ne!(a, e);
}

#[test]
fn delete_all() {
    let mut gv = GenerationalVector::default();

    let a = gv.push("a");
    let b = gv.push("b");
    let c = gv.push("c");

    gv.remove(&a);
    gv.remove(&b);
    gv.remove(&c);

    assert_eq!(gv.len(), 0);
    assert!(gv.is_empty());

    // Number of free elements is three, however
    // the internal list capacity is still higher.
    assert_eq!(gv.count_num_free(), 3);
    assert_eq!(gv.capacity(), 4);
}

#[test]
fn delete_all_reverse() {
    let mut gv = GenerationalVector::default();

    let a = gv.push("a");
    let b = gv.push("b");
    let c = gv.push("c");

    gv.remove(&c);
    gv.remove(&b);
    gv.remove(&a);

    assert_eq!(gv.len(), 0);
    assert!(gv.is_empty());

    // Number of free elements is three, however
    // the internal list capacity is still higher.
    assert_eq!(gv.count_num_free(), 3);
    assert_eq!(gv.capacity(), 4);
}

#[test]
fn delete_all_and_insert() {
    let mut gv = GenerationalVector::default();

    let a = gv.push("a");
    let b = gv.push("b");
    let c = gv.push("c");

    gv.remove(&a);
    gv.remove(&b);
    gv.remove(&c);

    let _d = gv.push("d");
    let _e = gv.push("e");

    // The last deleted element is assigned first.
    assert_eq!(gv.len(), 2);
    assert!(!gv.is_empty());

    assert_eq!(gv.count_num_free(), 1);
}
