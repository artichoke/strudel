use core::iter::{FromIterator, FusedIterator, IntoIterator};

use crate::st::map;
use crate::st::set::StHashSet;

/// This struct is created by the [`iter`](StHashSet::iter) method on
/// [`StHashSet`]. See its documentation for more.
#[derive(Debug, Clone)]
pub struct Iter<'a, T>(pub(crate) map::Keys<'a, T, ()>);

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

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

impl<'a, T> FusedIterator for Iter<'a, T> {}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n)
    }
}

/// This struct is created by the `into_iter` method on [`StHashSet`]. See its
/// documentation for more.
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct IntoIter<T>(map::IntoIter<T, ()>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(elem, _)| elem)
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
        self.0.last().map(|(elem, _)| elem)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n).map(|(elem, _)| elem)
    }

    #[inline]
    fn collect<B: FromIterator<Self::Item>>(self) -> B {
        self.0.map(|(elem, _)| elem).collect()
    }
}

impl<T> FusedIterator for IntoIter<T> {}

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(elem, _)| elem)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n).map(|(elem, _)| elem)
    }
}

/// This struct is created by the [`insert_ranks_from`] method on [`StHashSet`].
/// See its documentation for more.
///
/// [`insert_ranks_from`]: StHashSet::insert_ranks_from
#[derive(Debug)]
pub struct InsertRanks(pub(crate) map::InsertRanks);

impl<'a> Iterator for InsertRanks {
    type Item = usize;

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

impl<T, S> IntoIterator for StHashSet<T, S> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.map.into_iter())
    }
}

impl<'a, T, S> IntoIterator for &'a StHashSet<T, S> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
