use std::future::Future;

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

    type NextFut<'s> = impl 's + Future<Output = Step<Self::Item>>
    where
        Self: 's;

    #[inline]
    fn next(&mut self) -> Self::NextFut<'_> {
        async {
            match self.stream.next().await {
                Step::Ready(item) => Step::Ready((self.f)(item)),
                Step::NotYet => Step::NotYet,
                Step::Done => Step::Done,
            }
        }
    }
}
