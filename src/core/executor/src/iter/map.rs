use super::{Iterator, Step};

#[derive(Debug)]
pub struct Map<S, F> {
    stream: S,
    f: F,
}

impl<S, F> Map<S, F> {
    pub(crate) fn new(stream: S, f: F) -> Self {
        Self { stream, f }
    }
}

impl<'iter, I, F, B> Iterator<'iter> for Map<I, F>
where
    I: Iterator<'iter>,
    F: FnMut(I::Item) -> B,
    B: 'iter,
{
    type Item = B;
    type Return = I::Return;

    #[inline]
    fn next(&mut self) -> Step<Self::Item, Self::Return> {
        match self.stream.next() {
            Step::Ready(item) => Step::Ready((self.f)(item)),
            Step::NotYet => Step::NotYet,
            Step::Done(done) => Step::Done(done),
        }
    }
}
