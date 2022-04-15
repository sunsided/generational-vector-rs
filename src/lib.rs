mod default_generation_type;

use crate::default_generation_type::DefaultGenerationType;
use num_traits::One;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::ops::Add;

/// Alias for required traits on the type used for the generation value.
pub trait GenerationType: One + Copy + Add<Output = Self> + PartialEq {}

/// Automatic implementation of `GenerationType` for all matching types.
impl<T> GenerationType for T where T: One + Copy + Add<Output = T> + PartialEq {}

/// An index entry in the `GenerationalVector`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GenerationalIndex<TGeneration> {
    index: usize,
    generation: TGeneration,
}

/// An index entry
#[derive(Debug)]
struct GenerationalEntry<TEntry, TGeneration> {
    /// The generation of the entry. A value of zero always encodes an empty value.
    generation: TGeneration,
    entry: Option<TEntry>,
}

/// A vector that utilizes generational indexing to access the elements.
pub struct GenerationalVector<TEntry, TGeneration = DefaultGenerationType>
where
    TGeneration: GenerationType,
{
    data: Vec<GenerationalEntry<TEntry, TGeneration>>,
    free_list: Vec<usize>,
    len: usize,
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

/// A vector whose elements are addressed by both an index and an entry
/// generation.
impl<TEntry, TGeneration> GenerationalVector<TEntry, TGeneration>
where
    TGeneration: GenerationType,
{
    /// Initializes a new, empty vector.
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            free_list: Vec::new(),
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
            free_list: Vec::new(),
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
        self.free_list.len()
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
        let index = match self.free_list.is_empty() {
            true => self.insert_tail(value),
            false => {
                let free_index = self
                    .free_list
                    .pop()
                    .expect("expected free_list to contain values");
                self.data[free_index].reuse(value, free_index)
            }
        };

        self.len += 1;
        index
    }

    /// Inserts at the end of the vector.
    fn insert_tail(&mut self, value: TEntry) -> GenerationalIndex<TGeneration> {
        let generation = TGeneration::one();
        let index = GenerationalIndex::new(self.data.len(), generation);
        let gen_entry = GenerationalEntry::new_from_value(value, generation);
        self.data.push(gen_entry);
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
        if let Some(value) = &entry.entry {
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
            Some { .. } => {
                if *generation != index.generation {
                    return DeletionResult::InvalidGeneration;
                }

                *entry = None;
                *generation = generation.add(TGeneration::one());
                self.free_list.push(index.index);
                self.len -= 1;
                DeletionResult::Ok
            }
            _ => DeletionResult::NotFound,
        };
    }
}

impl<TEntry> Default for GenerationalVector<TEntry, DefaultGenerationType> {
    fn default() -> Self {
        GenerationalVector::<TEntry, DefaultGenerationType>::new()
    }
}

impl<TGeneration> GenerationalIndex<TGeneration> {
    fn new(index: usize, generation: TGeneration) -> Self {
        Self { index, generation }
    }
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

impl<TEntry, TGeneration> GenerationalEntry<TEntry, TGeneration>
where
    TGeneration: GenerationType,
{
    #[inline]
    fn new_from_value(value: TEntry, generation: TGeneration) -> Self {
        Self {
            entry: Some(value),
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
    pub fn reuse(&mut self, value: TEntry, vec_index: usize) -> GenerationalIndex<TGeneration> {
        if self.entry.is_none() {
            let key = GenerationalIndex::new(vec_index, self.generation);
            self.entry = Some(value);
            return key;
        }

        panic!("free list is corrupted");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::num::{NonZeroU8, NonZeroUsize};

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
        assert!(a.generation < e.generation);
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
        assert_eq!(gv.free_list.len(), 3);
        assert_eq!(*gv.free_list.last().unwrap(), 2);

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
        assert_eq!(gv.free_list.len(), 3);
        assert_eq!(*gv.free_list.last().unwrap(), 0);

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

    #[test]
    fn sizeof() {
        assert_eq!(std::mem::size_of::<GenerationalEntry<u8, usize>>(), 16);
        assert_eq!(std::mem::size_of::<GenerationalEntry<u8, u32>>(), 8);
        assert_eq!(std::mem::size_of::<GenerationalEntry<u8, u16>>(), 4);
        assert_eq!(std::mem::size_of::<GenerationalEntry<u8, u8>>(), 3);

        assert_eq!(
            std::mem::size_of::<GenerationalEntry<NonZeroU8, NonZeroUsize>>(),
            16
        );
        assert_eq!(
            std::mem::size_of::<GenerationalEntry<NonZeroU8, NonZeroU8>>(),
            2
        );
    }
}
