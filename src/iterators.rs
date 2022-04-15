use crate::vector::GenerationalEntry;
use crate::GenerationType;

///! Iterator implementations.

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
