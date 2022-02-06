use core::mem::size_of;

/// A type-safe typedef for data stored in the hashmap.
///
/// `st.h` defines `st_data_t` to be a type alias with the same size as `void*`.
///
/// # Examples
///
/// ```
/// # use core::mem;
/// # use strudel::api::st_data_t;
/// assert_eq!(mem::size_of::<st_data_t>(), mem::size_of::<usize>());
/// let data = st_data_t::from(usize::MAX);
/// assert_eq!(data, usize::MAX);
/// ```
///
/// # Declaration
///
/// ```c
/// #if SIZEOF_LONG == SIZEOF_VOIDP
/// typedef unsigned long st_data_t;
/// #elif SIZEOF_LONG_LONG == SIZEOF_VOIDP
/// typedef unsigned LONG_LONG st_data_t;
/// #else
/// # error ---->> st.c requires sizeof(void*) == sizeof(long) or sizeof(LONG_LONG) to be compiled. <<----
/// #endif
/// #define ST_DATA_T_DEFINED
/// ```
#[repr(transparent)]
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct st_data_t {
    inner: usize,
}

impl st_data_t {
    /// Convert an opaque `st_data_t` to a C char pointer (C String).
    ///
    /// # Safety
    ///
    /// This `st_data_t` must point to a C String allocation.
    #[inline]
    #[must_use]
    pub unsafe fn as_const_c_char(&self) -> *const libc::c_char {
        self.inner as *const _
    }
}

impl From<usize> for st_data_t {
    #[inline]
    fn from(data: usize) -> Self {
        Self { inner: data }
    }
}

impl From<st_data_t> for usize {
    #[inline]
    fn from(data: st_data_t) -> Self {
        data.inner
    }
}

impl PartialEq<usize> for st_data_t {
    fn eq(&self, other: &usize) -> bool {
        self.inner == *other
    }
}

impl PartialEq<st_data_t> for usize {
    fn eq(&self, other: &st_data_t) -> bool {
        *self == other.inner
    }
}

/// A type-safe typedef for indexes in the hashmap.
///
/// # Declaration
///
/// ```c
/// typedef st_data_t st_index_t;
/// ```
#[repr(transparent)]
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct st_index_t {
    inner: st_data_t,
}

// ```c
// typedef char st_check_for_sizeof_st_index_t[SIZEOF_VOIDP == (int)sizeof(st_index_t) ? 1 : -1];
// ```
const _: () = [()][(size_of::<usize>() == size_of::<st_index_t>()) as usize - 1];

impl From<st_data_t> for st_index_t {
    #[inline]
    fn from(data: st_data_t) -> Self {
        Self { inner: data }
    }
}

impl From<u64> for st_index_t {
    #[inline]
    fn from(hash: u64) -> Self {
        let hash = hash as usize;
        Self { inner: hash.into() }
    }
}

impl From<usize> for st_index_t {
    #[inline]
    fn from(index: usize) -> Self {
        Self {
            inner: index.into(),
        }
    }
}

impl From<st_index_t> for u64 {
    fn from(index: st_index_t) -> Self {
        index.inner.inner as u64
    }
}

impl From<st_index_t> for usize {
    #[inline]
    fn from(index: st_index_t) -> Self {
        index.inner.into()
    }
}

/// A type-safe typedef for hash values in the hashmap.
///
/// # Declaration
///
/// ```c
/// /* The type of hashes.  */
/// typedef st_index_t st_hash_t;
/// ```
#[repr(transparent)]
#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct st_hash_t {
    inner: st_index_t,
}

impl st_hash_t {
    /// Create a native endian integer value from its memory representation as a
    /// byte array in native endianness.
    ///
    /// **Note**: This function takes an array of length 2, 4 or 8 bytes
    /// depending on the target pointer size.
    #[inline]
    #[must_use]
    pub fn from_ne_bytes(bytes: [u8; size_of::<st_hash_t>()]) -> Self {
        usize::from_ne_bytes(bytes).into()
    }
}

impl From<st_index_t> for st_hash_t {
    #[inline]
    fn from(index: st_index_t) -> Self {
        Self { inner: index }
    }
}

impl From<u64> for st_hash_t {
    #[inline]
    fn from(hash: u64) -> Self {
        let hash = hash as usize;
        Self { inner: hash.into() }
    }
}

impl From<usize> for st_hash_t {
    #[inline]
    fn from(hash: usize) -> Self {
        Self { inner: hash.into() }
    }
}

impl From<st_hash_t> for usize {
    #[inline]
    fn from(hash: st_hash_t) -> Self {
        hash.inner.into()
    }
}
