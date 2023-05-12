use super::{Iterator, Step};

pub struct Map<S, F> {
    stream: S,
    f: F,
}

impl<S, F> Map<S, F> {
    pub(crate) fn new(stream: S, f: F) -> Self {
        Self { stream, f }
    }
}

impl<S, F, B> Iterator for Map<S, F>
where
    S: Iterator,
    F: FnMut(S::Item) -> B,
{
    type Item = B;

    #[inline]
    fn next(&mut self) -> Step<Self::Item> {
        match self.stream.next() {
            Step::Ready(item) => Step::Ready((self.f)(item)),
            Step::NotYet => Step::NotYet,
            Step::Done => Step::Done,
        }
    }
}
