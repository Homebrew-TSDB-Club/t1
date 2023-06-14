pub mod function;
pub mod plan;
pub mod scan;

use std::error::Error;

use chunk::mutable::Records;
use common::{context::Context, DynError};

use self::scan::Scan;

pub trait Planner {
    type Execution: Execution;
    type Error: Error;

    fn plan(self, inner: ExecutionImpl) -> Result<Self::Execution, Self::Error>;
}

#[derive(Debug)]
pub enum ExecutionImpl {
    Scan(Scan),
    Id(()),
}

pub trait Execution {
    async fn next(&mut self, cx: &mut Context) -> Option<Result<Records, DynError>>;
}

impl Execution for () {
    #[inline]
    async fn next(&mut self, _: &mut Context) -> Option<Result<Records, DynError>> {
        unreachable!()
    }
}
