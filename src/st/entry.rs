//! `StHashMap` [Entry API](crate::StHashMap::entry).
//!
//! The entry API allows for more complex methods of getting, setting, updating
//! and removing keys and their values.

use std::collections::hash_map::{
    OccupiedEntry as OccupiedHashEntry, VacantEntry as VacantHashEntry,
};

use crate::st::Key;

/// A view into a single entry in a map, which may either be vacant or occupied.
///
/// This enum is constructed from the [`entry`] method on [`StHash`].
///
/// [`entry`]: crate::StHash::entry
/// [`StHash`]: crate::StHash
#[derive(Debug)]
pub enum Entry<'a, K, V> {
    /// An occupied entry.
    Occupied(OccupiedEntry<'a, K, V>),

    /// A vacant entry.
    Vacant(VacantEntry<'a, K, V>),
}

/// A view into an occupied entry in a `StHash`. It is part of the [`Entry`]
/// enum.
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct OccupiedEntry<'a, K, V>(pub(crate) OccupiedHashEntry<'a, Key<K>, V>);

/// A view into a vacant entry in a `StHash`. It is part of the [`Entry`] enum.
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct VacantEntry<'a, K, V>(pub(crate) VacantHashEntry<'a, Key<K>, V>);

impl<'a, K, V> Entry<'a, K, V> {
    /// Ensures a value is in the entry by inserting the default if empty, and
    /// returns a mutable reference to the value in the entry.
    #[inline]
    #[must_use]
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Self::Occupied(entry) => entry.0.into_mut(),
            Self::Vacant(entry) => entry.0.insert(default),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default
    /// function if empty, and returns a mutable reference to the value in the
    /// entry.
    #[inline]
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Self::Occupied(entry) => entry.0.into_mut(),
            Self::Vacant(entry) => entry.0.insert(default()),
        }
    }

    /// Ensures a value is in the entry by inserting, if empty, the result of
    /// the default function, which takes the key as its argument, and returns a
    /// mutable reference to the value in the entry.
    #[inline]
    pub fn or_insert_with_key<F: FnOnce(&K) -> V>(self, default: F) -> &'a mut V {
        match self {
            Self::Occupied(entry) => entry.0.into_mut(),
            Self::Vacant(entry) => {
                let value = default(entry.0.key().inner());
                entry.insert(value)
            }
        }
    }

    /// Returns a reference to this entry's key.
    #[inline]
    #[must_use]
    pub fn key(&self) -> &K {
        match self {
            Self::Occupied(entry) => entry.0.key().inner(),
            Self::Vacant(entry) => entry.0.key().inner(),
        }
    }

    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the map.
    #[inline]
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
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

impl<'a, K, V> OccupiedEntry<'a, K, V> {
    /// Gets a reference to the key in the entry.
    #[inline]
    #[must_use]
    pub fn key(&self) -> &K {
        self.0.key().inner()
    }

    /// Take the ownership of the key and value from the map.
    #[inline]
    #[must_use]
    pub fn remove_entry(self) -> (K, V) {
        let (key, value) = self.0.remove_entry();
        (key.into_inner(), value)
    }

    /// Gets a reference to the value in the entry.
    #[inline]
    #[must_use]
    pub fn get(&self) -> &V {
        self.0.get()
    }

    /// Gets a mutable reference to the value in the entry.
    ///
    /// If you need a reference to the `OccupiedEntry` which may outlive the
    /// destruction of the `Entry` value, see [`into_mut`].
    ///
    /// [`into_mut`]: #method.into_mut
    #[inline]
    pub fn get_mut(&mut self) -> &mut V {
        self.0.get_mut()
    }

    /// Converts the `OccupiedEntry` into a mutable reference to the value in
    /// the entry with a lifetime bound to the map itself.
    ///
    /// If you need multiple references to the `OccupiedEntry`, see [`get_mut`].
    ///
    /// [`get_mut`]: #method.get_mut
    #[inline]
    #[must_use]
    pub fn into_mut(self) -> &'a mut V {
        self.0.into_mut()
    }

    /// Sets the value of the entry, and returns the entry's old value.
    #[inline]
    pub fn insert(&mut self, value: V) -> V {
        self.0.insert(value)
    }

    /// Takes the value out of the entry, and returns it.
    #[inline]
    #[must_use]
    pub fn remove(self) -> V {
        self.0.remove()
    }
}

impl<'a, K, V> VacantEntry<'a, K, V> {
    /// Gets a reference to the key that would be used when inserting a value
    /// through the `VacantEntry`.
    #[inline]
    #[must_use]
    pub fn key(&self) -> &K {
        self.0.key().inner()
    }

    /// Take ownership of the key.
    #[inline]
    #[must_use]
    pub fn into_key(self) -> K {
        self.0.into_key().into_inner()
    }

    /// Sets the value of the entry with the `VacantEntry`'s key, and returns a
    /// mutable reference to it.
    #[inline]
    #[must_use]
    pub fn insert(self, value: V) -> &'a mut V {
        self.0.insert(value)
    }
}
