use super::Iterator;

#[derive(Debug)]
pub struct FlatMap<I, F> {
    iter: I,
    f: F,
}

impl<'iter, I, F, B> Iterator<'iter> for FlatMap<I, F>
where
    I: Iterator<'iter>,
    I::Item: Iterator<'iter>,
    F: FnMut(<I::Item as Iterator<'iter>>::Item) -> B,
    B: 'iter,
{
    type Item = B;
    type Return = I::Return;
    type Error = I::Error;

    #[inline]
    fn next(&mut self) -> super::Step<Self::Item, Result<Self::Return, Self::Error>> {
        todo!()
        // (self.f)(iter)
    }
}
