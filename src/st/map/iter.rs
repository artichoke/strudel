use core::iter::{FromIterator, FusedIterator};
use core::ops::Range;
use core::slice;
use std::vec;

use crate::st::map::{InsertionEntry, StHashMap};

/// This struct is created by the [`iter`](StHashMap::iter) method on
/// [`StHashMap`]. See its documentation for more.
#[derive(Debug, Clone)]
pub struct Iter<'a, K, V>(pub(crate) slice::Iter<'a, InsertionEntry<K, V>>);

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next() {
                None => return None,
                Some(InsertionEntry::Dead) => continue,
                Some(InsertionEntry::Alive(key, value)) => return Some((key, value)),
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, K, V> FusedIterator for Iter<'a, K, V> {}

impl<'a, K, V> DoubleEndedIterator for Iter<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next_back() {
                None => return None,
                Some(InsertionEntry::Dead) => continue,
                Some(InsertionEntry::Alive(key, value)) => return Some((key, value)),
            }
        }
    }
}

/// This struct is created by the `into_iter` method on [`StHashMap`]. See its
/// documentation for more.
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct IntoIter<K, V>(vec::IntoIter<InsertionEntry<K, V>>);

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next() {
                None => return None,
                Some(InsertionEntry::Dead) => continue,
                Some(InsertionEntry::Alive(key, value)) => return Some((key, value)),
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<K, V> FusedIterator for IntoIter<K, V> {}

impl<K, V> DoubleEndedIterator for IntoIter<K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next_back() {
                None => return None,
                Some(InsertionEntry::Dead) => continue,
                Some(InsertionEntry::Alive(key, value)) => return Some((key, value)),
            }
        }
    }
}

/// This struct is created by the [`keys`](StHashMap::keys) method on
/// [`StHashMap`]. See its documentation for more.
#[derive(Debug, Clone)]
pub struct Keys<'a, K, V>(pub(crate) Iter<'a, K, V>);

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

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

impl<'a, K, V> FusedIterator for Keys<'a, K, V> {}

impl<'a, K, V> DoubleEndedIterator for Keys<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(key, _)| key)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n).map(|(key, _)| key)
    }
}

/// This struct is created by the [`values`](StHashMap::values) method on
/// [`StHashMap`]. See its documentation for more.
#[derive(Debug, Clone)]
pub struct Values<'a, K, V>(pub(crate) Iter<'a, K, V>);

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

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

impl<'a, K, V> FusedIterator for Values<'a, K, V> {}

impl<'a, K, V> DoubleEndedIterator for Values<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|(_, value)| value)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n).map(|(_, value)| value)
    }
}

/// This struct is created by the [`insert_ranks_from`] method on [`StHashMap`].
/// See its documentation for more.
///
/// [`insert_ranks_from`]: StHashMap::insert_ranks_from
#[derive(Debug, Clone)]
pub struct InsertRanks(pub(crate) Range<usize>);

impl Iterator for InsertRanks {
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

impl FusedIterator for InsertRanks {}

impl ExactSizeIterator for InsertRanks {}

impl DoubleEndedIterator for InsertRanks {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n)
    }
}

impl<K, V, S> IntoIterator for StHashMap<K, V, S> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.ordered.into_iter())
    }
}

impl<'a, K: 'a, V: 'a, S> IntoIterator for &'a StHashMap<K, V, S> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
