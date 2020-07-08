use core::convert;
use core::hash::{BuildHasher, Hasher};
use core::mem::size_of;
use std::collections::hash_map::{DefaultHasher, RandomState};

use crate::typedefs::*;

pub unsafe extern "C" fn default_compare(x: st_data_t, y: st_data_t) -> i32 {
    x.cmp(&y) as _
}

pub unsafe extern "C" fn default_hash(value: st_data_t) -> st_index_t {
    convert::identity(value)
}

impl Default for st_hash_type {
    #[inline]
    fn default() -> Self {
        Self {
            compare: default_compare,
            hash: default_hash,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct StBuildHasher {
    inner: RandomState,
    hash: st_hash_func,
}

impl StBuildHasher {
    #[inline]
    #[must_use]
    pub fn into_boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

impl Default for StBuildHasher {
    #[inline]
    fn default() -> Self {
        Self {
            inner: RandomState::default(),
            hash: default_hash,
        }
    }
}

impl From<*const st_hash_type> for StBuildHasher {
    #[inline]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn from(hash_type: *const st_hash_type) -> Self {
        let hash = unsafe { (*hash_type).hash };
        Self {
            inner: RandomState::new(),
            hash,
        }
    }
}

impl BuildHasher for StBuildHasher {
    type Hasher = StHasher;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Self::Hasher {
            state: self.inner.build_hasher(),
            hash: self.hash,
        }
    }
}

impl BuildHasher for Box<StBuildHasher> {
    type Hasher = StHasher;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Self::Hasher {
            state: self.inner.build_hasher(),
            hash: self.hash,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct StHasher {
    state: DefaultHasher,
    hash: st_hash_func,
}

impl Default for StHasher {
    #[inline]
    fn default() -> Self {
        Self {
            state: DefaultHasher::default(),
            hash: default_hash,
        }
    }
}

impl StHasher {
    #[inline]
    fn add_to_hash(&mut self, i: st_hash_t) {
        let i = unsafe { (self.hash)(i) };
        #[cfg(target_pointer_width = "32")]
        self.state.write_u32(i);
        #[cfg(target_pointer_width = "64")]
        self.state.write_u64(i);
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
