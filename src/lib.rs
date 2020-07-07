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

use core::borrow::Borrow;
use core::hash::{Hash, Hasher};
use core::iter::{FromIterator, FusedIterator};
use core::mem::size_of;
use std::collections::hash_map::Entry as HashEntry;
use std::collections::{btree_map, BTreeMap, HashMap};
use std::vec;

#[cfg(feature = "capi")]
pub mod capi;
mod entry;
mod fnv;
mod hasher;

use entry::{Entry, OccupiedEntry, VacantEntry};
pub use hasher::{st_hash_t, st_hash_type, StBuildHasher, StHasher};

pub mod st_hashmap {
    pub use crate::entry::{Entry, OccupiedEntry, VacantEntry};
}

#[cfg(target_pointer_width = "64")]
pub type st_data_t = u64;
#[cfg(target_pointer_width = "32")]
pub type st_data_t = u32;

pub type st_index_t = st_data_t;

#[derive(Debug, Clone)]
struct Key {
    insert_counter: st_index_t,
    lookup: LookupKey,
}

impl Key {
    #[inline]
    #[must_use]
    fn insert_counter(&self) -> st_index_t {
        self.insert_counter
    }

    #[inline]
    #[must_use]
    fn lookup_key(&self) -> &LookupKey {
        &self.lookup
    }

    #[inline]
    #[must_use]
    fn record(&self) -> &st_data_t {
        &self.lookup.record
    }

    #[inline]
    #[must_use]
    fn into_record(self) -> st_data_t {
        self.lookup.record
    }
}

impl PartialEq for Key {
    #[inline]
    fn eq(&self, other: &Key) -> bool {
        let cmp = self.lookup.eq;
        unsafe { (cmp)(self.lookup.record, other.lookup.record) == 0 }
    }
}

impl PartialEq<LookupKey> for Key {
    #[inline]
    fn eq(&self, other: &LookupKey) -> bool {
        let cmp = self.lookup.eq;
        unsafe { (cmp)(self.lookup.record, other.record) == 0 }
    }
}

impl Eq for Key {}

impl Hash for Key {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        #[cfg(target_pointer_width = "32")]
        state.write_u32(self.lookup.record);
        #[cfg(target_pointer_width = "64")]
        state.write_u64(self.lookup.record);
    }
}

#[derive(Debug, Clone)]
struct LookupKey {
    record: st_data_t,
    eq: unsafe extern "C" fn(st_data_t, st_data_t) -> i32,
}

impl PartialEq for LookupKey {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        let cmp = self.eq;
        unsafe { (cmp)(self.record, other.record) == 0 }
    }
}

impl PartialEq<Key> for LookupKey {
    #[inline]
    fn eq(&self, other: &Key) -> bool {
        let cmp = self.eq;
        unsafe { (cmp)(self.record, other.lookup.record) == 0 }
    }
}

impl Eq for LookupKey {}

impl Hash for LookupKey {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        #[cfg(target_pointer_width = "32")]
        state.write_u32(self.record);
        #[cfg(target_pointer_width = "64")]
        state.write_u64(self.record);
    }
}

impl Borrow<LookupKey> for Key {
    #[inline]
    fn borrow(&self) -> &LookupKey {
        self.lookup_key()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StHash {
    map: HashMap<Key, st_data_t, StBuildHasher>,
    ordered: BTreeMap<st_index_t, (st_data_t, st_data_t)>,
    eq: unsafe extern "C" fn(st_data_t, st_data_t) -> i32,
    insert_counter: st_index_t,
}

impl Default for StHash {
    #[inline]
    fn default() -> Self {
        Self {
            map: HashMap::default(),
            ordered: BTreeMap::default(),
            eq: hasher::default_compare,
            insert_counter: 0,
        }
    }
}

impl StHash {
    #[inline]
    #[must_use]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn with_hash_type(hash_type: *const st_hash_type) -> Self {
        let hasher = StBuildHasher::from(hash_type);
        let map = HashMap::with_hasher(hasher);
        Self {
            map,
            ordered: BTreeMap::new(),
            eq: unsafe { (*hash_type).compare },
            insert_counter: 0,
        }
    }

    #[inline]
    #[must_use]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn with_capacity_and_hash_type(capacity: usize, hash_type: *const st_hash_type) -> Self {
        let hasher = StBuildHasher::from(hash_type);
        let map = HashMap::with_capacity_and_hasher(capacity, hasher);
        Self {
            map,
            ordered: BTreeMap::new(),
            eq: unsafe { (*hash_type).compare },
            insert_counter: 0,
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
        self.ordered.clear();
        self.insert_counter = 0;
    }

    #[inline]
    #[must_use]
    pub fn contains_key(&self, key: st_data_t) -> bool {
        let key = LookupKey {
            record: key,
            eq: self.eq,
        };
        self.map.contains_key(&key)
    }

    #[inline]
    #[must_use]
    pub fn get(&self, key: st_data_t) -> Option<&st_data_t> {
        let key = LookupKey {
            record: key,
            eq: self.eq,
        };
        self.map.get(&key)
    }

    #[inline]
    #[must_use]
    pub fn get_key_value(&self, key: st_data_t) -> Option<(&st_data_t, &st_data_t)> {
        let key = LookupKey {
            record: key,
            eq: self.eq,
        };
        let (key, value) = self.map.get_key_value(&key)?;
        Some((key.record(), value))
    }

    #[inline]
    #[must_use]
    pub fn first(&self) -> Option<(&st_data_t, &st_data_t)> {
        self.iter().next()
    }

    #[inline]
    #[must_use]
    pub fn last(&self) -> Option<(&st_data_t, &st_data_t)> {
        self.iter().last()
    }

    #[inline]
    #[must_use]
    pub fn get_nth(&self, n: st_index_t) -> Option<(&st_data_t, &st_data_t)> {
        self.ordered.get(&n).map(|(key, value)| (key, value))
    }

    #[inline]
    #[must_use]
    pub fn max_insert_rank(&self) -> st_index_t {
        self.ordered.keys().last().copied().unwrap_or_default()
    }

    #[inline]
    #[must_use]
    pub fn min_insert_rank(&self) -> st_index_t {
        self.ordered.keys().next().copied().unwrap_or_default()
    }

    #[inline]
    pub fn entry(&mut self, key: st_data_t) -> Entry<'_> {
        let key = LookupKey {
            record: key,
            eq: self.eq,
        };
        let key = Key {
            lookup: key,
            insert_counter: self.insert_counter,
        };
        self.insert_counter += 1;
        match self.map.entry(key) {
            HashEntry::Occupied(base) => Entry::Occupied(OccupiedEntry(base)),
            HashEntry::Vacant(base) => Entry::Vacant(VacantEntry(base)),
        }
    }

    #[inline]
    #[must_use]
    pub fn insert(&mut self, key: st_data_t, value: st_data_t) -> Option<st_data_t> {
        let insert_counter = self.insert_counter;
        self.insert_counter += 1;
        let key_data = key;

        let key = LookupKey {
            record: key,
            eq: self.eq,
        };
        let key = Key {
            lookup: key,
            insert_counter,
        };
        let (counter, old_value) = match self.map.entry(key) {
            HashEntry::Occupied(mut base) => {
                let old_value = base.insert(value);
                (base.key().insert_counter(), Some(old_value))
            }
            HashEntry::Vacant(base) => {
                base.insert(value);
                (insert_counter, None)
            }
        };
        self.ordered.insert(counter, (key_data, value));
        old_value
    }

    #[inline]
    pub fn update(&mut self, key: st_data_t, value: st_data_t) {
        let key_data = key;
        let key = LookupKey {
            record: key,
            eq: self.eq,
        };
        if let Some((mut entry_key, _)) = self.map.remove_entry(&key) {
            self.ordered
                .insert(entry_key.insert_counter(), (key_data, value));
            if &key_data != entry_key.record() {
                entry_key.lookup.record = key_data;
                self.map.insert(entry_key, value);
            }
        } else {
            let _ = self.insert(key_data, value);
        }
    }

    #[inline]
    #[must_use]
    pub fn delete(&mut self, key: st_data_t) -> Option<(st_data_t, st_data_t)> {
        let key = LookupKey {
            record: key,
            eq: self.eq,
        };
        let (key, value) = self.map.remove_entry(&key)?;
        Some((key.into_record(), value))
    }

    #[inline]
    #[must_use]
    pub fn remove(&mut self, key: st_data_t) -> Option<st_data_t> {
        let key = LookupKey {
            record: key,
            eq: self.eq,
        };
        self.map.remove(&key)
    }

    #[inline]
    #[must_use]
    pub fn hasher(&self) -> &StBuildHasher {
        self.map.hasher()
    }

    #[inline]
    #[must_use]
    pub fn insert_ranks_from(&self, rank: st_index_t) -> InsertRanks {
        let ranks = self
            .ordered
            .range(rank..)
            .map(|(&rank, _)| rank)
            .collect::<Vec<_>>();
        InsertRanks(ranks.into_iter())
    }

    #[inline]
    #[must_use]
    pub fn iter(&self) -> Iter<'_> {
        Iter(self.ordered.values())
    }

    #[inline]
    #[must_use]
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        IterMut(self.ordered.values_mut())
    }

    #[inline]
    #[must_use]
    pub fn keys(&self) -> Keys<'_> {
        Keys(self.iter())
    }

    #[inline]
    #[must_use]
    pub fn values(&self) -> Values<'_> {
        Values(self.iter())
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.map.reserve(additional);
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.map.shrink_to_fit();
    }

    #[inline]
    pub fn estimated_memsize(&self) -> usize {
        size_of::<Self>()
            + (size_of::<Key>() + size_of::<st_data_t>()) * self.map.capacity()
            + (size_of::<st_index_t>() + size_of::<(st_data_t, st_data_t)>()) * self.ordered.len()
    }
}

#[derive(Debug, Clone)]
pub struct Iter<'a>(btree_map::Values<'a, st_index_t, (st_data_t, st_data_t)>);

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a st_data_t, &'a st_data_t);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(key, value)| (key, value))
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
        self.0.last().map(|(key, value)| (key, value))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n).map(|(key, value)| (key, value))
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B {
        self.0.map(|(key, value)| (key, value)).collect()
    }
}

impl<'a> FusedIterator for Iter<'a> {}

impl<'a> ExactSizeIterator for Iter<'a> {}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(key, value)| (key, value))
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n).map(|(key, value)| (key, value))
    }
}

#[derive(Debug)]
pub struct IterMut<'a>(btree_map::ValuesMut<'a, st_index_t, (st_data_t, st_data_t)>);

impl<'a> Iterator for IterMut<'a> {
    type Item = (&'a st_data_t, &'a mut st_data_t);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(key, value)| (&*key, value))
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
        self.0.last().map(|(key, value)| (&*key, value))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n).map(|(key, value)| (&*key, value))
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B {
        self.0.map(|(key, value)| (&*key, value)).collect()
    }
}

impl<'a> FusedIterator for IterMut<'a> {}

impl<'a> ExactSizeIterator for IterMut<'a> {}

impl<'a> DoubleEndedIterator for IterMut<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(key, value)| (&*key, value))
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n).map(|(key, value)| (&*key, value))
    }
}

#[derive(Debug)]
pub struct Keys<'a>(Iter<'a>);

impl<'a> Iterator for Keys<'a> {
    type Item = &'a st_data_t;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(key, _)| key)
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
        self.0.last().map(|(key, _)| key)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n).map(|(key, _)| key)
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B {
        self.0.map(|(key, _)| key).collect()
    }
}

impl<'a> FusedIterator for Keys<'a> {}

impl<'a> ExactSizeIterator for Keys<'a> {}

impl<'a> DoubleEndedIterator for Keys<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(key, _)| key)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n).map(|(key, _)| key)
    }
}

#[derive(Debug)]
pub struct Values<'a>(Iter<'a>);

impl<'a> Iterator for Values<'a> {
    type Item = &'a st_data_t;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, value)| value)
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
        self.0.last().map(|(_, value)| value)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n).map(|(_, value)| value)
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B {
        self.0.map(|(_, value)| value).collect()
    }
}

impl<'a> FusedIterator for Values<'a> {}

impl<'a> ExactSizeIterator for Values<'a> {}

impl<'a> DoubleEndedIterator for Values<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(_, value)| value)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n).map(|(_, value)| value)
    }
}

#[derive(Debug)]
pub struct InsertRanks(vec::IntoIter<st_index_t>);

impl<'a> Iterator for InsertRanks {
    type Item = st_index_t;

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

impl<'a> FusedIterator for InsertRanks {}

impl<'a> ExactSizeIterator for InsertRanks {}

impl<'a> DoubleEndedIterator for InsertRanks {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n)
    }
}

impl<'a> IntoIterator for &'a StHash {
    type Item = (&'a st_data_t, &'a st_data_t);
    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
