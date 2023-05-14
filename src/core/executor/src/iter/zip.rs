use std::mem::swap;

use super::{Iterator, Step};
use crate::either::Either;

#[derive(Debug)]
pub struct Zip<'i0, 'i1, I0, I1>
where
    I0: Iterator<'i0>,
    I1: Iterator<'i1>,
{
    i0: I0,
    i1: I1,
    buf: (Option<I0::Item>, Option<I1::Item>),
}

impl<'i0, 'i1, I0, I1> Zip<'i0, 'i1, I0, I1>
where
    I0: Iterator<'i0>,
    I1: Iterator<'i1>,
{
    #[inline]
    pub(crate) fn new(i0: I0, i1: I1) -> Self {
        Self {
            i0,
            i1,
            buf: (None, None),
        }
    }
}

impl<'i, 'i0: 'i, 'i1: 'i, I0, I1> Iterator<'i> for Zip<'i0, 'i1, I0, I1>
where
    I0: Iterator<'i0>,
    I1: Iterator<'i1>,
{
    type Item = (I0::Item, I1::Item);
    type Return = Either<I0::Return, I1::Return>;
    type Error = Either<I0::Error, I1::Error>;

    #[inline]
    fn next(&mut self) -> Step<Self::Item, Result<Self::Return, Self::Error>> {
        match (self.i0.next(), self.i1.next()) {
            (Step::NotYet, Step::NotYet) => Step::NotYet,
            (Step::NotYet, Step::Ready(mut i1)) => {
                self.buf.1.as_mut().map(|b| swap(b, &mut i1));
                if let Some(i0) = self.buf.0.take() {
                    Step::Ready((i0, i1))
                } else {
                    self.buf.1 = Some(i1);
                    Step::NotYet
                }
            }
            (Step::Ready(mut i0), Step::NotYet) => {
                self.buf.0.as_mut().map(|b| swap(b, &mut i0));
                if let Some(i1) = self.buf.1.take() {
                    Step::Ready((i0, i1))
                } else {
                    self.buf.0 = Some(i0);
                    Step::NotYet
                }
            }
            (Step::Ready(mut i0), Step::Ready(mut i1)) => {
                self.buf.0.as_mut().map(|b| swap(b, &mut i0));
                self.buf.1.as_mut().map(|b| swap(b, &mut i1));
                Step::Ready((i0, i1))
            }
            (Step::Done(done), _) => match done {
                Ok(ok) => Step::Done(Ok(Either::A(ok))),
                Err(e) => Step::Done(Err(Either::A(e))),
            },
            (_, Step::Done(done)) => match done {
                Ok(ok) => Step::Done(Ok(Either::B(ok))),
                Err(e) => Step::Done(Err(Either::B(e))),
            },
        }
    }
}
