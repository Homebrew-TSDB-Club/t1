use std::future::Future;

use chunk::mutable::MutableChunk;
use common::column::{FieldType, LabelType};
use executor::Stream;
use query::{execution::Execution, Source};

#[derive(Debug)]
pub struct Label {
    r#type: LabelType,
    name: String,
}

#[derive(Debug)]
pub struct Field {
    r#type: FieldType,
    name: String,
}

#[derive(Debug)]
pub struct Schema {
    pub label: Vec<Label>,
    pub field: Vec<Field>,
}

#[derive(Debug)]
pub struct Meta {
    mutable_chunk_num: usize,
    schema: Schema,
}

#[derive(Debug)]
pub struct Table {
    meta: Meta,
    mutable_chunks: Vec<MutableChunk>,
}

// pub struct TableSream {}

// impl Stream for TableSream {
//     type Output<'s>
//     where
//         Self: 's;

//     type NextFut<'s>
//     where
//         Self: 's;

//     fn next(&self) -> Self::NextFut<'_> {
//         todo!()
//     }
// }

// pub struct TableExecution {}

// impl Execution for TableExecution {
//     type Stream = impl Stream;

//     fn execute(&self) -> Self::Stream {

//     }
// }

// impl Source for Table {
//     type Execution = ;
//     type ScanFut<'future> = impl 'future + Future
//     where
//         Self: 'future;

//     fn scan(
//         &self,
//         context: &query::Context,
//         projection: &[usize],
//         filter: &[query::language::expression::Matcher<usize>],
//     ) -> Self::ScanFut<'_> {
//         async { todo!() }
//     }
// }
