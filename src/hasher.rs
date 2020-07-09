use core::hash::{BuildHasher, Hasher};
use core::mem::size_of;
use std::collections::hash_map::{DefaultHasher, RandomState};

use crate::typedefs::*;

#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct StBuildHasher {
    inner: RandomState,
    hash_type: *const st_hash_type,
}

impl StBuildHasher {
    #[inline]
    #[must_use]
    pub fn into_boxed(self) -> Box<Self> {
        Box::new(self)
    }

    #[inline]
    #[must_use]
    pub fn hash_type(&self) -> *const st_hash_type {
        self.hash_type
    }
}

impl From<*const st_hash_type> for StBuildHasher {
    #[inline]
    fn from(hash_type: *const st_hash_type) -> Self {
        Self {
            inner: RandomState::new(),
            hash_type,
        }
    }
}

impl BuildHasher for StBuildHasher {
    type Hasher = StHasher;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Self::Hasher {
            state: self.inner.build_hasher(),
            hash_type: self.hash_type,
        }
    }
}

impl BuildHasher for Box<StBuildHasher> {
    type Hasher = StHasher;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Self::Hasher {
            state: self.inner.build_hasher(),
            hash_type: self.hash_type,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct StHasher {
    state: DefaultHasher,
    hash_type: *const st_hash_type,
}

impl StHasher {
    #[inline]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn add_to_hash(&mut self, i: st_hash_t) {
        let i = unsafe {
            let hash = (*self.hash_type).hash;
            (hash)(i)
        };
        #[cfg(target_pointer_width = "32")]
        self.state.write_u32(i);
        #[cfg(target_pointer_width = "64")]
        self.state.write_u64(i);
    }

    #[inline]
    #[must_use]
    pub fn hash_type(&self) -> *const st_hash_type {
        self.hash_type
    }
}

impl Hasher for StHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let mut iter = bytes.chunks_exact(size_of::<st_hash_t>());
        let mut buf = [0_u8; size_of::<st_hash_t>()];
        while let Some(chunk) = iter.next() {
            buf.copy_from_slice(chunk);
            let i = st_hash_t::from_ne_bytes(buf);
            self.add_to_hash(i);
        }
        buf = [0_u8; size_of::<st_hash_t>()];
        buf[..iter.remainder().len()].copy_from_slice(iter.remainder());
        let i = st_hash_t::from_ne_bytes(buf);
        self.add_to_hash(i);
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.add_to_hash(i as st_hash_t);
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.add_to_hash(i as st_hash_t);
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.add_to_hash(i as st_hash_t);
    }

    #[inline]
    #[cfg(target_pointer_width = "32")]
    fn write_u64(&mut self, i: u64) {
        self.add_to_hash(i as st_hash_t);
        self.add_to_hash((i >> 32) as st_hash_t);
    }

    #[inline]
    #[cfg(target_pointer_width = "64")]
    fn write_u64(&mut self, i: u64) {
        self.add_to_hash(i as st_hash_t);
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.add_to_hash(i as st_hash_t);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.state.finish()
    }
}
