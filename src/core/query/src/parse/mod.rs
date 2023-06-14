pub mod promql;

use regex;
use thiserror::Error;

use crate::{plan::logical::Logical, Layer, Pass};

#[derive(Error, Debug)]
pub enum Error {
    #[error("parsing wrong: {}", .err)]
    ParsingWrong { err: String },
    #[error("query does not have a metric name")]
    NoName,
    #[error(transparent)]
    UpStream(#[from] Box<dyn std::error::Error>),
    #[error("invalid regex pattern: {}", .0)]
    InvalidRegex(#[from] regex::Error),
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
    type Output = Logical;
    type Error = Error;

    fn apply(&self, input: Input) -> Result<Self::Output, Self::Error> {
        promql::parse(
            self.inner
                .apply(input)
                .map_err(|e| Error::UpStream(Box::new(e)))?,
        )
    }
}
