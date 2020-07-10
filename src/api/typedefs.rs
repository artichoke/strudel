use core::fmt;
use core::hash::{Hash, Hasher};
use core::mem::size_of;
use core::ops::{Deref, DerefMut};

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
        #[cfg(target_pointer_width = "32")]
        state.write_u32(self.record);
        #[cfg(target_pointer_width = "64")]
        state.write_u64(self.record);
    }
}

/// Type alias for an [`StHashMap`] that stores opaque pointers with a
/// [`st_hash_type`] derived [`StBuildHasher`].
///
/// `ExternStHashMap` stores pointers to its keys and values. It owns hasher and
/// comaparator functions given at construction time to implement [`Hash`] and
/// [`Eq`] for these opaque keys. See [`StHashMap::with_hash_type`].
pub type ExternStHashMap = StHashMap<ExternKey, st_data_t, StBuildHasher>;

impl ExternStHashMap {
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
        Self::with_hasher(hasher)
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
        Self::with_capacity_and_hasher(capacity, hasher)
    }

    /// Wrapper around [`StHashMap::first`] that wraps a bare `st_data_t` in a
    /// key type that can be checked for equality.
    #[inline]
    #[must_use]
    pub fn first_raw(&self) -> Option<(&st_data_t, &st_data_t)> {
        let (key, value) = self.first()?;
        Some((&key.record, value))
    }

    /// Wrapper around [`StHashMap::get`] that wraps a bare `st_data_t` in a key
    /// type that can be checked for equality.
    #[inline]
    #[must_use]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn get_raw(&self, key: st_data_t) -> Option<&st_data_t> {
        let hash_type = self.hasher().hash_type();
        // Safety
        //
        // `StHashMap` assumes `hash_type` has `'static` lifetime.
        // `StHashMap` assumes `cmp` is a valid non-NULL function pointer.
        let eq = unsafe { (*hash_type).compare };
        let key = ExternKey { record: key, eq };
        self.get(&key)
    }

    /// Wrapper around [`StHashMap::get_key_value`] that wraps a bare
    /// `st_data_t` in a key type that can be checked for equality.
    #[inline]
    #[must_use]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn get_key_value_raw(&self, key: st_data_t) -> Option<(&st_data_t, &st_data_t)> {
        let hash_type = self.hasher().hash_type();
        // Safety
        //
        // `StHashMap` assumes `hash_type` has `'static` lifetime.
        // `StHashMap` assumes `cmp` is a valid non-NULL function pointer.
        let eq = unsafe { (*hash_type).compare };
        let key = ExternKey { record: key, eq };
        let (key, value) = self.get_key_value(&key)?;
        Some((&key.record, value))
    }

    /// Wrapper around [`StHashMap::insert`] that wraps a bare `st_data_t` in a
    /// key type that can be checked for equality.
    #[inline]
    #[must_use]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn insert_raw(&mut self, key: st_data_t, value: st_data_t) -> Option<st_data_t> {
        let hash_type = self.hasher().hash_type();
        // Safety
        //
        // `StHashMap` assumes `hash_type` has `'static` lifetime.
        // `StHashMap` assumes `cmp` is a valid non-NULL function pointer.
        let eq = unsafe { (*hash_type).compare };
        let key = ExternKey { record: key, eq };
        self.insert(key, value)
    }

    /// Wrapper around [`StHashMap::update`] that wraps a bare `st_data_t` in a
    /// key type that can be checked for equality.
    #[inline]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn update_raw(&mut self, key: st_data_t, value: st_data_t) {
        let hash_type = self.hasher().hash_type();
        // Safety
        //
        // `StHashMap` assumes `hash_type` has `'static` lifetime.
        // `StHashMap` assumes `cmp` is a valid non-NULL function pointer.
        let eq = unsafe { (*hash_type).compare };
        let key = ExternKey { record: key, eq };
        self.update(key, value)
    }

    /// Wrapper around [`StHashMap::remove`] that wraps a bare `st_data_t` in a
    /// key type that can be checked for equality.
    #[inline]
    #[must_use]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn remove_raw(&mut self, key: st_data_t) -> Option<st_data_t> {
        let hash_type = self.hasher().hash_type();
        // Safety
        //
        // `StHashMap` assumes `hash_type` has `'static` lifetime.
        // `StHashMap` assumes `cmp` is a valid non-NULL function pointer.
        let eq = unsafe { (*hash_type).compare };
        let key = ExternKey { record: key, eq };
        self.remove(&key)
    }

    /// Wrapper around [`StHashMap::remove_entry`] that wraps a bare `st_data_t`
    /// in a key type that can be checked for equality.
    #[inline]
    #[must_use]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn remove_entry_raw(&mut self, key: st_data_t) -> Option<(st_data_t, st_data_t)> {
        let hash_type = self.hasher().hash_type();
        // Safety
        //
        // `StHashMap` assumes `hash_type` has `'static` lifetime.
        // `StHashMap` assumes `cmp` is a valid non-NULL function pointer.
        let eq = unsafe { (*hash_type).compare };
        let key = ExternKey { record: key, eq };
        let (key, value) = self.remove_entry(&key)?;
        Some((key.into(), value))
    }
}

/// Type alias for pointers to keys and values.
///
/// Assumed to be equivalent to `usize`.
///
/// ```
/// # use core::mem::size_of;
/// # use strudel::api::st_data_t;
/// assert_eq!(size_of::<usize>(), size_of::<st_data_t>());
/// ```
#[cfg(target_pointer_width = "64")]
pub type st_data_t = u64;

/// Type alias for pointers to keys and values.
///
/// Assumed to be equivalent to `usize`.
///
/// ```
/// # use core::mem::size_of;
/// # use strudel::api::st_data_t;
/// assert_eq!(size_of::<usize>(), size_of::<st_data_t>());
/// ```
#[cfg(target_pointer_width = "32")]
pub type st_data_t = u32;

/// Type alias for insertion order indexes.
pub type st_index_t = st_data_t;

/// Type alias for hash values.
pub type st_hash_t = st_index_t;

/// Equality comparator function for `StHash` keys.
///
/// # Header declaration
///
/// ```c
/// typedef int st_compare_func(st_data_t, st_data_t);
/// ```
pub type st_compare_func = unsafe extern "C" fn(st_data_t, st_data_t) -> i32;

/// Hash function for `StHash` keys.
///
/// # Header declaration
///
/// ```c
/// typedef st_index_t st_hash_func(st_data_t);
/// ```
pub type st_hash_func = unsafe extern "C" fn(st_data_t) -> st_index_t;

// typedef char st_check_for_sizeof_st_index_t[SIZEOF_VOIDP == (int)sizeof(st_index_t) ? 1 : -1];
const _: () = [()][(size_of::<usize>() == size_of::<st_index_t>()) as usize - 1];

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
    unsafe extern "C" fn(*mut st_data_t, *mut st_data_t, st_data_t, i32) -> i32;

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

// These values enforced by test.
#[cfg(target_pointer_width = "64")]
const PADDING_TO_NUM_ENTRIES: usize = 0;
#[cfg(target_pointer_width = "64")]
const PADDING_TO_END: usize = 32;

#[cfg(target_pointer_width = "32")]
const PADDING_TO_NUM_ENTRIES: usize = 4;
#[cfg(target_pointer_width = "32")]
const PADDING_TO_END: usize = 32;

/// C struct wrapper around an [`ExternStHashMap`].
///
/// This wrapper is FFI compatible with the C definition for access to the
/// `hash->type` and `hash->num_entries` struct fields.
///
/// This wrapper has the same `size_of` the C definition.
///
/// `st_table` `deref`s and `deref_mut`s to [`ExternStHashMap`]
#[repr(C)]
pub struct st_table {
    table: *mut ExternStHashMap,
    _padding: [u8; PADDING_TO_NUM_ENTRIES],
    type_: *const st_hash_type,
    num_entries: st_index_t,
    _padding_end: [u8; PADDING_TO_END],
}

impl st_table {
    /// Sync the `num_entries` field on the FFI wrapper with the underlying
    /// table.
    ///
    /// This method should be called after mutable operations to the underlying
    /// [`StHashMap`].
    #[inline]
    pub fn ensure_num_entries_is_consistent_after_writes(&mut self) {
        self.num_entries = self.len() as st_index_t;
    }

    /// Consumes the table, returning a wrapped raw pointer.
    ///
    /// The pointer will be properly aligned and non-null.
    ///
    /// After calling this function, the caller is responsible for the allocated
    /// memory. In particular, the caller should properly destroy the `st_table`
    /// and release the memory, taking into account the memory layout used. The
    /// easiest way to do this is to convert the raw pointer back into a bosed
    /// `st_table` with the [`st_table::from_raw`] function, allowing the
    /// destructor to perform the cleanup.
    ///
    /// Note: this is an associated function, which means that you have to call
    /// it as `st_table::into_raw(table)` instead of `table.into_raw()`.
    #[inline]
    #[must_use]
    pub fn into_raw(table: Self) -> *mut Self {
        let table = Box::new(table);
        Box::into_raw(table)
    }

    /// Consumes the boxed table, returning a wrapped raw pointer.
    ///
    /// The pointer will be properly aligned and non-null.
    ///
    /// After calling this function, the caller is responsible for the allocated
    /// memory. In particular, the caller should properly destroy the `st_table`
    /// and release the memory, taking into account the memory layout used. The
    /// easiest way to do this is to convert the raw pointer back into a bosed
    /// `st_table` with the [`st_table::from_raw`] function, allowing the
    /// destructor to perform the cleanup.
    ///
    /// Note: this is an associated function, which means that you have to call
    /// it as `st_table::into_raw(table)` instead of `table.into_raw()`.
    #[inline]
    #[must_use]
    pub fn boxed_into_raw(table: Box<Self>) -> *mut Self {
        Box::into_raw(table)
    }

    /// Construct a boxed `st_table` from a raw pointer.
    ///
    /// # Safety
    ///
    /// This function is unsafe because improper use may lead to memory
    /// problems. For example, a double-free may occur if the function is
    /// called twice on the same raw pointer.
    ///
    /// The `table` pointer must be non-null and allocated using either
    /// [`st_table::into_raw`] or [`st_table::boxed_into_raw`].
    #[inline]
    #[must_use]
    pub unsafe fn from_raw(table: *mut Self) -> Box<Self> {
        Box::from_raw(table)
    }
}

impl Drop for st_table {
    fn drop(&mut self) {
        let inner = unsafe { Box::from_raw(self.table) };
        drop(inner);
    }
}

impl fmt::Debug for st_table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "st_table {{ opaque FFI type }}")
    }
}

impl From<ExternStHashMap> for st_table {
    #[inline]
    fn from(table: ExternStHashMap) -> Self {
        let num_entries = table.len() as st_index_t;
        let hash_type = table.hasher().hash_type();
        let table = Box::new(table);
        let table = Box::into_raw(table);
        Self {
            table,
            _padding: [0; PADDING_TO_NUM_ENTRIES],
            type_: hash_type,
            num_entries,
            _padding_end: [0; PADDING_TO_END],
        }
    }
}

impl Deref for st_table {
    type Target = ExternStHashMap;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.table) }
    }
}

impl DerefMut for st_table {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut (*self.table) }
    }
}

#[cfg(test)]
mod tests {
    use core::mem::size_of;

    use crate::api::typedefs::{ffi_types, st_table};

    #[test]
    fn num_entries_offset_ffi_compat() {
        let c_struct = memoffset::offset_of!(ffi_types::st_table, num_entries);
        let rust_struct = memoffset::offset_of!(st_table, num_entries);
        assert_eq!(c_struct, rust_struct);
    }

    #[test]
    fn type_offset_ffi_compat() {
        let c_struct = memoffset::offset_of!(ffi_types::st_table, type_);
        let rust_struct = memoffset::offset_of!(st_table, type_);
        assert_eq!(c_struct, rust_struct);
    }

    #[test]
    fn size_of_ffi_compat() {
        let c_struct = size_of::<ffi_types::st_table>();
        let rust_struct = size_of::<st_table>();
        assert_eq!(c_struct, rust_struct);
    }
}

#[cfg(test)]
mod ffi_types {
    use crate::api::typedefs::{st_data_t, st_hash_t, st_hash_type, st_index_t};

    /// `st_table` struct definition from C in `st.h`.
    ///
    /// # Header declaration
    ///
    /// ```c
    /// struct st_table {
    ///     /* Cached features of the table -- see st.c for more details.  */
    ///     unsigned char entry_power, bin_power, size_ind;
    ///     /* How many times the table was rebuilt.  */
    ///     unsigned int rebuilds_num;
    ///     const struct st_hash_type *type;
    ///     /* Number of entries currently in the table.  */
    ///     st_index_t num_entries;
    ///     /* Array of bins used for access by keys.  */
    ///     st_index_t *bins;
    ///     /* Start and bound index of entries in array entries.
    ///        entries_starts and entries_bound are in interval
    ///        [0,allocated_entries].  */
    ///     st_index_t entries_start, entries_bound;
    ///     /* Array of size 2^entry_power.  */
    ///     st_table_entry *entries;
    /// };
    /// ```
    #[repr(C)]
    pub struct st_table {
        pub entry_power: libc::c_uchar,
        pub bin_power: libc::c_uchar,
        pub size_ind: libc::c_uchar,
        pub rebuilds_num: libc::c_uint,
        pub type_: *const st_hash_type,
        pub num_entries: st_index_t,
        pub bins: *mut st_index_t,
        pub entries_start: st_index_t,
        pub entries_bound: st_index_t,
        pub entries: *mut st_table_entry,
    }

    /// `st_table_entry` struct definition from C in `st.c`.
    ///
    /// # Header declaration
    ///
    /// ```c
    /// struct st_table_entry {
    ///     st_hash_t hash;
    ///     st_data_t key;
    ///     st_data_t record;
    /// };
    /// ```
    #[repr(C)]
    pub struct st_table_entry {
        pub hash: st_hash_t,
        pub key: st_data_t,
        pub record: st_data_t,
    }
}
