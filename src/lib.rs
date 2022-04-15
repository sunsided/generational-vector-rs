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

impl<TEntry, TGeneration> IntoIterator for GenerationalVector<TEntry, TGeneration>
where
    TGeneration: GenerationType,
{
    type Item = TEntry;
    type IntoIter = iterators::EntryIntoIterator<TEntry, TGeneration>;

    fn into_iter(self) -> Self::IntoIter {
        iterators::EntryIntoIterator { vec: self.data }
    }
}

impl<'a, TEntry, TGeneration> IntoIterator for &'a GenerationalVector<TEntry, TGeneration>
where
    TGeneration: GenerationType,
{
    type Item = &'a TEntry;
    type IntoIter = iterators::EntryIterator<'a, TEntry, TGeneration>;

    fn into_iter(self) -> Self::IntoIter {
        iterators::EntryIterator {
            current: 0,
            vec: &self.data,
        }
    }
}

impl<'a, TEntry, TGeneration> IntoIterator for &'a mut GenerationalVector<TEntry, TGeneration>
where
    TGeneration: GenerationType,
{
    type Item = &'a mut TEntry;
    type IntoIter = iterators::EntryMutIterator<'a, TEntry, TGeneration>;

    fn into_iter(self) -> Self::IntoIter {
        iterators::EntryMutIterator {
            current: 0,
            vec: &mut self.data,
        }
    }
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

    /// Initializes the vector from an existing vector.
    pub fn new_from_vec(vec: Vec<TEntry>) -> Self {
        let len = vec.len();
        let mut data = Vec::with_capacity(len);
        for entry in vec {
            data.push(GenerationalEntry::new_from_value(entry, TGeneration::one()));
        }

        Self {
            data,
            free_list: Vec::new(),
            len,
        }
    }

    /// Initializes the vector from an iterator.
    pub fn new_from_iter<TIter: IntoIterator<Item = TEntry>>(vec: TIter) -> Self {
        let data: Vec<GenerationalEntry<TEntry, TGeneration>> = vec
            .into_iter()
            .map(|entry| GenerationalEntry::new_from_value(entry, TGeneration::one()))
            .collect();
        let len = data.len();

        Self {
            data,
            free_list: Vec::new(),
            len,
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

impl<TEntry, TGeneration> From<Vec<TEntry>> for GenerationalVector<TEntry, TGeneration>
where
    TGeneration: GenerationType,
{
    fn from(vec: Vec<TEntry>) -> Self {
        Self::new_from_vec(vec)
    }
}

/// Iterator implementations.
pub mod iterators {
    use super::*;

    /// Iterator for owned values.
    pub struct EntryIntoIterator<TEntry, TGeneration>
    where
        TGeneration: GenerationType,
    {
        pub(crate) vec: Vec<GenerationalEntry<TEntry, TGeneration>>,
    }

    /// Iterator for owned values.
    pub struct EntryIterator<'a, TEntry, TGeneration>
    where
        TGeneration: GenerationType,
    {
        pub(crate) current: usize,
        pub(crate) vec: &'a Vec<GenerationalEntry<TEntry, TGeneration>>,
    }

    pub struct EntryMutIterator<'a, TEntry, TGeneration>
    where
        TGeneration: GenerationType,
    {
        pub(crate) current: usize,
        pub(crate) vec: &'a mut Vec<GenerationalEntry<TEntry, TGeneration>>,
    }

    impl<TEntry, TGeneration> Iterator for EntryIntoIterator<TEntry, TGeneration>
    where
        TGeneration: GenerationType,
    {
        type Item = TEntry;

        fn next(&mut self) -> Option<Self::Item> {
            while !self.vec.is_empty() {
                match self.vec.pop() {
                    None => continue,
                    Some(entry) => return entry.entry,
                }
            }

            None
        }
    }

    impl<'a, TEntry, TGeneration> Iterator for EntryIterator<'a, TEntry, TGeneration>
    where
        TGeneration: GenerationType,
    {
        type Item = &'a TEntry;

        fn next(&mut self) -> Option<Self::Item> {
            while self.current < self.vec.len() {
                let entry = &self.vec[self.current];
                self.current += 1;
                let entry = entry.entry.as_ref();
                if entry.is_some() {
                    return Some(entry.unwrap());
                }
            }

            None
        }
    }

    impl<'a, TEntry, TGeneration> Iterator for EntryMutIterator<'a, TEntry, TGeneration>
    where
        TGeneration: GenerationType,
    {
        type Item = &'a mut TEntry;

        fn next(&mut self) -> Option<Self::Item> {
            let ptr = self.vec.as_mut_ptr();

            while self.current < self.vec.len() {
                let element = unsafe { &mut *ptr.add(self.current) };
                let entry = element.entry.as_mut();
                self.current += 1;

                if entry.is_some() {
                    return entry;
                }
            }

            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::num::{NonZeroU8, NonZeroUsize};

    #[test]
    fn insert_after_delete_generation_changes() {
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
    }

    #[test]
    fn delete_all_free_list_updates() {
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

        assert_eq!(gv.count_num_free(), 3);
    }

    #[test]
    fn delete_all_reverse_free_list_changes() {
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
        assert_eq!(gv.count_num_free(), 3);
    }

    #[test]
    fn delete_all_and_insert_indexes_are_set_in_order() {
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
