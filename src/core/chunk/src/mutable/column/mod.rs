pub mod label;

use std::ops::Generator;

use common::{
    array::{Array, NullableFixedSizedListArray},
    column::{
        label::{Label, LabelTypeMismatch, TryAsRef},
        Field,
    },
    expression::MatcherOp,
};
use croaring::Bitmap;
use executor::iter::Iterator;
use thiserror::Error;

use self::label::{BoolLabel, IPv4Label, IPv6Label, IntLabel, LabelColumn, StringLabel};

pub type UInt8Field = NullableFixedSizedListArray<u8>;
pub type UInt16Field = NullableFixedSizedListArray<u16>;
pub type UInt32Field = NullableFixedSizedListArray<u32>;
pub type UInt64Field = NullableFixedSizedListArray<u64>;
pub type Int8Field = NullableFixedSizedListArray<i8>;
pub type Int16Field = NullableFixedSizedListArray<i16>;
pub type Int32Field = NullableFixedSizedListArray<i32>;
pub type Int64Field = NullableFixedSizedListArray<i64>;
pub type Float32Field = NullableFixedSizedListArray<f32>;
pub type Float64Field = NullableFixedSizedListArray<f64>;
pub type BoolField = NullableFixedSizedListArray<bool>;

#[derive(Debug, Error)]
pub enum FilterError {
    #[error("regex match only supports string label")]
    RegexStringOnly,
    #[error("regex pattern error: {}", source)]
    PatternError {
        #[from]
        source: regex::Error,
    },
    #[error(transparent)]
    MismatchType {
        #[from]
        source: LabelTypeMismatch,
    },
}

#[derive(Debug, Clone)]
pub struct FieldColumn<A: Array> {
    array: A,
}

impl<A: Array> PartialEq for FieldColumn<A>
where
    for<'a> A::ItemRef<'a>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.array.iter().eq(other.array.iter())
    }
}

#[derive(Debug)]
pub struct LabelImpl(
    Label<
        LabelColumn<StringLabel>,
        LabelColumn<IPv4Label>,
        LabelColumn<IPv6Label>,
        LabelColumn<IntLabel>,
        LabelColumn<BoolLabel>,
    >,
);

impl LabelImpl {
    pub fn new(
        label: Label<
            LabelColumn<StringLabel>,
            LabelColumn<IPv4Label>,
            LabelColumn<IPv6Label>,
            LabelColumn<IntLabel>,
            LabelColumn<BoolLabel>,
        >,
    ) -> Self {
        Self(label)
    }

    pub fn filter<'s>(
        &'s self,
        matcher: &'s MatcherOp,
        superset: &'s mut Bitmap,
    ) -> Result<Box<dyn 's + Generator<(), Yield = (), Return = ()> + Unpin>, FilterError> {
        macro_rules! lookup {
            ($column:expr, $op:expr, $matcher:expr) => {{
                let matcher = match $matcher {
                    Some(matcher) => Some(matcher.try_as_ref()?),
                    None => None,
                };
                Ok(Box::new($column.lookup($op.positive(), matcher, superset)))
            }};
        }
        match matcher {
            op @ MatcherOp::LiteralEqual(matcher) | op @ MatcherOp::LiteralNotEqual(matcher) => {
                match &self.0 {
                    Label::String(col) => lookup!(col, op, matcher),
                    Label::IPv4(col) => lookup!(col, op, matcher),
                    Label::IPv6(col) => lookup!(col, op, matcher),
                    Label::Int(col) => lookup!(col, op, matcher),
                    Label::Bool(col) => lookup!(col, op, matcher),
                }
            }
            op @ MatcherOp::RegexMatch(matcher) | op @ MatcherOp::RegexNotMatch(matcher) => {
                match &self.0 {
                    Label::String(col) => col
                        .regex_match(op.positive(), matcher, superset)
                        .map(|g| Box::new(g) as Box<_>),
                    _ => Err(FilterError::RegexStringOnly),
                }
            }
        }
    }
}

pub type FieldImpl = Field<
    FieldColumn<UInt8Field>,
    FieldColumn<UInt16Field>,
    FieldColumn<UInt32Field>,
    FieldColumn<UInt64Field>,
    FieldColumn<Int8Field>,
    FieldColumn<Int16Field>,
    FieldColumn<Int32Field>,
    FieldColumn<Int64Field>,
    FieldColumn<Float32Field>,
    FieldColumn<Float64Field>,
    FieldColumn<BoolField>,
>;

#[cfg(test)]
mod tests {
    use std::{
        ops::{Generator, GeneratorState},
        pin::Pin,
    };

    use common::{column::label::Label, expression::MatcherOp};
    use croaring::Bitmap;

    use super::{LabelColumn, LabelImpl, StringLabel};
    use crate::mutable::index::IndexType;

    fn test_string_label() -> LabelColumn<StringLabel> {
        let mut column = LabelColumn::<StringLabel>::new(IndexType::Inverted(()));
        column.push(Some("test".as_ref()));
        column.push(None);
        column.push(Some("hello".as_ref()));
        column.push(Some("world".as_ref()));
        column.push(Some("hello".as_ref()));
        column
    }

    #[test]
    fn label_lookup_value() {
        let column = test_string_label();

        let mut result = Bitmap::from_range(0..=4);
        {
            let mut i = column.lookup(true, Some("hello".as_ref()), &mut result);
            loop {
                if let GeneratorState::Complete(_) = Pin::new(&mut i).resume(()) {
                    break;
                }
            }
        }
        assert_eq!(result, Bitmap::from_iter([2_u32, 4_u32]));

        let mut result = Bitmap::from_range(0..=4);
        {
            let mut i = column.lookup(true, Some("universe".as_ref()), &mut result);
            loop {
                if let GeneratorState::Complete(_) = Pin::new(&mut i).resume(()) {
                    break;
                }
            }
        }
        assert_eq!(result, Bitmap::create());

        let mut result = Bitmap::from_range(0..=4);
        {
            let mut i = column.lookup(true, None, &mut result);
            loop {
                if let GeneratorState::Complete(_) = Pin::new(&mut i).resume(()) {
                    break;
                }
            }
        }
        assert_eq!(result, Bitmap::from_iter([1_u32]));
    }

    #[test]
    fn label_impl() {
        let column = test_string_label();
        let limpl = LabelImpl::new(Label::String(column));
        let mut superset = Bitmap::from_range(0..6);
        let matcher = MatcherOp::LiteralEqual(Some(Label::String("hello".into())));
        {
            let mut round = 0;
            let mut i = limpl.filter(&matcher, &mut superset).unwrap();
            loop {
                round += 1;
                if let GeneratorState::Complete(_) = Pin::new(&mut i).resume(()) {
                    break;
                }
            }
            assert_eq!(round, 1);
        }
        assert_eq!(superset, Bitmap::from_iter([2_u32, 4_u32]));

        let mut superset = Bitmap::from_range(0..6);
        let matcher = MatcherOp::RegexNotMatch("he\\w+?".into());
        {
            let mut round = 0;
            let mut i = limpl.filter(&matcher, &mut superset).unwrap();
            loop {
                round += 1;
                if let GeneratorState::Complete(_) = Pin::new(&mut i).resume(()) {
                    break;
                }
            }
            assert_eq!(round, 6);
        }
        assert_eq!(superset, Bitmap::from_iter([0_u32, 3_u32]));
    }
}
