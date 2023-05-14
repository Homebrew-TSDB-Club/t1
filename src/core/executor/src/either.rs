use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum Either<E1, E2> {
    A(E1),
    B(E2),
}

impl<E1, E2> Display for Either<E1, E2>
where
    E1: Display,
    E2: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Either::A(e) => e.fmt(f),
            Either::B(e) => e.fmt(f),
        }
    }
}

impl<E1, E2> Error for Either<E1, E2>
where
    E1: Error,
    E2: Error,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Either::A(e) => e.source(),
            Either::B(e) => e.source(),
        }
    }
}
