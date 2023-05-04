use std::{cell::RefCell, rc::Rc, sync::Arc};

use chunk::mutable::MutableChunk;
use common::schema::Schema;
use executor::utils::ThreadLocal;

#[derive(Debug)]
pub struct MutableMeta {
    pub width: u32,
    pub length: u32,
    pub count: usize,
}

#[derive(Debug)]
pub struct ChunkMeta {
    pub mutable: MutableMeta,
}

#[derive(Debug)]
pub struct Meta {
    pub chunk: ChunkMeta,
    pub schema: Arc<Schema>,
}

#[derive(Debug, Default)]
pub struct DataShard {
    pub mutable: Vec<MutableChunk>,
}

impl DataShard {
    pub fn new(meta: &Meta) -> Self {
        Self {
            mutable: Vec::with_capacity(meta.chunk.mutable.count),
        }
    }
}

#[derive(Debug)]
pub struct Table {
    pub name: Arc<str>,
    pub meta: Meta,
    pub shards: ThreadLocal<Rc<RefCell<DataShard>>>,
}

impl Table {
    pub(crate) fn new(name: Arc<str>, meta: Meta) -> Self {
        let shards = ThreadLocal::new(|| Rc::new(RefCell::new(DataShard::new(&meta))));
        Self { name, meta, shards }
    }
}
