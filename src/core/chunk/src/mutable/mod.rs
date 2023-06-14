use common::{
    column::{
        field::Field,
        label::{Label, LabelValue},
    },
    context::Context,
    query::{MatcherOp, ProjectionRef},
    schema::Schema,
    time::{Duration, Instant, Range},
    Set,
};
use croaring::Bitmap;

use self::{
    column::{label::LabelImpl, FilterError},
    index::IndexImpl,
};
use crate::mutable::column::{
    field::{
        BoolField, FieldImpl, Float32Field, Float64Field, Int16Field, Int32Field, Int64Field,
        Int8Field, UInt16Field, UInt32Field, UInt64Field, UInt8Field,
    },
    label::{BoolLabel, IPv4Label, IPv6Label, IntLabel, LabelColumn, StringLabel},
};

pub mod column;
pub mod index;

#[derive(Debug, Clone, PartialEq)]
pub struct Meta {
    pub(crate) start_at: Instant,
    pub(crate) unit: Duration,
    pub(crate) length: u32,
    pub(crate) width: u32,
}

impl Meta {
    fn new(start_at: Instant, unit: Duration, length: u32, width: u32) -> Self {
        Self {
            start_at,
            unit,
            length,
            width,
        }
    }
}

#[derive(Debug)]
pub struct Records {
    pub labels: Vec<LabelImpl>,
    pub fields: Vec<FieldImpl>,
}

impl Records {
    fn new(schema: &Schema, width: u32) -> Self {
        let labels = schema
            .labels
            .iter()
            .map(|label| {
                match label.r#type {
                    Label::String(_) => Label::String(LabelColumn::<StringLabel>::new()),
                    Label::IPv4(_) => Label::IPv4(LabelColumn::<IPv4Label>::new()),
                    Label::IPv6(_) => Label::IPv6(LabelColumn::<IPv6Label>::new()),
                    Label::Int(_) => Label::Int(LabelColumn::<IntLabel>::new()),
                    Label::Bool(_) => Label::Bool(LabelColumn::<BoolLabel>::new()),
                }
                .into()
            })
            .collect();

        let fields = schema
            .fields
            .iter()
            .map(|field| {
                match field.r#type.as_ref() {
                    Field::UInt8(_) => Field::UInt8(UInt8Field::new(width)),
                    Field::UInt16(_) => Field::UInt16(UInt16Field::new(width)),
                    Field::UInt32(_) => Field::UInt32(UInt32Field::new(width)),
                    Field::UInt64(_) => Field::UInt64(UInt64Field::new(width)),
                    Field::Int8(_) => Field::Int8(Int8Field::new(width)),
                    Field::Int16(_) => Field::Int16(Int16Field::new(width)),
                    Field::Int32(_) => Field::Int32(Int32Field::new(width)),
                    Field::Int64(_) => Field::Int64(Int64Field::new(width)),
                    Field::Float32(_) => Field::Float32(Float32Field::new(width)),
                    Field::Float64(_) => Field::Float64(Float64Field::new(width)),
                    Field::Bool(_) => Field::Bool(BoolField::new(width)),
                }
                .into()
            })
            .collect();

        Self { labels, fields }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.fields[0].len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug)]
pub struct MutableChunk {
    pub records: Records,
    pub index: Vec<IndexImpl<usize>>,
    pub meta: Meta,
}

impl MutableChunk {
    pub fn new(
        schema: &Schema,
        start_at: Instant,
        duration: Duration,
        length: u32,
        width: u32,
    ) -> Self {
        let data = Records::new(schema, width);

        let mut index = Vec::with_capacity(schema.index.len());
        for r#type in &schema.index {
            index.push(IndexImpl::new(r#type))
        }

        let meta = Meta::new(start_at, duration, length, width);

        Self {
            records: data,
            index,
            meta,
        }
    }

    pub fn push(&mut self, labels: Vec<Option<LabelValue>>) {
        for ((value, column), index) in labels
            .into_iter()
            .zip(self.records.labels.iter_mut())
            .zip(self.index.iter_mut())
        {
            let value_id: usize = column.push(value);
            index.insert(column.len() - 1, value_id);
        }

        for column in &mut self.records.fields {
            column.push_zero();
        }
    }

    #[inline]
    unsafe fn filter_by_index(
        &self,
        row_set: &mut Bitmap,
        matcher: &[Option<MatcherOp>],
    ) -> Result<(), FilterError> {
        for ((label, matcher), index) in self
            .records
            .labels
            .iter()
            .zip(matcher.iter())
            .zip(self.index.iter())
        {
            if let Some(matcher) = matcher {
                match matcher {
                    MatcherOp::LiteralEqual(op) | MatcherOp::LiteralNotEqual(op) => {
                        let value_id = label.lookup_value_id_unchecked(op);
                        match value_id {
                            Some(value_id) => index.filter(matcher, value_id, row_set),
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
        Ok(())
    }

    async unsafe fn filter_rows(
        &self,
        cx: &mut Context,
        matcher: &[Option<MatcherOp>],
    ) -> Result<Bitmap, FilterError> {
        let mut row_set = Bitmap::from_range(0..self.records.labels[0].len() as u32);

        self.filter_by_index(&mut row_set, matcher)?;
        if row_set.is_empty() {
            return Ok(row_set);
        }

        if self.exactly(matcher) {
            return Ok(row_set);
        }

        for (label, matcher) in self.records.labels.iter().zip(matcher.iter()) {
            if let Some(matcher) = matcher {
                label.filter(cx, matcher, &mut row_set).await?;
            }
        }

        Ok(row_set)
    }

    async fn map(
        &self,
        cx: &mut Context,
        projection: ProjectionRef<'_>,
        set: Bitmap,
        range: Range,
    ) -> Records {
        let labels = match projection.labels {
            Set::Universe => {
                let mut mapped = Vec::with_capacity(set.cardinality() as usize);
                for label in self.records.labels.iter() {
                    mapped.push(label.map(cx, &set).await);
                }
                mapped
            }
            Set::Some(predicate) => {
                let mut mapped = Vec::with_capacity(set.cardinality() as usize);
                for label in predicate.iter() {
                    mapped.push(self.records.labels[*label].map(cx, &set).await);
                }
                mapped
            }
        };

        let range = self.trim_range(range.clone());
        let fields = match projection.fields {
            Set::Universe => {
                let mut mapped = Vec::with_capacity(set.cardinality() as usize);
                for field in self.records.fields.iter() {
                    mapped.push(field.map(cx, &set, range.clone()).await)
                }
                mapped
            }
            Set::Some(predicate) => {
                let mut mapped = Vec::with_capacity(set.cardinality() as usize);
                for field in predicate.iter() {
                    mapped.push(
                        self.records.fields[*field]
                            .map(cx, &set, range.clone())
                            .await,
                    );
                }
                mapped
            }
        };

        Records { labels, fields }
    }

    #[inline]
    fn trim_range(&self, range: Range) -> std::ops::Range<usize> {
        let start = match range.start {
            Some(start) => ((start - self.meta.start_at) / self.meta.unit) as usize,
            None => 0,
        };
        let end = match range.end {
            Some(end) => ((end - self.end_at()) / self.meta.unit) as usize,
            None => self.meta.width as usize,
        };
        start..end
    }

    #[allow(clippy::missing_safety_doc)]
    pub async unsafe fn filter(
        &self,
        cx: &mut Context,
        matcher: &[Option<MatcherOp>],
        projection: ProjectionRef<'_>,
        range: Range,
    ) -> Result<Records, FilterError> {
        let set = self.filter_rows(cx, matcher).await?;
        Ok(self.map(cx, projection, set, range).await)
    }

    #[inline]
    fn exactly<V>(&self, matcher: &[Option<MatcherOp<V>>]) -> bool {
        for (index, matcher) in self.index.iter().zip(matcher.iter()) {
            if matcher.is_some() && !index.exactly() {
                return false;
            }
        }
        true
    }

    #[inline]
    pub fn end_at(&self) -> Instant {
        self.meta.start_at + self.meta.unit * self.meta.width
    }

    #[inline]
    pub fn range(&self) -> Range {
        Range {
            start: Some(self.meta.start_at),
            end: Some(self.meta.start_at + self.meta.unit * self.meta.width),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr};

    use common::{
        column::label::{Label, LabelValue},
        context::Context,
        index::Index,
        query::MatcherOp,
        time::{Duration, Instant},
    };
    use croaring::Bitmap;

    use super::{
        column::label::{BoolLabel, IPv4Label, IPv6Label, IntLabel, LabelColumn, StringLabel},
        index::IndexImpl,
        Meta, MutableChunk, Records,
    };

    #[test]
    fn chunk_filter() {
        let mut chunk = MutableChunk {
            records: Records {
                labels: vec![
                    Label::String(LabelColumn::<StringLabel>::new()).into(),
                    Label::IPv4(LabelColumn::<IPv4Label>::new()).into(),
                    Label::IPv6(LabelColumn::<IPv6Label>::new()).into(),
                    Label::Int(LabelColumn::<IntLabel>::new()).into(),
                    Label::Bool(LabelColumn::<BoolLabel>::new()).into(),
                ],
                fields: vec![],
            },
            index: vec![IndexImpl::new(&Index::Inverted(()))],
            meta: Meta {
                start_at: Instant::now(),
                unit: Duration::from_secs(1),
                length: 0,
                width: 0,
            },
        };

        let others = vec![
            Some(LabelValue::IPv4(
                "127.0.0.1".parse::<Ipv4Addr>().unwrap().octets(),
            )),
            Some(LabelValue::IPv6(
                "::1".parse::<Ipv6Addr>().unwrap().octets(),
            )),
            Some(LabelValue::Int(1)),
            Some(LabelValue::Bool(true)),
        ];

        chunk.push(vec![None, None, None, None, None]);
        chunk.push(
            [
                vec![Some(LabelValue::String(Vec::from("hello")))],
                others.clone(),
            ]
            .concat(),
        );
        chunk.push(
            [
                vec![Some(LabelValue::String(Vec::from("world")))],
                others.clone(),
            ]
            .concat(),
        );
        chunk.push(
            [
                vec![Some(LabelValue::String(Vec::from("hello")))],
                others.clone(),
            ]
            .concat(),
        );

        futures_lite::future::block_on(async move {
            unsafe {
                let mut cx = Context::new(256);
                let set = chunk
                    .filter_rows(
                        &mut cx,
                        &[Some(MatcherOp::LiteralEqual(None)), None, None, None, None],
                    )
                    .await
                    .unwrap();
                assert_eq!(set, Bitmap::from_range(0..1));

                let set = chunk
                    .filter_rows(
                        &mut cx,
                        &[
                            Some(MatcherOp::LiteralEqual(Some(LabelValue::String(
                                Vec::from("hello"),
                            )))),
                            Some(MatcherOp::LiteralEqual(Some(LabelValue::IPv4(
                                "127.0.0.1".parse::<Ipv4Addr>().unwrap().octets(),
                            )))),
                            None,
                            Some(MatcherOp::LiteralEqual(Some(LabelValue::Int(1)))),
                            None,
                        ],
                    )
                    .await
                    .unwrap();
                assert_eq!(set, Bitmap::from_iter([1, 3]));
            }
        })
    }
}
