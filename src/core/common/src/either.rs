use std::{
    error::Error,
    fmt,
    fmt::{Debug, Display, Formatter},
};

pub enum Either<A, B> {
    A(A),
    B(B),
}

impl<A: Debug, B: Debug> Debug for Either<A, B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::A(a) => Debug::fmt(a, f),
            Self::B(b) => Debug::fmt(b, f),
        }
    }
}

impl<A: Display, B: Display> Display for Either<A, B> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::A(a) => Display::fmt(a, f),
            Self::B(b) => Display::fmt(b, f),
        }
    }
}

impl<A: Error, B: Error> Error for Either<A, B> {}
