#![allow(non_camel_case_types)]

use core::ffi::c_void;
use core::hash::Hasher;
use core::mem::size_of;
use core::slice;
use std::ffi::CStr;

use crate::{st_data_t, st_hash_t, st_hash_type, st_index_t, StHash, StHasher};

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
struct st_table_entry {
    hash: st_hash_t,
    key: st_data_t,
    record: st_data_t,
}

#[repr(C)]
struct __st_table {
    /* Cached features of the table -- see st.c for more details.  */
    // unsigned char entry_power, bin_power, size_ind;
    entry_power: libc::c_uchar,
    bin_power: libc::c_uchar,
    size_ind: libc::c_uchar,
    /* How many times the table was rebuilt.  */
    // unsigned int rebuilds_num;
    rebuilds_num: libc::c_uint,
    // const struct st_hash_type *type;
    type_: *const st_hash_type,
    /* Number of entries currently in the table.  */
    // st_index_t num_entries;
    num_entries: st_index_t,
    /* Array of bins used for access by keys.  */
    // st_index_t *bins;
    bins: *mut st_index_t,
    /* Start and bound index of entries in array entries.
    entries_starts and entries_bound are in interval
    [0,allocated_entries].  */
    // st_index_t entries_start, entries_bound;
    entries_start: st_index_t,
    entries_bound: st_index_t,
    /* Array of size 2^entry_power.  */
    // st_table_entry *entries;
    entries: *mut st_table_entry,
}

// ensure that `StdHash` fits in `st_table` for an opaque FFI container.
const _: () = [()][!(size_of::<__st_table>() >= size_of::<StHash>()) as usize];
const ST_TABLE_PADDING_LEN: usize = size_of::<__st_table>() - size_of::<*mut StHash>();

#[repr(C)]
pub struct st_table {
    table: StHash,
    padding: [u8; ST_TABLE_PADDING_LEN],
}

impl From<StHash> for st_table {
    fn from(table: StHash) -> Self {
        Self {
            table,
            padding: [0; ST_TABLE_PADDING_LEN],
        }
    }
}

impl From<Box<StHash>> for st_table {
    fn from(table: Box<StHash>) -> Self {
        let table = *table;
        Self {
            table,
            padding: [0; ST_TABLE_PADDING_LEN],
        }
    }
}

// st_table *st_init_table(const struct st_hash_type *);
#[no_mangle]
pub unsafe extern "C" fn st_init_table(hash_type: *const st_hash_type) -> *mut st_table {
    let map = StHash::with_hash_type(hash_type);
    let table = todo!();
}

// st_table *st_init_table_with_size(const struct st_hash_type *, st_index_t);
#[no_mangle]
pub unsafe extern "C" fn st_init_table_with_size(
    hash_type: *const st_hash_type,
    size: st_index_t,
) -> *mut st_table {
    todo!();
}

// st_table *st_init_numtable(void);
#[no_mangle]
pub unsafe extern "C" fn st_init_numtable() -> *mut st_table {
    todo!();
}

// st_table *st_init_numtable_with_size(st_index_t);
#[no_mangle]
pub unsafe extern "C" fn st_init_numtable_with_size(size: st_index_t) -> *mut st_table {
    todo!();
}

// st_table *st_init_strtable(void);
#[no_mangle]
pub unsafe extern "C" fn st_init_strtable() -> *mut st_table {
    todo!();
}

// st_table *st_init_strtable_with_size(st_index_t);
#[no_mangle]
pub unsafe extern "C" fn st_init_strtable_with_size(size: st_index_t) -> *mut st_table {
    todo!();
}

// st_table *st_init_strcasetable(void);
#[no_mangle]
pub unsafe extern "C" fn st_init_strcasetable() -> *mut st_table {
    todo!();
}

// st_table *st_init_strcasetable_with_size(st_index_t);
#[no_mangle]
pub unsafe extern "C" fn st_init_strcasetable_with_size(size: st_index_t) -> *mut st_table {
    todo!();
}

// int st_delete(st_table *, st_data_t *, st_data_t *); /* returns 0:notfound 1:deleted */
#[no_mangle]
pub unsafe extern "C" fn st_delete(
    table: *mut st_table,
    key: *mut st_data_t,
    value: *mut st_data_t,
) -> libc::c_int {
    todo!();
}

// int st_delete_safe(st_table *, st_data_t *, st_data_t *, st_data_t);
#[no_mangle]
pub unsafe extern "C" fn st_delete_safe(
    table: *mut st_table,
    key: *mut st_data_t,
    value: *mut st_data_t,
    _never: *const st_data_t,
) -> libc::c_int {
    todo!();
}

// int st_shift(st_table *, st_data_t *, st_data_t *); /* returns 0:notfound 1:deleted */
#[no_mangle]
pub unsafe extern "C" fn st_shift(
    table: *mut st_table,
    key: *mut st_data_t,
    value: *mut st_data_t,
) -> libc::c_int {
    todo!();
}

// int st_insert(st_table *, st_data_t, st_data_t);
#[no_mangle]
pub unsafe extern "C" fn st_insert(
    table: *mut st_table,
    key: *mut st_data_t,
    value: *mut st_data_t,
) -> libc::c_int {
    todo!();
}

// int st_insert2(st_table *, st_data_t, st_data_t, st_data_t (*)(st_data_t));
#[no_mangle]
pub unsafe extern "C" fn st_insert2(
    table: *mut st_table,
    key: st_data_t,
    value: st_data_t,
    func: fn(st_data_t) -> st_data_t,
) -> libc::c_int {
    todo!();
}

// int st_lookup(st_table *, st_data_t, st_data_t *);
#[no_mangle]
pub unsafe extern "C" fn st_lookup(
    table: *mut st_table,
    key: st_data_t,
    value: *mut st_data_t,
) -> libc::c_int {
    todo!();
}

// int st_get_key(st_table *, st_data_t, st_data_t *);
#[no_mangle]
pub unsafe extern "C" fn st_get_key(
    table: *mut st_table,
    key: st_data_t,
    result: *mut st_data_t,
) -> libc::c_int {
    todo!();
}

// typedef int st_update_callback_func(st_data_t *key, st_data_t *value, st_data_t arg, int existing);
pub type st_update_callback_func =
    fn(*mut st_data_t, *mut st_data_t, st_data_t, libc::c_int) -> libc::c_int;

// /* *key may be altered, but must equal to the old key, i.e., the
//  * results of hash() are same and compare() returns 0, otherwise the
//  * behavior is undefined */
// int st_update(st_table *table, st_data_t key, st_update_callback_func *func, st_data_t arg);
#[no_mangle]
pub unsafe extern "C" fn st_update(
    table: *mut st_table,
    key: st_data_t,
    func: st_update_callback_func,
    arg: st_data_t,
) -> libc::c_int {
    todo!();
}

// int (*)(ANYARGS)
pub type st_foreach_callback_func = fn(st_data_t, st_data_t, st_data_t, libc::c_int) -> libc::c_int;

// int st_foreach(st_table *, int (*)(ANYARGS), st_data_t);
pub unsafe extern "C" fn st_foreach(
    table: *mut st_table,
    func: st_foreach_callback_func,
    arg: st_data_t,
) -> libc::c_int {
    todo!();
}

// int st_foreach_check(st_table *, int (*)(ANYARGS), st_data_t, st_data_t);
pub unsafe extern "C" fn st_foreach_check(
    table: *mut st_table,
    func: st_foreach_callback_func,
    arg: st_data_t,
    _never: st_data_t,
) -> libc::c_int {
    todo!();
}

// st_index_t st_keys(st_table *table, st_data_t *keys, st_index_t size);
#[no_mangle]
pub unsafe extern "C" fn st_keys(
    table: *mut st_table,
    keys: *mut st_data_t,
    size: st_index_t,
) -> st_index_t {
    todo!();
}

// st_index_t st_keys_check(st_table *table, st_data_t *keys, st_index_t size, st_data_t never);
#[no_mangle]
pub unsafe extern "C" fn st_keys_check(
    table: *mut st_table,
    keys: *mut st_data_t,
    size: st_index_t,
    _never: st_data_t,
) -> st_index_t {
    todo!();
}

// st_index_t st_values(st_table *table, st_data_t *values, st_index_t size);
#[no_mangle]
pub unsafe extern "C" fn st_values(
    table: *mut st_table,
    values: *mut st_data_t,
    size: st_index_t,
) -> st_index_t {
    todo!();
}

// st_index_t st_values_check(st_table *table, st_data_t *values, st_index_t size, st_data_t never);
#[no_mangle]
pub unsafe extern "C" fn st_values_check(
    table: *mut st_table,
    values: *mut st_data_t,
    size: st_index_t,
    _never: st_data_t,
) -> st_index_t {
    todo!();
}

// void st_add_direct(st_table *, st_data_t, st_data_t);
#[no_mangle]
pub unsafe extern "C" fn st_add_direct(table: *mut st_table, key: st_data_t, value: st_data_t) {
    todo!();
}

// void st_free_table(st_table *);
#[no_mangle]
pub unsafe extern "C" fn st_free_table(table: *mut st_table) {
    todo!();
}

// void st_cleanup_safe(st_table *, st_data_t);
#[no_mangle]
pub unsafe extern "C" fn st_cleanup_safe(table: *mut st_table, _never: st_data_t) {
    let _ = table;
}

// void st_clear(st_table *);
#[no_mangle]
pub unsafe extern "C" fn st_clear(table: *mut st_table) {
    todo!();
}

// st_table *st_copy(st_table *);
#[no_mangle]
pub unsafe extern "C" fn st_copy(table: *mut st_table) -> *mut st_table {
    todo!();
}

// CONSTFUNC(int st_numcmp(st_data_t, st_data_t));
#[no_mangle]
pub unsafe extern "C" fn st_numcmp(x: st_data_t, y: st_data_t) -> libc::c_int {
    x.cmp(&y) as libc::c_int
}

// CONSTFUNC(st_index_t st_numhash(st_data_t));
#[no_mangle]
#[allow(trivial_casts)]
pub unsafe extern "C" fn st_numhash(n: st_data_t) -> st_index_t {
    let mut hasher = StHasher::default();
    hasher.write_u64(n as st_index_t);
    hasher.finish() as st_index_t
}

// PUREFUNC(int st_locale_insensitive_strcasecmp(const char *s1, const char *s2));
#[no_mangle]
pub unsafe extern "C" fn st_locale_insensitive_strcasecmp(
    s1: *const libc::c_char,
    s2: *const libc::c_char,
) -> libc::c_int {
    let s1 = CStr::from_ptr(s1);
    let s2 = CStr::from_ptr(s2);
    match (s1.to_bytes().len(), s2.to_bytes().len()) {
        (left, right) if left == right => {}
        (left, right) if left > right => return 1,
        _ => return -1,
    }

    for (&left, &right) in s1.to_bytes().iter().zip(s2.to_bytes().iter()) {
        // there are guaranteed to be no interior NULs in this loop
        let c1 = left.to_ascii_lowercase();
        let c2 = right.to_ascii_lowercase();
        match (c1, c2) {
            (a, b) if a == b => {}
            (a, b) if a > b => return 1,
            _ => return -1,
        }
    }
    0
}

// PUREFUNC(int st_locale_insensitive_strncasecmp(const char *s1, const char *s2, size_t n));
#[no_mangle]
pub unsafe extern "C" fn st_locale_insensitive_strncasecmp(
    s1: *const libc::c_char,
    s2: *const libc::c_char,
    n: libc::size_t,
) -> libc::c_int {
    let s1 = slice::from_raw_parts(s1 as *const u8, n as usize);
    let s2 = slice::from_raw_parts(s2 as *const u8, n as usize);

    for (&left, &right) in s1.iter().zip(s2.iter()) {
        match (left, right) {
            (b'\0', b'\0') => return 0,
            (_, b'\0') => return 1,
            (b'\0', _) => return -1,
            (mut c1, mut c2) => {
                c1 = c1.to_ascii_lowercase();
                c2 = c2.to_ascii_lowercase();
                match (c1, c2) {
                    (a, b) if a == b => {}
                    (a, b) if a > b => return 1,
                    _ => return -1,
                }
            }
        }
    }
    0
}

// #define st_strcasecmp st_locale_insensitive_strcasecmp
#[no_mangle]
pub unsafe extern "C" fn st_strcasecmp(
    s1: *const libc::c_char,
    s2: *const libc::c_char,
) -> libc::c_int {
    st_locale_insensitive_strcasecmp(s1, s2)
}

// #define st_strncasecmp st_locale_insensitive_strncasecmp
#[no_mangle]
pub unsafe extern "C" fn st_strncasecmp(
    s1: *const libc::c_char,
    s2: *const libc::c_char,
    n: libc::size_t,
) -> libc::c_int {
    st_locale_insensitive_strncasecmp(s1, s2, n)
}

// PUREFUNC(size_t st_memsize(const st_table *));
#[no_mangle]
pub unsafe extern "C" fn st_memsize(table: *const st_table) -> libc::size_t {
    todo!();
}

// PUREFUNC(st_index_t st_hash(const void *ptr, size_t len, st_index_t h));
#[no_mangle]
pub unsafe extern "C" fn st_hash(
    ptr: *const c_void,
    len: libc::size_t,
    h: st_index_t,
) -> st_index_t {
    todo!();
}

// CONSTFUNC(st_index_t st_hash_uint32(st_index_t h, uint32_t i));
#[no_mangle]
pub unsafe extern "C" fn st_hash_uint32(h: st_index_t, i: u32) -> st_index_t {
    todo!();
}

// CONSTFUNC(st_index_t st_hash_uint(st_index_t h, st_index_t i));
#[no_mangle]
pub unsafe extern "C" fn st_hash_uint(h: st_index_t, i: st_index_t) -> st_index_t {
    todo!();
}

// CONSTFUNC(st_index_t st_hash_end(st_index_t h));
#[no_mangle]
pub unsafe extern "C" fn st_hash_end(h: st_index_t) -> st_index_t {
    todo!();
}

// CONSTFUNC(st_index_t st_hash_start(st_index_t h));
#[no_mangle]
pub unsafe extern "C" fn st_hash_start(h: st_index_t) -> st_index_t {
    todo!();
}

// void rb_hash_bulk_insert_into_st_table(long, const VALUE *, VALUE);
