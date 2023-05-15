use std::{
    ops::{Generator, GeneratorState},
    pin::Pin,
};

use common::{
    column::label::LabelTypeMismatch,
    expression::MatcherOp,
    time::{Duration, Instant},
};
use croaring::Bitmap;
use query::Context;

use self::{
    column::{
        label::{ItemRefImpl, LabelImpl},
        FilterError,
    },
    index::{IndexImpl, IndexType},
};
use crate::mutable::column::field::FieldImpl;

pub mod column;
pub mod index;

#[derive(Debug, Clone, PartialEq)]
pub struct ChunkMeta {
    pub(crate) start_at: Instant,
    time_interval: Duration,
    series_len: u32,
}

#[derive(Debug)]
pub struct MutableChunk {
    pub labels: Vec<LabelImpl>,
    pub fields: Vec<FieldImpl>,
    pub index: Vec<IndexImpl<usize>>,
    pub meta: ChunkMeta,
}

impl MutableChunk {
    pub fn push(&mut self, labels: Vec<Option<ItemRefImpl<'_>>>) -> Result<(), LabelTypeMismatch> {
        for ((value, column), index) in labels
            .into_iter()
            .zip(self.labels.iter_mut())
            .zip(self.index.iter_mut())
        {
            let value_id: usize = column.push(value);
            index.insert(column.len() - 1, value_id);
        }
        Ok(())
    }

    pub fn filter_rows<'s>(
        &'s self,
        _context: Context,
        matchers: &'s [Option<MatcherOp>],
    ) -> impl 's + Generator<Return = Result<(), FilterError>> {
        static move || {
            let mut row_set = Bitmap::from_range(0..self.labels[0].len() as u32);

            for ((label, matcher), index) in self
                .labels
                .iter()
                .zip(matchers.iter())
                .zip(self.index.iter())
            {
                if let Some(matcher) = matcher {
                    match matcher {
                        MatcherOp::LiteralEqual(op) | MatcherOp::LiteralNotEqual(op) => {
                            let value_id = label.lookup_value_id(op)?;
                            match value_id {
                                Some(value_id) => index.filter(matcher, value_id, &mut row_set),
                                None => {
                                    row_set.clear();
                                    return Ok(());
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            if self.exactly(&matchers) {
                return Ok(());
            }

            for (label, matcher) in self.labels.iter().zip(matchers.iter()) {
                if let Some(matcher) = matcher {
                    let mut generator = label.filter(matcher, &mut row_set)?;
                    loop {
                        match Pin::new(&mut generator).resume(()) {
                            GeneratorState::Yielded(_) => yield,
                            GeneratorState::Complete(_) => break,
                        }
                    }
                }
            }

            Ok(())
        }
    }

    pub fn exactly(&self, matchers: &[Option<MatcherOp>]) -> bool {
        let mut exactly = true;
        for (index, matcher) in self.index.iter().zip(matchers.iter()) {
            if let Some(_) = matcher {
                match index {
                    IndexType::Sparse(_) => {
                        exactly = false;
                        break;
                    }
                    _ => {}
                }
            }
        }
        exactly
    }
}
