pub mod hir;
pub mod promql;

use thiserror::Error;

use self::hir::Hir;
use crate::{Layer, Pass};

#[derive(Error, Debug)]
pub enum Error<E: std::error::Error> {
    #[error("parsing wrong: {}", .err)]
    ParsingWrong { err: String },
    #[error("query does not have a metric name")]
    NoName,
    #[error(transparent)]
    UpStream(#[from] E),
}

#[derive(Default)]
pub struct Parser;

impl Parser {
    pub(crate) fn new() -> Self {
        Default::default()
    }
}

impl<'input, Input, Inner> Layer<Input, Inner> for Parser
where
    Inner: Pass<Input, Output = &'input str>,
{
    type Pass = Parse<Inner>;

    fn layer(&self, inner: Inner) -> Self::Pass {
        Parse { inner }
    }
}

pub struct Parse<P> {
    inner: P,
}

impl<'input, Input, P> Pass<Input> for Parse<P>
where
    P: Pass<Input, Output = &'input str>,
{
    type Output = Hir;
    type Error = Error<P::Error>;

    fn apply(&self, input: Input) -> Result<Self::Output, Self::Error> {
        promql::parse(self.inner.apply(input)?)
    }
}
