pub mod filter;
pub mod map;

use std::future::Future;

use self::{filter::Filter, map::Map};

#[derive(Debug)]
pub enum Step<T> {
    NotYet,
    Ready(T),
    Done,
}

pub trait Iterator {
    type Item;
    type NextFut<'s>: 's + Future<Output = Step<Self::Item>>
    where
        Self: 's;

    fn next(&mut self) -> Self::NextFut<'_>;

    #[inline]
    fn filter<P>(self, predicate: P) -> Filter<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        Filter::new(self, predicate)
    }

    #[inline]
    fn map<P, B>(self, f: P) -> Map<Self, P>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> B,
    {
        Map::new(self, f)
    }
}

pub struct IterStream<I> {
    iter: I,
}

impl<I: std::iter::Iterator> From<I> for IterStream<I> {
    fn from(value: I) -> Self {
        Self { iter: value }
    }
}

impl<I: std::iter::Iterator> Iterator for IterStream<I> {
    type Item = I::Item;

    type NextFut<'s> = impl 's + Future<Output = Step<Self::Item>>
    where
        Self: 's;

    #[inline]
    fn next(&mut self) -> Self::NextFut<'_> {
        async {
            match self.iter.next() {
                Some(item) => Step::Ready(item),
                None => Step::Done,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;

    use super::{IterStream, Iterator};
    use crate::stream::Step;

    #[test]
    fn from_iter() {
        let v = vec![0, 1, 2, 3];
        let mut stream = IterStream::from(v.iter())
            .filter(|item| *item % 2 == 0)
            .map(|item| *item + 1);
        block_on(async move {
            loop {
                match stream.next().await {
                    Step::Done => break,
                    Step::Ready(i) => println!("{:?}", i),
                    _ => {}
                }
            }
        });
    }
}
