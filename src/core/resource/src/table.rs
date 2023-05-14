use std::rc::Rc;

use chunk::mutable::{column::FilterError, Morsel, MutableChunk};
use common::{
    column::{label::LabelType, FieldType},
    expression::MatcherOp,
};
use croaring::Bitmap;
use executor::iter::{Iterator, IteratorFusion, StdIter, Step};
use query::Context;
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
    pub schema: Schema,
}

#[derive(Debug)]
pub struct Table {
    pub meta: Meta,
    pub mutable_chunks: Vec<MutableChunk>,
}

#[derive(Error, Debug)]
pub enum TableScanError {
    #[error("filter column error {}", .source)]
    FilterError {
        #[from]
        source: FilterError,
    },
}

#[derive(Debug)]
pub struct TableScan {
    table: Rc<Table>,
    context: Context,
    projection: Vec<usize>,
    filter: Vec<MatcherOp>,
    limit: Option<usize>,

    set: Bitmap,
}

impl<'iter> Iterator<'iter> for TableScan {
    type Item = Result<Morsel<'iter>, FilterError>;
    type Return = ();
    type Error = TableScanError;

    fn next(&mut self) -> Step<Self::Item, Result<(), Self::Error>> {
        let iter = self
            .table
            .mutable_chunks
            .iter()
            .zip(self.filter.iter())
            .fusion()
            .fold(Bitmap::create(), |superset, (chunk, matcher)| {
                chunk.labels.iter().fusion().map(|label| {
                    label.filter(matcher, superset)
                });
            });
        todo!()
    }
}

// impl Source for Table {
//     type Execution;

//     type ScanFut<'future>
//     where
//         Self: 'future;

//     fn scan(
//         &self,
//         context: &query::Context,
//         projection: &[usize],
//         filter: &[common::expression::Matcher<usize>],
//     ) -> Self::ScanFut<'_> {
//         todo!()
//     }
// }
