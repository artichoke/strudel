//! FFI helpers.

use core::mem::ManuallyDrop;

use super::ffi::st_table;
use super::StTable;

pub trait Repack {
    unsafe fn repack(&mut self);
}

/// Wrapper around a boxed type that is owned by a foreign caller.
///
/// This struct will repack metadata on drop but will not free the underlying
/// table.
#[derive(Debug, Clone)]
pub struct Foreign<T>
where
    T: Repack,
{
    inner: ManuallyDrop<Box<T>>,
}

impl<T> Foreign<T>
where
    T: Repack,
{
    /// Construct a new foreign wrapper for a previously allocated table.
    ///
    /// # Safety
    ///
    /// The given pointer must be non-null and a active allocation derived from
    /// [`Box::into_raw`].
    pub unsafe fn new_from_raw(table: *mut T) -> Self {
        let table = Box::from_raw(table);
        let inner = ManuallyDrop::new(table);
        Self { inner }
    }

    /// Return the inner owned `st_table`.
    ///
    /// # Safety
    ///
    /// Callers must ensure the table is not owned by foreign code so it is not
    /// prematurely dropped.
    #[must_use]
    pub unsafe fn take(mut self) -> Box<T> {
        ManuallyDrop::take(&mut self.inner)
    }
}

impl Foreign<st_table> {
    /// Retrieve a mutable, potentially aliased pointer to the inner `StHashMap`.
    ///
    /// This pointer is guaranteed to not be modified as long as `st_free_table`
    /// is not called.
    ///
    /// # Safety
    ///
    /// Callers must not create mutable references from the returned pointer as
    /// it is not guaranteed to be unaliased.
    pub unsafe fn as_inner_mut(&mut self) -> *mut StTable {
        self.inner.as_mut().table
    }
}

impl<T> Drop for Foreign<T>
where
    T: Repack,
{
    fn drop(&mut self) {
        unsafe {
            self.inner.repack();
        }
    }
}
