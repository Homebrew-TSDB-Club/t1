use super::{Iterator, Step};

#[derive(Debug)]
pub struct Filter<I, P> {
    stream: I,
    predicate: P,
}

impl<I, P> Filter<I, P> {
    pub(crate) fn new(stream: I, predicate: P) -> Self {
        Self { stream, predicate }
    }
}

impl<'iter, I, P> Iterator<'iter> for Filter<I, P>
where
    I: Iterator<'iter>,
    P: FnMut(&I::Item) -> bool,
{
    type Item = I::Item;
    type Return = I::Return;

    #[inline]
    fn next(&mut self) -> Step<Self::Item, Self::Return> {
        match self.stream.next() {
            Step::Ready(item) => {
                if (self.predicate)(&item) {
                    Step::Ready(item)
                } else {
                    Step::NotYet
                }
            }
            other => other,
        }
    }
}
