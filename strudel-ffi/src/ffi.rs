#![allow(non_upper_case_globals)]

//! `st_hash`-compatible C API bindings for [`StHashMap`].
//!
//! These bindings require activating the **capi** Cargo feature.
//!
//! This module's functions are exported via `#[no_mangle]` symbol bindings.
//! These functions are callable from C by including `st.h` and linking in
//! `libstrudel`.
//!
//! [`StHashMap`]: crate::StHashMap

use core::ffi::c_void;
use std::os::raw::c_int;

use crate::bindings::{st_foreach_callback_func, st_hash_type, st_update_callback_func};
use crate::primitives::{st_data_t, st_hash_t, st_index_t};
use crate::st_table::ffi::st_table;

mod imp;
mod init;

/// # Header declaration
///
/// ```c
/// st_table *st_init_table(const struct st_hash_type *);
/// ```
#[no_mangle]
unsafe extern "C" fn st_init_table(hash_type: *const st_hash_type) -> *mut st_table {
    imp::st_init_table(hash_type)
}

/// # Header declaration
///
/// ```c
/// st_table *st_init_table_with_size(const struct st_hash_type *, st_index_t);
/// ```
#[no_mangle]
unsafe extern "C" fn st_init_table_with_size(
    hash_type: *const st_hash_type,
    size: st_index_t,
) -> *mut st_table {
    imp::st_init_table_with_size(hash_type, size)
}

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
) -> c_int {
    imp::st_delete(table, key, value)
}

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
) -> c_int {
    imp::st_delete_safe(table, key, value, never)
}

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
) -> c_int {
    imp::st_shift(table, key, value)
}

/// # Header declaration
///
/// ```c
/// int st_insert(st_table *, st_data_t, st_data_t);
/// ```
#[no_mangle]
unsafe extern "C" fn st_insert(table: *mut st_table, key: st_data_t, value: st_data_t) -> c_int {
    imp::st_insert(table, key, value)
}

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
) -> c_int {
    imp::st_insert2(table, key, value, func)
}

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
) -> c_int {
    imp::st_lookup(table, key, value)
}

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
) -> c_int {
    imp::st_get_key(table, key, result)
}

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
) -> c_int {
    imp::st_update(table, key, func, arg)
}

/// # Header declaration
///
/// ```c
/// int st_foreach(st_table *, int (*)(ANYARGS), st_data_t);
/// ```
#[no_mangle]
unsafe extern "C" fn st_foreach(
    table: *mut st_table,
    func: st_foreach_callback_func,
    arg: st_data_t,
) -> c_int {
    imp::st_foreach(table, func, arg)
}

/// # Header declaration
///
/// ```c
/// int st_foreach_check(st_table *, int (*)(ANYARGS), st_data_t, st_data_t);
/// ```
#[no_mangle]
unsafe extern "C" fn st_foreach_check(
    table: *mut st_table,
    func: st_foreach_callback_func,
    arg: st_data_t,
    never: st_data_t,
) -> c_int {
    imp::st_foreach_check(table, func, arg, never)
}

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
    imp::st_keys(table, keys, size)
}

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
    imp::st_keys_check(table, keys, size, never)
}

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
    imp::st_values(table, values, size)
}

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
    imp::st_values_check(table, values, size, never)
}

/// # Header declaration
///
/// ```c
/// void st_add_direct(st_table *, st_data_t, st_data_t);
/// ```
#[no_mangle]
unsafe extern "C" fn st_add_direct(table: *mut st_table, key: st_data_t, value: st_data_t) {
    imp::st_add_direct(table, key, value);
}

/// # Header declaration
///
/// ```c
/// void st_add_direct_with_hash(st_table *tab, st_data_t key, st_data_t value, st_hash_t hash)
/// ```
#[no_mangle]
unsafe extern "C" fn st_add_direct_with_hash(
    table: *mut st_table,
    key: st_data_t,
    value: st_data_t,
    hash: st_hash_t,
) {
    let _ = hash;
    imp::st_add_direct(table, key, value);
}

/// # Header declaration
///
/// ```c
/// void st_free_table(st_table *);
/// ```
#[no_mangle]
unsafe extern "C" fn st_free_table(table: *mut st_table) {
    imp::st_free_table(table);
}

/// # Header declaration
///
/// ```c
/// void st_cleanup_safe(st_table *, st_data_t);
/// ```
#[no_mangle]
unsafe extern "C" fn st_cleanup_safe(table: *mut st_table, never: st_data_t) {
    imp::st_cleanup_safe(table, never);
}

/// # Header declaration
///
/// ```c
/// void st_clear(st_table *);
/// ```
#[no_mangle]
unsafe extern "C" fn st_clear(table: *mut st_table) {
    imp::st_clear(table);
}

/// # Header declaration
///
/// ```c
/// st_table *st_copy(st_table *);
/// ```
#[no_mangle]
unsafe extern "C" fn st_copy(table: *mut st_table) -> *mut st_table {
    imp::st_copy(table)
}

/// # Header declaration
///
/// ```c
/// PUREFUNC(size_t st_memsize(const st_table *));
/// ```
#[no_mangle]
unsafe extern "C" fn st_memsize(table: *const st_table) -> libc::size_t {
    imp::st_memsize(table)
}

/// # Header declaration
///
/// ```c
/// PUREFUNC(st_index_t st_hash(const void *ptr, size_t len, st_index_t h));
/// ```
#[no_mangle]
unsafe extern "C" fn st_hash(ptr: *const c_void, len: libc::size_t, h: st_index_t) -> st_index_t {
    imp::st_hash(ptr, len, h)
}

/// # Header declaration
///
/// ```c
/// CONSTFUNC(st_index_t st_hash_uint32(st_index_t h, uint32_t i));
/// ```
#[no_mangle]
unsafe extern "C" fn st_hash_uint32(h: st_index_t, i: u32) -> st_index_t {
    imp::st_hash_uint32(h, i)
}

/// # Header declaration
///
/// ```c
/// CONSTFUNC(st_index_t st_hash_uint(st_index_t h, st_index_t i));
/// ```
#[no_mangle]
unsafe extern "C" fn st_hash_uint(h: st_index_t, i: st_index_t) -> st_index_t {
    imp::st_hash_uint(h, i)
}

/// # Header declaration
///
/// ```c
/// CONSTFUNC(st_index_t st_hash_end(st_index_t h));
/// ```
#[no_mangle]
unsafe extern "C" fn st_hash_end(h: st_index_t) -> st_index_t {
    imp::st_hash_end(h)
}

/// # Header declaration
///
/// ```c
/// CONSTFUNC(st_index_t st_hash_start(st_index_t h));
/// ```
#[no_mangle]
unsafe extern "C" fn st_hash_start(h: st_index_t) -> st_index_t {
    imp::st_hash_start(h)
}

// void rb_hash_bulk_insert_into_st_table(long, const VALUE *, VALUE);
