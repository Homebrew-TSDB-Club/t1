use std::{convert::Infallible, hash::Hash, unreachable};

use common::{
    array::{Array, ConstFixedSizedListArray, IdArray, ListArray, PrimitiveArray},
    column::label::Label,
    expression::MatcherOp,
};
use croaring::Bitmap;
use executor::iter::{Iterator, StdIter, Step};
use regex::Regex;

use super::FilterError;
use crate::mutable::index::{IndexImpl, IndexType};

pub trait AsStr {
    fn as_str(&self) -> &str;
}

impl AsStr for &[u8] {
    fn as_str(&self) -> &str {
        std::str::from_utf8(self).unwrap()
    }
}

pub type StringLabel = ListArray<u8>;
pub type IPv4Label = ConstFixedSizedListArray<u8, 4>;
pub type IPv6Label = ConstFixedSizedListArray<u8, 16>;
pub type IntLabel = PrimitiveArray<i64>;
pub type BoolLabel = PrimitiveArray<bool>;

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

impl<A: Array + std::fmt::Debug> LabelColumn<A>
where
    for<'a, 'b> A::ItemRef<'a>: PartialEq<A::ItemRef<'b>>,
    for<'a> A::ItemRef<'a>: Hash + AsStr,
{
    pub(crate) fn regex_match<'s>(
        &'s self,
        positive: bool,
        pattern: &str,
        superset: &'s mut Bitmap,
    ) -> Result<impl Iterator<'s>, FilterError> {
        let regex = Regex::new(pattern)?;
        Ok(self
            .array
            .iter()
            .enumerate()
            .filter(move |(_, item)| {
                if let Some(item) = item {
                    !(positive ^ regex.is_match(item.as_str()))
                } else {
                    false
                }
            })
            .map(|(id, _)| id as u32)
            .fold(Bitmap::create(), |set, row_id| {
                set.add(row_id);
            })
            .and_then(|set| superset.and_inplace(&set)))
    }
}

#[derive(Debug)]
pub enum LookupIter<I> {
    Exactly,
    Approximately { iter: I },
}

impl<'iter, I: Iterator<'iter, Return = Bitmap, Error = Infallible>> Iterator<'iter>
    for LookupIter<I>
{
    type Item = I::Item;
    type Return = Option<Bitmap>;
    type Error = Infallible;

    fn next(&mut self) -> Step<Self::Item, Result<Self::Return, Self::Error>> {
        match self {
            LookupIter::Exactly => Step::Done(Ok(None)),
            LookupIter::Approximately { iter } => match iter.next() {
                Step::NotYet => Step::NotYet,
                Step::Ready(ready) => Step::Ready(ready),
                Step::Done(done) => Step::Done(done.map(Some)),
            },
        }
    }
}

impl<A: Array> LabelColumn<A>
where
    for<'a, 'b> A::ItemRef<'a>: PartialEq<A::ItemRef<'b>>,
    for<'a> A::ItemRef<'a>: Hash,
{
    pub(crate) fn lookup<'s>(
        &'s self,
        positive: bool,
        value: Option<A::ItemRef<'s>>,
        superset: &'s mut Bitmap,
    ) -> impl Iterator<'s, Return = Option<Bitmap>, Error = Infallible> {
        let id = value
            .as_ref()
            .map(|v| self.array.lookup_id(v))
            .unwrap_or(Some(0));
        match id {
            Some(id) => self.index.lookup(&id, |set| {
                if positive {
                    superset.and_inplace(set)
                } else {
                    superset.andnot_inplace(set)
                }
            }),
            None => superset.clear(),
        }

        if !self.index.exactly() {
            LookupIter::Approximately {
                iter: StdIter::from(superset.iter())
                    .filter(
                        move |row_id| match self.array.get_unchecked(*row_id as usize) {
                            Some(item) => match &value {
                                Some(value) => !(positive ^ (*value == item)),
                                None => false,
                            },
                            None => match value {
                                Some(_) => false,
                                None => true,
                            },
                        },
                    )
                    .fold(Bitmap::create(), |superset, row_id| {
                        superset.add(row_id);
                    }),
            }
        } else {
            LookupIter::Exactly
        }
    }

    pub fn push(&mut self, value: Option<A::ItemRef<'_>>) {
        let id = self.array.push_and_get_id(value);
        self.index.insert(self.array.len() - 1, id);
    }
}

pub type Impl = Label<
    LabelColumn<StringLabel>,
    LabelColumn<IPv4Label>,
    LabelColumn<IPv6Label>,
    LabelColumn<IntLabel>,
    LabelColumn<BoolLabel>,
>;

#[derive(Debug)]
pub struct LabelImpl(Impl);

#[derive(Debug)]
pub enum FilterIter<I1, I2, I3, I4, I5, I6> {
    Regex(I1),
    Lookup(Label<I2, I3, I4, I5, I6>),
}

impl<'iter, I1, I2, I3, I4, I5, I6> Iterator<'iter> for FilterIter<I1, I2, I3, I4, I5, I6>
where
    I1: Iterator<'iter>,
    I2: Iterator<'iter, Return = Option<Bitmap>, Error = Infallible>,
    I3: Iterator<'iter, Return = Option<Bitmap>, Error = Infallible>,
    I4: Iterator<'iter, Return = Option<Bitmap>, Error = Infallible>,
    I5: Iterator<'iter, Return = Option<Bitmap>, Error = Infallible>,
    I6: Iterator<'iter, Return = Option<Bitmap>, Error = Infallible>,
{
    type Item = ();
    type Return = Option<Bitmap>;
    type Error = FilterError;

    fn next(&mut self) -> Step<Self::Item, Result<Self::Return, Self::Error>> {
        macro_rules! label_iter {
            ($iter:expr) => {
                match $iter.next() {
                    Step::NotYet => Step::NotYet,
                    Step::Ready(_) => unreachable!(),
                    Step::Done(done) => Step::Done(Ok(done.unwrap())),
                }
            };
        }

        match self {
            FilterIter::Regex(iter) => match iter.next() {
                Step::NotYet => Step::NotYet,
                Step::Ready(_) => unreachable!(),
                Step::Done(_) => Step::Done(Ok(None)),
            },
            FilterIter::Lookup(iter) => match iter {
                Label::String(iter) => label_iter!(iter),
                Label::IPv4(iter) => label_iter!(iter),
                Label::IPv6(iter) => label_iter!(iter),
                Label::Int(iter) => label_iter!(iter),
                Label::Bool(iter) => label_iter!(iter),
            },
        }
    }
}

impl From<Impl> for LabelImpl {
    fn from(value: Impl) -> Self {
        Self(value)
    }
}

impl LabelImpl {
    pub fn filter<'s>(
        &'s self,
        matcher: &'s MatcherOp,
        superset: &'s mut Bitmap,
    ) -> Result<
        FilterIter<
            impl Iterator<'s>,
            impl Iterator<'s, Return = Option<Bitmap>, Error = Infallible>,
            impl Iterator<'s, Return = Option<Bitmap>, Error = Infallible>,
            impl Iterator<'s, Return = Option<Bitmap>, Error = Infallible>,
            impl Iterator<'s, Return = Option<Bitmap>, Error = Infallible>,
            impl Iterator<'s, Return = Option<Bitmap>, Error = Infallible>,
        >,
        FilterError,
    > {
        use common::column::label::TryAsRef;

        macro_rules! lookup {
            ($column:expr, $op:expr, $matcher:expr, $label_type:expr) => {{
                let matcher = match $matcher {
                    Some(matcher) => Some(matcher.try_as_ref()?),
                    None => None,
                };
                Ok(FilterIter::Lookup($label_type($column.lookup(
                    $op.positive(),
                    matcher,
                    superset,
                ))))
            }};
        }

        match matcher {
            op @ MatcherOp::LiteralEqual(matcher) | op @ MatcherOp::LiteralNotEqual(matcher) => {
                match &self.0 {
                    Label::String(col) => lookup!(col, op, matcher, Label::String),
                    Label::IPv4(col) => lookup!(col, op, matcher, Label::IPv4),
                    Label::IPv6(col) => lookup!(col, op, matcher, Label::IPv6),
                    Label::Int(col) => lookup!(col, op, matcher, Label::Int),
                    Label::Bool(col) => lookup!(col, op, matcher, Label::Bool),
                }
            }
            op @ MatcherOp::RegexMatch(matcher) | op @ MatcherOp::RegexNotMatch(matcher) => {
                match &self.0 {
                    Label::String(col) => col
                        .regex_match(op.positive(), matcher, superset)
                        .map(FilterIter::Regex),
                    _ => Err(FilterError::RegexStringOnly),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use common::{column::label::Label, expression::MatcherOp};
    use croaring::Bitmap;
    use executor::iter::{Iterator, Step};

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
                if let Step::Done(_) = i.next() {
                    break;
                }
            }
        }
        assert_eq!(result, Bitmap::from_iter([2_u32, 4_u32]));

        let mut result = Bitmap::from_range(0..=4);
        {
            let mut i = column.lookup(true, Some("universe".as_ref()), &mut result);
            loop {
                if let Step::Done(_) = i.next() {
                    break;
                }
            }
        }
        assert_eq!(result, Bitmap::create());

        let mut result = Bitmap::from_range(0..=4);
        {
            let mut i = column.lookup(true, None, &mut result);
            loop {
                if let Step::Done(_) = i.next() {
                    break;
                }
            }
        }
        assert_eq!(result, Bitmap::from_iter([1_u32]));
    }

    #[test]
    fn label_impl() {
        let column = test_string_label();
        let limpl = LabelImpl::from(Label::String(column));
        let mut superset = Bitmap::from_range(0..6);
        let matcher = MatcherOp::LiteralEqual(Some(Label::String("hello".into())));
        {
            let mut round = 0;
            let mut i = limpl.filter(&matcher, &mut superset).unwrap();
            loop {
                round += 1;
                if let Step::Done(_) = i.next() {
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
                if let Step::Done(_) = i.next() {
                    break;
                }
            }
            assert_eq!(round, 6);
        }
        assert_eq!(superset, Bitmap::from_iter([0_u32, 3_u32]));
    }
}
