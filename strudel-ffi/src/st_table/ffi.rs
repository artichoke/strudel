//! FFI helpers.

use core::fmt;

use super::foreign::{Foreign, Repack};
use super::StTable;
use crate::bindings::st_hash_type;
use crate::primitives::st_index_t;

// These values enforced by test.
#[cfg(target_pointer_width = "64")]
const PADDING_TO_NUM_ENTRIES: usize = 0;
#[cfg(target_pointer_width = "64")]
const PADDING_TO_END: usize = 32;

#[cfg(target_pointer_width = "32")]
const PADDING_TO_NUM_ENTRIES: usize = 4;
#[cfg(target_pointer_width = "32")]
const PADDING_TO_END: usize = 16;

/// C struct wrapper around an [`StHashMap`].
///
/// This wrapper is FFI compatible with the C definition for access to the
/// `hash->type` and `hash->num_entries` struct fields.
///
/// This wrapper has the same `size_of` the C definition.
///
/// [`StHashMap`]: crate::StHashMap
#[repr(C)]
pub struct st_table {
    pub(super) table: *mut StTable,
    _padding: [u8; PADDING_TO_NUM_ENTRIES],
    type_: *const st_hash_type,
    num_entries: st_index_t,
    _padding_end: [u8; PADDING_TO_END],
}

impl Clone for st_table {
    fn clone(&self) -> Self {
        let inner = unsafe { (*self.table).clone() };
        inner.into()
    }
}

impl Repack for st_table {
    unsafe fn repack(&mut self) {
        self.repack();
    }
}

impl st_table {
    /// Sync the `num_entries` field on the FFI wrapper with the underlying
    /// table.
    ///
    /// This method should be called after mutable operations to the underlying
    /// [`StHashMap`].
    ///
    /// # Safety
    ///
    /// Callers must not invalidate other in-use pointers.
    ///
    /// [`StHashMap`]: crate::StHashMap
    #[inline]
    pub unsafe fn repack(&mut self) {
        let len = (*self.table).len();
        self.num_entries = len.into();
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
    pub unsafe fn from_raw(table: *mut Self) -> Foreign<Self> {
        Foreign::new_from_raw(table)
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
        f.debug_struct("st_table").field("_private", &()).finish()
    }
}

impl From<StTable> for st_table {
    #[inline]
    fn from(table: StTable) -> Self {
        let num_entries = st_index_t::from(table.inner.len());
        let hash_type = table.inner.hasher().hash_type();
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

#[cfg(test)]
mod tests {
    use core::mem::size_of;
    use std::os::raw::{c_uchar, c_uint};

    use crate::bindings::st_hash_type;
    use crate::primitives::{st_data_t, st_hash_t, st_index_t};

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
        pub entry_power: c_uchar,
        pub bin_power: c_uchar,
        pub size_ind: c_uchar,
        pub rebuilds_num: c_uint,
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

    #[test]
    fn num_entries_offset_ffi_compat() {
        let c_struct = memoffset::offset_of!(st_table, num_entries);
        let rust_struct = memoffset::offset_of!(super::st_table, num_entries);
        assert_eq!(c_struct, rust_struct);
    }

    #[test]
    fn type_offset_ffi_compat() {
        let c_struct = memoffset::offset_of!(st_table, type_);
        let rust_struct = memoffset::offset_of!(super::st_table, type_);
        assert_eq!(c_struct, rust_struct);
    }

    #[test]
    fn size_of_ffi_compat() {
        let c_struct = size_of::<st_table>();
        let rust_struct = size_of::<super::st_table>();
        assert_eq!(c_struct, rust_struct);
    }
}
