#![feature(async_fn_in_trait)]
#![feature(impl_trait_in_assoc_type)]
#![feature(impl_trait_projections)]

use std::{
    convert::Infallible,
    error::Error,
    fmt::{Debug, Display},
    marker::PhantomData,
};

pub mod checker;
pub mod executor;
pub mod parser;

pub trait Layer<Input, Inner: Pass<Input>> {
    type Pass: Pass<Input>;

    fn layer(&self, inner: Inner) -> Self::Pass;
}

pub trait Pass<Input> {
    type Output;
    type Error: 'static + Error;

    fn apply(&self, input: Input) -> Result<Self::Output, Self::Error>;
}

pub enum Either<A, B> {
    A(A),
    B(B),
}

impl<A: Debug, B: Debug> Debug for Either<A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::A(a) => Debug::fmt(a, f),
            Self::B(b) => Debug::fmt(b, f),
        }
    }
}

impl<A: Display, B: Display> Display for Either<A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::A(a) => Display::fmt(a, f),
            Self::B(b) => Display::fmt(b, f),
        }
    }
}

impl<A: Error, B: Error> Error for Either<A, B> {}

pub struct Stack<Input, I, O> {
    inner: I,
    outer: O,
    _marker: PhantomData<Input>,
}

impl<Input, I, O> Pass<Input> for Stack<Input, I, O>
where
    I: Pass<Input>,
    O: Pass<I::Output>,
{
    type Output = O::Output;
    type Error = Either<I::Error, O::Error>;

    fn apply(&self, input: Input) -> Result<Self::Output, Self::Error> {
        self.outer
            .apply(self.inner.apply(input).map_err(Either::A)?)
            .map_err(Either::B)
    }
}

impl<T> Pass<T> for () {
    type Output = T;
    type Error = Infallible;

    fn apply(&self, input: T) -> Result<Self::Output, Self::Error> {
        Ok(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::{parser::Parser, Layer};

    #[test]
    fn make_passes() {
        let _ = Parser {}.layer(());
    }
}
