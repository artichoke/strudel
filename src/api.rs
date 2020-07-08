#![allow(non_upper_case_globals)]

use core::ffi::c_void;
use core::hash::Hasher;
use core::mem;
use core::ptr;
use core::slice;

use crate::fnv::Fnv1a32;
use crate::typedefs::*;
use crate::StHash;

// st_table *st_init_table(const struct st_hash_type *);
#[inline]
pub unsafe fn st_init_table(hash_type: *const st_hash_type) -> *mut st_table {
    let table = StHash::with_hash_type(hash_type);
    st_table::into_raw(table.into())
}

// st_table *st_init_table_with_size(const struct st_hash_type *, st_index_t);
#[inline]
pub unsafe fn st_init_table_with_size(
    hash_type: *const st_hash_type,
    size: st_index_t,
) -> *mut st_table {
    let table = StHash::with_capacity_and_hash_type(size as usize, hash_type);
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
#[inline]
pub unsafe fn st_delete(
    table: *mut st_table,
    key: *mut st_data_t,
    value: *mut st_data_t,
) -> libc::c_int {
    let mut table = st_table::from_raw(table);
    let ret = if let Some((entry_key, entry_value)) = table.delete(*key) {
        ptr::write(key, entry_key);
        if !value.is_null() {
            ptr::write(value, entry_value);
        }
        1
    } else {
        if !value.is_null() {
            ptr::write(value, 0);
        }
        0
    };
    #[cfg(feature = "capi")]
    table.ensure_num_entries_is_consistent_after_writes();
    mem::forget(table);
    ret
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
#[inline]
pub unsafe fn st_shift(
    table: *mut st_table,
    key: *mut st_data_t,
    value: *mut st_data_t,
) -> libc::c_int {
    let mut table = st_table::from_raw(table);
    if let Some((&first_key, _)) = table.first() {
        if let Some((entry_key, entry_value)) = table.delete(first_key) {
            ptr::write(key, entry_key);
            if !value.is_null() {
                ptr::write(value, entry_value);
            }
            #[cfg(feature = "capi")]
            table.ensure_num_entries_is_consistent_after_writes();
            mem::forget(table);
            return 1;
        }
    }
    if !value.is_null() {
        ptr::write(value, 0);
    }
    mem::forget(table);
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
#[inline]
pub unsafe fn st_insert(table: *mut st_table, key: st_data_t, value: st_data_t) -> libc::c_int {
    let mut table = st_table::from_raw(table);
    let ret = if table.insert(key, value).is_some() {
        1
    } else {
        #[cfg(feature = "capi")]
        table.ensure_num_entries_is_consistent_after_writes();
        0
    };
    mem::forget(table);
    ret
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
#[inline]
pub unsafe fn st_insert2(
    table: *mut st_table,
    key: st_data_t,
    value: st_data_t,
    func: unsafe extern "C" fn(st_data_t) -> st_data_t,
) -> libc::c_int {
    let mut table = st_table::from_raw(table);
    if table.get(key).is_some() {
        if table.insert(key, value).is_none() {
            #[cfg(feature = "capi")]
            table.ensure_num_entries_is_consistent_after_writes();
        }
        mem::forget(table);
        1
    } else {
        let table = st_table::boxed_into_raw(table);
        // `func` might mutate this table, so make sure we don't
        // alias the `Box`.
        let key = func(key);
        let mut table = st_table::from_raw(table);
        if table.insert(key, value).is_none() {
            #[cfg(feature = "capi")]
            table.ensure_num_entries_is_consistent_after_writes();
        }
        mem::forget(table);
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
#[inline]
pub unsafe fn st_lookup(
    table: *mut st_table,
    key: st_data_t,
    value: *mut st_data_t,
) -> libc::c_int {
    let table = st_table::from_raw(table);
    let ret = if let Some(&entry_value) = table.get(key) {
        if !value.is_null() {
            ptr::write(value, entry_value);
        }
        1
    } else {
        0
    };
    mem::forget(table);
    ret
}

/// Find an entry with `key` in table `table`. Return non-zero if we found it.
/// Set up `*RESULT` to the found table entry key.
///
/// # Header declaration
///
/// ```c
/// int st_get_key(st_table *, st_data_t, st_data_t *);
/// ```
#[inline]
pub unsafe fn st_get_key(
    table: *mut st_table,
    key: st_data_t,
    result: *mut st_data_t,
) -> libc::c_int {
    let table = st_table::from_raw(table);
    let ret = if let Some((&entry_key, _)) = table.get_key_value(key) {
        if !result.is_null() {
            ptr::write(result, entry_key);
        }
        1
    } else {
        0
    };
    mem::forget(table);
    ret
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
#[inline]
pub unsafe fn st_update(
    table: *mut st_table,
    key: st_data_t,
    func: st_update_callback_func,
    arg: st_data_t,
) -> libc::c_int {
    use st_retval::*;

    let table = st_table::from_raw(table);
    let (existing, mut key, mut value) =
        if let Some((&entry_key, &entry_value)) = table.get_key_value(key) {
            (true, entry_key, entry_value)
        } else {
            (false, key, 0)
        };
    let old_key = key;
    let table = st_table::boxed_into_raw(table);
    let update = func(&mut key, &mut value, arg, existing as libc::c_int);
    let mut table = st_table::from_raw(table);
    match update {
        ret if ret == ST_CONTINUE && !existing => {
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
            if table.insert(key, value).is_none() {
                #[cfg(feature = "capi")]
                table.ensure_num_entries_is_consistent_after_writes();
            }
        }
        ret if ret == ST_CONTINUE => {
            table.update(key, value);
        }
        ret if ret == ST_DELETE && existing => {
            let _ = table.remove(old_key);
            #[cfg(feature = "capi")]
            table.ensure_num_entries_is_consistent_after_writes();
        }
        _ => {}
    };
    mem::forget(table);
    existing as libc::c_int
}

// int st_foreach(st_table *, int (*)(ANYARGS), st_data_t);
#[inline]
pub unsafe fn st_foreach(
    table: *mut st_table,
    func: st_foreach_callback_func,
    arg: st_data_t,
) -> libc::c_int {
    use st_retval::*;

    let table_ptr = table;
    let table = st_table::from_raw(table_ptr);
    let mut insertion_ranks = table.insert_ranks_from(0).peekable();
    let mut last_seen_rank = 0;
    mem::forget(table);

    loop {
        let table = st_table::from_raw(table_ptr);

        // skip any ranks that have been removed from the table.
        let min = table.min_insert_rank();
        if last_seen_rank < min {
            insertion_ranks = table.insert_ranks_from(min).peekable();
        }
        mem::forget(table);

        if let Some(rank) = insertion_ranks.next() {
            let table = st_table::from_raw(table_ptr);
            last_seen_rank = rank;
            let nth = table.get_nth(rank).map(|(&key, &value)| (key, value));
            mem::forget(table);

            if let Some((key, value)) = nth {
                let retval = func(key, value, arg, 0);
                match retval {
                    retval if ST_CONTINUE == retval => {}
                    retval if ST_CHECK == retval || ST_STOP == retval => return 0,
                    retval if ST_DELETE == retval => {
                        let mut table = st_table::from_raw(table_ptr);
                        let _ = table.remove(key);
                        #[cfg(feature = "capi")]
                        table.ensure_num_entries_is_consistent_after_writes();
                        mem::forget(table);
                    }
                    _ => {}
                }
            }
        } else {
            let table = st_table::from_raw(table_ptr);
            let current_max = table.max_insert_rank();
            if current_max <= last_seen_rank {
                mem::forget(table);
                break;
            }
            insertion_ranks = table.insert_ranks_from(last_seen_rank).peekable();
            mem::forget(table);
        }
    }
    0
}

// int st_foreach_check(st_table *, int (*)(ANYARGS), st_data_t, st_data_t);
#[inline]
pub unsafe fn st_foreach_check(
    table: *mut st_table,
    func: st_foreach_callback_func,
    arg: st_data_t,
    _never: st_data_t,
) -> libc::c_int {
    use st_retval::*;

    let table_ptr = table;
    let table = st_table::from_raw(table_ptr);
    let mut insertion_ranks = table.insert_ranks_from(0).peekable();
    let mut last_seen_rank = 0;
    mem::forget(table);

    loop {
        let table = st_table::from_raw(table_ptr);

        // skip any ranks that have been removed from the table.
        let min = table.min_insert_rank();
        if last_seen_rank < min {
            insertion_ranks = table.insert_ranks_from(min).peekable();
        }
        mem::forget(table);

        if let Some(rank) = insertion_ranks.next() {
            let table = st_table::from_raw(table_ptr);
            last_seen_rank = rank;
            let nth = table.get_nth(rank).map(|(&key, &value)| (key, value));
            mem::forget(table);

            if let Some((key, value)) = nth {
                let retval = func(key, value, arg, 0);
                match retval {
                    retval if ST_CONTINUE == retval || ST_CHECK == retval => {}
                    retval if ST_STOP == retval => return 0,
                    retval if ST_DELETE == retval => {
                        let mut table = st_table::from_raw(table_ptr);
                        let _ = table.remove(key);
                        #[cfg(feature = "capi")]
                        table.ensure_num_entries_is_consistent_after_writes();
                        mem::forget(table);
                    }
                    _ => {}
                }
            }
        } else {
            let table = st_table::from_raw(table_ptr);
            let current_max = table.max_insert_rank();
            if current_max <= last_seen_rank {
                mem::forget(table);
                break;
            }
            insertion_ranks = table.insert_ranks_from(last_seen_rank).peekable();
            mem::forget(table);
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
#[inline]
pub unsafe fn st_keys(table: *mut st_table, keys: *mut st_data_t, size: st_index_t) -> st_index_t {
    let table = st_table::from_raw(table);
    let keys = slice::from_raw_parts_mut(keys, size as usize);
    let mut count = 0;
    for (counter, (slot, &key)) in keys.iter_mut().zip(table.keys()).enumerate() {
        ptr::write(slot, key);
        count = counter;
    }
    mem::forget(table);
    count as st_index_t
}

/// No-op. See comments for function [`st_delete_safe`].
///
/// # Header declaration
///
/// ```c
/// st_index_t st_keys_check(st_table *table, st_data_t *keys, st_index_t size, st_data_t never);
/// ```
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
#[inline]
pub unsafe fn st_values(
    table: *mut st_table,
    values: *mut st_data_t,
    size: st_index_t,
) -> st_index_t {
    let table = st_table::from_raw(table);
    let keys = slice::from_raw_parts_mut(values, size as usize);
    let mut count = 0;
    for (counter, (slot, &value)) in keys.iter_mut().zip(table.values()).enumerate() {
        ptr::write(slot, value);
        count = counter;
    }
    mem::forget(table);
    count as st_index_t
}

/// No-op. See comments for function [`st_delete_safe`].
///
/// # Header declaration
///
/// ```c
/// st_index_t st_values_check(st_table *table, st_data_t *values, st_index_t size, st_data_t never);
/// ```
#[inline]
pub unsafe fn st_values_check(
    table: *mut st_table,
    values: *mut st_data_t,
    size: st_index_t,
    _never: st_data_t,
) -> st_index_t {
    st_values(table, values, size)
}

// void st_add_direct(st_table *, st_data_t, st_data_t);
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
    if table.insert(key, value).is_none() {
        #[cfg(feature = "capi")]
        table.ensure_num_entries_is_consistent_after_writes();
    }
    mem::forget(table);
}

/// Free table `table` space.
///
/// # Header declaration
///
/// ```c
/// void st_free_table(st_table *);
/// ```
#[inline]
pub unsafe fn st_free_table(table: *mut st_table) {
    let table = st_table::from_raw(table);
    drop(table)
}

/// No-op. See comments for function [`st_delete_safe`].
///
/// # Header declaration
///
/// ```c
/// void st_cleanup_safe(st_table *, st_data_t);
/// ```
#[inline]
pub unsafe fn st_cleanup_safe(table: *mut st_table, _never: st_data_t) {
    let _ = table;
}

/// Make table `table` empty.
///
/// # Header declaration
///
/// ```c
/// void st_clear(st_table *);
/// ```
#[inline]
pub unsafe fn st_clear(table: *mut st_table) {
    let mut table = st_table::from_raw(table);
    table.clear();
    #[cfg(feature = "capi")]
    table.ensure_num_entries_is_consistent_after_writes();
    mem::forget(table);
}

// st_table *st_copy(st_table *);
#[inline]
pub unsafe fn st_copy(table: *mut st_table) -> *mut st_table {
    let table = st_table::from_raw(table);
    let copy = table.clone();
    mem::forget(table);
    st_table::boxed_into_raw(copy.into())
}

#[inline]
pub unsafe fn st_memsize(table: *const st_table) -> libc::size_t {
    let table = st_table::from_raw(table as *mut st_table);
    let memsize = table.estimated_memsize();
    mem::forget(table);
    memsize as _
}

// PUREFUNC(st_index_t st_hash(const void *ptr, size_t len, st_index_t h));
#[inline]
pub unsafe fn st_hash(ptr: *const c_void, len: libc::size_t, h: st_index_t) -> st_index_t {
    let mut hasher = Fnv1a32::with_seed(h as u32);
    let data = slice::from_raw_parts(ptr as *const u8, len as usize);
    hasher.write(data);
    hasher.finish() as st_index_t
}

// CONSTFUNC(st_index_t st_hash_uint32(st_index_t h, uint32_t i));
#[inline]
pub unsafe fn st_hash_uint32(h: st_index_t, i: u32) -> st_index_t {
    let mut hasher = Fnv1a32::with_seed(h as u32);
    hasher.write_u32(i);
    hasher.finish() as st_index_t
}

// CONSTFUNC(st_index_t st_hash_uint(st_index_t h, st_index_t i));
#[inline]
pub unsafe fn st_hash_uint(h: st_index_t, i: st_index_t) -> st_index_t {
    let mut hasher = Fnv1a32::with_seed(h as u32);
    hasher.write_u64(i as u64);
    hasher.finish() as st_index_t
}

// CONSTFUNC(st_index_t st_hash_end(st_index_t h));
#[inline]
pub unsafe fn st_hash_end(h: st_index_t) -> st_index_t {
    h
}

// CONSTFUNC(st_index_t st_hash_start(st_index_t h));
#[inline]
pub unsafe fn st_hash_start(h: st_index_t) -> st_index_t {
    let mut hasher = Fnv1a32::new();
    hasher.write_u64(h as u64);
    hasher.finish() as st_index_t
}
