use core::hash::{BuildHasher, Hash};
use core::mem::size_of;
use std::collections::hash_map::RandomState;

use crate::st::map::StHashMap;

mod iter;

pub use iter::{InsertRanks, IntoIter, Iter};

/// An insertion-ordered hash set implemented as an `StHashMap` where the value
/// is `()`.
///
/// As with the [`StHashMap`] type, a `StHashSet` requires that the elements
/// implement the [`Eq`] and [`Hash`] traits.
#[derive(Default, Debug, Clone)]
pub struct StHashSet<T, S = RandomState> {
    map: StHashMap<T, (), S>,
}

impl<T, S> PartialEq for StHashSet<T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        // Only map contents contribute to equality
        self.map == other.map
    }
}

impl<T, S> Eq for StHashSet<T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
}

impl<T> StHashSet<T, RandomState> {
    /// Creates an empty `StHashSet`.
    ///
    /// The hash set is initially created with a capacity of 0, so it will not
    /// allocate until it is first inserted into.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashSet;
    /// let mut set: StHashSet<i32> = StHashSet::new();
    /// assert_eq!(0, set.capacity());
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let map = StHashMap::new();
        Self { map }
    }

    /// Creates an empty `StHashSet` with the specified capacity.
    ///
    /// The hash set will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the hash set will not allocate.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashSet;
    /// let mut set: StHashSet<&str, i32> = StHashSet::with_capacity(10);
    /// assert!(set.capacity() >= 10);
    /// ```
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let map = StHashMap::with_capacity(capacity);
        Self { map }
    }
}

impl<T, S> StHashSet<T, S> {
    /// Creates an empty `StHashSet` which will use the given hash builder to
    /// hash keys.
    ///
    /// The created set has the default initial capacity.
    ///
    /// Warning: `hash_builder` is normally randomly generated, and is designed
    /// to allow `StHashSet`s to be resistant to attacks that cause many
    /// collisions and very poor performance. Setting it manually using this
    /// function can expose a DoS attack vector.
    ///
    /// The `hash_builder` passed should implement the [`BuildHasher`] trait for
    /// the `StHashSet` to be useful, see its documentation for details.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashSet;
    /// use std::collections::hash_map::RandomState;
    ///
    /// let s = RandomState::new();
    /// let mut set = StHashSet::with_hasher(s);
    /// assert_eq!(0, set.capacity());
    /// set.insert(1, 2);
    /// ```
    #[inline]
    #[must_use]
    pub fn with_hasher(hash_builder: S) -> Self {
        let map = StHashMap::with_hasher(hash_builder);
        Self { map }
    }

    /// Creates an empty `StHashSet` with the specified capacity, using the
    /// given hash builder to hash keys.
    ///
    /// The hash set will be able to hold at least `capacity` elements without
    /// reallocating. If `capacity` is 0, the hash set will not allocate.
    ///
    /// Warning: `hash_builder` is normally randomly generated, and is designed
    /// to allow `StHashSet`s to be resistant to attacks that cause many
    /// collisions and very poor performance. Setting it manually using this
    /// function can expose a DoS attack vector.
    ///
    /// The `hash_builder` passed should implement the [`BuildHasher`] trait for
    /// the `StHashSet` to be useful, see its documentation for details.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashSet;
    /// use std::collections::hash_map::RandomState;
    ///
    /// let s = RandomState::new();
    /// let mut set = StHashSet::with_capacity_and_hasher(10, s);
    /// assert!(set.capacity() >= 10);
    /// set.insert(1, 2);
    /// ```
    #[inline]
    #[must_use]
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        let map = StHashMap::with_capacity_and_hasher(capacity, hash_builder);
        Self { map }
    }

    /// Returns the number of elements the set can hold without reallocating.
    ///
    /// This number is a lower bound; the `StHashSet` might be able to hold
    /// more, but is guaranteed to be able to hold at least this many.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashSet;
    /// let mut set: StHashSet<i32> = StHashSet::with_capacity(100);
    /// assert!(set.capacity() >= 100);
    /// ```
    #[inline]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }

    /// An iterator for visiting all elements in insertion order. The iterator
    /// element type is `&'a T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashSet;
    ///
    /// let mut set = StHashSet::new();
    /// set.insert("a");
    /// set.insert("b");
    /// set.insert("c");
    ///
    /// for elem in set.iter() {
    ///     println!("element: {}", elem);
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter(self.map.keys())
    }

    /// An iterator for visiting all insertion counters in insertion order
    /// starting from the given rank. The iterator element type is `usize`.
    ///
    /// The yielded elements may be passed to [`get_nth`](StHashSet::get_nth) to
    /// retrieve the `element` in the nth insertion slot.
    ///
    /// This API can be used to build a mutable iterator over the set that can
    /// safely be invalidated. This is safe because new inserts always have
    /// higher insert rank. See `api::st_foreach`.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashSet;
    ///
    /// let mut set = StHashSet::new();
    /// set.insert("a");
    /// set.insert("b");
    /// set.insert("c");
    ///
    /// set.remove(&"a");
    /// set.insert("b");
    ///
    /// let insert_ranks = set.insert_ranks_from(0).collect::<Vec<_>>();
    /// assert_eq!(vec![1, 2], insert_ranks);
    ///
    /// assert_eq!(None, set.get_nth(0));
    /// assert_eq!(Some(&"b"), set.get_nth(1));
    /// assert_eq!(Some(&"c"), set.get_nth(2));
    /// assert_eq!(None, set.get_nth(4));
    ///
    /// assert_eq!(0, set.insert_ranks_from(100).count());
    /// ```
    #[inline]
    #[must_use]
    pub fn insert_ranks_from(&self, rank: usize) -> InsertRanks {
        InsertRanks(self.map.insert_ranks_from(rank))
    }

    /// Returns the first element in the set. The element is equal to the
    /// element inserted earliest into the set.
    ///
    /// Elements are ordered by insertion order. Insertion order is maintained
    /// if there are deletions. Insertion order is by slot, so
    /// [in-place updates to elements](StHashSet::update) maintain the same
    /// insertion position.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashSet;
    ///
    /// let mut set = StHashSet::new();
    /// set.insert("a");
    /// set.insert("b");
    /// set.insert("c");
    /// assert_eq!(Some(&"a"), set.first());
    ///
    /// set.remove(&"a");
    /// set.insert("b");
    /// assert_eq!(Some(&"b"), set.first());
    /// ```
    #[inline]
    #[must_use]
    pub fn first(&self) -> Option<&T> {
        self.iter().next()
    }

    /// Returns the last element in the set. The element is equal to the element
    /// inserted most recently into the set.
    ///
    /// Elements are ordered by insertion order. Insertion order is maintained
    /// if there are deletions. Insertion order is by slot, so
    /// [in-place updates to elements](StHashSet::update) maintain the same
    /// insertion position.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashSet;
    ///
    /// let mut set = StHashSet::new();
    /// set.insert("a");
    /// set.insert("b");
    /// set.insert("c");
    /// assert_eq!(Some(&"c"), set.last());
    ///
    /// set.remove(&"a");
    /// set.insert("b");
    /// assert_eq!(Some(&"c"), set.last());
    /// ```
    #[inline]
    #[must_use]
    pub fn last(&self) -> Option<&T> {
        self.iter().last()
    }

    /// Returns the nth element in the set. The element is equal to the element
    /// inserted nth earliest into the set.
    ///
    /// Elements are ordered by insertion order. Insertion order is maintained
    /// if there are deletions. Insertion order is by slot, so
    /// [in-place updates to elements](StHashSet::update) maintain the same
    /// insertion position.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashSet;
    ///
    /// let mut set = StHashSet::new();
    /// set.insert("a");
    /// set.insert("b");
    /// set.insert("c");
    ///
    /// set.remove(&"a");
    /// set.insert("b");
    ///
    /// let insert_ranks = set.insert_ranks_from(0).collect::<Vec<_>>();
    /// assert_eq!(vec![1, 2], insert_ranks);
    ///
    /// assert_eq!(None, set.get_nth(0));
    /// assert_eq!(Some(&"b"), set.get_nth(1));
    /// assert_eq!(Some(&"c"), set.get_nth(2));
    /// assert_eq!(None, set.get_nth(4));
    ///
    /// assert_eq!(0, set.insert_ranks_from(100).count());
    /// ```
    ///
    /// [in-place updates to keys]: StHash::update
    #[inline]
    #[must_use]
    pub fn get_nth(&self, n: usize) -> Option<&T> {
        self.map.get_nth(n).map(|(elem, _)| elem)
    }

    /// Insertion counter for the [first](StHashSet::first) element in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashSet;
    ///
    /// let mut set = StHashSet::new();
    /// assert_eq!(0, set.min_insert_rank());
    ///
    /// set.insert("a");
    /// set.insert("b");
    /// set.insert("c");
    /// assert_eq!(0, set.min_insert_rank());
    ///
    /// set.remove(&"a");
    /// set.insert("b");
    /// assert_eq!(1, set.min_insert_rank());
    /// ```
    #[inline]
    #[must_use]
    pub fn min_insert_rank(&self) -> usize {
        self.map.min_insert_rank()
    }

    /// Insertion counter for the [last](StHashSet::last) element in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashSet;
    ///
    /// let mut set = StHashSet::new();
    /// assert_eq!(0, set.max_insert_rank());
    ///
    /// set.insert("a");
    /// set.insert("b");
    /// set.insert("c");
    /// assert_eq!(2, set.max_insert_rank());
    ///
    /// set.remove(&"a");
    /// set.insert("b");
    /// assert_eq!(2, set.max_insert_rank());
    /// ```
    #[inline]
    #[must_use]
    pub fn max_insert_rank(&self) -> usize {
        self.map.max_insert_rank()
    }

    /// Returns the number of elements in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashSet;
    ///
    /// let mut set = StHashSet::new();
    /// assert_eq!(0, set.len());
    /// set.insert(1);
    /// assert_eq!(1, set.len());
    /// ```
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the set contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashSet;
    ///
    /// let mut set = StHashSet::new();
    /// assert!(set.is_empty());
    /// set.insert(1);
    /// assert!(!set.is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Clears the set, removing all elements. Keeps the allocated memory for
    /// reuse.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashSet;
    ///
    /// let mut set = StHashSet::new();
    /// set.insert(1);
    /// set.clear();
    /// assert!(set.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Returns a reference to the set's [`BuildHasher`].
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashSet;
    /// use std::collections::hash_map::RandomState;
    ///
    /// let hasher = RandomState::new();
    /// let set: StHashSet<i32> = StHashSet::with_hasher(hasher);
    /// let hasher: &RandomState = set.hasher();
    /// ```
    #[inline]
    #[must_use]
    pub fn hasher(&self) -> &S {
        self.map.hasher()
    }

    /// Return an estimate of the byte size of memory allocted for this set.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashSet;
    /// let empty: StHashSet<i32> = StHashSet::with_capacity(0);
    /// let set: StHashMap<i32> = StHashSet::with_capacity(100);
    /// assert!(set.estimated_memsize() > empty.estimated_memsize());
    /// ```
    #[inline]
    #[must_use]
    pub fn estimated_memsize(&self) -> usize {
        let additional_stack_size = size_of::<Self>() - size_of::<StHashMap<T, (), S>>();
        self.map.estimated_memsize() + additional_stack_size
    }
}

impl<T, S> StHashSet<T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the `StHashSet`. The collection may reserve more space to avoid
    /// frequent reallocations.
    ///
    /// # Panics
    ///
    /// Panics if the new allocation size overflows `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashSet;
    /// let mut set: StHashSet<&str> = StHashSet::new();
    /// assert_eq!(0, set.capacity());
    /// set.reserve(10);
    /// assert!(set.capacity() >= 10);
    /// ```
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.map.reserve(additional);
    }

    /// Shrinks the capacity of the set as much as possible. It will drop down
    /// as much as possible while maintaining the internal rules and possibly
    /// leaving some space in accordance with the resize policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashSet;
    /// let mut set: StHashSet<i32> = StHashSet::with_capacity(100);
    /// set.insert(1);
    /// set.insert(3);
    /// assert!(set.capacity() >= 100);
    /// set.shrink_to_fit();
    /// assert!(set.capacity() >= 2);
    /// ```
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.map.shrink_to_fit();
    }

    /// Returns `true` if the set contains the specified element.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashSet;
    ///
    /// let mut set = StHashSet::new();
    /// set.insert(1);
    /// assert_eq!(set.contains(&1), true);
    /// assert_eq!(set.contains(&2), false);
    /// ```
    #[inline]
    #[must_use]
    pub fn contains(&self, element: &T) -> bool {
        self.map.contains_key(element)
    }

    /// Returns a reference to the element in the set corresponding to the given
    /// value.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashSet;
    ///
    /// let mut set = StHashSet::new();
    /// set.insert(1);
    /// assert_eq!(set.get(&1), Some(&1));
    /// assert_eq!(set.get(&2), None);
    /// ```
    #[inline]
    #[must_use]
    pub fn get(&self, element: &T) -> Option<&T> {
        let (element, _) = self.map.get_key_value(element)?;
        Some(element)
    }
}

impl<T, S> StHashSet<T, S>
where
    T: Eq + Hash + Clone,
    S: BuildHasher,
{
    /// Adds an element into the set.
    ///
    /// If the set did not have this element present, `false` is returned.
    ///
    /// If the set did have this element present, `true` is returned. The
    /// element is not updated, though. To update the element in-place, use
    /// [`StHashSet::update`].
    #[inline]
    #[must_use]
    pub fn insert(&mut self, element: T) -> bool {
        self.map.insert(element, ()).is_some()
    }

    /// Inserts an element into the set and update the element in place if an
    /// entry is already present.
    ///
    /// This function maintains the insertion rank of the element.
    ///
    /// If you do not wish to update the element in-place, use [`StHashSet::insert`].
    #[inline]
    pub fn update(&mut self, element: T) {
        self.map.update(element, ());
    }

    /// Removes an element from the set, returning the stored element if the
    /// element was previously in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use strudel::StHashSet;
    ///
    /// let mut set = StHashSet::new();
    /// set.insert(1, "a");
    /// assert_eq!(set.remove(&1), Some("a"));
    /// assert_eq!(set.remove(&1), None);
    /// ```
    #[inline]
    #[must_use]
    pub fn remove(&mut self, element: &T) -> bool {
        self.map.remove(element).is_some()
    }
}
