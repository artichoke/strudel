#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use core::ffi::c_void;
use core::hash::Hasher;
use core::mem::{self, size_of, MaybeUninit};
use core::ptr;
use core::slice;
use std::ffi::CStr;

use crate::fnv::{self, Fnv1a32};
use crate::{st_data_t, st_hash_t, st_hash_type, st_index_t, StHash};

#[repr(C)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum st_retval {
    ST_CONTINUE,
    ST_STOP,
    ST_DELETE,
    ST_CHECK,
}

impl PartialEq<libc::c_int> for st_retval {
    fn eq(&self, other: &libc::c_int) -> bool {
        *self as libc::c_int == *other
    }
}

impl PartialEq<st_retval> for libc::c_int {
    fn eq(&self, other: &st_retval) -> bool {
        *self == *other as libc::c_int
    }
}

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

pub struct st_table(StHash);

impl st_table {
    #[inline]
    pub fn into_raw(table: Self) -> *mut Self {
        let table = Box::new(table);
        Box::into_raw(table)
    }

    #[inline]
    pub fn boxed_into_raw(table: Box<Self>) -> *mut Self {
        Box::into_raw(table)
    }

    #[inline]
    pub unsafe fn from_raw(table: *mut Self) -> Box<Self> {
        Box::from_raw(table)
    }
}

impl From<StHash> for st_table {
    #[inline]
    fn from(table: StHash) -> Self {
        Self(table)
    }
}

// CONSTFUNC(int st_numcmp(st_data_t, st_data_t));
#[no_mangle]
pub unsafe extern "C" fn st_numcmp(x: st_data_t, y: st_data_t) -> libc::c_int {
    x.cmp(&y) as libc::c_int
}

// CONSTFUNC(st_index_t st_numhash(st_data_t));
#[no_mangle]
pub unsafe extern "C" fn st_numhash(n: st_data_t) -> st_index_t {
    let s1 = 11;
    let s2 = 3;
    let hash = ((n >> s1) | (n << s2)) ^ (n >> s2);
    hash as st_index_t
}

static st_hashtype_num: st_hash_type = st_hash_type {
    compare: st_numcmp,
    hash: st_numhash,
};

// CONSTFUNC(int st_numcmp(st_data_t, st_data_t));
unsafe extern "C" fn strcmp(x: st_data_t, y: st_data_t) -> libc::c_int {
    libc::strcmp(x as *const i8, y as *const i8)
}

unsafe extern "C" fn strhash(arg: st_data_t) -> st_index_t {
    let string = CStr::from_ptr(arg as *const libc::c_char);
    fnv::hash(string.to_bytes()) as st_index_t
}

static type_strhash: st_hash_type = st_hash_type {
    compare: strcmp,
    hash: strhash,
};

unsafe extern "C" fn strcasehash(arg: st_data_t) -> st_index_t {
    let string = CStr::from_ptr(arg as *const libc::c_char);
    let hval = fnv::hash(string.to_bytes());
    hval as st_index_t
}

static type_strcasehash: st_hash_type = st_hash_type {
    compare: st_locale_insensitive_strcasecmp,
    hash: strcasehash,
};

// st_table *st_init_table(const struct st_hash_type *);
#[no_mangle]
pub unsafe extern "C" fn st_init_table(hash_type: *const st_hash_type) -> *mut st_table {
    let table = StHash::with_hash_type(hash_type);
    st_table::into_raw(table.into())
}

// st_table *st_init_table_with_size(const struct st_hash_type *, st_index_t);
#[no_mangle]
pub unsafe extern "C" fn st_init_table_with_size(
    hash_type: *const st_hash_type,
    size: st_index_t,
) -> *mut st_table {
    let table = StHash::with_capacity_and_hash_type(size as usize, hash_type);
    st_table::into_raw(table.into())
}

// st_table *st_init_numtable(void);
#[no_mangle]
pub unsafe extern "C" fn st_init_numtable() -> *mut st_table {
    let table = StHash::with_hash_type(&st_hashtype_num as *const _);
    st_table::into_raw(table.into())
}

// st_table *st_init_numtable_with_size(st_index_t);
#[no_mangle]
pub unsafe extern "C" fn st_init_numtable_with_size(size: st_index_t) -> *mut st_table {
    let table = StHash::with_capacity_and_hash_type(size as usize, &st_hashtype_num as *const _);
    st_table::into_raw(table.into())
}

// st_table *st_init_strtable(void);
#[no_mangle]
pub unsafe extern "C" fn st_init_strtable() -> *mut st_table {
    let table = StHash::with_hash_type(&type_strhash as *const _);
    st_table::into_raw(table.into())
}

// st_table *st_init_strtable_with_size(st_index_t);
#[no_mangle]
pub unsafe extern "C" fn st_init_strtable_with_size(size: st_index_t) -> *mut st_table {
    let table = StHash::with_capacity_and_hash_type(size as usize, &type_strhash as *const _);
    st_table::into_raw(table.into())
}

// st_table *st_init_strcasetable(void);
#[no_mangle]
pub unsafe extern "C" fn st_init_strcasetable() -> *mut st_table {
    let table = StHash::with_hash_type(&type_strcasehash as *const _);
    st_table::into_raw(table.into())
}

// st_table *st_init_strcasetable_with_size(st_index_t);
#[no_mangle]
pub unsafe extern "C" fn st_init_strcasetable_with_size(size: st_index_t) -> *mut st_table {
    let table = StHash::with_capacity_and_hash_type(size as usize, &type_strcasehash as *const _);
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
#[no_mangle]
pub unsafe extern "C" fn st_delete(
    table: *mut st_table,
    key: *mut st_data_t,
    value: *mut st_data_t,
) -> libc::c_int {
    let mut table = st_table::from_raw(table);
    let ret = if let Some((entry_key, entry_value)) = table.0.delete(*key) {
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
#[no_mangle]
pub unsafe extern "C" fn st_delete_safe(
    table: *mut st_table,
    key: *mut st_data_t,
    value: *mut st_data_t,
    _never: *const st_data_t,
) -> libc::c_int {
    // This impl should be identical to `st_delete`.
    // ```c
    // st_delete(table, key, value)
    // ```
    //
    // The implementation is inlined below.
    let mut table = st_table::from_raw(table);
    let ret = if let Some((entry_key, entry_value)) = table.0.delete(*key) {
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
    mem::forget(table);
    ret
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
pub unsafe extern "C" fn st_shift(
    table: *mut st_table,
    key: *mut st_data_t,
    value: *mut st_data_t,
) -> libc::c_int {
    let mut table = st_table::from_raw(table);
    // The `StHash::keys` iterator returns keys in insertion order.
    if let Some(&first_key) = table.0.keys().next() {
        if let Some((entry_key, entry_value)) = table.0.delete(first_key) {
            ptr::write(key, entry_key);
            if !value.is_null() {
                ptr::write(value, entry_value);
            }
            return 1;
        }
    }
    if !value.is_null() {
        ptr::write(value, 0);
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
#[no_mangle]
pub unsafe extern "C" fn st_insert(
    table: *mut st_table,
    key: st_data_t,
    value: st_data_t,
) -> libc::c_int {
    let mut table = st_table::from_raw(table);
    let ret = table.0.insert(key, value).is_some() as libc::c_int;
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
#[no_mangle]
pub unsafe extern "C" fn st_insert2(
    table: *mut st_table,
    key: st_data_t,
    value: st_data_t,
    func: unsafe extern "C" fn(st_data_t) -> st_data_t,
) -> libc::c_int {
    let mut table = st_table::from_raw(table);
    let ret = if table.0.get(key).is_some() {
        let _ = table.0.insert(key, value);
        1
    } else {
        let _ = table.0.insert(func(key), value);
        0
    };
    mem::forget(table);
    ret
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
pub unsafe extern "C" fn st_update(
    table: *mut st_table,
    key: st_data_t,
    func: st_update_callback_func,
    arg: st_data_t,
) -> libc::c_int {
    use st_retval::*;

    let mut table = st_table::from_raw(table);
    let (existing, mut key, mut value) =
        if let Some((&entry_key, &entry_value)) = table.0.get_key_value(key) {
            (true, entry_key, entry_value)
        } else {
            (false, key, 0)
        };
    let old_key = key;
    let update = func(&mut key, &mut value, arg, 1);
    match update {
        ret if ret == ST_CONTINUE && !existing => {
            st_add_direct_with_hash(table, key, value, hash);
        }
        ret if ret == ST_CONTINUE => {
            if old_key == key {
                table.0.insert(key, value);
            } else {
                table.0.update(key, value);
            }
        }
        ret if ret == ST_DELETE && existing => {
            let _ = table.0.remove(old_key);
        }
        _ => {}
    };
    mem::forget(table);
    existing as libc::c_int
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

/// Free table `table` space.
///
/// # Header declaration
///
/// ```c
/// void st_free_table(st_table *);
/// ```
#[no_mangle]
pub unsafe extern "C" fn st_free_table(table: *mut st_table) {
    let table = Box::from_raw(table);
    mem::drop(table)
}

/// No-op. See comments for function [`st_delete_safe`].
///
/// # Header declaration
///
/// ```c
/// void st_cleanup_safe(st_table *, st_data_t);
/// ```
#[no_mangle]
pub unsafe extern "C" fn st_cleanup_safe(table: *mut st_table, _never: st_data_t) {
    let _ = table;
}

/// Make table `table` empty.
///
/// # Header declaration
///
/// ```c
/// void st_clear(st_table *);
/// ```
#[no_mangle]
pub unsafe extern "C" fn st_clear(table: *mut st_table) {
    let mut table = Box::from_raw(table);
    table.0.clear();
    mem::forget(table);
}

// st_table *st_copy(st_table *);
#[no_mangle]
pub unsafe extern "C" fn st_copy(table: *mut st_table) -> *mut st_table {
    let table = Box::from_raw(table);
    let copy = table.0.clone();
    mem::forget(table);
    let new_table = Box::new(copy.into());
    Box::into_raw(new_table)
}

// PUREFUNC(int st_locale_insensitive_strcasecmp(const char *s1, const char *s2));
#[no_mangle]
pub unsafe extern "C" fn st_locale_insensitive_strcasecmp(
    s1: st_data_t,
    s2: st_data_t,
) -> libc::c_int {
    let s1 = CStr::from_ptr(s1 as *const libc::c_char);
    let s2 = CStr::from_ptr(s2 as *const libc::c_char);
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
    s1: st_data_t,
    s2: st_data_t,
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
pub unsafe extern "C" fn st_strcasecmp(s1: st_data_t, s2: st_data_t) -> libc::c_int {
    st_locale_insensitive_strcasecmp(s1, s2)
}

// #define st_strncasecmp st_locale_insensitive_strncasecmp
#[no_mangle]
pub unsafe extern "C" fn st_strncasecmp(
    s1: st_data_t,
    s2: st_data_t,
    n: libc::size_t,
) -> libc::c_int {
    st_locale_insensitive_strncasecmp(s1, s2, n)
}

#[no_mangle]
pub unsafe extern "C" fn st_memsize(table: *const st_table) -> libc::size_t {
    let table = Box::from_raw(table as *mut st_table);
    let mut size = size_of::<st_table>();
    size += table.0.capacity() * size_of::<st_data_t>() * 2;
    mem::forget(table);
    size as _
}

// PUREFUNC(st_index_t st_hash(const void *ptr, size_t len, st_index_t h));
#[no_mangle]
pub unsafe extern "C" fn st_hash(
    ptr: *const c_void,
    len: libc::size_t,
    h: st_index_t,
) -> st_index_t {
    let mut hasher = Fnv1a32::with_seed(h as u32);
    let data = slice::from_raw_parts(ptr as *const u8, len as usize);
    hasher.write(data);
    hasher.finish() as st_index_t
}

// CONSTFUNC(st_index_t st_hash_uint32(st_index_t h, uint32_t i));
#[no_mangle]
pub unsafe extern "C" fn st_hash_uint32(h: st_index_t, i: u32) -> st_index_t {
    let mut hasher = Fnv1a32::with_seed(h as u32);
    hasher.write_u32(i);
    hasher.finish() as st_index_t
}

// CONSTFUNC(st_index_t st_hash_uint(st_index_t h, st_index_t i));
#[no_mangle]
pub unsafe extern "C" fn st_hash_uint(h: st_index_t, i: st_index_t) -> st_index_t {
    let mut hasher = Fnv1a32::with_seed(h as u32);
    hasher.write_u64(i as u64);
    hasher.finish() as st_index_t
}

// CONSTFUNC(st_index_t st_hash_end(st_index_t h));
#[no_mangle]
pub unsafe extern "C" fn st_hash_end(h: st_index_t) -> st_index_t {
    h
}

// CONSTFUNC(st_index_t st_hash_start(st_index_t h));
#[no_mangle]
pub unsafe extern "C" fn st_hash_start(h: st_index_t) -> st_index_t {
    let mut hasher = Fnv1a32::new();
    hasher.write_u64(h as u64);
    hasher.finish() as st_index_t
}

// void rb_hash_bulk_insert_into_st_table(long, const VALUE *, VALUE);
