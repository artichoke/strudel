use core::ops::{Deref, DerefMut};

#[cfg(target_pointer_width = "64")]
pub type st_data_t = u64;
#[cfg(target_pointer_width = "32")]
pub type st_data_t = u32;

pub type st_index_t = st_data_t;

pub type st_hash_t = st_index_t;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct st_hash_type {
    /// `st_compare_func`
    ///
    /// # Header declaration
    ///
    /// ```c
    /// (*compare)(ANYARGS /*st_data_t, st_data_t*/); /* st_compare_func* */
    /// ```
    pub compare: unsafe extern "C" fn(st_data_t, st_data_t) -> i32,

    /// `st_hash_func`
    ///
    /// # Header declaration
    ///
    /// ```c
    /// st_index_t (*hash)(ANYARGS /*st_data_t*/);        /* st_hash_func* */
    /// ```
    pub hash: unsafe extern "C" fn(st_data_t) -> st_index_t,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum st_retval {
    ST_CONTINUE,
    ST_STOP,
    ST_DELETE,
    ST_CHECK,
}

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
