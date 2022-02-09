#![warn(unsafe_op_in_unsafe_fn)]

use core::hash::{BuildHasher, Hasher};
use core::mem::size_of;

use crate::api::primitives::st_data_t;
use crate::api::typedefs::st_hash_type;

/// `StBuildHasher` is the default state for `ExternStHashMap`s.
///
/// A particular instance of `StBuildHasher` will create the same instances of
/// [`Hasher`], but hashers created by two different `StBuildHasher` instances
/// are unlikely to produce the same result for the same values.
#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct StBuildHasher {
    hash_type: *const st_hash_type,
}

impl StBuildHasher {
    /// Return the underlying equality comparator and hash function used to
    /// construct this [`BuildHasher`].
    #[inline]
    #[must_use]
    pub fn hash_type(&self) -> *const st_hash_type {
        self.hash_type
    }
}

impl From<*const st_hash_type> for StBuildHasher {
    #[inline]
    fn from(hash_type: *const st_hash_type) -> Self {
        Self { hash_type }
    }
}

impl BuildHasher for StBuildHasher {
    type Hasher = StHasher;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Self::Hasher {
            hash_type: self.hash_type,
            state: 0,
        }
    }
}

impl BuildHasher for Box<StBuildHasher> {
    type Hasher = StHasher;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Self::Hasher {
            hash_type: self.hash_type,
            state: 0,
        }
    }
}

/// The default [`Hasher`] used by [`StBuildHasher`].
#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct StHasher {
    hash_type: *const st_hash_type,
    state: u64,
}

impl StHasher {
    #[inline]
    unsafe fn add_to_hash(&mut self, i: st_data_t) {
        // `StHasher` should only be called with one round.
        debug_assert!(self.state == 0);

        // Safety:
        //
        // `StHasher` assumes the `*const st_hash_type` pointer has `'static`
        // lifetime.
        // `StHasher` assumes that the `hash` function pointer is non-NULL.
        let hash_val = unsafe {
            let hash = (*self.hash_type).hash;
            (hash)(i)
        };
        self.state += u64::from(hash_val);
    }

    /// Return the underlying equality comparator and hash function used to
    /// construct this [`Hasher`].
    #[inline]
    #[must_use]
    pub fn hash_type(&self) -> *const st_hash_type {
        self.hash_type
    }
}

impl Hasher for StHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let mut iter = bytes.chunks_exact(size_of::<st_data_t>());
        let mut buf = [0_u8; size_of::<st_data_t>()];
        for chunk in &mut iter {
            buf.copy_from_slice(chunk);

            let i = st_data_t::from_ne_bytes(buf);
            unsafe {
                self.add_to_hash(i);
            }
        }

        let remainder = iter.remainder();
        if !remainder.is_empty() {
            buf = [0_u8; size_of::<st_data_t>()];
            buf[..remainder.len()].copy_from_slice(remainder);

            let i = st_data_t::from_ne_bytes(buf);
            unsafe {
                self.add_to_hash(i);
            }
        }
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        let i = i as usize;
        unsafe {
            self.add_to_hash(i.into());
        }
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        let i = i as usize;
        unsafe {
            self.add_to_hash(i.into());
        }
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        let i = i as usize;
        unsafe {
            self.add_to_hash(i.into());
        }
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        if cfg!(target_pointer_width = "32") {
            self.write(&i.to_ne_bytes());
        } else if cfg!(target_pointer_width = "64") {
            let i = i as usize;
            unsafe {
                self.add_to_hash(i.into());
            }
        }
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        unsafe {
            self.add_to_hash(i.into());
        }
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.state
    }
}
