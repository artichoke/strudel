use core::fmt;
use core::ops::{Deref, DerefMut};

use crate::StHash;

#[cfg(target_pointer_width = "64")]
pub type st_data_t = u64;
#[cfg(target_pointer_width = "32")]
pub type st_data_t = u32;

pub type st_index_t = st_data_t;

pub type st_hash_t = st_index_t;

// typedef int st_compare_func(st_data_t, st_data_t);
pub type st_compare_func = unsafe extern "C" fn(st_data_t, st_data_t) -> i32;

// typedef st_index_t st_hash_func(st_data_t);
pub type st_hash_func = unsafe extern "C" fn(st_data_t) -> st_index_t;

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

#[repr(C)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum st_retval {
    ST_CONTINUE,
    ST_STOP,
    ST_DELETE,
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

// typedef int st_update_callback_func(st_data_t *key, st_data_t *value, st_data_t arg, int existing);
pub type st_update_callback_func =
    unsafe extern "C" fn(*mut st_data_t, *mut st_data_t, st_data_t, i32) -> i32;

// int (*)(ANYARGS)
pub type st_foreach_callback_func =
    unsafe extern "C" fn(st_data_t, st_data_t, st_data_t, i32) -> i32;

// These values enforced by test.
const PADDING_TO_NUM_ENTRIES: usize = 0;
const PADDING_TO_END: usize = 32;

/// C struct wrapper around an `StHash`.
///
/// This wrapper allows property access to `hash->type` and `hash->num_entries`
/// from C callers.
#[repr(C)]
pub struct st_table {
    table: *mut StHash,
    _padding: [u8; PADDING_TO_NUM_ENTRIES],
    type_: *const st_hash_type,
    num_entries: st_index_t,
    _padding_end: [u8; PADDING_TO_END],
}

impl st_table {
    #[inline]
    pub fn ensure_num_entries_is_consistent_after_writes(&mut self) {
        self.num_entries = self.len() as st_index_t;
    }

    #[inline]
    #[must_use]
    pub fn into_raw(table: Self) -> *mut Self {
        let table = Box::new(table);
        Box::into_raw(table)
    }

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

impl From<StHash> for st_table {
    #[inline]
    fn from(table: StHash) -> Self {
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
    type Target = StHash;

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

    use crate::typedefs::{ffi_types, st_table};

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
    use crate::typedefs::*;

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
