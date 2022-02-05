/// A type-safe typedef for data stored in the hashmap.
///
/// `st.h` defines `st_data_t` to be a type alias with the same size as `void*`.
///
/// # Examples
///
/// ```
/// # use core::mem;
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

impl From<st_data_t> for st_index_t {
    #[inline]
    fn from(data: st_data_t) -> Self {
        Self { inner: data }
    }
}

impl From<usize> for st_index_t {
    #[inline]
    fn from(data: usize) -> Self {
        Self { inner: data.into() }
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
