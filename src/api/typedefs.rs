use core::hash::{Hash, Hasher};

use std::os::raw::c_int;

use crate::api::primitives::{st_data_t, st_index_t};
use crate::api::StBuildHasher;
use crate::StHashMap;

/// A wrapper around a raw `st_data_t` key that includes a vtable for equality
/// comparisons.
#[derive(Debug, Clone)]
pub struct ExternKey {
    record: st_data_t,
    eq: st_compare_func,
}

impl ExternKey {
    /// Return a reference to the inner key.
    #[inline]
    #[must_use]
    pub fn inner(&self) -> &st_data_t {
        &self.record
    }
}

impl From<ExternKey> for st_data_t {
    #[inline]
    fn from(key: ExternKey) -> Self {
        key.record
    }
}

impl PartialEq for ExternKey {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        if self.record == other.record {
            return true;
        }
        let cmp = self.eq;
        // Safety
        //
        // `StHashMap` assumes `cmp` is a valid non-NULL function pointer.
        unsafe { (cmp)(self.record, other.record) == 0 }
    }
}

impl PartialEq<&ExternKey> for ExternKey {
    #[inline]
    fn eq(&self, other: &&Self) -> bool {
        self == *other
    }
}

impl Eq for ExternKey {}

impl Hash for ExternKey {
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
pub type ExternStHashMap = StHashMap<ExternKey, st_data_t, StBuildHasher>;

#[derive(Debug, Clone)]
pub struct ExternHashMap {
    pub(crate) inner: ExternStHashMap,
}

impl ExternHashMap {
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
        let map = ExternStHashMap::with_hasher(hasher);
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
        let map = ExternStHashMap::with_capacity_and_hasher(capacity, hasher);
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
        let key = ExternKey { record: key, eq };
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
        let key = ExternKey { record: key, eq };
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
        let key = ExternKey { record: key, eq };
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
        let key = ExternKey { record: key, eq };
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
        let key = ExternKey { record: key, eq };
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
        let key = ExternKey { record: key, eq };
        let (key, value) = self.inner.remove_entry(&key)?;
        Some((key.into(), value))
    }
}

/// Equality comparator function for `StHash` keys.
///
/// # Header declaration
///
/// ```c
/// typedef int st_compare_func(st_data_t, st_data_t);
/// ```
pub type st_compare_func = unsafe extern "C" fn(st_data_t, st_data_t) -> c_int;

/// Hash function for `StHash` keys.
///
/// # Header declaration
///
/// ```c
/// typedef st_index_t st_hash_func(st_data_t);
/// ```
pub type st_hash_func = unsafe extern "C" fn(st_data_t) -> st_index_t;

/// Equality comparator and hash function used to build a [`StHashMap`] hasher.
///
/// These functions are `unsafe extern "C" fn` and expected to be supplied via
/// FFI.
///
/// # Safety
///
/// `st_hash_type` are expected to have `'static` lifetime. This assumption is
/// exploited by [`StHashMap`] and [`StBuildHasher`].
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct st_hash_type {
    /// `st_compare_func`
    ///
    /// # Header declaration
    ///
    /// ```c
    /// (*compare)(ANYARGS /*st_data_t, st_data_t*/); /* st_compare_func* */
    /// ```
    pub compare: st_compare_func,

    /// `st_hash_func`
    ///
    /// # Header declaration
    ///
    /// ```c
    /// st_index_t (*hash)(ANYARGS /*st_data_t*/);        /* st_hash_func* */
    /// ```
    pub hash: st_hash_func,
}

/// Return values from [`st_foreach_callback_func`] and
/// [`st_update_callback_func`] callback function pointers.
#[repr(C)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[allow(clippy::upper_case_acronyms)]
pub enum st_retval {
    /// Continue iteration.
    ST_CONTINUE,

    /// Stop iteration.
    ST_STOP,

    /// Delete current iteration `(key, value)` pair and continue iteration.
    ST_DELETE,

    /// Continue or stop iteration.
    ///
    /// This return value has slightly different behavior depending on API. See
    /// [`st_foreach`] and [`st_foreach_check`].
    ///
    /// [`st_foreach`]: crate::api::st_foreach
    /// [`st_foreach_check`]: crate::api::st_foreach_check
    ST_CHECK,
}

impl PartialEq<i32> for st_retval {
    fn eq(&self, other: &i32) -> bool {
        *self as i32 == *other
    }
}

impl PartialEq<st_retval> for i32 {
    fn eq(&self, other: &st_retval) -> bool {
        *self == *other as i32
    }
}

/// [`st_update`] callback function.
///
/// # Header declaration
///
/// ```c
/// typedef int st_update_callback_func(st_data_t *key, st_data_t *value, st_data_t arg, int existing);
/// ```
///
/// [`st_update`]: crate::api::st_update
pub type st_update_callback_func =
    unsafe extern "C" fn(*mut st_data_t, *mut st_data_t, st_data_t, c_int) -> c_int;

/// [`st_foreach`] and [`st_foreach_check`] callback function.
///
/// # Header declaration
///
/// ```c
/// int (*)(ANYARGS)
/// ```
///
/// [`st_foreach`]: crate::api::st_foreach
/// [`st_foreach_check`]: crate::api::st_foreach_check
pub type st_foreach_callback_func =
    unsafe extern "C" fn(st_data_t, st_data_t, st_data_t, i32) -> i32;
