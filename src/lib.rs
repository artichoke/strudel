/* This is a public domain general purpose hash table package
originally written by Peter Moore @ UCB.

The hash table data structures were redesigned and the package was
rewritten by Vladimir Makarov <vmakarov@redhat.com>.  */

/* The original package implemented classic bucket-based hash tables
   with entries doubly linked for an access by their insertion order.
   To decrease pointer chasing and as a consequence to improve a data
   locality the current implementation is based on storing entries in
   an array and using hash tables with open addressing.  The current
   entries are more compact in comparison with the original ones and
   this also improves the data locality.

   The hash table has two arrays called *bins* and *entries*.

     bins:
    -------
   |       |                  entries array:
   |-------|            --------------------------------
   | index |           |      | entry:  |        |      |
   |-------|           |      |         |        |      |
   | ...   |           | ...  | hash    |  ...   | ...  |
   |-------|           |      | key     |        |      |
   | empty |           |      | record  |        |      |
   |-------|            --------------------------------
   | ...   |                   ^                  ^
   |-------|                   |_ entries start   |_ entries bound
   |deleted|
    -------

   o The entry array contains table entries in the same order as they
     were inserted.

     When the first entry is deleted, a variable containing index of
     the current first entry (*entries start*) is changed.  In all
     other cases of the deletion, we just mark the entry as deleted by
     using a reserved hash value.

     Such organization of the entry storage makes operations of the
     table shift and the entries traversal very fast.

   o The bins provide access to the entries by their keys.  The
     key hash is mapped to a bin containing *index* of the
     corresponding entry in the entry array.

     The bin array size is always power of two, it makes mapping very
     fast by using the corresponding lower bits of the hash.
     Generally it is not a good idea to ignore some part of the hash.
     But alternative approach is worse.  For example, we could use a
     modulo operation for mapping and a prime number for the size of
     the bin array.  Unfortunately, the modulo operation for big
     64-bit numbers are extremely slow (it takes more than 100 cycles
     on modern Intel CPUs).

     Still other bits of the hash value are used when the mapping
     results in a collision.  In this case we use a secondary hash
     value which is a result of a function of the collision bin
     index and the original hash value.  The function choice
     guarantees that we can traverse all bins and finally find the
     corresponding bin as after several iterations the function
     becomes a full cycle linear congruential generator because it
     satisfies requirements of the Hull-Dobell theorem.

     When an entry is removed from the table besides marking the
     hash in the corresponding entry described above, we also mark
     the bin by a special value in order to find entries which had
     a collision with the removed entries.

     There are two reserved values for the bins.  One denotes an
     empty bin, another one denotes a bin for a deleted entry.

   o The length of the bin array is at least two times more than the
     entry array length.  This keeps the table load factor healthy.
     The trigger of rebuilding the table is always a case when we can
     not insert an entry anymore at the entries bound.  We could
     change the entries bound too in case of deletion but than we need
     a special code to count bins with corresponding deleted entries
     and reset the bin values when there are too many bins
     corresponding deleted entries

     Table rebuilding is done by creation of a new entry array and
     bins of an appropriate size.  We also try to reuse the arrays
     in some cases by compacting the array and removing deleted
     entries.

   o To save memory very small tables have no allocated arrays
     bins.  We use a linear search for an access by a key.

   o To save more memory we use 8-, 16-, 32- and 64- bit indexes in
     bins depending on the current hash table size.

   o The implementation takes into account that the table can be
     rebuilt during hashing or comparison functions.  It can happen if
     the functions are implemented in Ruby and a thread switch occurs
     during their execution.

   This implementation speeds up the Ruby hash table benchmarks in
   average by more 40% on Intel Haswell CPU.

*/

#![allow(non_camel_case_types)]

use core::iter::{FromIterator, FusedIterator};
use std::collections::{hash_map, HashMap};

#[cfg(feature = "capi")]
pub mod capi;
mod hasher;

pub use hasher::{st_hash_t, st_hash_type, StHasher};

#[cfg(target_pointer_width = "64")]
pub type st_data_t = u64;
#[cfg(target_pointer_width = "32")]
pub type st_data_t = u32;

pub type st_index_t = st_data_t;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct StHash {
    map: HashMap<st_data_t, st_data_t, StHasher>,
}

impl StHash {
    #[inline]
    #[must_use]
    pub fn with_hash_type(hash_type: *const st_hash_type) -> Self {
        let hasher = StHasher::from(hash_type);
        Self {
            map: HashMap::with_hasher(hasher),
        }
    }

    #[inline]
    #[must_use]
    pub fn with_capacity_and_hash_type(capacity: usize, hash_type: *const st_hash_type) -> Self {
        let hasher = StHasher::from(hash_type);
        Self {
            map: HashMap::with_capacity_and_hasher(capacity, hasher),
        }
    }

    #[inline]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.map.clear();
    }

    #[inline]
    #[must_use]
    pub fn contains_key(&self, key: st_data_t) -> bool {
        self.map.contains_key(&key)
    }

    #[inline]
    #[must_use]
    pub fn get(&self, key: st_data_t) -> Option<&st_data_t> {
        self.map.get(&key)
    }

    #[inline]
    #[must_use]
    pub fn insert(&mut self, key: st_data_t, value: st_data_t) -> Option<st_data_t> {
        self.map.insert(key, value)
    }

    #[inline]
    #[must_use]
    pub fn remove(&mut self, key: st_data_t) -> Option<st_data_t> {
        self.map.remove(&key)
    }

    #[inline]
    #[must_use]
    pub fn hasher(&self) -> &StHasher {
        self.map.hasher()
    }

    #[inline]
    #[must_use]
    pub fn iter(&self) -> Iter<'_> {
        Iter(self.map.iter())
    }

    #[inline]
    #[must_use]
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        IterMut(self.map.iter_mut())
    }

    #[inline]
    #[must_use]
    pub fn keys(&self) -> Keys<'_> {
        Keys(self.map.keys())
    }

    #[inline]
    #[must_use]
    pub fn values(&mut self) -> Values<'_> {
        Values(self.map.values())
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.map.reserve(additional);
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.map.shrink_to_fit();
    }
}

#[derive(Debug, Clone)]
pub struct Iter<'a>(hash_map::Iter<'a, st_data_t, st_data_t>);

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a st_data_t, &'a st_data_t);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.0.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.0.last()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n)
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B {
        self.0.collect()
    }
}

impl<'a> FusedIterator for Iter<'a> {}

impl<'a> ExactSizeIterator for Iter<'a> {}

#[derive(Debug)]
pub struct IterMut<'a>(hash_map::IterMut<'a, st_data_t, st_data_t>);

impl<'a> Iterator for IterMut<'a> {
    type Item = (&'a st_data_t, &'a mut st_data_t);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.0.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.0.last()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n)
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B {
        self.0.collect()
    }
}

impl<'a> FusedIterator for IterMut<'a> {}

impl<'a> ExactSizeIterator for IterMut<'a> {}

#[derive(Debug)]
pub struct Keys<'a>(hash_map::Keys<'a, st_data_t, st_data_t>);

impl<'a> Iterator for Keys<'a> {
    type Item = &'a st_data_t;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.0.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.0.last()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n)
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B {
        self.0.collect()
    }
}

impl<'a> FusedIterator for Keys<'a> {}

impl<'a> ExactSizeIterator for Keys<'a> {}

#[derive(Debug)]
pub struct Values<'a>(hash_map::Values<'a, st_data_t, st_data_t>);

impl<'a> Iterator for Values<'a> {
    type Item = &'a st_data_t;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.0.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.0.last()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n)
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B {
        self.0.collect()
    }
}

impl<'a> FusedIterator for Values<'a> {}

impl<'a> ExactSizeIterator for Values<'a> {}

impl<'a> IntoIterator for &'a StHash {
    type Item = (&'a st_data_t, &'a st_data_t);
    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
