use std::{
    ops::{Generator, GeneratorState},
    pin::Pin,
    rc::Rc,
    sync::Arc,
};

use chunk::mutable::{column::FilterError, MutableChunk};
use common::{
    column::{label::LabelType, FieldType},
    expression::MatcherOp,
};
use croaring::Bitmap;
use executor::iter::Step;
use query::{Context, Projection};
use thiserror::Error;

#[derive(Debug)]
pub struct Label {
    pub r#type: LabelType,
    pub name: String,
}

#[derive(Debug)]
pub struct Field {
    pub r#type: FieldType,
    pub name: String,
}

#[derive(Debug)]
pub struct Schema {
    pub label: Vec<Label>,
    pub field: Vec<Field>,
}

#[derive(Debug)]
pub struct Meta {
    pub mutable_chunk_num: usize,
    pub schema: Arc<Schema>,
}

#[derive(Debug)]
pub struct Table {
    pub meta: Meta,
    pub mutable_chunks: Vec<MutableChunk>,
}

impl Table {
    #[inline]
    pub fn schema(&self) -> &Arc<Schema> {
        &self.meta.schema
    }
}

#[derive(Error, Debug)]
pub enum TableScanError {
    #[error("filter column error {}", .source)]
    FilterError {
        #[from]
        source: FilterError,
    },
}
