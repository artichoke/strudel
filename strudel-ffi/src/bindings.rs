use std::os::raw::c_int;

use crate::primitives::{st_data_t, st_index_t};

/// Equality comparator function for `StHash` keys.
///
/// # Header declaration
///
/// ```c
/// typedef int st_compare_func(st_data_t, st_data_t);
/// ```
pub type st_compare_func = unsafe extern "C" fn(st_data_t, st_data_t) -> c_int;

/// Hash function for `StHash` keys.
///
/// # Header declaration
///
/// ```c
/// typedef st_index_t st_hash_func(st_data_t);
/// ```
pub type st_hash_func = unsafe extern "C" fn(st_data_t) -> st_index_t;

/// Equality comparator and hash function used to build a [`StHashMap`] hasher.
///
/// These functions are `unsafe extern "C" fn` and expected to be supplied via
/// FFI.
///
/// # Safety
///
/// `st_hash_type` are expected to have `'static` lifetime. This assumption is
/// exploited by [`StHashMap`] and [`StBuildHasher`].
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

/// Return values from [`st_foreach_callback_func`] and
/// [`st_update_callback_func`] callback function pointers.
#[repr(C)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[allow(clippy::upper_case_acronyms)]
pub enum st_retval {
    /// Continue iteration.
    ST_CONTINUE,

    /// Stop iteration.
    ST_STOP,

    /// Delete current iteration `(key, value)` pair and continue iteration.
    ST_DELETE,

    /// Continue or stop iteration.
    ///
    /// This return value has slightly different behavior depending on API. See
    /// [`st_foreach`] and [`st_foreach_check`].
    ///
    /// [`st_foreach`]: crate::ffi::st_foreach
    /// [`st_foreach_check`]: crate::ffi::st_foreach_check
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

/// [`st_update`] callback function.
///
/// # Header declaration
///
/// ```c
/// typedef int st_update_callback_func(st_data_t *key, st_data_t *value, st_data_t arg, int existing);
/// ```
///
/// [`st_update`]: crate::ffi::st_update
pub type st_update_callback_func =
    unsafe extern "C" fn(*mut st_data_t, *mut st_data_t, st_data_t, c_int) -> c_int;

/// [`st_foreach`] and [`st_foreach_check`] callback function.
///
/// # Header declaration
///
/// ```c
/// int (*)(ANYARGS)
/// ```
///
/// [`st_foreach`]: crate::ffi::st_foreach
/// [`st_foreach_check`]: crate::ffi::st_foreach_check
pub type st_foreach_callback_func =
    unsafe extern "C" fn(st_data_t, st_data_t, st_data_t, i32) -> i32;
