use core::ops::{Deref, DerefMut};

#[cfg(target_pointer_width = "64")]
pub type st_data_t = u64;
#[cfg(target_pointer_width = "32")]
pub type st_data_t = u32;

pub type st_index_t = st_data_t;

pub type st_hash_t = st_index_t;

// typedef int st_compare_func(st_data_t, st_data_t);
pub type st_compare_func = unsafe extern "C" fn(st_data_t, st_data_t) -> i32;

// typedef st_index_t st_hash_func(st_data_t);
pub type st_hash_func = unsafe extern "C" fn(st_data_t) -> st_index_t;

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

#[repr(C)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum st_retval {
    ST_CONTINUE,
    ST_STOP,
    ST_DELETE,
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

// typedef int st_update_callback_func(st_data_t *key, st_data_t *value, st_data_t arg, int existing);
pub type st_update_callback_func =
    unsafe extern "C" fn(*mut st_data_t, *mut st_data_t, st_data_t, i32) -> i32;

// int (*)(ANYARGS)
pub type st_foreach_callback_func =
    unsafe extern "C" fn(st_data_t, st_data_t, st_data_t, i32) -> i32;

/// Opaque FFI wrapper around an `StHash`.
#[derive(Debug)]
pub struct st_table(crate::StHash);

impl st_table {
    #[inline]
    #[must_use]
    pub fn into_raw(table: Self) -> *mut Self {
        let table = Box::new(table);
        Box::into_raw(table)
    }

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
    pub unsafe fn from_raw(table: *mut Self) -> Box<Self> {
        Box::from_raw(table)
    }
}

impl From<crate::StHash> for st_table {
    #[inline]
    fn from(table: crate::StHash) -> Self {
        Self(table)
    }
}

impl Deref for st_table {
    type Target = crate::StHash;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for st_table {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
