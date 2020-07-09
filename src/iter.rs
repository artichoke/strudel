use core::iter::{FromIterator, FusedIterator};
use std::collections::btree_map;
use std::vec;

use crate::typedefs::*;
use crate::StHash;

#[derive(Debug, Clone)]
pub struct Iter<'a>(pub(crate) btree_map::Values<'a, st_index_t, (st_data_t, st_data_t)>);

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a st_data_t, &'a st_data_t);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(key, value)| (key, value))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.0.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.0.last().map(|(key, value)| (key, value))
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n).map(|(key, value)| (key, value))
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B {
        self.0.map(|(key, value)| (key, value)).collect()
    }
}

impl<'a> FusedIterator for Iter<'a> {}

impl<'a> ExactSizeIterator for Iter<'a> {}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(key, value)| (key, value))
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n).map(|(key, value)| (key, value))
    }
}

#[derive(Debug)]
pub struct Keys<'a>(pub(crate) Iter<'a>);

impl<'a> Iterator for Keys<'a> {
    type Item = &'a st_data_t;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(key, _)| key)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.0.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.0.last().map(|(key, _)| key)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n).map(|(key, _)| key)
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B {
        self.0.map(|(key, _)| key).collect()
    }
}

impl<'a> FusedIterator for Keys<'a> {}

impl<'a> ExactSizeIterator for Keys<'a> {}

impl<'a> DoubleEndedIterator for Keys<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(key, _)| key)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n).map(|(key, _)| key)
    }
}

#[derive(Debug)]
pub struct Values<'a>(pub(crate) Iter<'a>);

impl<'a> Iterator for Values<'a> {
    type Item = &'a st_data_t;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, value)| value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.0.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.0.last().map(|(_, value)| value)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n).map(|(_, value)| value)
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B {
        self.0.map(|(_, value)| value).collect()
    }
}

impl<'a> FusedIterator for Values<'a> {}

impl<'a> ExactSizeIterator for Values<'a> {}

impl<'a> DoubleEndedIterator for Values<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(_, value)| value)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n).map(|(_, value)| value)
    }
}

#[derive(Debug)]
pub struct InsertRanks(pub(crate) vec::IntoIter<st_index_t>);

impl<'a> Iterator for InsertRanks {
    type Item = st_index_t;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.0.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.0.last()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n)
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B {
        self.0.collect()
    }
}

impl<'a> FusedIterator for InsertRanks {}

impl<'a> ExactSizeIterator for InsertRanks {}

impl<'a> DoubleEndedIterator for InsertRanks {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n)
    }
}

impl<'a> IntoIterator for &'a StHash {
    type Item = (&'a st_data_t, &'a st_data_t);
    type IntoIter = Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
