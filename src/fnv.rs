#![allow(dead_code)]

use core::fmt;
use core::hash::{BuildHasher, Hasher};

pub const FNV1_32A_INIT: u32 = 0x811c_9dc5;
pub const FNV_32_PRIME: u32 = 1677_7619;

#[derive(Default, Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[allow(clippy::module_name_repetitions)]
pub struct Fnv1a32BuildHasher;

impl BuildHasher for Fnv1a32BuildHasher {
    type Hasher = Fnv1a32;

    #[inline]
    #[must_use]
    fn build_hasher(&self) -> Self::Hasher {
        Self::Hasher::default()
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct Fnv1a32(u32);

impl Fnv1a32 {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    #[must_use]
    pub fn with_seed(seed: u32) -> Self {
        Self(seed)
    }
}

impl Default for Fnv1a32 {
    #[inline]
    #[must_use]
    fn default() -> Self {
        Self(FNV1_32A_INIT)
    }
}

impl fmt::Debug for Fnv1a32 {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fnv1a32 {{}}")
    }
}

impl Hasher for Fnv1a32 {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes.iter() {
            self.0 ^= byte as u32;
            self.0 = self.0.wrapping_mul(FNV_32_PRIME);
        }
    }

    #[inline]
    #[must_use]
    fn finish(&self) -> u64 {
        self.0 as u64
    }
}

#[inline]
#[must_use]
pub fn hash<T: AsRef<[u8]>>(data: T) -> u64 {
    let mut hasher = Fnv1a32::default();
    hasher.write(data.as_ref());
    hasher.finish()
}
