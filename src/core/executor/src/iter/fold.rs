use super::{Iterator, Step};

#[derive(Debug)]
pub struct Fold<I, F, B> {
    iter: I,
    f: F,
    accum: Option<B>,
}

impl<I, F, B> Fold<I, F, B> {
    pub(crate) fn new(iter: I, accum: B, f: F) -> Self {
        Self {
            iter,
            f,
            accum: Some(accum),
        }
    }
}

impl<'iter, I, F, B> Iterator<'iter> for Fold<I, F, B>
where
    I: Iterator<'iter, Return = ()>,
    F: FnMut(&mut B, I::Item),
{
    type Item = I::Item;
    type Return = B;
    type Error = I::Error;

    #[inline]
    fn next(&mut self) -> Step<Self::Item, Result<Self::Return, Self::Error>> {
        let accum = self.accum.as_mut().expect("fold iterator has beed done");
        match self.iter.next() {
            Step::NotYet => Step::NotYet,
            Step::Ready(ready) => {
                (self.f)(accum, ready);
                Step::NotYet
            }
            Step::Done(done) => match done {
                Ok(_) => Step::Done(Ok(unsafe { self.accum.take().unwrap_unchecked() })),
                Err(e) => Step::Done(Err(e)),
            },
        }
    }
}
