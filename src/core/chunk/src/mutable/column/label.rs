use std::hash::Hash;

use common::{
    array::{
        fixed::ConstFixedListArray, id::IdArray, list::ListArray, primitive::PrimitiveArray, Array,
    },
    column::label::{AnyValue, Label, LabelType, LabelValue},
    context::Context,
    query::MatcherOp,
    scalar::{Scalar, ScalarRef},
    try_yield,
};
use croaring::Bitmap;
use paste::paste;
use regex::Regex;

use super::FilterError;

pub trait AsStr {
    fn as_str(&self) -> &str;
}

impl AsStr for &[u8] {
    #[inline]
    fn as_str(&self) -> &str {
        std::str::from_utf8(self).unwrap()
    }
}

pub type StringLabel = ListArray<u8>;
pub type IPv4Label = ConstFixedListArray<u8, 4>;
pub type IPv6Label = ConstFixedListArray<u8, 16>;
pub type IntLabel = PrimitiveArray<i64>;
pub type BoolLabel = PrimitiveArray<bool>;

#[derive(Debug)]
pub struct LabelColumn<A> {
    array: IdArray<A>,
}

impl<A> From<Vec<Option<A::Item>>> for LabelColumn<A>
where
    A: Array + Default,
    for<'a, 'b> A::ItemRef<'a>: PartialEq<A::ItemRef<'b>>,
    for<'a> A::ItemRef<'a>: Hash,
{
    #[inline]
    fn from(v: Vec<Option<A::Item>>) -> Self {
        let mut column = Self::new();
        for item in v {
            column.push(item);
        }
        column
    }
}

impl<A: Array + Default> Default for LabelColumn<A> {
    #[inline]
    fn default() -> Self {
        Self {
            array: IdArray::<A>::new(A::default()),
        }
    }
}

impl<A: Array + Default> LabelColumn<A> {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn with_capacity(capacity: usize, array: A) -> Self {
        Self {
            array: IdArray::<A>::with_capacity(capacity, array),
        }
    }
}

impl<A: Array> LabelColumn<A>
where
    for<'a, 'b> A::ItemRef<'a>: PartialEq<A::ItemRef<'b>>,
    for<'a> A::ItemRef<'a>: Hash + AsStr,
{
    pub(crate) async fn regex_match(
        &self,
        cx: &mut Context,
        positive: bool,
        pattern: &str,
        superset: &mut Bitmap,
    ) -> Result<(), FilterError> {
        let regex = Regex::new(pattern)?;
        let mut set = Bitmap::create();

        for row_id in superset.iter() {
            if let Some(item) = unsafe { self.array.get_unchecked(row_id as usize) } {
                if !(positive ^ regex.is_match(item.as_str())) {
                    set.add(row_id);
                }
            }
            try_yield!(cx);
        }

        *superset = set;
        Ok(())
    }
}

impl<A: Array> LabelColumn<A>
where
    for<'a, 'b> A::ItemRef<'a>: PartialEq<A::ItemRef<'b>>,
    for<'a> A::ItemRef<'a>: Hash,
{
    #[inline]
    pub fn push(&mut self, item: Option<A::Item>) {
        self.array.push(item)
    }

    pub(crate) async fn lookup<'s>(
        &'s self,
        cx: &mut Context,
        positive: bool,
        value: Option<A::ItemRef<'s>>,
        superset: &mut Bitmap,
    ) {
        let mut set = Bitmap::create();
        for row_id in superset.iter() {
            if !(positive ^ (unsafe { self.array.get_unchecked(row_id as usize) } == value)) {
                set.add(row_id);
            }
            try_yield!(cx);
        }

        *superset = set;
    }
}

pub type ArrayImpl = Label<
    LabelColumn<StringLabel>,
    LabelColumn<IPv4Label>,
    LabelColumn<IPv6Label>,
    LabelColumn<IntLabel>,
    LabelColumn<BoolLabel>,
>;

#[derive(Debug)]
pub struct LabelImpl(ArrayImpl);

impl From<ArrayImpl> for LabelImpl {
    fn from(value: ArrayImpl) -> Self {
        Self(value)
    }
}

impl LabelImpl {
    pub fn push(&mut self, value: Option<LabelValue>) -> usize {
        macro_rules! push {
            ($($label_type:ident), *) => {
                paste! {
                match &mut self.0 {
                    $(
                    Label::$label_type(column) => {
                        match value {
                            Some(value) => match value {
                                Label::$label_type(v) => column.array.push_and_get_id(Some(v)),
                                others => panic!(
                                    "expect type: {}, found: {}",
                                    LabelType::from(Label::$label_type(())),
                                    others.r#type()
                                ),
                            },
                            None => column.array.push_and_get_id(None),
                        }
                    }
                    )*
                }
                }
            };
        }

        push!(String, IPv4, IPv6, Int, Bool)
    }

    pub unsafe fn lookup_value_id_unchecked(&self, matcher: &Option<AnyValue>) -> Option<usize> {
        macro_rules! lookup_value_id {
            ($($label_type:ident), * => $op:expr, $matcher:expr) => {
                paste! {
                match &self.0 {
                    $(
                    Label::$label_type(column) => {
                        $matcher
                            .as_ref()
                            .map(|matcher| Scalar::as_ref(matcher.cast::<<[<$label_type Label>] as Array>::Item>()))
                            .map(|v| column.array.lookup_id(&v))
                            .unwrap_or(Some(0))
                    }
                    )*
                }
                }
            };
        }

        lookup_value_id!(String, IPv4, IPv6, Int, Bool => op, matcher)
    }

    #[inline]
    pub fn len(&self) -> usize {
        macro_rules! len {
            ($($label_type:ident), *) => {
                paste! {
                match &self.0 {
                    $(Label::$label_type(column) => column.array.len(),)*
                }
                }
            };
        }

        len!(String, IPv4, IPv6, Int, Bool)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub async unsafe fn filter(
        &self,
        cx: &mut Context,
        matcher: &MatcherOp<AnyValue>,
        superset: &mut Bitmap,
    ) -> Result<(), FilterError> {
        macro_rules! label_lookup {
            ($($label_type:ident), * => $op:expr, $matcher:expr) => {
                paste! {
                match &self.0 {
                    $(
                    Label::$label_type(column) => {
                        let matcher = $matcher
                            .as_ref()
                            .map(|matcher| Scalar::as_ref(matcher.cast::<<[<$label_type Label>] as Array>::Item>()));
                        column.lookup(cx, $op.positive(), matcher, superset).await;
                        Ok(())
                    }
                    )*
                }
                }
            };
        }

        match matcher {
            op @ MatcherOp::LiteralEqual(matcher) | op @ MatcherOp::LiteralNotEqual(matcher) => {
                label_lookup!(String, IPv4, IPv6, Int, Bool => op, matcher)
            }
            op @ MatcherOp::RegexMatch(matcher) | op @ MatcherOp::RegexNotMatch(matcher) => {
                match &self.0 {
                    Label::String(column) => {
                        column
                            .regex_match(cx, op.positive(), matcher, superset)
                            .await
                    }
                    _ => Err(FilterError::RegexStringOnly),
                }
            }
        }
    }

    pub fn map(&self, row_set: &Bitmap) -> Self {
        macro_rules! map {
            ($($label_type:ident), *) => {
                paste! {
                match &self.0 {
                    $(
                    Label::$label_type(column) => {
                        let len = row_set.cardinality() as usize;
                        let mut new = LabelColumn::with_capacity(len, [<$label_type Label>]::with_capacity(len));
                        for id in row_set.iter() {
                            new.push(
                                unsafe {
                                    column
                                    .array
                                    .get_unchecked(id as usize)
                                    .map(ScalarRef::to_owned)
                                },
                            );
                        }
                        Self(Label::$label_type(new))
                    }
                    )*
                }
                }
            };
        }

        map!(String, IPv4, IPv6, Int, Bool)
    }
}
#[cfg(test)]
mod tests {

    use common::context::Context;
    use croaring::Bitmap;

    use super::{LabelColumn, StringLabel};

    #[test]
    fn label_lookup_value() {
        futures_lite::future::block_on(async {
            let column = LabelColumn::<StringLabel>::from(vec![
                Some("test".into()),
                None,
                Some("hello".into()),
                Some("world".into()),
                Some("hello".into()),
            ]);
            let mut cx = Context::new(256);

            let mut result = Bitmap::from_range(0..5);
            column
                .lookup(&mut cx, true, Some("hello".as_ref()), &mut result)
                .await;
            assert_eq!(result, Bitmap::from_iter([2_u32, 4_u32]));

            let mut result = Bitmap::from_range(0..5);
            column
                .lookup(&mut cx, true, Some("universe".as_ref()), &mut result)
                .await;
            assert_eq!(result, Bitmap::create());

            let mut result = Bitmap::from_range(0..5);
            column.lookup(&mut cx, true, None, &mut result).await;
            assert_eq!(result, Bitmap::from_iter([1_u32]));

            let mut result = Bitmap::from_range(0..5);
            column
                .regex_match(&mut cx, true, "\\w+?", &mut result)
                .await
                .unwrap();
            assert_eq!(result, Bitmap::from_iter([0, 2, 3, 4]));
        })
    }
}
