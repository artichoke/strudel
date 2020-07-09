use core::borrow::Borrow;
use core::hash::{Hash, Hasher};
use core::mem::size_of;
use std::collections::hash_map::Entry as HashEntry;
use std::collections::{BTreeMap, HashMap};

pub use crate::entry::{Entry, OccupiedEntry, VacantEntry};
pub use crate::hasher::{StBuildHasher, StHasher};
pub use crate::iter::{InsertRanks, Iter, Keys, Values};

use crate::typedefs::*;

#[derive(Debug, Clone)]
pub(crate) struct Key {
    insert_counter: st_index_t,
    lookup: LookupKey,
}

impl Key {
    #[inline]
    #[must_use]
    pub fn insert_counter(&self) -> st_index_t {
        self.insert_counter
    }

    #[inline]
    #[must_use]
    pub fn lookup_key(&self) -> &LookupKey {
        &self.lookup
    }

    #[inline]
    #[must_use]
    pub fn record(&self) -> &st_data_t {
        &self.lookup.record
    }

    #[inline]
    #[must_use]
    pub fn into_record(self) -> st_data_t {
        self.lookup.record
    }
}

impl PartialEq for Key {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.lookup == other.lookup
    }
}

impl PartialEq<LookupKey> for Key {
    #[inline]
    fn eq(&self, other: &LookupKey) -> bool {
        self.lookup == other
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
pub(crate) struct LookupKey {
    record: st_data_t,
    eq: st_compare_func,
}

impl PartialEq for LookupKey {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        if self.record == other.record {
            return true;
        }
        let cmp = self.eq;
        // Safety
        //
        // `StHash` assumes `cmp` is a valid non-NULL function pointer.
        unsafe { (cmp)(self.record, other.record) == 0 }
    }
}

impl PartialEq<&LookupKey> for LookupKey {
    #[inline]
    fn eq(&self, other: &&Self) -> bool {
        self == *other
    }
}

impl PartialEq<Key> for LookupKey {
    #[inline]
    fn eq(&self, other: &Key) -> bool {
        *self == other.lookup
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

/// An insertion-ordered hash map that implements the `st_hash` API.
///
/// `StHash` stores pointers to its keys and values and owns hasher and
/// comaparator functions given at construction time to implement [`Hash`] and
/// [`Eq`] for these opaque keys. See [`StHash::with_hash_type`].
///
/// `StHash` is built on top of a hashing algorithm selected to provide
/// resistance against HashDoS attacks. See [`RandomState`].
///
/// `StHash` supports updating keys in place. See [`StHash::update`].
///
/// The hashing algorithm must be set on a per-`StHash` basis using the
/// [`with_hash_type`] or `with_capacity_and_hash_type` methods. See
/// [`st_hash_type`].
///
/// The optional `api` and `capi` modules in `strudel` build on top of `StHash`
/// to implement a compatible C API to `st_hash`. This API includes support for
/// iterating over a mutable map and inplace updates of (`key`, `value`) pairs.
/// These features distinguish it from the [`HashMap`] in Rust `std`.
///
/// [`RandomState`]: std::collections::hash_map::RandomState
/// [`with_hash_type`]: StHash::with_hash_type
/// [`with_capacity_and_hash_type`]: StHash::with_capacity_and_hash_type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StHash {
    map: HashMap<Key, st_data_t, StBuildHasher>,
    ordered: BTreeMap<st_index_t, (st_data_t, st_data_t)>,
    eq: st_compare_func,
    insert_counter: st_index_t,
}

impl StHash {
    /// Creates an empty `StHash` which will use the given `st_hash_type` to
    /// hash keys.
    ///
    /// The created map has the default initial capacity.
    ///
    /// A [`Hasher`] is constructed from an [`StBuildHasher`].
    #[inline]
    #[must_use]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn with_hash_type(hash_type: *const st_hash_type) -> Self {
        let hasher = StBuildHasher::from(hash_type);
        let map = HashMap::with_hasher(hasher);
        let ordered = BTreeMap::new();
        // Safety:
        //
        // `StHash` assumes the `*const st_hash_type` pointer has `'static`
        // lifetime.
        // `StHash` assumes that the `compare` function pointer is non-NULL.
        let eq = unsafe { (*hash_type).compare };
        Self {
            map,
            ordered,
            eq,
            insert_counter: 0,
        }
    }

    /// Creates an empty `StHash` with the specified capacity which will use the
    /// given `st_hash_type` to hash keys.
    ///
    /// The hash map will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the hash map will not allocate.
    ///
    /// A [`Hasher`] is constructed from an [`StBuildHasher`].
    #[inline]
    #[must_use]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn with_capacity_and_hash_type(capacity: usize, hash_type: *const st_hash_type) -> Self {
        let hasher = StBuildHasher::from(hash_type);
        let map = HashMap::with_capacity_and_hasher(capacity, hasher);
        let ordered = BTreeMap::new();
        // Safety:
        //
        // `StHash` assumes the `*const st_hash_type` pointer has `'static`
        // lifetime.
        // `StHash` assumes that the `compare` function pointer is non-NULL.
        let eq = unsafe { (*hash_type).compare };
        Self {
            map,
            ordered,
            eq,
            insert_counter: 0,
        }
    }

    /// Returns the number of elements the map can hold without reallocating.
    ///
    /// This number is a lower bound; the `StHash` might be able to hold more,
    /// but is guaranteed to be able to hold at least this many.
    #[inline]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }

    /// Returns the number of elements in the map.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the map contains no elements.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Clears the map, removing all key-value pairs. Keeps the allocated memory
    /// for reuse.
    #[inline]
    pub fn clear(&mut self) {
        self.map.clear();
        self.ordered.clear();
        self.insert_counter = 0;
    }

    /// Returns `true` if the map contains a value for the specified key.
    ///
    /// The key may be any `st_data_t` as long as but [`Hash`] and [`Eq`] on the
    /// key match the stored key.
    #[inline]
    #[must_use]
    pub fn contains_key(&self, key: st_data_t) -> bool {
        let key = LookupKey {
            record: key,
            eq: self.eq,
        };
        self.map.contains_key(&key)
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// The key may be any `st_data_t` as long as but [`Hash`] and [`Eq`] on the
    /// key match the stored key.
    #[inline]
    #[must_use]
    pub fn get(&self, key: st_data_t) -> Option<&st_data_t> {
        let key = LookupKey {
            record: key,
            eq: self.eq,
        };
        self.map.get(&key)
    }

    /// Returns the key-value pair corresponding to the supplied key.
    ///
    /// The key may be any `st_data_t` as long as but [`Hash`] and [`Eq`] on the
    /// key match the stored key.
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

    /// Returns the first key-value pair in the map. The key in this pair is
    /// equal to the key inserted earliest into the map.
    ///
    /// Key-value pairs are ordered by insertion order. Insertion order is
    /// maintained if there are deletions. Insertion order is by slot, so
    /// [in-place updates to keys] maintain the same insertion position.
    ///
    /// [in-place updates to keys]: StHash::update
    #[inline]
    #[must_use]
    pub fn first(&self) -> Option<(&st_data_t, &st_data_t)> {
        self.iter().next()
    }

    /// Returns the last key-value pair in the map. The key in this pair is
    /// equal to the key inserted most recently into the map.
    ///
    /// Key-value pairs are ordered by insertion order. Insertion order is
    /// maintained if there are deletions. Insertion order is by slot, so
    /// [in-place updates to keys] maintain the same insertion position.
    ///
    /// [in-place updates to keys]: StHash::update
    #[inline]
    #[must_use]
    pub fn last(&self) -> Option<(&st_data_t, &st_data_t)> {
        self.iter().last()
    }

    /// Returns the nth key-value pair in the map. The key in this pair is
    /// equal to the key inserted nth earliest into the map.
    ///
    /// Key-value pairs are ordered by insertion order. Insertion order is
    /// maintained if there are deletions. Insertion order is by slot, so
    /// [in-place updates to keys] maintain the same insertion position.
    ///
    /// [in-place updates to keys]: StHash::update
    #[inline]
    #[must_use]
    pub fn get_nth(&self, n: st_index_t) -> Option<(&st_data_t, &st_data_t)> {
        self.ordered.get(&n).map(|(key, value)| (key, value))
    }

    /// Insertion counter for the [first] key-value pair in the map.
    ///
    /// [first]: StHash::first
    #[inline]
    #[must_use]
    pub fn min_insert_rank(&self) -> st_index_t {
        self.ordered.keys().next().copied().unwrap_or_default()
    }

    /// Insertion counter for the [last] key-value pair in the map.
    ///
    /// [last]: StHash::last
    #[inline]
    #[must_use]
    pub fn max_insert_rank(&self) -> st_index_t {
        self.ordered.keys().last().copied().unwrap_or_default()
    }

    /// Gets the given key's corresponding entry in the map for in-place
    /// manipulation.
    #[inline]
    pub fn entry(&mut self, key: st_data_t) -> Entry<'_> {
        let insert_counter = self.insert_counter;
        self.insert_counter += 1;
        let key = LookupKey {
            record: key,
            eq: self.eq,
        };
        let key = Key {
            lookup: key,
            insert_counter,
        };
        match self.map.entry(key) {
            HashEntry::Occupied(base) => Entry::Occupied(OccupiedEntry(base)),
            HashEntry::Vacant(base) => Entry::Vacant(VacantEntry(base)),
        }
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, [`None`] is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though. To update the key
    /// in-place, use [`StHash::update`].
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

        match self.map.entry(key) {
            HashEntry::Occupied(mut base) => {
                let counter = base.key().insert_counter();
                self.ordered.insert(counter, (key_data, value));
                let old_value = base.insert(value);
                Some(old_value)
            }
            HashEntry::Vacant(base) => {
                let counter = base.key().insert_counter();
                self.ordered.insert(counter, (key_data, value));
                base.insert(value);
                None
            }
        }
    }

    /// Inserts a key-value pair into the map and update the key in place if an
    /// entry is already present.
    ///
    /// This function maintains the insertion rank of the key-value pair.
    ///
    /// If you do not wish to update the key in-place, use [`StHash::insert`].
    #[inline]
    pub fn update(&mut self, key: st_data_t, value: st_data_t) {
        let key_data = key;
        let key = LookupKey {
            record: key,
            eq: self.eq,
        };

        if let Some((entry_key, _)) = self.map.remove_entry(&key) {
            let insert_counter = entry_key.insert_counter();
            // Maintain insert rank with new key-value pair.
            self.ordered.insert(insert_counter, (key_data, value));
            let key = Key {
                lookup: key,
                insert_counter,
            };
            self.map.insert(key, value);
        } else {
            let _ = self.insert(key_data, value);
        }
    }

    /// Removes a key from the map, returning the stored key if the key was
    /// previously in the map.
    ///
    /// The key may be any `st_data_t` as long as but [`Hash`] and [`Eq`] on the
    /// key match the stored key.
    #[inline]
    #[must_use]
    pub fn remove(&mut self, key: st_data_t) -> Option<st_data_t> {
        let key = LookupKey {
            record: key,
            eq: self.eq,
        };
        let (key, value) = self.map.remove_entry(&key)?;
        self.ordered.remove(&key.insert_counter());
        Some(value)
    }

    /// Removes a key from the map, returning the stored key and value if the
    /// key was previously in the map.
    ///
    /// The key may be any `st_data_t` as long as but [`Hash`] and [`Eq`] on the
    /// key match the stored key.
    #[inline]
    #[must_use]
    pub fn remove_entry(&mut self, key: st_data_t) -> Option<(st_data_t, st_data_t)> {
        let key = LookupKey {
            record: key,
            eq: self.eq,
        };
        let (key, value) = self.map.remove_entry(&key)?;
        self.ordered.remove(&key.insert_counter());
        Some((key.into_record(), value))
    }

    /// Returns a reference to the map's [`StBuildHasher`].
    ///
    /// This can be used to recover the original [`st_hash_type`] used to
    /// construct the `StHash`. See [`StBuildHasher::hash_type`].
    #[inline]
    #[must_use]
    pub fn hasher(&self) -> &StBuildHasher {
        self.map.hasher()
    }

    /// An iterator for visiting all insertion counters in insertion order
    /// starting from the given rank. The iterator element type is `st_index_t`.
    ///
    /// The yielded elements may be passed to [`get_nth`] to retrieve the
    /// (`key`, `value`) pair in the nth insertion slot.
    ///
    /// This API can be used to build a mutable iterator over the map that can
    /// safely be invalidated. This is safe because new inserts always have
    /// higher insert rank. See `api::st_foreach`.
    ///
    /// [`get_nth`]: StHash::get_nth
    #[inline]
    #[must_use]
    pub fn insert_ranks_from(&self, rank: st_index_t) -> InsertRanks {
        let ordered_range = self.ordered.range(rank..);
        let ranks_from = ordered_range.map(|(&rank, _)| rank);
        let ranks = ranks_from.collect::<Vec<_>>();
        InsertRanks(ranks.into_iter())
    }

    /// An iterator for visiting all key-value pairs in insertion order. The
    /// iterator element type is (`&'a st_data_t`, `&'a st_data_t`).
    #[inline]
    #[must_use]
    pub fn iter(&self) -> Iter<'_> {
        Iter(self.ordered.values())
    }

    /// An iterator visiting all keys in insertion order. The iterator element
    /// type is `&'a st_data_t`.
    #[inline]
    #[must_use]
    pub fn keys(&self) -> Keys<'_> {
        Keys(self.iter())
    }

    /// An iterator visiting all values in insertion order. The iterator element
    /// type is `&'a st_data_t`.
    #[inline]
    #[must_use]
    pub fn values(&self) -> Values<'_> {
        Values(self.iter())
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the `StHash`. The collection may reserve more space to avoid frequent
    /// reallocations.
    ///
    /// # Panics
    ///
    /// Panics if the new allocation size overflows `usize`.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.map.reserve(additional);
    }

    /// Shrinks the capacity of the map as much as possible. It will drop down
    /// as much as possible while maintaining the internal rules and possibly
    /// leaving some space in accordance with the resize policy.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.map.shrink_to_fit();
    }

    /// Return an estimate of the byte size of memory allocted for this map.
    #[inline]
    #[must_use]
    pub fn estimated_memsize(&self) -> usize {
        let stack_size = size_of::<Self>();
        let hashmap_size = (size_of::<Key>() + size_of::<st_data_t>()) * self.map.capacity();
        let btreemap_size =
            (size_of::<st_index_t>() + size_of::<(st_data_t, st_data_t)>()) * self.ordered.len();

        stack_size + hashmap_size + btreemap_size
    }
}
