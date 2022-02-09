use core::hash::{Hash, Hasher};

use strudel::StHashMap;

use crate::bindings::{st_compare_func, st_hash_type};
use crate::hasher::StBuildHasher;
use crate::primitives::st_data_t;

pub mod ffi;
pub mod foreign;

/// A wrapper around a raw `st_data_t` key that includes a vtable for equality
/// comparisons.
#[derive(Debug, Clone)]
pub struct Key {
    record: st_data_t,
    eq: st_compare_func,
}

impl Key {
    /// Return a reference to the inner key record.
    #[inline]
    #[must_use]
    pub fn inner(&self) -> &st_data_t {
        &self.record
    }
}

impl From<Key> for st_data_t {
    #[inline]
    fn from(key: Key) -> Self {
        key.record
    }
}

impl PartialEq for Key {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        if self.record == other.record {
            return true;
        }
        let cmp = self.eq;
        // Safety:
        //
        // `StHashMap` assumes `cmp` is a valid non-NULL function pointer.
        unsafe { (cmp)(self.record, other.record) == 0 }
    }
}

impl PartialEq<&Key> for Key {
    #[inline]
    fn eq(&self, other: &&Self) -> bool {
        self == *other
    }
}

impl Eq for Key {}

impl Hash for Key {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.record.into());
    }
}

/// Type alias for an [`StHashMap`] that stores opaque pointers with a
/// [`st_hash_type`] derived [`StBuildHasher`].
///
/// `ExternStHashMap` stores pointers to its keys and values. It owns hasher and
/// comaparator functions given at construction time to implement [`Hash`] and
/// [`Eq`] for these opaque keys. See [`StHashMap::with_hash_type`].
pub type Table = StHashMap<Key, st_data_t, StBuildHasher>;

#[derive(Debug, Clone)]
pub struct StTable {
    pub(crate) inner: Table,
}

impl StTable {
    /// Creates an empty `StHashMap` which will use the given `st_hash_type` to
    /// hash keys.
    ///
    /// The created map has the default initial capacity.
    ///
    /// A [`Hasher`] is constructed from an [`StBuildHasher`].
    #[inline]
    #[must_use]
    pub fn with_hash_type(hash_type: *const st_hash_type) -> Self {
        let hasher = StBuildHasher::from(hash_type);
        let map = Table::with_hasher(hasher);
        Self { inner: map }
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
    pub fn with_capacity_and_hash_type(capacity: usize, hash_type: *const st_hash_type) -> Self {
        let hasher = StBuildHasher::from(hash_type);
        let map = Table::with_capacity_and_hasher(capacity, hasher);
        Self { inner: map }
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Wrapper around [`StHashMap::first`] that wraps a bare `st_data_t` in a
    /// key type that can be checked for equality.
    #[inline]
    #[must_use]
    pub fn first_raw(&self) -> Option<(&st_data_t, &st_data_t)> {
        let (key, value) = self.inner.first()?;
        Some((&key.record, value))
    }

    /// Wrapper around [`StHashMap::get`] that wraps a bare `st_data_t` in a key
    /// type that can be checked for equality.
    #[inline]
    #[must_use]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn get_raw(&self, key: st_data_t) -> Option<&st_data_t> {
        let hash_type = self.inner.hasher().hash_type();
        // Safety
        //
        // `StHashMap` assumes `hash_type` has `'static` lifetime.
        // `StHashMap` assumes `cmp` is a valid non-NULL function pointer.
        let eq = unsafe { (*hash_type).compare };
        let key = Key { record: key, eq };
        self.inner.get(&key)
    }

    /// Wrapper around [`StHashMap::get_key_value`] that wraps a bare
    /// `st_data_t` in a key type that can be checked for equality.
    #[inline]
    #[must_use]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn get_key_value_raw(&self, key: st_data_t) -> Option<(&st_data_t, &st_data_t)> {
        let hash_type = self.inner.hasher().hash_type();
        // Safety
        //
        // `StHashMap` assumes `hash_type` has `'static` lifetime.
        // `StHashMap` assumes `cmp` is a valid non-NULL function pointer.
        let eq = unsafe { (*hash_type).compare };
        let key = Key { record: key, eq };
        let (key, value) = self.inner.get_key_value(&key)?;
        Some((&key.record, value))
    }

    /// Wrapper around [`StHashMap::insert`] that wraps a bare `st_data_t` in a
    /// key type that can be checked for equality.
    #[inline]
    #[must_use]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn insert_raw(&mut self, key: st_data_t, value: st_data_t) -> Option<st_data_t> {
        let hash_type = self.inner.hasher().hash_type();
        // Safety
        //
        // `StHashMap` assumes `hash_type` has `'static` lifetime.
        // `StHashMap` assumes `cmp` is a valid non-NULL function pointer.
        let eq = unsafe { (*hash_type).compare };
        let key = Key { record: key, eq };
        self.inner.insert(key, value)
    }

    /// Wrapper around [`StHashMap::update`] that wraps a bare `st_data_t` in a
    /// key type that can be checked for equality.
    #[inline]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn update_raw(&mut self, key: st_data_t, value: st_data_t) {
        let hash_type = self.inner.hasher().hash_type();
        // Safety
        //
        // `StHashMap` assumes `hash_type` has `'static` lifetime.
        // `StHashMap` assumes `cmp` is a valid non-NULL function pointer.
        let eq = unsafe { (*hash_type).compare };
        let key = Key { record: key, eq };
        self.inner.update(key, value);
    }

    /// Wrapper around [`StHashMap::remove`] that wraps a bare `st_data_t` in a
    /// key type that can be checked for equality.
    #[inline]
    #[must_use]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn remove_raw(&mut self, key: st_data_t) -> Option<st_data_t> {
        let hash_type = self.inner.hasher().hash_type();
        // Safety
        //
        // `StHashMap` assumes `hash_type` has `'static` lifetime.
        // `StHashMap` assumes `cmp` is a valid non-NULL function pointer.
        let eq = unsafe { (*hash_type).compare };
        let key = Key { record: key, eq };
        self.inner.remove(&key)
    }

    /// Wrapper around [`StHashMap::remove_entry`] that wraps a bare `st_data_t`
    /// in a key type that can be checked for equality.
    #[inline]
    #[must_use]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn remove_entry_raw(&mut self, key: st_data_t) -> Option<(st_data_t, st_data_t)> {
        let hash_type = self.inner.hasher().hash_type();
        // Safety
        //
        // `StHashMap` assumes `hash_type` has `'static` lifetime.
        // `StHashMap` assumes `cmp` is a valid non-NULL function pointer.
        let eq = unsafe { (*hash_type).compare };
        let key = Key { record: key, eq };
        let (key, value) = self.inner.remove_entry(&key)?;
        Some((key.into(), value))
    }
}
