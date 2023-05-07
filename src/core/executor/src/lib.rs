#![feature(impl_trait_in_assoc_type)]
#![feature(async_fn_in_trait)]

use std::future::Future;

pub enum Step<T> {
    NotYet,
    Ready(T),
    Done,
}

pub trait Stream {
    type Output<'s>
    where
        Self: 's;
    type NextFut<'s>: Future<Output = Step<Self::Output<'s>>>
    where
        Self: 's;

    fn next(&self) -> Self::NextFut<'_>;
}
