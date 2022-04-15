use num_traits::int::PrimInt;
use std::borrow::Borrow;

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
    generation: TGeneration, // TODO: Make non-zero for optimizations?
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
    #[inline]
    fn new_from_value(value: TEntry, generation: TGeneration) -> Self {
        Self {
            entry: Slot::Occupied(value),
            generation,
        }
    }

    /// Replaces the content of an empty slot with a new value.
    ///
    /// ## Panics
    /// Will panic if the slot is already occupied.
    ///
    /// ## Arguments
    /// * `value` - The new value.
    /// * `free_head` - A mutable reference to the free head pointer of the vector.
    ///   This value will be overwritten.
    ///
    /// ## Returns
    /// The index pointing to the new element.
    pub fn reuse(
        &mut self,
        value: TEntry,
        free_head: &mut usize,
    ) -> GenerationalIndex<TGeneration> {
        if let Slot::Free { next_free } = self.entry {
            let key = GenerationalIndex::new(*free_head, self.generation);
            self.entry = Slot::Occupied(value);
            *free_head = next_free;
            return key;
        }

        panic!("free list is corrupted");
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

/// A vector whose elements are addressed by both an index and an entry
/// generation.
impl<TEntry, TGeneration> GenerationalVector<TEntry, TGeneration>
where
    TGeneration: PrimInt,
{
    /// Initializes a new, empty vector.
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            free_head: 0,
            len: 0,
        }
    }

    /// Constructs a new, empty `Vec<T>` with the specified capacity.
    ///
    /// The vector will be able to hold exactly `capacity` elements without
    /// reallocating. If `capacity` is 0, the vector will not allocate.
    ///
    /// It is important to note that although the returned vector has the
    /// *capacity* specified, the vector will have a zero *length*.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            free_head: 0,
            len: 0,
        }
    }

    /// Returns the number of elements in the vector, also referred to
    /// as its 'length'.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut v = generational_vector::GenerationalVector::default();
    /// let _a = v.push("a");
    /// let _b = v.push("b");
    /// let _c = v.push("c");
    /// assert_eq!(v.len(), 3);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the vector contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut v = generational_vector::GenerationalVector::default();
    /// assert!(v.is_empty());
    ///
    /// v.push("a");
    /// assert!(!v.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Walks the list to determine the number of free elements.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut v = generational_vector::GenerationalVector::default();
    ///
    /// let _a = v.push("a");
    /// let _b = v.push("b");
    /// let _c = v.push("c");
    ///
    /// v.remove(&_a);
    /// v.remove(&_b);
    ///
    /// assert_eq!(v.len(), 1);
    /// assert_eq!(v.count_num_free(), 2);
    /// ```
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

    /// Returns the number of elements the vector can hold without
    /// reallocating.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_vector::GenerationalVector;
    /// let vec: GenerationalVector<i32> = GenerationalVector::with_capacity(10);
    /// assert_eq!(vec.capacity(), 10);
    /// ```
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Inserts an element into the vector. This method will prefer
    /// replacing empty slots over growing the underlying array.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_vector::{GenerationalVector, DeletionResult};
    ///
    /// let mut v = GenerationalVector::default();
    ///
    /// let a = v.push("a");
    /// let b = v.push("b");
    /// assert_eq!(v.len(), 2);
    /// ```
    pub fn push(&mut self, value: TEntry) -> GenerationalIndex<TGeneration> {
        let index = match self.data.get_mut(self.free_head) {
            None => self.insert_tail(value),
            Some(entry) => entry.reuse(value, &mut self.free_head),
        };

        self.len += 1;
        index
    }

    /// Inserts at the end of the vector.
    fn insert_tail(&mut self, value: TEntry) -> GenerationalIndex<TGeneration> {
        let generation = TGeneration::zero();
        let index = GenerationalIndex::new(self.data.len(), generation);
        let gen_entry = GenerationalEntry::new_from_value(value, generation);
        self.data.push(gen_entry);
        self.free_head = index.index + 1;
        index
    }

    /// Retrieves the element at the specified index.
    ///
    /// ## Arguments
    /// * `index` - The index of the element.
    ///
    /// ## Returns
    /// `None` if the element does not exist; `Some` element otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_vector::{GenerationalVector, DeletionResult};
    ///
    /// let mut v = GenerationalVector::default();
    /// let a = v.push("a");
    /// let b = v.push("b");
    ///
    /// assert_eq!(v.get(&a).unwrap(), &"a");
    /// assert_eq!(v.get(&b).unwrap(), &"b");
    ///
    /// v.remove(&b);
    /// assert_eq!(v.get(&b), None);
    ///
    /// let c = v.push("c");
    /// assert_eq!(v.get(&b), None);
    /// assert_eq!(v.get(&c).unwrap(), &"c");
    /// ```
    pub fn get<Index>(&self, index: Index) -> Option<&TEntry>
    where
        Index: Borrow<GenerationalIndex<TGeneration>>,
    {
        let index = index.borrow();

        // Apply boundary check for the index.
        let entry = self.data.get(index.index);
        if entry.is_none() {
            return None;
        }

        let entry = entry.unwrap();
        if let Slot::Occupied(value) = &entry.entry {
            if entry.generation == index.generation {
                return Some(value);
            }
        }

        None
    }

    /// Removes an element from the vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_vector::{GenerationalVector, DeletionResult};
    ///
    /// let mut v = GenerationalVector::default();
    ///
    /// let a = v.push("a");
    /// let b = v.push("b");
    ///
    /// assert_eq!(v.remove(&a), DeletionResult::Ok);
    /// assert_eq!(v.remove(&b), DeletionResult::Ok);
    /// assert_eq!(v.remove(&b), DeletionResult::NotFound);
    /// assert_eq!(v.len(), 0);
    ///
    /// let c = v.push("c");
    /// assert_eq!(v.remove(&b), DeletionResult::InvalidGeneration);
    /// assert_eq!(v.len(), 1);
    /// ```
    pub fn remove(&mut self, index: &GenerationalIndex<TGeneration>) -> DeletionResult {
        let GenerationalEntry { entry, generation } = &mut self.data[index.index];

        return match entry {
            Slot::Occupied { .. } => {
                if *generation != index.generation {
                    return DeletionResult::InvalidGeneration;
                }

                *generation = generation.add(TGeneration::one());
                *entry = Slot::Free {
                    next_free: self.free_head,
                };
                self.free_head = index.index;
                self.len -= 1;
                DeletionResult::Ok
            }
            _ => DeletionResult::NotFound,
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
