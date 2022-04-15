use num_traits::int::PrimInt;

/// An index entry in the `GenerationalVector`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GenerationalIndex<TGeneration> {
    index: usize,
    generation: TGeneration,
}

/// An entry slot encoding whether the list index
/// is free to be reused or occupied by a value.
#[derive(Debug)]
enum Slot<TEntry> {
    Free { next_free: usize },
    Occupied(TEntry),
}

/// An index entry
#[derive(Debug)]
struct GenerationalEntry<TEntry, TGeneration> {
    generation: TGeneration, // TODO: Make non-zero
    entry: Slot<TEntry>,
}

/// A vector that utilizes generational indexing to access the elements.
pub struct GenerationalVector<TEntry, TGeneration = usize>
where
    TGeneration: PrimInt,
{
    data: Vec<GenerationalEntry<TEntry, TGeneration>>,
    free_head: usize,
    len: usize,
}

impl<TEntry> Default for GenerationalVector<TEntry, usize> {
    fn default() -> Self {
        GenerationalVector::<TEntry, usize>::new()
    }
}

impl<TGeneration> GenerationalIndex<TGeneration> {
    fn new(index: usize, generation: TGeneration) -> Self {
        Self { index, generation }
    }
}

impl<TEntry, TGeneration> GenerationalEntry<TEntry, TGeneration>
where
    TGeneration: PrimInt,
{
    fn new(entry: Slot<TEntry>, generation: TGeneration) -> Self {
        Self { entry, generation }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum DeletionResult {
    /// The entry was successfully deleted.
    Ok,
    /// The entry was already deleted before.
    NotFound,
    /// Attempted to delete an entry of a different generation.
    InvalidGeneration,
}

impl DeletionResult {
    /// Determines whether the result was a valid deletion attempt,
    /// i.e. the entry was deleted or did not exist.
    ///
    /// ## Returns
    /// `false` if an invalid attempt was made at deleting a different generation.
    pub fn is_valid(&self) -> bool {
        !(*self == Self::InvalidGeneration)
    }
}

impl<TEntry, TGeneration> GenerationalVector<TEntry, TGeneration>
where
    TGeneration: PrimInt,
{
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            free_head: 0,
            len: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Walks the list to determine the number of free elements.
    ///
    /// ## Returns
    /// The number of empty slots.
    pub fn count_num_free(&self) -> usize {
        let mut head = self.free_head;
        let mut free_count = 0;
        while let Some(GenerationalEntry { entry, .. }) = &self.data.get(head) {
            if let Slot::Free { next_free } = entry {
                free_count += 1;
                head = *next_free;
            } else {
                break;
            }
        }
        free_count
    }

    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    pub fn push(&mut self, value: TEntry) -> GenerationalIndex<TGeneration> {
        let key = match self.data.get_mut(self.free_head) {
            Some(GenerationalEntry { entry, generation }) => {
                // Update existing entry.
                match entry {
                    Slot::Free { next_free } => {
                        let key = GenerationalIndex::new(self.free_head, *generation);
                        self.free_head = *next_free; // TODO: ensure this one never points to a used element after deletion!
                        *entry = Slot::Occupied(value);
                        key
                    }
                    _ => {
                        // We have found an occupied entry.
                        panic!("corrupt free list");
                    }
                }
            }
            None => {
                // Insert to the end.
                let generation = TGeneration::zero();
                let key = GenerationalIndex::new(self.data.len(), generation);
                let entry = Slot::Occupied(value);
                let gen_entry = GenerationalEntry::new(entry, generation);
                self.data.push(gen_entry);
                self.free_head = key.index + 1;
                key
            }
        };

        self.len += 1;
        key
    }

    pub fn get(&self, key: &GenerationalIndex<TGeneration>) -> Option<&TEntry> {
        let GenerationalEntry { entry, generation } = &self.data[key.index];

        if let Slot::Occupied(value) = entry {
            if *generation == key.generation {
                return Some(value);
            }
        }

        None
    }

    pub fn remove(&mut self, key: &GenerationalIndex<TGeneration>) -> DeletionResult {
        let GenerationalEntry { entry, generation } = &mut self.data[key.index];

        return match entry {
            Slot::Occupied { .. } => {
                if *generation != key.generation {
                    return DeletionResult::InvalidGeneration;
                }

                *generation = generation.add(TGeneration::one());
                *entry = Slot::Free {
                    next_free: self.free_head,
                };
                self.free_head = key.index;
                self.len = self.len - 1;
                DeletionResult::Ok
            }
            _ => {
                // If we get there it mean's that the user is trying to remove an already
                // removed key, just do nothing.
                DeletionResult::NotFound
            }
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
        assert_eq!(a.index, d.index);
        assert!(a.generation < d.generation);
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
        assert_eq!(a.index, e.index);
        assert_eq!(a.generation + 2, e.generation);
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

        // The free head now points at the last element.
        assert_eq!(gv.free_head, 2);

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

        // The free head now points at the first element.
        assert_eq!(gv.free_head, 0);

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

        let d = gv.push("d");
        let e = gv.push("e");

        // The last deleted element is assigned first.
        assert_eq!(c.index, d.index);
        assert_eq!(b.index, e.index);
        assert_eq!(gv.len(), 2);
        assert!(!gv.is_empty());

        assert_eq!(gv.count_num_free(), 1);
    }
}
