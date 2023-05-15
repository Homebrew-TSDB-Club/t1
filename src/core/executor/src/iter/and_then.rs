use std::{marker::PhantomData, unreachable};

use super::{Iterator, Step};

#[derive(Debug)]
pub struct AndThen<I, F, B> {
    iter: I,
    then: F,
    _marker: PhantomData<B>,
}

impl<I, F, B> AndThen<I, F, B> {
    pub(crate) fn new(iter: I, then: F) -> Self {
        Self {
            iter,
            then,
            _marker: PhantomData,
        }
    }
}

impl<'iter, I, F, B> Iterator<'iter> for AndThen<I, F, B>
where
    I: Iterator<'iter>,
    F: FnMut(I::Return) -> B,
{
    type Item = I::Item;
    type Return = B;

    fn next(&mut self) -> Step<Self::Item, Self::Return> {
        match self.iter.next() {
            Step::NotYet => Step::NotYet,
            Step::Ready(_) => unreachable!(),
            Step::Done(done) => Step::Done((self.then)(done)),
        }
    }
}
