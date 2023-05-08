use std::hash::Hash;

use common::{
    array::{
        Array, ConstFixedSizedListArray, IdArray, ListArray, NullableFixedSizedListArray,
        PrimitiveArray,
    },
    column::{Field, Label, LabelType},
    expression::MatcherOp,
};
use croaring::Bitmap;
use regex::Regex;
use thiserror::Error;

use super::index::{IndexImpl, IndexType};

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

pub type StringLabel = ListArray<u8>;
pub type IPv4Label = ConstFixedSizedListArray<u8, 4>;
pub type IPv6Label = ConstFixedSizedListArray<u8, 16>;
pub type IntLabel = PrimitiveArray<i64>;
pub type BoolLabel = PrimitiveArray<bool>;

pub trait IntoStr {
    fn into_str(&self) -> &str;
}

impl IntoStr for &[u8] {
    fn into_str(&self) -> &str {
        std::str::from_utf8(self).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct LabelColumn<A> {
    array: IdArray<A>,
    index: IndexImpl<usize>,
}

impl<A: Array + Default> LabelColumn<A> {
    pub fn new(index: IndexType<(), u32>) -> Self {
        Self {
            array: IdArray::<A>::new(A::default()),
            index: IndexImpl::new(index),
        }
    }
}

#[derive(Debug, Error)]
pub enum LookupError {
    #[error("regex match only supports string label")]
    RegexStringOnly,
    #[error("regex pattern error: {}", source)]
    PatternError {
        #[from]
        source: regex::Error,
    },
    #[error(
        "mismatch type between value with column, expect {}, found {}",
        expect,
        found
    )]
    MismatchType { expect: LabelType, found: LabelType },
}

impl<A: Array> PartialEq for LabelColumn<A>
where
    for<'a, 'b> A::ItemRef<'a>: PartialEq<A::ItemRef<'b>>,
    for<'a> A::ItemRef<'a>: Hash,
{
    fn eq(&self, other: &Self) -> bool {
        self.array.iter().eq(other.array.iter())
    }
}

impl<A: Array> LabelColumn<A>
where
    for<'a, 'b> A::ItemRef<'a>: PartialEq<A::ItemRef<'b>>,
    for<'a> A::ItemRef<'a>: Hash + IntoStr,
{
    fn regex_match(
        &self,
        should_match: bool,
        pattern: &str,
        superset: &mut Bitmap,
    ) -> Result<(), LookupError> {
        let regex = Regex::new(pattern)?;
        let set = self
            .array
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                if let Some(item) = item {
                    !(should_match ^ regex.is_match(item.into_str()))
                } else {
                    false
                }
            })
            .map(|(id, _)| id as u32)
            .collect();
        superset.and_inplace(&set);
        Ok(())
    }
}

impl<A: Array> LabelColumn<A>
where
    for<'a, 'b> A::ItemRef<'a>: PartialEq<A::ItemRef<'b>>,
    for<'a> A::ItemRef<'a>: Hash,
{
    fn lookup<'s: 'value, 'value>(
        &'s self,
        should_equal: bool,
        value: Option<A::ItemRef<'value>>,
        superset: &mut Bitmap,
    ) {
        let id = value
            .as_ref()
            .map(|v| self.array.lookup_id(v))
            .unwrap_or(Some(0));
        match id {
            Some(id) => self.index.lookup(&id, |set| {
                if should_equal {
                    superset.and_inplace(set)
                } else {
                    superset.andnot_inplace(set)
                }
            }),
            None => superset.clear(),
        }

        if !self.index.exactly() {
            *superset = superset
                .iter()
                .filter(|item| {
                    !(should_equal ^ (self.array.get_unchecked(*item as usize) == value))
                })
                .collect()
        }
    }

    pub fn push(&mut self, value: Option<A::ItemRef<'_>>) {
        let id = self.array.push_and_get_id(value);
        self.index.insert(self.array.len() - 1, id);
    }
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

    pub fn lookup(&self, matcher: MatcherOp, superset: &mut Bitmap) -> Result<(), LookupError> {
        match &self.0 {
            Label::String(column) => match &matcher {
                op @ (MatcherOp::LiteralEqual(matcher) | MatcherOp::LiteralNotEqual(matcher)) => {
                    let matcher = match matcher {
                        Some(matcher) => match matcher {
                            Label::String(s) => Some(s.as_ref()),
                            m @ _ => {
                                return Err(LookupError::MismatchType {
                                    expect: self.0.r#type(),
                                    found: m.r#type(),
                                });
                            }
                        },
                        None => None,
                    };
                    Ok(column.lookup(op.positive(), matcher, superset))
                }
                op @ (MatcherOp::RegexMatch(matcher) | MatcherOp::RegexNotMatch(matcher)) => {
                    column.regex_match(op.positive(), &matcher, superset)
                }
            },
            Label::IPv4(column) => match &matcher {
                op @ (MatcherOp::LiteralEqual(matcher) | MatcherOp::LiteralNotEqual(matcher)) => {
                    let matcher = match matcher {
                        Some(matcher) => match matcher {
                            Label::IPv4(s) => Some(s.octets()),
                            m @ _ => {
                                return Err(LookupError::MismatchType {
                                    expect: self.0.r#type(),
                                    found: m.r#type(),
                                });
                            }
                        },
                        None => None,
                    };
                    Ok(column.lookup(op.positive(), matcher.as_ref(), superset))
                }
                _ => return Err(LookupError::RegexStringOnly),
            },
            Label::IPv6(column) => match &matcher {
                op @ (MatcherOp::LiteralEqual(matcher) | MatcherOp::LiteralNotEqual(matcher)) => {
                    let matcher = match matcher {
                        Some(matcher) => match matcher {
                            Label::IPv6(s) => Some(s.octets()),
                            m @ _ => {
                                return Err(LookupError::MismatchType {
                                    expect: self.0.r#type(),
                                    found: m.r#type(),
                                });
                            }
                        },
                        None => None,
                    };
                    Ok(column.lookup(op.positive(), matcher.as_ref(), superset))
                }
                _ => return Err(LookupError::RegexStringOnly),
            },
            Label::Int(column) => match &matcher {
                op @ (MatcherOp::LiteralEqual(matcher) | MatcherOp::LiteralNotEqual(matcher)) => {
                    let matcher = match matcher {
                        Some(matcher) => match matcher {
                            Label::Int(s) => Some(*s),
                            m @ _ => {
                                return Err(LookupError::MismatchType {
                                    expect: self.0.r#type(),
                                    found: m.r#type(),
                                });
                            }
                        },
                        None => None,
                    };
                    Ok(column.lookup(op.positive(), matcher.as_ref(), superset))
                }
                _ => return Err(LookupError::RegexStringOnly),
            },
            Label::Bool(column) => match &matcher {
                op @ (MatcherOp::LiteralEqual(matcher) | MatcherOp::LiteralNotEqual(matcher)) => {
                    let matcher = match matcher {
                        Some(matcher) => match matcher {
                            Label::Bool(s) => Some(*s),
                            m @ _ => {
                                return Err(LookupError::MismatchType {
                                    expect: self.0.r#type(),
                                    found: m.r#type(),
                                });
                            }
                        },
                        None => None,
                    };
                    Ok(column.lookup(op.positive(), matcher.as_ref(), superset))
                }
                _ => return Err(LookupError::RegexStringOnly),
            },
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
    use common::{column::Label, expression::MatcherOp};
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
        column.lookup(true, Some("hello".as_ref()), &mut result);
        assert_eq!(result, Bitmap::from_iter([2_u32, 4_u32]));

        let mut result = Bitmap::from_range(0..=4);
        column.lookup(true, Some("universe".as_ref()), &mut result);
        assert_eq!(result, Bitmap::create());

        let mut result = Bitmap::from_range(0..=4);
        column.lookup(true, None, &mut result);
        assert_eq!(result, Bitmap::from_iter([1_u32]));
    }

    #[test]
    fn label_impl() {
        let column = test_string_label();
        let limpl = LabelImpl::new(Label::String(column));
        let mut superset = Bitmap::from_range(0..6);
        limpl
            .lookup(
                MatcherOp::LiteralEqual(Some(Label::String("hello".into()))),
                &mut superset,
            )
            .unwrap();
        assert_eq!(superset, Bitmap::from_iter([2_u32, 4_u32]));

        let mut superset = Bitmap::from_range(0..6);
        limpl
            .lookup(MatcherOp::RegexNotMatch("he\\w+?".into()), &mut superset)
            .unwrap();
        assert_eq!(superset, Bitmap::from_iter([0_u32, 3_u32]));
    }
}
