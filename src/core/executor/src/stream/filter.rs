use std::future::Future;

use super::{Iterator, Step};

pub struct Filter<S, P> {
    stream: S,
    predicate: P,
}

impl<S, P> Filter<S, P> {
    pub(crate) fn new(stream: S, predicate: P) -> Self {
        Self { stream, predicate }
    }
}

impl<S, P> Iterator for Filter<S, P>
where
    S: Iterator,
    P: FnMut(&S::Item) -> bool,
{
    type Item = S::Item;

    type NextFut<'s> = impl 's + Future<Output = Step<Self::Item>>
    where
        Self: 's;

    #[inline]
    fn next(&mut self) -> Self::NextFut<'_> {
        async {
            match self.stream.next().await {
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
}
