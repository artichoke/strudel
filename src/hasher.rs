use core::convert;
use core::hash::{BuildHasher, Hasher};
use core::mem::size_of;
use core::ops::BitXor;

use crate::{st_data_t, st_index_t};

#[cfg(target_pointer_width = "32")]
const K: st_hash_t = 0x9e3779b9;
#[cfg(target_pointer_width = "64")]
const K: st_hash_t = 0x517cc1b727220a95;

pub type st_hash_t = st_index_t;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct st_hash_type {
    // (*compare)(ANYARGS /*st_data_t, st_data_t*/); /* st_compare_func* */
    compare: fn(st_data_t, st_data_t) -> i32,
    // st_index_t (*hash)(ANYARGS /*st_data_t*/);        /* st_hash_func* */
    hash: fn(st_data_t) -> st_index_t,
}

fn default_compare(x: st_data_t, y: st_data_t) -> i32 {
    x.cmp(&y) as _
}

static default_hash_type: st_hash_type = st_hash_type {
    compare: default_compare,
    hash: convert::identity,
};

impl Default for st_hash_type {
    #[inline]
    fn default() -> Self {
        Self {
            compare: default_compare,
            hash: convert::identity,
        }
    }
}

pub struct StHasher {
    hash: st_hash_t,
    hash_type: *const st_hash_type,
}

impl Default for StHasher {
    fn default() -> Self {
        Self {
            hash: 0,
            hash_type: &default_hash_type as *const _,
        }
    }
}

impl From<*const st_hash_type> for StHasher {
    fn from(hash_type: *const st_hash_type) -> Self {
        let mut buf = [0_u8; size_of::<st_hash_t>()];
        let seed = if getrandom::getrandom(&mut buf).is_ok() {
            st_hash_t::from_ne_bytes(buf)
        } else {
            0
        };
        Self {
            hash: seed,
            hash_type,
        }
    }
}

impl BuildHasher for StHasher {
    type Hasher = Self;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Self::from(self.hash_type)
    }
}

impl Clone for StHasher {
    #[inline]
    fn clone(&self) -> Self {
        self.build_hasher()
    }
}

impl StHasher {
    #[inline]
    fn add_to_hash(&mut self, i: st_hash_t) {
        let hash = unsafe { (*self.hash_type).hash };
        let i = (hash)(i);
        self.hash = self.hash.rotate_left(5).bitxor(i).wrapping_mul(K);
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
        self.hash as u64
    }
}
