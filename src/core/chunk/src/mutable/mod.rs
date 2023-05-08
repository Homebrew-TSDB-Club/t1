use common::time::{Duration, Instant};

use self::column::{FieldImpl, LabelImpl};

pub mod column;
pub mod index;

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkMeta {
    pub(crate) start_at: Instant,
    time_interval: Duration,
    series_len: u32,
}

impl ChunkMeta {
    #[allow(unused)]
    #[inline]
    pub(crate) fn end_at(&self) -> Instant {
        self.start_at + self.time_interval * (self.series_len - 1)
    }
}

#[derive(Debug)]
pub struct MutableChunk {
    pub label: Vec<LabelImpl>,
    pub field: Vec<FieldImpl>,
    pub meta: ChunkMeta,
}
