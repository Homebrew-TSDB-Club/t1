#![feature(impl_trait_in_assoc_type)]
#![feature(async_fn_in_trait)]

use std::{error::Error, future::Future};

pub enum Step<T> {
    NotYet,
    Ready(T),
    Done,
}

pub trait Stream {
    type Error: Error;
    type Output<'s>
    where
        Self: 's;
    type NextFut<'s>: 's + Future<Output = Result<Step<Self::Output<'s>>, Self::Error>>
    where
        Self: 's;

    fn next(&mut self) -> Self::NextFut<'_>;
}
