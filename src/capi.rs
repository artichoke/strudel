#![allow(non_upper_case_globals)]

use core::ffi::c_void;
use core::fmt;
use core::mem;
use core::ops::{Deref, DerefMut};

use crate::api;
use crate::typedefs::*;
use crate::StHash;

#[cfg(feature = "capi-specialized-init")]
mod specialized_init;

// These values enforced by test.
const PADDING_TO_NUM_ENTRIES: usize = 0;
const PADDING_TO_END: usize = 32;

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
        let table = unsafe { Box::from_raw(self.table) };
        self.num_entries = table.len() as st_index_t;
        mem::forget(table);
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

// st_table *st_init_table(const struct st_hash_type *);
#[no_mangle]
unsafe extern "C" fn st_init_table(hash_type: *const st_hash_type) -> *mut st_table {
    #[cfg(feature = "debug")]
    dbg!("st_init_table");

    api::st_init_table(hash_type)
}

// st_table *st_init_table_with_size(const struct st_hash_type *, st_index_t);
#[no_mangle]
unsafe extern "C" fn st_init_table_with_size(
    hash_type: *const st_hash_type,
    size: st_index_t,
) -> *mut st_table {
    #[cfg(feature = "debug")]
    dbg!("st_init_table_with_size");

    api::st_init_table_with_size(hash_type, size)
}

/// Delete entry with `key` from table `table`.
///
/// Set up `*VALUE` (unless `VALUE` is zero) from deleted table entry, and
/// return non-zero. If there is no entry with `key` in the table, clear
/// `*VALUE` (unless `VALUE` is zero), and return zero.
///
/// # Header declaration
///
/// ```c
/// int st_delete(st_table *, st_data_t *, st_data_t *); /* returns 0:notfound 1:deleted */
/// ```
#[no_mangle]
unsafe extern "C" fn st_delete(
    table: *mut st_table,
    key: *mut st_data_t,
    value: *mut st_data_t,
) -> libc::c_int {
    #[cfg(feature = "debug")]
    dbg!("st_delete");

    api::st_delete(table, key, value)
}

/// The function and other functions with suffix '_safe' or '_check' are
/// originated from the previous implementation of the hash tables.
///
/// It was necessary for correct deleting entries during traversing tables. The
/// current implementation permits deletion during traversing without a specific
/// way to do this.
///
/// This function has an identical implementation to `st_delete`. The
/// implementation is inlined.
///
/// # Header declaration
///
/// ```c
/// int st_delete_safe(st_table *, st_data_t *, st_data_t *, st_data_t);
/// ```
#[no_mangle]
unsafe extern "C" fn st_delete_safe(
    table: *mut st_table,
    key: *mut st_data_t,
    value: *mut st_data_t,
    never: *const st_data_t,
) -> libc::c_int {
    #[cfg(feature = "debug")]
    dbg!("st_delete_safe");

    api::st_delete_safe(table, key, value, never)
}

/// If table `table` is empty, clear `*VALUE` (unless `VALUE` is zero), and
/// return zero. Otherwise, remove the first entry in the table.  Return its key
/// through `KEY` and its record through `VALUE` (unless `VALUE` is zero).
///
/// # Header declaration
///
/// ```c
/// int st_shift(st_table *, st_data_t *, st_data_t *); /* returns 0:notfound 1:deleted */
/// ```
#[no_mangle]
unsafe extern "C" fn st_shift(
    table: *mut st_table,
    key: *mut st_data_t,
    value: *mut st_data_t,
) -> libc::c_int {
    #[cfg(feature = "debug")]
    dbg!("st_shift");

    api::st_shift(table, key, value)
}

/// Insert (KEY, VALUE) into table TAB and return zero. If there is already
/// entry with KEY in the table, return nonzero and and update the value of the
/// found entry.
///
/// # Header declaration
///
/// ```c
/// int st_insert(st_table *, st_data_t, st_data_t);
/// ```
#[no_mangle]
unsafe extern "C" fn st_insert(
    table: *mut st_table,
    key: st_data_t,
    value: st_data_t,
) -> libc::c_int {
    #[cfg(feature = "debug")]
    dbg!("st_insert");

    api::st_insert(table, key, value)
}

/// Insert (FUNC(KEY), VALUE) into table TAB and return zero. If there is
/// already entry with KEY in the table, return nonzero and and update the value
/// of the found entry.
///
/// # Header declaration
///
/// ```c
/// int st_insert2(st_table *, st_data_t, st_data_t, st_data_t (*)(st_data_t));
/// ```
#[no_mangle]
unsafe extern "C" fn st_insert2(
    table: *mut st_table,
    key: st_data_t,
    value: st_data_t,
    func: unsafe extern "C" fn(st_data_t) -> st_data_t,
) -> libc::c_int {
    #[cfg(feature = "debug")]
    dbg!("st_insert2");

    api::st_insert2(table, key, value, func)
}

/// Find an entry with `key` in table `table`. Return non-zero if we found it.
/// Set up `*RECORD` to the found entry record.
///
/// # Header declaration
///
/// ```c
/// int st_lookup(st_table *, st_data_t, st_data_t *);
/// ```
#[no_mangle]
unsafe extern "C" fn st_lookup(
    table: *mut st_table,
    key: st_data_t,
    value: *mut st_data_t,
) -> libc::c_int {
    #[cfg(feature = "debug")]
    dbg!("st_lookup");

    api::st_lookup(table, key, value)
}

/// Find an entry with `key` in table `table`. Return non-zero if we found it.
/// Set up `*RESULT` to the found table entry key.
///
/// # Header declaration
///
/// ```c
/// int st_get_key(st_table *, st_data_t, st_data_t *);
/// ```
#[no_mangle]
unsafe extern "C" fn st_get_key(
    table: *mut st_table,
    key: st_data_t,
    result: *mut st_data_t,
) -> libc::c_int {
    #[cfg(feature = "debug")]
    dbg!("st_get_key");

    api::st_get_key(table, key, result)
}

/// Find entry with `key` in table `table`, call `func` with the key and the
/// value of the found entry, and non-zero as the 3rd argument. If the entry is
/// not found, call `func` with `key`, and 2 zero arguments.
///
/// If the call returns `ST_CONTINUE`, the table will have an entry with key and
/// value returned by `func` through the 1st and 2nd parameters.  If the call of
/// `func` returns `ST_DELETE`, the table will not have entry with `key`. The
/// function returns flag of that the entry with `key` was in the table before
/// the call.
///
/// # Notes
///
/// `*key` may be altered, but must equal to the old key, i.e., the results of
/// `hash()` are same and `compare()` returns 0, otherwise the behavior is
/// undefined.
///
/// # Header declaration
///
/// ```c
/// int st_update(st_table *table, st_data_t key, st_update_callback_func *func, st_data_t arg);
/// ```
#[no_mangle]
unsafe extern "C" fn st_update(
    table: *mut st_table,
    key: st_data_t,
    func: st_update_callback_func,
    arg: st_data_t,
) -> libc::c_int {
    #[cfg(feature = "debug")]
    dbg!("st_update");

    api::st_update(table, key, func, arg)
}

// int st_foreach(st_table *, int (*)(ANYARGS), st_data_t);
#[no_mangle]
unsafe extern "C" fn st_foreach(
    table: *mut st_table,
    func: st_foreach_callback_func,
    arg: st_data_t,
) -> libc::c_int {
    #[cfg(feature = "debug")]
    dbg!("st_foreach");

    api::st_foreach(table, func, arg)
}

// int st_foreach_check(st_table *, int (*)(ANYARGS), st_data_t, st_data_t);
#[no_mangle]
unsafe extern "C" fn st_foreach_check(
    table: *mut st_table,
    func: st_foreach_callback_func,
    arg: st_data_t,
    never: st_data_t,
) -> libc::c_int {
    #[cfg(feature = "debug")]
    dbg!("st_foreach_check");

    api::st_foreach_check(table, func, arg, never)
}

/// Set up array `keys` by at most `size` keys of head table `table` entries.
/// Return the number of keys set up in array `keys`.
///
/// # Header declaration
///
/// ```c
/// st_index_t st_keys(st_table *table, st_data_t *keys, st_index_t size);
/// ```
#[no_mangle]
unsafe extern "C" fn st_keys(
    table: *mut st_table,
    keys: *mut st_data_t,
    size: st_index_t,
) -> st_index_t {
    #[cfg(feature = "debug")]
    dbg!("st_keys");

    api::st_keys(table, keys, size)
}

/// No-op. See comments for function [`st_delete_safe`].
///
/// # Header declaration
///
/// ```c
/// st_index_t st_keys_check(st_table *table, st_data_t *keys, st_index_t size, st_data_t never);
/// ```
#[no_mangle]
unsafe extern "C" fn st_keys_check(
    table: *mut st_table,
    keys: *mut st_data_t,
    size: st_index_t,
    never: st_data_t,
) -> st_index_t {
    #[cfg(feature = "debug")]
    dbg!("st_keys_check");

    api::st_keys_check(table, keys, size, never)
}

/// Set up array `values` by at most `size` values of head table `table`
/// entries. Return the number of values set up in array `values`.
///
/// # Header declaration
///
/// ```c
/// st_index_t st_values(st_table *table, st_data_t *values, st_index_t size);
/// ```
#[no_mangle]
unsafe extern "C" fn st_values(
    table: *mut st_table,
    values: *mut st_data_t,
    size: st_index_t,
) -> st_index_t {
    #[cfg(feature = "debug")]
    dbg!("st_values");

    api::st_values(table, values, size)
}

/// No-op. See comments for function [`st_delete_safe`].
///
/// # Header declaration
///
/// ```c
/// st_index_t st_values_check(st_table *table, st_data_t *values, st_index_t size, st_data_t never);
/// ```
#[no_mangle]
unsafe extern "C" fn st_values_check(
    table: *mut st_table,
    values: *mut st_data_t,
    size: st_index_t,
    never: st_data_t,
) -> st_index_t {
    #[cfg(feature = "debug")]
    dbg!("st_values_check");

    api::st_values_check(table, values, size, never)
}

// void st_add_direct(st_table *, st_data_t, st_data_t);
#[no_mangle]
unsafe extern "C" fn st_add_direct(table: *mut st_table, key: st_data_t, value: st_data_t) {
    #[cfg(feature = "debug")]
    dbg!("st_add_direct");

    api::st_add_direct(table, key, value)
}

// void st_add_direct_with_hash(st_table *tab, st_data_t key, st_data_t value, st_hash_t hash)
#[no_mangle]
unsafe extern "C" fn st_add_direct_with_hash(
    table: *mut st_table,
    key: st_data_t,
    value: st_data_t,
    hash: st_hash_t,
) {
    let _ = hash;
    #[cfg(feature = "debug")]
    dbg!("st_add_direct_with_hash");

    api::st_add_direct(table, key, value)
}

/// Free table `table` space.
///
/// # Header declaration
///
/// ```c
/// void st_free_table(st_table *);
/// ```
#[no_mangle]
unsafe extern "C" fn st_free_table(table: *mut st_table) {
    #[cfg(feature = "debug")]
    dbg!("st_free_table");

    api::st_free_table(table)
}

/// No-op. See comments for function [`st_delete_safe`].
///
/// # Header declaration
///
/// ```c
/// void st_cleanup_safe(st_table *, st_data_t);
/// ```
#[no_mangle]
unsafe extern "C" fn st_cleanup_safe(table: *mut st_table, never: st_data_t) {
    #[cfg(feature = "debug")]
    dbg!("st_cleanup_safe");

    api::st_cleanup_safe(table, never)
}

/// Make table `table` empty.
///
/// # Header declaration
///
/// ```c
/// void st_clear(st_table *);
/// ```
#[no_mangle]
unsafe extern "C" fn st_clear(table: *mut st_table) {
    #[cfg(feature = "debug")]
    dbg!("St_clear");

    api::st_clear(table)
}

// st_table *st_copy(st_table *);
#[no_mangle]
unsafe extern "C" fn st_copy(table: *mut st_table) -> *mut st_table {
    #[cfg(feature = "debug")]
    dbg!("st_copy");

    api::st_copy(table)
}

#[no_mangle]
unsafe extern "C" fn st_memsize(table: *const st_table) -> libc::size_t {
    #[cfg(feature = "debug")]
    dbg!("st_memsize");

    api::st_memsize(table)
}

// PUREFUNC(st_index_t st_hash(const void *ptr, size_t len, st_index_t h));
#[no_mangle]
unsafe extern "C" fn st_hash(ptr: *const c_void, len: libc::size_t, h: st_index_t) -> st_index_t {
    #[cfg(feature = "debug")]
    dbg!("st_hash");

    api::st_hash(ptr, len, h)
}

// CONSTFUNC(st_index_t st_hash_uint32(st_index_t h, uint32_t i));
#[no_mangle]
unsafe extern "C" fn st_hash_uint32(h: st_index_t, i: u32) -> st_index_t {
    #[cfg(feature = "debug")]
    dbg!("st_hash_uint32");

    api::st_hash_uint32(h, i)
}

// CONSTFUNC(st_index_t st_hash_uint(st_index_t h, st_index_t i));
#[no_mangle]
unsafe extern "C" fn st_hash_uint(h: st_index_t, i: st_index_t) -> st_index_t {
    #[cfg(feature = "debug")]
    dbg!("st_hash_uint");

    api::st_hash_uint(h, i)
}

// CONSTFUNC(st_index_t st_hash_end(st_index_t h));
#[no_mangle]
unsafe extern "C" fn st_hash_end(h: st_index_t) -> st_index_t {
    #[cfg(feature = "debug")]
    dbg!("st_hash_end");

    api::st_hash_end(h)
}

// CONSTFUNC(st_index_t st_hash_start(st_index_t h));
#[no_mangle]
unsafe extern "C" fn st_hash_start(h: st_index_t) -> st_index_t {
    #[cfg(feature = "debug")]
    dbg!("st_hash_start");

    api::st_hash_start(h)
}

// void rb_hash_bulk_insert_into_st_table(long, const VALUE *, VALUE);

#[cfg(test)]
mod tests {
    use core::mem::size_of;

    use crate::capi::ffi_types;
    use crate::typedefs::st_table;

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
