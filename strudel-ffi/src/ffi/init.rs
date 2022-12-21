use core::hash::Hasher;
use core::mem::transmute;
use core::slice;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use fnv::FnvHasher;

use super::{st_init_table, st_init_table_with_size};
use crate::bindings::st_hash_type;
use crate::primitives::{st_data_t, st_index_t};
use crate::st_table::ffi::st_table;

/// # Header declaration
///
/// ```c
/// CONSTFUNC(int st_numcmp(st_data_t, st_data_t));
/// ```
#[no_mangle]
unsafe extern "C" fn st_numcmp(x: st_data_t, y: st_data_t) -> c_int {
    c_int::from(x != y)
}

/// # Header declaration
///
/// ```c
/// CONSTFUNC(st_index_t st_numhash(st_data_t));
/// ```
#[no_mangle]
unsafe extern "C" fn st_numhash(n: st_data_t) -> st_index_t {
    n.into()
}

static st_hashtype_num: st_hash_type = st_hash_type {
    compare: st_numcmp,
    hash: st_numhash,
};

/// # Declaration
///
/// ```c
/// static st_index_t strhash(st_data_t);
/// ```
unsafe extern "C" fn strhash(arg: st_data_t) -> st_index_t {
    let string = CStr::from_ptr(arg.as_const_c_char());
    let mut hasher = FnvHasher::default();
    hasher.write(string.to_bytes());
    hasher.finish().into()
}

static type_strhash: st_hash_type = st_hash_type {
    compare: unsafe {
        transmute(libc::strcmp as unsafe extern "C" fn(*const c_char, *const c_char) -> c_int)
    },
    hash: strhash,
};

unsafe extern "C" fn strcasehash(arg: st_data_t) -> st_index_t {
    let string = CStr::from_ptr(arg.as_const_c_char());
    let mut hasher = FnvHasher::default();
    for byte in string.to_bytes() {
        hasher.write_u8(byte.to_ascii_lowercase());
    }
    hasher.finish().into()
}

static type_strcasehash: st_hash_type = st_hash_type {
    compare: st_locale_insensitive_strcasecmp,
    hash: strcasehash,
};

/// # Header declaration
///
/// ```c
/// st_table *st_init_numtable(void);
/// ```
#[no_mangle]
unsafe extern "C" fn st_init_numtable() -> *mut st_table {
    st_init_table(&st_hashtype_num)
}

/// # Header declaration
///
/// ```c
/// st_table *st_init_numtable_with_size(st_index_t);
/// ```
#[no_mangle]
unsafe extern "C" fn st_init_numtable_with_size(size: st_index_t) -> *mut st_table {
    st_init_table_with_size(&st_hashtype_num, size)
}

/// # Header declaration
///
/// ```c
/// st_table *st_init_strtable(void);
/// ```
#[no_mangle]
unsafe extern "C" fn st_init_strtable() -> *mut st_table {
    st_init_table(&type_strhash)
}

/// # Header declaration
///
/// ```c
/// st_table *st_init_strtable_with_size(st_index_t);
/// ```
#[no_mangle]
unsafe extern "C" fn st_init_strtable_with_size(size: st_index_t) -> *mut st_table {
    st_init_table_with_size(&type_strhash, size)
}

/// # Header declaration
///
/// ```c
/// st_table *st_init_strcasetable(void);
/// ```
#[no_mangle]
unsafe extern "C" fn st_init_strcasetable() -> *mut st_table {
    st_init_table(&type_strcasehash)
}

/// # Header declaration
///
/// ```c
/// st_table *st_init_strcasetable_with_size(st_index_t);
/// ```
#[no_mangle]
unsafe extern "C" fn st_init_strcasetable_with_size(size: st_index_t) -> *mut st_table {
    st_init_table_with_size(&type_strcasehash, size)
}

/// # Header declaration
///
/// ```c
/// PUREFUNC(int st_locale_insensitive_strcasecmp(const char *s1, const char *s2));
/// ```
#[no_mangle]
unsafe extern "C" fn st_locale_insensitive_strcasecmp(s1: st_data_t, s2: st_data_t) -> libc::c_int {
    let s1 = CStr::from_ptr(s1.as_const_c_char());
    let s2 = CStr::from_ptr(s2.as_const_c_char());
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

/// # Header declaration
///
/// ```c
/// PUREFUNC(int st_locale_insensitive_strncasecmp(const char *s1, const char *s2, size_t n));
/// ```
#[no_mangle]
unsafe extern "C" fn st_locale_insensitive_strncasecmp(
    s1: st_data_t,
    s2: st_data_t,
    n: libc::size_t,
) -> libc::c_int {
    let s1 = slice::from_raw_parts(s1.as_const_c_char(), n);
    let s2 = slice::from_raw_parts(s2.as_const_c_char(), n);

    for (&left, &right) in s1.iter().zip(s2.iter()) {
        match (transmute(left), transmute(right)) {
            (b'\0', b'\0') => return 0,
            (_, b'\0') => return 1,
            (b'\0', _) => return -1,
            (c1, c2) => match (c1.to_ascii_lowercase(), c2.to_ascii_lowercase()) {
                (a, b) if a == b => {}
                (a, b) if a > b => return 1,
                _ => return -1,
            },
        }
    }
    0
}
