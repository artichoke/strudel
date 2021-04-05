use core::borrow::Borrow;
use core::hash::{BuildHasher, Hash, Hasher};
use core::mem::size_of;
use core::ops::Index;
use std::collections::hash_map::{Entry as HashEntry, RandomState};
use std::collections::HashMap;

mod iter;

pub use iter::{InsertRanks, IntoIter, Iter, Keys, Values};

// Tables with `LINEAR_SCAN_THRESHOLD` or fewer insertions will perform lookups
// by scanning the `ordered` insertion list.
const LINEAR_SCAN_THRESHOLD: usize = 8;

#[derive(Debug, Clone)]
pub(crate) struct Key<T> {
    inner: T,
    insert_rank: usize,
}

impl<T> Key<T> {
    #[inline]
    #[must_use]
    pub fn insert_rank(&self) -> usize {
        self.insert_rank
    }

    #[inline]
    #[must_use]
    pub fn inner(&self) -> &T {
        &self.inner
    }

    #[inline]
    #[must_use]
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> PartialEq for Key<T>
where
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner() == other.inner()
    }
}

impl<T> Eq for Key<T> where T: Eq {}

impl<T> Hash for Key<T>
where
    T: Hash,
{
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner().hash(state);
    }
}

impl<T> Borrow<T> for Key<T> {
    #[inline]
    fn borrow(&self) -> &T {
        self.inner()
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Value<T> {
    inner: T,
}

impl<T> From<T> for Value<T> {
    fn from(value: T) -> Self {
        Self { inner: value }
    }
}

impl<T> Value<T> {
    #[inline]
    #[must_use]
    pub fn inner(&self) -> &T {
        &self.inner
    }

    #[inline]
    #[must_use]
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> Borrow<T> for Value<T> {
    #[inline]
    fn borrow(&self) -> &T {
        self.inner()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum InsertionEntry<K, V> {
    Alive(K, V),
    Dead,
}

/// An insertion-ordered hash map implemented with [`HashMap`] and [`Vec`].
///
/// `StHashMap` is designed to implement the `st_hash` C API and be
/// FFI-friendly.
///
/// `StHashMap` is built on top of a hashing algorithm selected to provide
/// resistance against HashDoS attacks. See [`RandomState`].
///
/// `StHashMap` supports updating keys in place. See [`StHashMap::update`].
///
/// The optional `api` and `capi` modules in `strudel` build on top of
/// `StHashMap` to implement a compatible C API to `st_hash`. This API includes
/// support for iterating over a mutable map and inplace updates of
/// `(key, value)` pairs. These features distinguish it from the [`HashMap`] in
/// Rust `std`.
///
/// [`RandomState`]: std::collections::hash_map::RandomState
#[derive(Default, Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct StHashMap<K, V, S = RandomState> {
    map: HashMap<Key<K>, Value<V>, S>,
    ordered: Vec<InsertionEntry<K, V>>,
}

impl<K, V, S> PartialEq for StHashMap<K, V, S>
where
    K: Eq + Hash,
    V: PartialEq,
    S: BuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        // Only map contents contribute to equality
        self.map == other.map
    }
}

impl<K, V, S> Eq for StHashMap<K, V, S>
where
    K: Eq + Hash,
    V: PartialEq,
    S: BuildHasher,
{
}

impl<K, V, S> Index<&K> for StHashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    type Output = V;

    /// Returns a reference to the value corresponding to the supplied key.
    ///
    /// # Panics
    ///
    /// Panics if the key is not present in the `HashMap`.
    #[inline]
    fn index(&self, key: &K) -> &V {
        self.get(key).expect("no entry found for key")
    }
}

impl<K, V> StHashMap<K, V, RandomState> {
    /// Creates an empty `StHashMap`.
    ///
    /// The hash map is initially created with a capacity of 0, so it will not
    /// allocate until it is first inserted into.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    /// let mut map: StHashMap<&str, i32> = StHashMap::new();
    /// assert_eq!(0, map.capacity());
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let map = HashMap::new();
        let ordered = Vec::new();
        Self { map, ordered }
    }

    /// Creates an empty `StHashMap` with the specified capacity.
    ///
    /// The hash map will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the hash map will not allocate.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    /// let mut map: StHashMap<&str, i32> = StHashMap::with_capacity(10);
    /// assert!(map.capacity() >= 10);
    /// ```
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let map = HashMap::with_capacity(capacity);
        let ordered = Vec::with_capacity(capacity);
        Self { map, ordered }
    }
}

impl<K, V, S> StHashMap<K, V, S> {
    /// Creates an empty `StHashMap` which will use the given hash builder to
    /// hash keys.
    ///
    /// The created map has the default initial capacity.
    ///
    /// Warning: `hash_builder` is normally randomly generated, and is designed
    /// to allow `StHashMap`s to be resistant to attacks that cause many
    /// collisions and very poor performance. Setting it manually using this
    /// function can expose a DoS attack vector.
    ///
    /// The `hash_builder` passed should implement the [`BuildHasher`] trait for
    /// the `StHashMap` to be useful, see its documentation for details.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    /// use std::collections::hash_map::RandomState;
    ///
    /// let s = RandomState::new();
    /// let mut map = StHashMap::with_hasher(s);
    /// assert_eq!(0, map.capacity());
    /// map.insert(1, 2);
    /// ```
    #[inline]
    #[must_use]
    pub fn with_hasher(hash_builder: S) -> Self {
        let map = HashMap::with_hasher(hash_builder);
        let ordered = Vec::new();
        Self { map, ordered }
    }

    /// Creates an empty `StHashMap` with the specified capacity, using the
    /// given hash builder to hash keys.
    ///
    /// The hash map will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the hash map will not allocate.
    ///
    /// Warning: `hash_builder` is normally randomly generated, and is designed
    /// to allow `StHashMap`s to be resistant to attacks that cause many
    /// collisions and very poor performance. Setting it manually using this
    /// function can expose a DoS attack vector.
    ///
    /// The `hash_builder` passed should implement the [`BuildHasher`] trait for
    /// the `StHashMap` to be useful, see its documentation for details.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    /// use std::collections::hash_map::RandomState;
    ///
    /// let s = RandomState::new();
    /// let mut map = StHashMap::with_capacity_and_hasher(10, s);
    /// assert!(map.capacity() >= 10);
    /// map.insert(1, 2);
    /// ```
    #[inline]
    #[must_use]
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        let map = HashMap::with_capacity_and_hasher(capacity, hash_builder);
        let ordered = Vec::with_capacity(capacity);
        Self { map, ordered }
    }

    /// Returns the number of elements the map can hold without reallocating.
    ///
    /// This number is a lower bound; the `StHashMap` might be able to hold
    /// more, but is guaranteed to be able to hold at least this many.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    /// let mut map: StHashMap<&str, i32> = StHashMap::with_capacity(100);
    /// assert!(map.capacity() >= 100);
    /// ```
    #[inline]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.map.capacity().min(self.ordered.capacity())
    }

    /// An iterator visiting all keys in insertion order. The iterator element
    /// type is `&'a K`.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    ///
    /// let mut map = StHashMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// for key in map.keys() {
    ///     println!("key: {}", key);
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys(self.iter())
    }

    /// An iterator visiting all values in insertion order. The iterator element
    /// type is `&'a V`.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    ///
    /// let mut map = StHashMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// for val in map.values() {
    ///     println!("val: {}", val);
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn values(&self) -> Values<'_, K, V> {
        Values(self.iter())
    }

    /// An iterator for visiting all key-value pairs in insertion order. The
    /// iterator element type is `(&'a K, &'a V)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    ///
    /// let mut map = StHashMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// for (key, val) in map.iter() {
    ///     println!("key: {} val: {}", key, val);
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter(self.ordered.iter())
    }

    /// An iterator for visiting all insertion counters in insertion order
    /// starting from the given rank. The iterator element type is `usize`.
    ///
    /// This iterator may return insertion counters to dead elements.
    ///
    /// The yielded elements may be passed to [`get_nth`] to retrieve the
    /// `(key, value)` pair in the nth insertion slot.
    ///
    /// This API can be used to build a mutable iterator over the map that can
    /// safely be invalidated. This is safe because new inserts always have
    /// higher insert rank. See `api::st_foreach`.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    ///
    /// let mut map = StHashMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// map.remove(&"a");
    /// map.insert("b", 100);
    ///
    /// let insert_ranks = map.insert_ranks_from(0).collect::<Vec<_>>();
    /// assert_eq!(vec![0, 1, 2], insert_ranks);
    ///
    /// assert_eq!(None, map.get_nth(0));
    /// assert_eq!(Some((&"b", &100)), map.get_nth(1));
    /// assert_eq!(Some((&"c", &3)), map.get_nth(2));
    /// assert_eq!(None, map.get_nth(4));
    ///
    /// assert_eq!(0, map.insert_ranks_from(100).count());
    /// ```
    ///
    /// [`get_nth`]: StHashMap::get_nth
    #[inline]
    #[must_use]
    pub fn insert_ranks_from(&self, rank: usize) -> InsertRanks {
        InsertRanks(rank..self.ordered.len())
    }

    /// Returns the first key-value pair in the map. The key in this pair is
    /// equal to the key inserted earliest into the map.
    ///
    /// Key-value pairs are ordered by insertion order. Insertion order is
    /// maintained if there are deletions. Insertion order is by slot, so
    /// [in-place updates to keys] maintain the same insertion position.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    ///
    /// let mut map = StHashMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    /// assert_eq!(Some((&"a", &1)), map.first());
    ///
    /// map.remove(&"a");
    /// map.insert("b", 100);
    /// assert_eq!(Some((&"b", &100)), map.first());
    /// ```
    ///
    /// [in-place updates to keys]: StHashMap::update
    #[inline]
    #[must_use]
    pub fn first(&self) -> Option<(&K, &V)> {
        self.iter().next()
    }

    /// Returns the last key-value pair in the map. The key in this pair is
    /// equal to the key inserted most recently into the map.
    ///
    /// Key-value pairs are ordered by insertion order. Insertion order is
    /// maintained if there are deletions. Insertion order is by slot, so
    /// [in-place updates to keys] maintain the same insertion position.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    ///
    /// let mut map = StHashMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    /// assert_eq!(Some((&"c", &3)), map.last());
    ///
    /// map.remove(&"a");
    /// map.insert("b", 100);
    /// assert_eq!(Some((&"c", &3)), map.last());
    /// ```
    ///
    /// [in-place updates to keys]: StHashMap::update
    #[inline]
    #[must_use]
    pub fn last(&self) -> Option<(&K, &V)> {
        self.iter().last()
    }

    /// Returns the nth key-value pair in the map. The key in this pair is
    /// equal to the key inserted nth earliest into the map.
    ///
    /// Key-value pairs are ordered by insertion order. Insertion order is
    /// maintained if there are deletions. Insertion order is by slot, so
    /// [in-place updates to keys] maintain the same insertion position.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    ///
    /// let mut map = StHashMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// map.remove(&"a");
    /// map.insert("b", 100);
    ///
    /// let insert_ranks = map.insert_ranks_from(0).collect::<Vec<_>>();
    /// assert_eq!(vec![0, 1, 2], insert_ranks);
    ///
    /// assert_eq!(None, map.get_nth(0));
    /// assert_eq!(Some((&"b", &100)), map.get_nth(1));
    /// assert_eq!(Some((&"c", &3)), map.get_nth(2));
    /// assert_eq!(None, map.get_nth(4));
    ///
    /// assert_eq!(0, map.insert_ranks_from(100).count());
    /// ```
    ///
    /// [in-place updates to keys]: StHashMap::update
    #[inline]
    #[must_use]
    pub fn get_nth(&self, n: usize) -> Option<(&K, &V)> {
        match self.ordered.get(n) {
            None | Some(InsertionEntry::Dead) => None,
            Some(InsertionEntry::Alive(key, value)) => Some((key, value)),
        }
    }

    /// Insertion counter for the [first](StHashMap::first) key-value pair in
    /// the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    ///
    /// let mut map = StHashMap::new();
    /// assert_eq!(0, map.min_insert_rank());
    ///
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    /// assert_eq!(0, map.min_insert_rank());
    ///
    /// map.remove(&"a");
    /// map.insert("b", 100);
    /// assert_eq!(1, map.min_insert_rank());
    /// ```
    #[inline]
    #[must_use]
    pub fn min_insert_rank(&self) -> usize {
        for (idx, pair) in self.ordered.iter().enumerate() {
            if let InsertionEntry::Alive(_, _) = pair {
                return idx;
            }
        }
        0
    }

    /// Insertion counter for the [last](StHashMap::last) key-value pair in the
    /// map.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    ///
    /// let mut map = StHashMap::new();
    /// assert_eq!(0, map.max_insert_rank());
    ///
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    /// assert_eq!(2, map.max_insert_rank());
    ///
    /// map.remove(&"a");
    /// map.insert("b", 100);
    /// assert_eq!(2, map.max_insert_rank());
    /// ```
    #[inline]
    #[must_use]
    pub fn max_insert_rank(&self) -> usize {
        for (idx, pair) in self.ordered.iter().enumerate().rev() {
            if let InsertionEntry::Alive(_, _) = pair {
                return idx;
            }
        }
        0
    }

    /// Returns the number of elements in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    ///
    /// let mut map = StHashMap::new();
    /// assert_eq!(0, map.len());
    /// map.insert(1, "a");
    /// assert_eq!(1, map.len());
    /// ```
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the map contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    ///
    /// let mut map = StHashMap::new();
    /// assert!(map.is_empty());
    /// map.insert(1, "a");
    /// assert!(!map.is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Clears the map, removing all key-value pairs. Keeps the allocated memory
    /// for reuse.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    ///
    /// let mut map = StHashMap::new();
    /// map.insert(1, "a");
    /// map.clear();
    /// assert!(map.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.map.clear();
        self.ordered.clear();
    }

    /// Returns a reference to the map's [`BuildHasher`].
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    /// use std::collections::hash_map::RandomState;
    ///
    /// let hasher = RandomState::new();
    /// let map: StHashMap<i32, i32> = StHashMap::with_hasher(hasher);
    /// let hasher: &RandomState = map.hasher();
    /// ```
    #[inline]
    #[must_use]
    pub fn hasher(&self) -> &S {
        self.map.hasher()
    }

    /// Return an estimate of the byte size of memory allocted for this map.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    /// let empty: StHashMap<i32, i32> = StHashMap::with_capacity(0);
    /// let map: StHashMap<i32, i32> = StHashMap::with_capacity(100);
    /// assert!(map.estimated_memsize() > empty.estimated_memsize());
    /// ```
    #[inline]
    #[must_use]
    pub fn estimated_memsize(&self) -> usize {
        let stack_size = size_of::<Self>();
        let hashmap_size = (size_of::<Key<K>>() + size_of::<V>()) * self.map.capacity();
        let vec_size = size_of::<InsertionEntry<K, V>>() * self.ordered.len();

        stack_size + hashmap_size + vec_size
    }
}

impl<K, V, S> StHashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the `StHashMap`. The collection may reserve more space to avoid
    /// frequent reallocations.
    ///
    /// # Panics
    ///
    /// Panics if the new allocation size overflows `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    /// let mut map: StHashMap<&str, i32> = StHashMap::new();
    /// assert_eq!(0, map.capacity());
    /// map.reserve(10);
    /// assert!(map.capacity() >= 10);
    /// ```
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.map.reserve(additional);
        self.ordered.reserve(additional);
    }

    /// Shrinks the capacity of the map as much as possible. It will drop down
    /// as much as possible while maintaining the internal rules and possibly
    /// leaving some space in accordance with the resize policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    /// let mut map: StHashMap<i32, i32> = StHashMap::with_capacity(100);
    /// map.insert(1, 2);
    /// map.insert(3, 4);
    /// assert!(map.capacity() >= 100);
    /// map.shrink_to_fit();
    /// assert!(map.capacity() >= 2);
    /// ```
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.map.shrink_to_fit();
        self.ordered.shrink_to_fit();
    }

    /// Returns `true` if the map contains a value for the specified key.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    ///
    /// let mut map = StHashMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.contains_key(&1), true);
    /// assert_eq!(map.contains_key(&2), false);
    /// ```
    #[inline]
    #[must_use]
    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    ///
    /// let mut map = StHashMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.get(&1), Some(&"a"));
    /// assert_eq!(map.get(&2), None);
    /// ```
    #[inline]
    #[must_use]
    pub fn get(&self, key: &K) -> Option<&V> {
        if self.ordered.len() < LINEAR_SCAN_THRESHOLD {
            self.ordered.iter().find_map(|entry| match entry {
                InsertionEntry::Alive(entry_key, entry_value) if entry_key == key => {
                    Some(entry_value)
                }
                _ => None,
            })
        } else {
            self.map.get(key).map(Value::inner)
        }
    }

    /// Returns the key-value pair corresponding to the supplied key.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    ///
    /// let mut map = StHashMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.get_key_value(&1), Some((&1, &"a")));
    /// assert_eq!(map.get_key_value(&2), None);
    /// ```
    #[inline]
    #[must_use]
    pub fn get_key_value(&self, key: &K) -> Option<(&K, &V)> {
        let (key, value) = self.map.get_key_value(key)?;
        Some((key.inner(), value.inner()))
    }
}

impl<K, V, S> StHashMap<K, V, S>
where
    K: Eq + Hash + Clone,
    V: PartialEq + Clone,
    S: BuildHasher,
{
    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, [`None`] is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though. To update the key
    /// in-place, use [`StHashMap::update`].
    #[inline]
    #[must_use]
    #[allow(clippy::clippy::missing_panics_doc)]
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let insert_rank = self.ordered.len();

        let key = Key {
            inner: key,
            insert_rank,
        };

        match self.map.entry(key) {
            HashEntry::Occupied(mut base) => {
                let insert_rank = base.key().insert_rank();
                // Maintain insert rank with new key-value pair.
                if let Some(InsertionEntry::Alive(_, stored_value)) =
                    self.ordered.get_mut(insert_rank)
                {
                    if *stored_value != value {
                        *stored_value = value.clone();
                    }
                } else {
                    panic!("already inserted pair not alive in ordered storage");
                }
                Some(base.insert(value.into()).into_inner())
            }
            HashEntry::Vacant(base) => {
                self.ordered.push(InsertionEntry::Alive(
                    base.key().inner().clone(),
                    value.clone(),
                ));
                base.insert(value.into());
                None
            }
        }
    }

    /// Inserts a key-value pair into the map and update the key in place if an
    /// entry is already present.
    ///
    /// This function maintains the insertion rank of the key-value pair.
    ///
    /// If you do not wish to update the key in-place, use
    /// [`StHashMap::insert`].
    #[allow(clippy::clippy::missing_panics_doc)]
    #[inline]
    pub fn update(&mut self, key: K, value: V) {
        if let Some((entry_key, _)) = self.map.remove_entry(&key) {
            let insert_rank = entry_key.insert_rank();
            // Maintain insert rank with new key-value pair.
            if let Some(InsertionEntry::Alive(stored_key, stored_value)) =
                self.ordered.get_mut(insert_rank)
            {
                if *stored_key != key {
                    *stored_key = key.clone();
                }
                if *stored_value != value {
                    *stored_value = value.clone();
                }
            } else {
                panic!("already inserted pair not alive in ordered storage");
            }
            let key = Key {
                inner: key,
                insert_rank,
            };
            self.map.insert(key, value.into());
        } else {
            let _ = self.insert(key, value);
        }
    }

    /// Removes a key from the map, returning the stored key if the key was
    /// previously in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    ///
    /// let mut map = StHashMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.remove(&1), Some("a"));
    /// assert_eq!(map.remove(&1), None);
    /// ```
    #[inline]
    #[must_use]
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.remove_entry(key).map(|(_, value)| value)
    }

    /// Removes a key from the map, returning the stored key and value if the
    /// key was previously in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashMap;
    ///
    /// let mut map = StHashMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.remove_entry(&1), Some((1, "a")));
    /// assert_eq!(map.remove(&1), None);
    /// ```
    #[inline]
    #[must_use]
    pub fn remove_entry(&mut self, key: &K) -> Option<(K, V)> {
        let (key, value) = self.map.remove_entry(key)?;
        self.ordered[key.insert_rank()] = InsertionEntry::Dead;
        Some((key.into_inner(), value.into_inner()))
    }
}
