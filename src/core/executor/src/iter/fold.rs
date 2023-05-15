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

    #[inline]
    fn next(&mut self) -> Step<Self::Item, Self::Return> {
        let accum = self.accum.as_mut().expect("fold iterator has beed done");
        match self.iter.next() {
            Step::NotYet => Step::NotYet,
            Step::Ready(ready) => {
                (self.f)(accum, ready);
                Step::NotYet
            }
            Step::Done(done) => Step::Done(unsafe { self.accum.take().unwrap_unchecked() }),
        }
    }
}
