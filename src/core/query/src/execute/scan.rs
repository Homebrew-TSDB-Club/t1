use std::sync::Arc;

use chunk::mutable::{MutableChunk, Records};
use common::{
    context::Context,
    query::{MatcherOp, Projection},
    time::Range,
};
use resource::{table::Table, TableScanError};
use thiserror::Error;

use super::{DynError, Execution, ExecutionImpl, Planner};

#[derive(Error, Debug)]
pub enum Error {
    #[error("no resource name: {}", .name)]
    NoResource { name: String },
    #[error("no column name: {}", .name)]
    NoColumn { name: String },
}

#[derive(Debug)]
pub struct ScanPlanner {
    pub(crate) resource: Arc<Table>,
    pub(crate) matcher: Vec<Option<MatcherOp>>,
    pub(crate) limit: Option<usize>,
    pub(crate) projection: Projection<usize>,
    pub(crate) range: Range,
}

impl Planner for ScanPlanner {
    type Execution = async_channel::Receiver<Result<Records, TableScanError>>;
    type Error = Error;

    #[inline]
    fn plan(self, _: ExecutionImpl) -> Result<Self::Execution, Self::Error> {
        let (send, recv) = async_channel::bounded(1);
        for id in 0..executor::worker_num() {
            let mut context = Context::new(256);
            let send = send.clone();
            let resource = self.resource.clone();
            let projection = self.projection.clone();
            let matcher = self.matcher.clone();
            let range = self.range.clone();
            executor::spawn_to(id, move || async move {
                let shards = resource.shards.get().borrow();
                let mut worker = ScanWorker {
                    iter: shards.mutable.iter(),
                    projection,
                    matcher,
                    limit: self.limit,
                    count: 0,
                    range,
                };
                while let Some(records) = worker.next(&mut context).await {
                    send.send(records).await.unwrap();
                }
            })
            .detach();
        }

        Ok(recv)
    }
}

pub type Scan = async_channel::Receiver<Result<Records, TableScanError>>;

impl Execution for Scan {
    async fn next(&mut self, _: &mut Context) -> Option<Result<Records, DynError>> {
        match self.recv().await {
            Ok(ok) => Some(ok.map_err(|e| Box::new(e) as Box<_>)),
            Err(_) => None,
        }
    }
}

#[derive(Debug)]
pub struct ScanWorker<'chunks> {
    iter: std::slice::Iter<'chunks, MutableChunk>,
    projection: Projection,
    matcher: Vec<Option<MatcherOp>>,
    limit: Option<usize>,
    count: usize,
    range: Range,
}

impl<'chunks> ScanWorker<'chunks> {
    async fn next(&mut self, cx: &mut Context) -> Option<Result<Records, TableScanError>> {
        if let Some(limit) = self.limit {
            if self.count >= limit {
                return None;
            }
        }
        let chunk = self.iter.next()?;
        let range = chunk.range() & self.range.clone();
        if range.is_empty() {
            return None;
        }
        let column =
            unsafe { chunk.filter(cx, &self.matcher, self.projection.as_ref(), range) }.await;
        match column {
            Ok(records) => {
                self.count += records.len();
                Some(Ok(records))
            }
            Err(e) => Some(Err(e.into())),
        }
    }
}
