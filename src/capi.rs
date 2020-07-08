#![allow(non_upper_case_globals)]

use core::ffi::c_void;

use crate::api;
use crate::typedefs::*;

#[cfg(feature = "capi-specialized-init")]
mod specialized_init;

// st_table *st_init_table(const struct st_hash_type *);
#[no_mangle]
unsafe extern "C" fn st_init_table(hash_type: *const st_hash_type) -> *mut st_table {
    #[cfg(feature = "debug")]
    dbg!();
    api::st_init_table(hash_type)
}

// st_table *st_init_table_with_size(const struct st_hash_type *, st_index_t);
#[no_mangle]
unsafe extern "C" fn st_init_table_with_size(
    hash_type: *const st_hash_type,
    size: st_index_t,
) -> *mut st_table {
    #[cfg(feature = "debug")]
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
    api::st_values_check(table, values, size, never)
}

// void st_add_direct(st_table *, st_data_t, st_data_t);
#[no_mangle]
unsafe extern "C" fn st_add_direct(table: *mut st_table, key: st_data_t, value: st_data_t) {
    #[cfg(feature = "debug")]
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
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
    dbg!();
    api::st_clear(table)
}

// st_table *st_copy(st_table *);
#[no_mangle]
unsafe extern "C" fn st_copy(table: *mut st_table) -> *mut st_table {
    #[cfg(feature = "debug")]
    dbg!();
    api::st_copy(table)
}

#[no_mangle]
unsafe extern "C" fn st_memsize(table: *const st_table) -> libc::size_t {
    #[cfg(feature = "debug")]
    dbg!();
    api::st_memsize(table)
}

// PUREFUNC(st_index_t st_hash(const void *ptr, size_t len, st_index_t h));
#[no_mangle]
unsafe extern "C" fn st_hash(ptr: *const c_void, len: libc::size_t, h: st_index_t) -> st_index_t {
    #[cfg(feature = "debug")]
    dbg!();
    api::st_hash(ptr, len, h)
}

// CONSTFUNC(st_index_t st_hash_uint32(st_index_t h, uint32_t i));
#[no_mangle]
unsafe extern "C" fn st_hash_uint32(h: st_index_t, i: u32) -> st_index_t {
    #[cfg(feature = "debug")]
    dbg!();
    api::st_hash_uint32(h, i)
}

// CONSTFUNC(st_index_t st_hash_uint(st_index_t h, st_index_t i));
#[no_mangle]
unsafe extern "C" fn st_hash_uint(h: st_index_t, i: st_index_t) -> st_index_t {
    #[cfg(feature = "debug")]
    dbg!();
    api::st_hash_uint(h, i)
}

// CONSTFUNC(st_index_t st_hash_end(st_index_t h));
#[no_mangle]
unsafe extern "C" fn st_hash_end(h: st_index_t) -> st_index_t {
    #[cfg(feature = "debug")]
    dbg!();
    api::st_hash_end(h)
}

// CONSTFUNC(st_index_t st_hash_start(st_index_t h));
#[no_mangle]
unsafe extern "C" fn st_hash_start(h: st_index_t) -> st_index_t {
    #[cfg(feature = "debug")]
    dbg!();
    api::st_hash_start(h)
}

// void rb_hash_bulk_insert_into_st_table(long, const VALUE *, VALUE);
