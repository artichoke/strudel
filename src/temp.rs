
#[derive(Debug, Clone)]
pub(crate) struct LookupKey {
    record: st_data_t,
    eq: st_compare_func,
}

impl PartialEq for LookupKey {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        if self.record == other.record {
            return true;
        }
        let cmp = self.eq;
        // Safety
        //
        // `StHash` assumes `cmp` is a valid non-NULL function pointer.
        unsafe { (cmp)(self.record, other.record) == 0 }
    }
}

impl PartialEq<&LookupKey> for LookupKey {
    #[inline]
    fn eq(&self, other: &&Self) -> bool {
        self == *other
    }
}

impl PartialEq<Key> for LookupKey {
    #[inline]
    fn eq(&self, other: &Key) -> bool {
        *self == other.lookup
    }
}

impl Eq for LookupKey {}

impl Hash for LookupKey {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        #[cfg(target_pointer_width = "32")]
        state.write_u32(self.record);
        #[cfg(target_pointer_width = "64")]
        state.write_u64(self.record);
    }
}
