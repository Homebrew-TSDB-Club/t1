#![feature(async_fn_in_trait)]

use std::{
    convert::Infallible,
    error::Error,
    sync::{Arc, RwLock},
};

use check::Checker;
use parse::Parser;
use resource::db::DB;

pub mod check;
pub mod execute;
pub mod parse;
pub mod plan;

pub fn query(db: Arc<RwLock<DB>>) {
    let _ = Checker::new(db).layer(Parser::new().layer(()));
}

pub trait Layer<Input, Inner: Pass<Input>> {
    type Pass: Pass<Input>;

    fn layer(&self, inner: Inner) -> Self::Pass;
}

pub trait Pass<Input> {
    type Output;
    type Error: 'static + Error;

    fn apply(&self, input: Input) -> Result<Self::Output, Self::Error>;
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
    use crate::{parse::Parser, Layer};

    #[test]
    fn make_passes() {
        let _ = Parser {}.layer(());
    }
}
