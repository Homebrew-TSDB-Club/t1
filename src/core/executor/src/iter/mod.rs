pub mod and_then;
pub mod enumerate;
pub mod filter;
pub mod fold;
pub mod map;
mod zip;

use self::{
    and_then::AndThen, enumerate::Enumerate, filter::Filter, fold::Fold, map::Map, zip::Zip,
};

#[derive(Debug)]
pub enum Step<R, D = ()> {
    NotYet,
    Ready(R),
    Done(D),
}

pub trait Iterator<'iter>: Sized {
    type Item: 'iter;
    type Return;

    fn next(&mut self) -> Step<Self::Item, Self::Return>;

    #[inline]
    fn filter<P>(self, predicate: P) -> Filter<Self, P>
    where
        P: FnMut(&Self::Item) -> bool,
    {
        Filter::new(self, predicate)
    }

    #[inline]
    fn map<P, B>(self, f: P) -> Map<Self, P>
    where
        P: FnMut(Self::Item) -> B,
    {
        Map::new(self, f)
    }

    #[inline]
    fn enumerate(self) -> Enumerate<Self> {
        Enumerate::new(self)
    }

    #[inline]
    fn fold<B, F>(self, accum: B, f: F) -> Fold<Self, F, B>
    where
        F: FnMut(&mut B, Self::Item),
    {
        Fold::new(self, accum, f)
    }

    #[inline]
    fn and_then<B, F>(self, f: F) -> AndThen<Self, F, B>
    where
        F: FnMut(Self::Return) -> B,
    {
        AndThen::new(self, f)
    }

    #[inline]
    fn zip<'rhs, I>(self, rhs: I) -> Zip<'iter, 'rhs, Self, I>
    where
        I: Iterator<'rhs>,
    {
        Zip::new(self, rhs)
    }

    #[inline]
    fn eq<I>(mut self, mut another: I) -> bool
    where
        I: Iterator<'iter, Item = Self::Item>,
        Self::Item: PartialEq,
    {
        loop {
            match self.next() {
                Step::NotYet => continue,
                Step::Ready(lhs) => loop {
                    match another.next() {
                        Step::NotYet => continue,
                        Step::Ready(rhs) => {
                            if lhs != rhs {
                                return false;
                            }
                        }
                        Step::Done(_) => return false,
                    }
                },
                Step::Done(_) => {
                    if let Step::Done(_) = another.next() {
                        return true;
                    } else {
                        return false;
                    }
                }
            }
        }
    }
}

impl<'iter, T> Iterator<'iter> for &mut T
where
    T: Iterator<'iter>,
{
    type Item = T::Item;
    type Return = T::Return;

    fn next(&mut self) -> Step<Self::Item, Self::Return> {
        (*self).next()
    }
}

pub trait IteratorFusion<'iter> {
    type Iterator: Iterator<'iter>;

    fn fusion(self) -> Self::Iterator;
}

impl<'iter, I: 'iter + std::iter::Iterator> IteratorFusion<'iter> for I {
    type Iterator = StdIter<I>;

    #[inline]
    fn fusion(self) -> Self::Iterator {
        StdIter::from(self)
    }
}

pub struct StdIter<I> {
    iter: I,
}

impl<I: std::iter::Iterator> From<I> for StdIter<I> {
    #[inline]
    fn from(value: I) -> Self {
        Self { iter: value }
    }
}

impl<'iter, I: 'iter + std::iter::Iterator> Iterator<'iter> for StdIter<I> {
    type Item = I::Item;
    type Return = ();

    #[inline]
    fn next(&mut self) -> Step<Self::Item, Self::Return> {
        match self.iter.next() {
            Some(item) => Step::Ready(item),
            None => Step::Done(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Iterator, StdIter};
    use crate::iter::Step;

    #[test]
    fn from_iter() {
        let v = vec![0, 1, 2, 3];
        let mut stream = StdIter::from(v.iter())
            .filter(|item| *item % 2 == 0)
            .map(|item| *item + 1);
        let mut round = 0;
        loop {
            round += 1;
            match stream.next() {
                Step::Done(_) => break,
                Step::Ready(i) => {
                    let _ = i;
                }
                _ => {}
            }
        }
        assert_eq!(round, 5);
    }
}
