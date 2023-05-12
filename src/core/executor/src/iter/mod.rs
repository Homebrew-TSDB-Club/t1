pub mod enumerate;
pub mod filter;
pub mod map;

use self::{enumerate::Enumerate, filter::Filter, map::Map};

#[derive(Debug)]
pub enum Step<T> {
    NotYet,
    Ready(T),
    Done,
}

pub trait Iterator: Sized {
    type Item;

    fn next(&mut self) -> Step<Self::Item>;

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
    fn enumerate(self) -> Enumerate<Self>
where {
        Enumerate::new(self)
    }

    #[inline]
    fn eq<I>(mut self, mut another: I) -> bool
    where
        I: Iterator<Item = Self::Item>,
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
                        Step::Done => return false,
                    }
                },
                Step::Done => {
                    if let Step::Done = another.next() {
                        return true;
                    } else {
                        return false;
                    }
                }
            }
        }
    }
}

pub struct IterStream<I> {
    iter: I,
}

impl<I: std::iter::Iterator> From<I> for IterStream<I> {
    #[inline]
    fn from(value: I) -> Self {
        Self { iter: value }
    }
}

impl<I: std::iter::Iterator> Iterator for IterStream<I> {
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Step<Self::Item> {
        match self.iter.next() {
            Some(item) => Step::Ready(item),
            None => Step::Done,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{IterStream, Iterator};
    use crate::iter::Step;

    #[test]
    fn from_iter() {
        let v = vec![0, 1, 2, 3];
        let mut stream = IterStream::from(v.iter())
            .filter(|item| *item % 2 == 0)
            .map(|item| *item + 1);
        let mut round = 0;
        loop {
            round += 1;
            match stream.next() {
                Step::Done => break,
                Step::Ready(i) => {
                    let _ = i;
                }
                _ => {}
            }
        }
        assert_eq!(round, 5);
    }
}
