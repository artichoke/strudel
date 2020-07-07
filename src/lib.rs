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
use std::collections::{hash_map, HashMap};

#[cfg(feature = "capi")]
pub mod capi;
mod fnv;
mod hasher;

pub use hasher::{st_hash_t, st_hash_type, StBuildHasher, StHasher};

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
    fn lookup_key(&self) -> &LookupKey {
        &self.lookup
    }

    #[inline]
    fn record(&self) -> &st_data_t {
        &self.lookup.record
    }

    #[inline]
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
    eq: unsafe extern "C" fn(st_data_t, st_data_t) -> i32,
    insert_counter: st_index_t,
}

impl Default for StHash {
    #[inline]
    fn default() -> Self {
        Self {
            map: HashMap::default(),
            eq: hasher::default_compare,
            insert_counter: 0,
        }
    }
}

impl StHash {
    #[inline]
    #[must_use]
    pub fn with_hash_type(hash_type: *const st_hash_type) -> Self {
        let hasher = StBuildHasher::from(hash_type);
        let map = HashMap::with_hasher(hasher);
        Self {
            map,
            eq: unsafe { (*hash_type).compare },
            insert_counter: 0,
        }
    }

    #[inline]
    #[must_use]
    pub fn with_capacity_and_hash_type(capacity: usize, hash_type: *const st_hash_type) -> Self {
        let hasher = StBuildHasher::from(hash_type);
        let map = HashMap::with_capacity_and_hasher(capacity, hasher);
        Self {
            map,
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
            hash_map::Entry::Occupied(base) => Entry::Occupied(OccupiedEntry(base)),
            hash_map::Entry::Vacant(base) => Entry::Vacant(VacantEntry(base)),
        }
    }

    #[inline]
    #[must_use]
    pub fn insert(&mut self, key: st_data_t, value: st_data_t) -> Option<st_data_t> {
        let key = LookupKey {
            record: key,
            eq: self.eq,
        };
        let key = Key {
            lookup: key,
            insert_counter: self.insert_counter,
        };
        self.insert_counter += 1;
        self.map.insert(key, value)
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
}

#[derive(Debug, Clone)]
pub struct Iter<'a>(hash_map::Iter<'a, Key, st_data_t>);

impl<'a> Iter<'a> {
    pub fn ordered(self) -> Vec<(&'a st_data_t, &'a st_data_t)> {
        let mut pairs = self.0.collect::<Vec<_>>();
        pairs.sort_by(|(left, _), (right, _)| left.insert_counter.cmp(&right.insert_counter));
        pairs
            .into_iter()
            .map(|(key, value)| (key.record(), value))
            .collect()
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a st_data_t, &'a st_data_t);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (key, value) = self.0.next()?;
        Some((key.record(), value))
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
        let (key, value) = self.0.last()?;
        Some((key.record(), value))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let (key, value) = self.0.nth(n)?;
        Some((key.record(), value))
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B {
        self.0.map(|(key, value)| (key.record(), value)).collect()
    }
}

impl<'a> FusedIterator for Iter<'a> {}

impl<'a> ExactSizeIterator for Iter<'a> {}

#[derive(Debug)]
pub struct IterMut<'a>(hash_map::IterMut<'a, Key, st_data_t>);

impl<'a> IterMut<'a> {
    pub fn ordered(self) -> Vec<(&'a st_data_t, &'a mut st_data_t)> {
        let mut pairs = self.0.collect::<Vec<_>>();
        pairs.sort_by(|(left, _), (right, _)| left.insert_counter.cmp(&right.insert_counter));
        pairs
            .into_iter()
            .map(|(key, value)| (key.record(), value))
            .collect()
    }
}

impl<'a> Iterator for IterMut<'a> {
    type Item = (&'a st_data_t, &'a mut st_data_t);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (key, value) = self.0.next()?;
        Some((key.record(), value))
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
        let (key, value) = self.0.last()?;
        Some((key.record(), value))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let (key, value) = self.0.nth(n)?;
        Some((key.record(), value))
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B {
        self.0.map(|(key, value)| (key.record(), value)).collect()
    }
}

impl<'a> FusedIterator for IterMut<'a> {}

impl<'a> ExactSizeIterator for IterMut<'a> {}

#[derive(Debug)]
pub struct Keys<'a>(hash_map::Keys<'a, Key, st_data_t>);

impl<'a> Keys<'a> {
    pub fn ordered(self) -> Vec<&'a st_data_t> {
        let mut pairs = self.0.collect::<Vec<_>>();
        pairs.sort_by(|left, right| left.insert_counter.cmp(&right.insert_counter));
        pairs.into_iter().map(|key| key.record()).collect()
    }
}

impl<'a> Iterator for Keys<'a> {
    type Item = &'a st_data_t;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let key = self.0.next()?;
        Some(key.record())
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
        let key = self.0.last()?;
        Some(key.record())
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let key = self.0.nth(n)?;
        Some(key.record())
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B {
        self.0.map(|key| key.record()).collect()
    }
}

impl<'a> FusedIterator for Keys<'a> {}

impl<'a> ExactSizeIterator for Keys<'a> {}

#[derive(Debug)]
pub struct Values<'a>(Iter<'a>);

impl<'a> Values<'a> {
    pub fn ordered(self) -> Vec<&'a st_data_t> {
        let pairs = self.0.ordered();
        pairs.into_iter().map(|(_, value)| value).collect()
    }
}

impl<'a> Iterator for Values<'a> {
    type Item = &'a st_data_t;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (_, value) = self.0.next()?;
        Some(value)
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
        let (_, value) = self.0.last()?;
        Some(value)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let (_, value) = self.0.nth(n)?;
        Some(value)
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B {
        self.0.map(|(_, value)| value).collect()
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

#[derive(Debug)]
pub enum Entry<'a> {
    /// An occupied entry.
    Occupied(OccupiedEntry<'a>),

    /// A vacant entry.
    Vacant(VacantEntry<'a>),
}

#[derive(Debug)]
pub struct OccupiedEntry<'a>(hash_map::OccupiedEntry<'a, Key, st_data_t>);

#[derive(Debug)]
pub struct VacantEntry<'a>(hash_map::VacantEntry<'a, Key, st_data_t>);

impl<'a> Entry<'a> {
    /// Ensures a value is in the entry by inserting the default if empty, and
    /// returns a mutable reference to the value in the entry.
    #[inline]
    pub fn or_insert(self, default: st_data_t) -> &'a mut st_data_t {
        match self {
            Self::Occupied(entry) => entry.0.into_mut(),
            Self::Vacant(entry) => entry.0.insert(default),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default
    /// function if empty, and returns a mutable reference to the value in the
    /// entry.
    #[inline]
    pub fn or_insert_with<F: FnOnce() -> st_data_t>(self, default: F) -> &'a mut st_data_t {
        match self {
            Self::Occupied(entry) => entry.0.into_mut(),
            Self::Vacant(entry) => entry.0.insert(default()),
        }
    }

    /// Ensures a value is in the entry by inserting, if empty, the result of
    /// the default function, which takes the key as its argument, and returns a
    /// mutable reference to the value in the entry.
    #[inline]
    pub fn or_insert_with_key<F: FnOnce(&st_data_t) -> st_data_t>(
        self,
        default: F,
    ) -> &'a mut st_data_t {
        match self {
            Self::Occupied(entry) => entry.0.into_mut(),
            Self::Vacant(entry) => {
                let value = default(entry.0.key().record());
                entry.insert(value)
            }
        }
    }

    /// Returns a reference to this entry's key.
    #[inline]
    pub fn key(&self) -> &st_data_t {
        match self {
            Self::Occupied(entry) => entry.0.key().record(),
            Self::Vacant(entry) => entry.0.key().record(),
        }
    }

    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the map.
    #[inline]
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut st_data_t),
    {
        match self {
            Self::Occupied(mut entry) => {
                f(entry.0.get_mut());
                Self::Occupied(entry)
            }
            Self::Vacant(entry) => Self::Vacant(entry),
        }
    }
}

impl<'a> OccupiedEntry<'a> {
    /// Gets a reference to the key in the entry.
    #[inline]
    pub fn key(&self) -> &st_data_t {
        self.0.key().record()
    }

    /// Take the ownership of the key and value from the map.
    #[inline]
    pub fn remove_entry(self) -> (st_data_t, st_data_t) {
        let (key, value) = self.0.remove_entry();
        (key.into_record(), value)
    }

    /// Gets a reference to the value in the entry.
    #[inline]
    pub fn get(&self) -> &st_data_t {
        self.0.get()
    }

    /// Gets a mutable reference to the value in the entry.
    ///
    /// If you need a reference to the `OccupiedEntry` which may outlive the
    /// destruction of the `Entry` value, see [`into_mut`].
    ///
    /// [`into_mut`]: #method.into_mut
    #[inline]
    pub fn get_mut(&mut self) -> &mut st_data_t {
        self.0.get_mut()
    }

    /// Converts the OccupiedEntry into a mutable reference to the value in the
    /// entry with a lifetime bound to the map itself.
    ///
    /// If you need multiple references to the `OccupiedEntry`, see [`get_mut`].
    ///
    /// [`get_mut`]: #method.get_mut
    #[inline]
    pub fn into_mut(self) -> &'a mut st_data_t {
        self.0.into_mut()
    }

    /// Sets the value of the entry, and returns the entry's old value.
    #[inline]
    pub fn insert(&mut self, value: st_data_t) -> st_data_t {
        self.0.insert(value)
    }

    /// Takes the value out of the entry, and returns it.
    #[inline]
    pub fn remove(self) -> st_data_t {
        self.0.remove()
    }
}

impl<'a> VacantEntry<'a> {
    /// Gets a reference to the key that would be used when inserting a value
    /// through the `VacantEntry`.
    #[inline]
    pub fn key(&self) -> &st_data_t {
        self.0.key().record()
    }

    /// Take ownership of the key.
    #[inline]
    pub fn into_key(self) -> st_data_t {
        self.0.into_key().into_record()
    }

    /// Sets the value of the entry with the VacantEntry's key, and returns a
    /// mutable reference to it.
    #[inline]
    pub fn insert(self, value: st_data_t) -> &'a mut st_data_t {
        self.0.insert(value)
    }
}
