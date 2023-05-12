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

impl<I> Iterator for Enumerate<I>
where
    I: Iterator,
{
    type Item = (usize, I::Item);

    #[inline]
    fn next(&mut self) -> Step<Self::Item> {
        match self.iter.next() {
            Step::Ready(item) => {
                let item = Step::Ready((self.count, item));
                self.count += 1;
                item
            }
            Step::NotYet => Step::NotYet,
            Step::Done => Step::Done,
        }
    }
}
