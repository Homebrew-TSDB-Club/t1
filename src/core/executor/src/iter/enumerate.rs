use super::{Iterator, Step};

pub struct Enumerate<I> {
    iter: I,
    count: usize,
}

impl<I> Enumerate<I> {
    pub(crate) fn new(iter: I) -> Self {
        Self { iter, count: 0 }
    }
}

impl<'iter, I> Iterator<'iter> for Enumerate<I>
where
    I: Iterator<'iter>,
{
    type Item = (usize, I::Item);
    type Return = I::Return;

    #[inline]
    fn next(&mut self) -> Step<Self::Item, Self::Return> {
        match self.iter.next() {
            Step::Ready(item) => {
                let item = Step::Ready((self.count, item));
                self.count += 1;
                item
            }
            Step::NotYet => Step::NotYet,
            Step::Done(done) => Step::Done(done),
        }
    }
}
