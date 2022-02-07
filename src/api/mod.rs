#![allow(non_upper_case_globals)]

//! `st_hash`-compatible Rust API bindings for [`ExternHashMap`].
//!
//! For the C API bindings, see the `capi` module.

use core::ffi::c_void;
use core::hash::Hasher;
use core::mem;
use core::ptr;
use core::slice;
use fnv::FnvHasher;

mod hasher;
mod primitives;
mod typedefs;

pub use hasher::{StBuildHasher, StHasher};
pub use primitives::{st_data_t, st_hash_t, st_index_t};
pub(crate) use typedefs::{
    st_foreach_callback_func, st_hash_type, st_retval, st_update_callback_func, ExternHashMap,
};

use crate::ffi::st_table;

const DEFAULT_CAPACITY: usize = 8;

/// Create and return table with `type` which can hold a minimal number of
/// entries.
///
/// # Header declaration
///
/// ```c
/// st_table *st_init_table(const struct st_hash_type *);
/// ```
#[inline]
#[must_use]
pub fn st_init_table(hash_type: *const st_hash_type) -> *mut st_table {
    let table = ExternHashMap::with_capacity_and_hash_type(DEFAULT_CAPACITY, hash_type);
    st_table::into_raw(table.into())
}

/// Create and return table with `type` which can hold at least `size` entries.
/// The real number of entries which the table can hold is the nearest power of
/// two for `size`.
///
/// # Header declaration
///
/// ```c
/// st_table *st_init_table_with_size(const struct st_hash_type *, st_index_t);
/// ```
#[inline]
#[must_use]
pub fn st_init_table_with_size(hash_type: *const st_hash_type, size: st_index_t) -> *mut st_table {
    let table = ExternHashMap::with_capacity_and_hash_type(size.into(), hash_type);
    st_table::into_raw(table.into())
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
///
/// # Safety
///
/// `table` must be non-null and point to a valid `st_table` created by the
/// `st_init_table` family of functions.
#[inline]
pub unsafe fn st_delete(
    table: *mut st_table,
    key: *mut st_data_t,
    value: *mut st_data_t,
) -> libc::c_int {
    let mut table = st_table::from_raw(table);
    let inner = table.as_inner_mut();

    if let Some((entry_key, entry_value)) = (*inner).remove_entry_raw(*key) {
        ptr::write(key, entry_key);
        if !value.is_null() {
            ptr::write(value, entry_value);
        }
        1
    } else {
        if !value.is_null() {
            ptr::write(value, 0_usize.into());
        }
        0
    }
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
///
/// # Safety
///
/// `table` must be non-null and point to a valid `st_table` created by the
/// `st_init_table` family of functions.
#[inline]
pub unsafe fn st_delete_safe(
    table: *mut st_table,
    key: *mut st_data_t,
    value: *mut st_data_t,
    _never: *const st_data_t,
) -> libc::c_int {
    st_delete(table, key, value)
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
///
/// # Safety
///
/// `table` must be non-null and point to a valid `st_table` created by the
/// `st_init_table` family of functions.
#[inline]
pub unsafe fn st_shift(
    table: *mut st_table,
    key: *mut st_data_t,
    value: *mut st_data_t,
) -> libc::c_int {
    let mut table = st_table::from_raw(table);
    let inner = table.as_inner_mut();

    if let Some((&first_key, _)) = (*inner).first_raw() {
        if let Some((entry_key, entry_value)) = (*inner).remove_entry_raw(first_key) {
            ptr::write(key, entry_key);
            if !value.is_null() {
                ptr::write(value, entry_value);
            }
            return 1;
        }
    }
    if !value.is_null() {
        ptr::write(value, 0_usize.into());
    }
    0
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
///
/// # Safety
///
/// `table` must be non-null and point to a valid `st_table` created by the
/// `st_init_table` family of functions.
#[inline]
pub unsafe fn st_insert(table: *mut st_table, key: st_data_t, value: st_data_t) -> libc::c_int {
    let mut table = st_table::from_raw(table);
    let inner = table.as_inner_mut();

    if (*inner).insert_raw(key, value).is_some() {
        1
    } else {
        0
    }
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
///
/// # Safety
///
/// `table` must be non-null and point to a valid `st_table` created by the
/// `st_init_table` family of functions.
///
/// `func` must be a callback function with the
/// `unsafe extern "C" fn(st_data_t) -> st_data_t` signature.
#[inline]
pub unsafe fn st_insert2(
    table: *mut st_table,
    key: st_data_t,
    value: st_data_t,
    func: unsafe extern "C" fn(st_data_t) -> st_data_t,
) -> libc::c_int {
    let table_raw = table;
    let mut table = st_table::from_raw(table);
    let inner = table.as_inner_mut();

    if (*inner).get_raw(key).is_some() {
        let _ = (*inner).insert_raw(key, value);
        1
    } else {
        // `func` might mutate this table, so make sure we don't alias the `Box`.
        drop(table);
        let key = func(key);

        let _ = (*inner).insert_raw(key, value);
        // We can reuse `inner` because it is guaranteed to not change for the
        // life of the table, but we reify the table so we can repack its size
        // and other metadata into the FFI struct.
        drop(st_table::from_raw(table_raw));

        0
    }
}

/// Find an entry with `key` in table `table`. Return non-zero if we found it.
/// Set up `*RECORD` to the found entry record.
///
/// # Header declaration
///
/// ```c
/// int st_lookup(st_table *, st_data_t, st_data_t *);
/// ```
///
/// # Safety
///
/// `table` must be non-null and point to a valid `st_table` created by the
/// `st_init_table` family of functions.
#[inline]
pub unsafe fn st_lookup(
    table: *mut st_table,
    key: st_data_t,
    value: *mut st_data_t,
) -> libc::c_int {
    let mut table = st_table::from_raw(table);
    let inner = table.as_inner_mut();

    if let Some(&entry_value) = (*inner).get_raw(key) {
        if !value.is_null() {
            ptr::write(value, entry_value);
        }
        1
    } else {
        0
    }
}

/// Find an entry with `key` in table `table`. Return non-zero if we found it.
/// Set up `*RESULT` to the found table entry key.
///
/// # Header declaration
///
/// ```c
/// int st_get_key(st_table *, st_data_t, st_data_t *);
/// ```
///
/// # Safety
///
/// `table` must be non-null and point to a valid `st_table` created by the
/// `st_init_table` family of functions.
#[inline]
pub unsafe fn st_get_key(
    table: *mut st_table,
    key: st_data_t,
    result: *mut st_data_t,
) -> libc::c_int {
    let mut table = st_table::from_raw(table);
    let inner = table.as_inner_mut();

    if let Some((&entry_key, _)) = (*inner).get_key_value_raw(key) {
        if !result.is_null() {
            ptr::write(result, entry_key);
        }
        1
    } else {
        0
    }
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
///
/// # Safety
///
/// `table` must be non-null and point to a valid `st_table` created by the
/// `st_init_table` family of functions.
///
/// `func` must be a callback function with the `st_update_callback_func`
/// signature.
#[inline]
pub unsafe fn st_update(
    table: *mut st_table,
    key: st_data_t,
    func: st_update_callback_func,
    arg: st_data_t,
) -> libc::c_int {
    use st_retval::{ST_CONTINUE, ST_DELETE};

    let table_raw = table;
    let mut table = st_table::from_raw(table);
    let inner = table.as_inner_mut();

    let (existing, mut key, mut value) =
        if let Some((&entry_key, &entry_value)) = (*inner).get_key_value_raw(key) {
            (true, entry_key, entry_value)
        } else {
            (false, key, 0_usize.into())
        };

    let old_key = key;
    let old_value = value;

    // `func` might mutate this table, so make sure we don't alias the `Box`.
    drop(table);
    let update = func(&mut key, &mut value, arg, existing as libc::c_int);

    if update == ST_CONTINUE {
        match (key, value) {
            (key, value) if existing && key == old_key && value == old_value => {}
            (key, value) if key == old_key => {
                let _ = (*inner).insert_raw(key, value);
            }
            (key, value) => {
                // In the MRI implementation, `st_add_direct_with_hash` is called in
                // this position, which has the following docs:
                //
                // > Insert (KEY, VALUE, HASH) into table TAB. The table should not
                // > have entry with KEY before the insertion.
                //
                // Rust maps do not expose direct insert with hash APIs, so we go
                // through the normal insert route. This is semantically different
                // behavior because `hash` of `key` might have changed when calling
                // `func`.
                //
                // # Header declaration
                //
                // ```c
                // st_add_direct_with_hash(table, key, value, hash);
                // ```
                (*inner).update_raw(key, value);
            }
        }
    } else if update == ST_DELETE && existing {
        let _ = (*inner).remove_raw(old_key);
    }

    // We can reuse `inner` above because it is guaranteed to not change for the
    // life of the table, but we reify the table so we can repack its size and
    // other metadata into the FFI struct.
    drop(st_table::from_raw(table_raw));

    existing as libc::c_int
}

/// Traverse all entries in table `table` calling `func` with current entry key
/// and value and zero. If the call returns `ST_STOP`, stop traversing. If the
/// call returns `ST_DELETE`, delete the current entry from the table. In case
/// of `ST_CHECK` or `ST_CONTINUE`, continue traversing. The function returns
/// zero unless an error is found.
///
/// The behavior is a bit different from [`st_foreach_check`] when `ST_CHECK`
/// is returned from `func` and when the current element is removed during
/// traversing.
///
/// # Header declaration
///
/// ```c
/// int st_foreach(st_table *, int (*)(ANYARGS), st_data_t);
/// ```
///
/// # Safety
///
/// `table` must be non-null and point to a valid `st_table` created by the
/// `st_init_table` family of functions.
///
/// `func` must be a callback function with the `st_foreach_callback_func`
/// signature.
#[inline]
pub unsafe fn st_foreach(
    table: *mut st_table,
    func: st_foreach_callback_func,
    arg: st_data_t,
) -> libc::c_int {
    use st_retval::{ST_CHECK, ST_CONTINUE, ST_DELETE, ST_STOP};

    let table_raw = table;
    let mut table = st_table::from_raw(table);
    let inner = table.as_inner_mut();

    let mut insertion_ranks = (*inner).inner.insert_ranks_from(0);
    let mut last_seen_rank = 0;

    // `func` might mutate this table, so make sure we don't alias the `Box`.
    drop(table);

    loop {
        if let Some(rank) = insertion_ranks.next() {
            last_seen_rank = rank;
            let nth = (*inner)
                .inner
                .get_nth(rank)
                .map(|(key, &value)| (*key.inner(), value));

            if let Some((key, value)) = nth {
                let retval = func(key, value, arg, 0);

                match retval {
                    retval if ST_CONTINUE == retval => {}
                    retval if ST_CHECK == retval || ST_STOP == retval => return 0,
                    retval if ST_DELETE == retval => {
                        let _ = (*inner).remove_raw(key);
                        // We can reuse `inner` above because it is guaranteed
                        // to not change for the life of the table, but we reify
                        // the table so we can repack its size and other metadata
                        // into the FFI struct.
                        //
                        // We must do this since we are looping and `func` may be
                        // called multiple times.
                        drop(st_table::from_raw(table_raw));
                    }
                    _ => {}
                }
            }
        } else {
            let current_max = (*inner).inner.max_insert_rank();
            if current_max <= last_seen_rank {
                break;
            }
            insertion_ranks = (*inner).inner.insert_ranks_from(last_seen_rank);
            // We can reuse `inner` above because it is guaranteed to not change
            // for the life of the table, but we reify the table so we can repack
            // its size and other metadata into the FFI struct.
            //
            // We must do this since we are looping and `func` may be called
            // multiple times.
            drop(st_table::from_raw(table_raw));
        }
    }
    0
}

/// Traverse all entries in table `table` calling `func` with current entry key
/// and value and zero. If the call returns `ST_STOP`, stop traversing. If the
/// call returns `ST_DELETE`, delete the current entry from the table. In case
/// of `ST_CHECK` or `ST_CONTINUE`, continue traversing. The function returns
/// zero unless an error is found.
///
/// The behavior is a bit different from [`st_foreach`] when `ST_CHECK` is
/// returned from `func` and when the current element is removed during
/// traversing.
///
/// # Header declaration
///
/// ```c
/// int st_foreach_check(st_table *, int (*)(ANYARGS), st_data_t, st_data_t);
/// ```
///
/// # Safety
///
/// `table` must be non-null and point to a valid `st_table` created by the
/// `st_init_table` family of functions.
///
/// `func` must be a callback function with the `st_foreach_callback_func`
/// signature.
#[inline]
pub unsafe fn st_foreach_check(
    table: *mut st_table,
    func: st_foreach_callback_func,
    arg: st_data_t,
    _never: st_data_t,
) -> libc::c_int {
    use st_retval::{ST_CHECK, ST_CONTINUE, ST_DELETE, ST_STOP};

    let table_raw = table;
    let mut table = st_table::from_raw(table);
    let inner = table.as_inner_mut();

    let mut insertion_ranks = (*inner).inner.insert_ranks_from(0);
    let mut last_seen_rank = 0;

    // `func` might mutate this table, so make sure we don't alias the `Box`.
    drop(table);

    loop {
        if let Some(rank) = insertion_ranks.next() {
            last_seen_rank = rank;
            let nth = (*inner)
                .inner
                .get_nth(rank)
                .map(|(key, &value)| (*key.inner(), value));

            if let Some((key, value)) = nth {
                let retval = func(key, value, arg, 0);
                match retval {
                    retval if ST_CONTINUE == retval || ST_CHECK == retval => {}
                    retval if ST_STOP == retval => return 0,
                    retval if ST_DELETE == retval => {
                        let _ = (*inner).remove_raw(key);
                        // We can reuse `inner` above because it is guaranteed
                        // to not change for the life of the table, but we reify
                        // the table so we can repack its size and other metadata
                        // into the FFI struct.
                        //
                        // We must do this since we are looping and `func` may be
                        // called multiple times.
                        drop(st_table::from_raw(table_raw));
                    }
                    _ => {}
                }
            }
        } else {
            let current_max = (*inner).inner.max_insert_rank();
            if current_max <= last_seen_rank {
                break;
            }
            insertion_ranks = (*inner).inner.insert_ranks_from(last_seen_rank);
            // We can reuse `inner` above because it is guaranteed to not change
            // for the life of the table, but we reify the table so we can repack
            // its size and other metadata into the FFI struct.
            //
            // We must do this since we are looping and `func` may be called
            // multiple times.
            drop(st_table::from_raw(table_raw));
        }
    }
    0
}

/// Set up array `keys` by at most `size` keys of head table `table` entries.
/// Return the number of keys set up in array `keys`.
///
/// # Header declaration
///
/// ```c
/// st_index_t st_keys(st_table *table, st_data_t *keys, st_index_t size);
/// ```
///
/// # Safety
///
/// `table` must be non-null and point to a valid `st_table` created by the
/// `st_init_table` family of functions.
///
/// `keys` must be non-null and point to an array.
#[inline]
pub unsafe fn st_keys(table: *mut st_table, keys: *mut st_data_t, size: st_index_t) -> st_index_t {
    let mut table = st_table::from_raw(table);
    let inner = table.as_inner_mut();

    let keys = slice::from_raw_parts_mut(keys, size.into());
    let mut count = 0;
    for (counter, (slot, key)) in keys.iter_mut().zip((*inner).inner.keys()).enumerate() {
        ptr::write(slot, *key.inner());
        count = counter;
    }
    mem::forget(table);
    count.into()
}

/// No-op. See comments for function [`st_delete_safe`].
///
/// # Header declaration
///
/// ```c
/// st_index_t st_keys_check(st_table *table, st_data_t *keys, st_index_t size, st_data_t never);
/// ```
///
/// # Safety
///
/// `table` must be non-null and point to a valid `st_table` created by the
/// `st_init_table` family of functions.
///
/// `keys` must be non-null and point to an array.
#[inline]
pub unsafe fn st_keys_check(
    table: *mut st_table,
    keys: *mut st_data_t,
    size: st_index_t,
    _never: st_data_t,
) -> st_index_t {
    st_keys(table, keys, size)
}

/// Set up array `values` by at most `size` values of head table `table`
/// entries. Return the number of values set up in array `values`.
///
/// # Header declaration
///
/// ```c
/// st_index_t st_values(st_table *table, st_data_t *values, st_index_t size);
/// ```
///
/// # Safety
///
/// `table` must be non-null and point to a valid `st_table` created by the
/// `st_init_table` family of functions.
///
/// `values` must be non-null and point to an array.
#[inline]
pub unsafe fn st_values(
    table: *mut st_table,
    values: *mut st_data_t,
    size: st_index_t,
) -> st_index_t {
    let mut table = st_table::from_raw(table);
    let inner = table.as_inner_mut();

    let keys = slice::from_raw_parts_mut(values, size.into());
    let mut count = 0;
    for (counter, (slot, &value)) in keys.iter_mut().zip((*inner).inner.values()).enumerate() {
        ptr::write(slot, value);
        count = counter;
    }
    count.into()
}

/// No-op. See comments for function [`st_delete_safe`].
///
/// # Header declaration
///
/// ```c
/// st_index_t st_values_check(st_table *table, st_data_t *values, st_index_t size, st_data_t never);
/// ```
///
/// # Safety
///
/// `table` must be non-null and point to a valid `st_table` created by the
/// `st_init_table` family of functions.
///
/// `values` must be non-null and point to an array.
#[inline]
pub unsafe fn st_values_check(
    table: *mut st_table,
    values: *mut st_data_t,
    size: st_index_t,
    _never: st_data_t,
) -> st_index_t {
    st_values(table, values, size)
}

/// Insert (`key`, `value`) into table `table`. The table should not have entry
/// with `key` before the insertion.
///
/// # Notes
///
/// This implementation delegates to
/// [`StHashMap::insert`](crate::StHashMap::insert) directly.
///
/// # Header declaration
///
/// ```c
/// void st_add_direct(st_table *, st_data_t, st_data_t);
/// ```
///
/// # Safety
///
/// `table` must be non-null and point to a valid `st_table` created by the
/// `st_init_table` family of functions.
#[inline]
pub unsafe fn st_add_direct(table: *mut st_table, key: st_data_t, value: st_data_t) {
    // The original C implementation uses `st_add_direct_with_hash` to implement
    // this function.
    //
    // ```c
    // st_hash_t hash_value;
    // hash_value = do_hash(key, tab);
    // st_add_direct_with_hash(tab, key, value, hash_value);
    // ```
    //
    // Unlike `st_update`, there is no semantic difference here because there
    // are no callbacks.
    let mut table = st_table::from_raw(table);
    let inner = table.as_inner_mut();

    let _ = (*inner).insert_raw(key, value);
}

/// Free table `table` space.
///
/// # Header declaration
///
/// ```c
/// void st_free_table(st_table *);
/// ```
///
/// # Safety
///
/// `table` must be non-null and point to a valid `st_table` created by the
/// `st_init_table` family of functions.
#[inline]
pub unsafe fn st_free_table(table: *mut st_table) {
    let table = st_table::from_raw(table);
    let table = table.take();
    drop(table);
}

/// No-op. See comments for function [`st_delete_safe`].
///
/// # Header declaration
///
/// ```c
/// void st_cleanup_safe(st_table *, st_data_t);
/// ```
#[inline]
pub fn st_cleanup_safe(table: *mut st_table, _never: st_data_t) {
    let _ = table;
}

/// Make table `table` empty.
///
/// # Header declaration
///
/// ```c
/// void st_clear(st_table *);
/// ```
///
/// # Safety
///
/// `table` must be non-null and point to a valid `st_table` created by the
/// `st_init_table` family of functions.
#[inline]
pub unsafe fn st_clear(table: *mut st_table) {
    let mut table = st_table::from_raw(table);
    let inner = table.as_inner_mut();

    (*inner).inner.clear();
}

/// Create and return a copy of table `old_table`.
///
/// # Header declaration
///
/// ```c
/// st_table *st_copy(st_table *);
/// ```
///
/// # Safety
///
/// `table` must be non-null and point to a valid `st_table` created by the
/// `st_init_table` family of functions.
#[inline]
pub unsafe fn st_copy(old_table: *mut st_table) -> *mut st_table {
    let old_table = st_table::from_raw(old_table);
    let table = old_table.clone();
    drop(old_table);
    let table = table.take();
    st_table::boxed_into_raw(table)
}

/// Return byte size of memory allocted for table `table`.
///
/// # Notes
///
/// This implementation is a best effort approximation because Rust collection
/// types do not expose their memsize. See
/// [`StHashMap::estimated_memsize`](crate::StHashMap::estimated_memsize).
///
/// # Header declaration
///
/// ```c
/// PUREFUNC(size_t st_memsize(const st_table *));
/// ```
///
/// # Safety
///
/// `table` must be non-null and point to a valid `st_table` created by the
/// `st_init_table` family of functions.
#[inline]
#[must_use]
pub unsafe fn st_memsize(table: *const st_table) -> libc::size_t {
    let mut table = st_table::from_raw(table as *mut st_table);
    let inner = table.as_inner_mut();

    (*inner).inner.estimated_memsize()
}

/// Hash a byte array with FNV.
///
/// # Header declaration
///
/// ```c
/// PUREFUNC(st_index_t st_hash(const void *ptr, size_t len, st_index_t h));
/// ```
///
/// # Safety
///
/// `ptr` must be a non-null `*const u8` pointer.
#[inline]
#[must_use]
pub unsafe fn st_hash(ptr: *const c_void, len: libc::size_t, h: st_index_t) -> st_index_t {
    let mut hasher = FnvHasher::with_key(h.into());
    let data = slice::from_raw_parts(ptr.cast::<u8>(), len);
    hasher.write(data);
    hasher.finish().into()
}

/// Hash one round of FNV with `h` as the initial state.
///
/// # Header declaration
///
/// ```c
/// CONSTFUNC(st_index_t st_hash_uint32(st_index_t h, uint32_t i));
/// ```
#[inline]
#[must_use]
pub fn st_hash_uint32(h: st_index_t, i: u32) -> st_index_t {
    let mut hasher = FnvHasher::with_key(h.into());
    hasher.write_u32(i);
    hasher.finish().into()
}

/// Hash one round of FNV with `h` as the initial state.
///
/// # Header declaration
///
/// ```c
/// CONSTFUNC(st_index_t st_hash_uint(st_index_t h, st_index_t i));
/// ```
#[inline]
#[must_use]
pub fn st_hash_uint(h: st_index_t, i: st_index_t) -> st_index_t {
    let mut hasher = FnvHasher::with_key(h.into());
    hasher.write_usize(i.into());
    hasher.finish().into()
}

/// Finalize FNV hash.
///
/// # Header declaration
///
/// ```c
/// CONSTFUNC(st_index_t st_hash_end(st_index_t h));
/// ```
#[inline]
#[must_use]
pub const fn st_hash_end(h: st_index_t) -> st_index_t {
    h
}

/// Initialize an FNV hash with `h` data.
///
/// # Header declaration
///
/// ```c
/// CONSTFUNC(st_index_t st_hash_start(st_index_t h));
/// ```
#[inline]
#[must_use]
pub fn st_hash_start(h: st_index_t) -> st_index_t {
    let mut hasher = FnvHasher::default();
    hasher.write_usize(h.into());
    hasher.finish().into()
}
