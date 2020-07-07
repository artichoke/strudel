use core::convert;
use core::hash::{BuildHasher, Hasher};
use std::collections::hash_map::{DefaultHasher, RandomState};

use crate::{st_data_t, st_index_t};

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
pub struct StBuildHasher {
    inner: RandomState,
    hash: unsafe extern "C" fn(st_data_t) -> st_index_t,
}

impl StBuildHasher {
    pub fn into_boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

impl Default for StBuildHasher {
    fn default() -> Self {
        Self {
            inner: RandomState::default(),
            hash: default_hash,
        }
    }
}

impl From<*const st_hash_type> for StBuildHasher {
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
pub struct StHasher {
    state: DefaultHasher,
    hash: unsafe extern "C" fn(st_data_t) -> st_index_t,
}

impl Default for StHasher {
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
    #[cfg(target_pointer_width = "32")]
    fn write(&mut self, bytes: &[u8]) {
        let mut iter = bytes.chunks_exact(4);
        while let Some(&[a, b, c, d]) = iter.next() {
            let i = st_hash_t::from_ne_bytes([a, b, c, d]);
            self.add_to_hash(i);
        }
        match iter.remainder() {
            &[a] => {
                let i = st_hash_t::from_ne_bytes([a, 0, 0, 0]);
                self.add_to_hash(i);
            }
            &[a, b] => {
                let i = st_hash_t::from_ne_bytes([a, b, 0, 0]);
                self.add_to_hash(i);
            }
            &[a, b, c] => {
                let i = st_hash_t::from_ne_bytes([a, b, c, 0]);
                self.add_to_hash(i);
            }
            _ => {}
        }
    }

    #[inline]
    #[cfg(target_pointer_width = "64")]
    fn write(&mut self, bytes: &[u8]) {
        let mut iter = bytes.chunks_exact(8);
        while let Some(&[a, b, c, d, e, f, g, h]) = iter.next() {
            let i = st_hash_t::from_ne_bytes([a, b, c, d, e, f, g, h]);
            self.add_to_hash(i);
        }
        match iter.remainder() {
            &[a] => {
                let i = st_hash_t::from_ne_bytes([a, 0, 0, 0, 0, 0, 0, 0]);
                self.add_to_hash(i);
            }
            &[a, b] => {
                let i = st_hash_t::from_ne_bytes([a, b, 0, 0, 0, 0, 0, 0]);
                self.add_to_hash(i);
            }
            &[a, b, c] => {
                let i = st_hash_t::from_ne_bytes([a, b, c, 0, 0, 0, 0, 0]);
                self.add_to_hash(i);
            }
            &[a, b, c, d] => {
                let i = st_hash_t::from_ne_bytes([a, b, c, d, 0, 0, 0, 0]);
                self.add_to_hash(i);
            }
            &[a, b, c, d, e] => {
                let i = st_hash_t::from_ne_bytes([a, b, c, d, e, 0, 0, 0]);
                self.add_to_hash(i);
            }
            &[a, b, c, d, e, f] => {
                let i = st_hash_t::from_ne_bytes([a, b, c, d, e, f, 0, 0]);
                self.add_to_hash(i);
            }
            &[a, b, c, d, e, f, g] => {
                let i = st_hash_t::from_ne_bytes([a, b, c, d, e, f, g, 0]);
                self.add_to_hash(i);
            }
            _ => {}
        }
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

    #[cfg(target_pointer_width = "32")]
    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.add_to_hash(i as st_hash_t);
        self.add_to_hash((i >> 32) as st_hash_t);
    }

    #[cfg(target_pointer_width = "64")]
    #[inline]
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
