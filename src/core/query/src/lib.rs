pub mod execution;
pub mod language;

use std::future::Future;

use common::expression::Matcher;
use execution::Execution;
use uuid::Uuid;

#[derive(Debug)]
pub struct Context {
    session_id: Uuid,
}

pub trait Source {
    type Execution: Execution;
    type ScanFut<'future>: 'future + Future<Output = Self::Execution>
    where
        Self: 'future;

    fn scan(
        &self,
        context: &Context,
        projection: &[usize],
        filter: &[Matcher<usize>],
    ) -> Self::ScanFut<'_>;
}
