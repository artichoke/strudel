//! An insertion-ordered hash map implemented with [`HashMap`] and [`BTreeMap`].

use core::borrow::Borrow;
use core::hash::{Hash, Hasher};
use core::mem::size_of;
use std::collections::hash_map::Entry as HashEntry;
use std::collections::{BTreeMap, HashMap};

pub use crate::entry::{Entry, OccupiedEntry, VacantEntry};
pub use crate::hasher::{StBuildHasher, StHasher};
pub use crate::iter::{InsertRanks, Iter, IterMut, Keys, Values};

use crate::typedefs::*;

#[derive(Debug, Clone)]
pub struct Key {
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
pub struct LookupKey {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StHash {
    map: HashMap<Key, st_data_t, StBuildHasher>,
    ordered: BTreeMap<st_index_t, (st_data_t, st_data_t)>,
    eq: unsafe extern "C" fn(st_data_t, st_data_t) -> i32,
    insert_counter: st_index_t,
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
        self.ordered.remove(&key.insert_counter());
        Some((key.into_record(), value))
    }

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
    #[must_use]
    pub fn estimated_memsize(&self) -> usize {
        size_of::<Self>()
            + (size_of::<Key>() + size_of::<st_data_t>()) * self.map.capacity()
            + (size_of::<st_index_t>() + size_of::<(st_data_t, st_data_t)>()) * self.ordered.len()
    }
}
