use std::{
    hash::Hash,
    ops::{Generator, GeneratorState},
    pin::Pin,
    unreachable,
};

use common::{
    array::{Array, ConstFixedSizedListArray, IdArray, ListArray, PrimitiveArray},
    column::label::{Label, LabelType, LabelValue},
    expression::MatcherOp,
};
use croaring::Bitmap;
use executor::iter::{Iterator, IteratorFusion, Step};
use regex::Regex;

use super::FilterError;

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

pub type ItemRefImpl<'r> = Label<
    <StringLabel as Array>::ItemRef<'r>,
    <IPv4Label as Array>::ItemRef<'r>,
    <IPv6Label as Array>::ItemRef<'r>,
    <IntLabel as Array>::ItemRef<'r>,
    <BoolLabel as Array>::ItemRef<'r>,
>;

#[derive(Debug, Clone)]
pub struct LabelColumn<A> {
    array: IdArray<A>,
}

impl<A: Array + Default> LabelColumn<A> {
    pub fn new() -> Self {
        Self {
            array: IdArray::<A>::new(A::default()),
        }
    }
}

impl<A: Array> LabelColumn<A>
where
    for<'a, 'b> A::ItemRef<'a>: PartialEq<A::ItemRef<'b>>,
    for<'a> A::ItemRef<'a>: Hash + AsStr,
{
    pub(crate) fn regex_match<'s>(
        &'s self,
        positive: bool,
        pattern: &'s str,
        superset: &'s mut Bitmap,
    ) -> Result<impl 's + Generator<Yield = (), Return = ()> + Unpin, FilterError> {
        let regex = Regex::new(pattern)?;
        Ok(move || {
            let mut iter = self
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
                .and_then(|set| superset.and_inplace(&set));
            loop {
                match iter.next() {
                    Step::NotYet => yield,
                    Step::Ready(_) => unreachable!(),
                    Step::Done(done) => return done,
                }
            }
        })
    }
}

impl<A: Array> LabelColumn<A>
where
    for<'a, 'b> A::ItemRef<'a>: PartialEq<A::ItemRef<'b>>,
    for<'a> A::ItemRef<'a>: Hash,
{
    pub fn push(&mut self, item: Option<A::ItemRef<'_>>) {
        self.array.push(item)
    }

    pub(crate) fn lookup<'s>(
        &'s self,
        positive: bool,
        value: Option<A::ItemRef<'s>>,
        superset: &'s mut Bitmap,
    ) -> impl 's + Generator<Yield = (), Return = ()> + Unpin {
        move || {
            let mut iter = superset
                .iter()
                .fusion()
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
                });
            loop {
                match iter.next() {
                    Step::NotYet => yield,
                    Step::Ready(_) => unreachable!(),
                    Step::Done(set) => {
                        *superset = set;
                        return;
                    }
                }
            }
        }
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
    pub fn push(&mut self, value: Option<ItemRefImpl<'_>>) -> usize {
        use paste::paste;

        macro_rules! push {
            ($($label_type:ident), *) => {
                paste! {
                        match &mut self.0 {
                        $(
                            Label::$label_type(col) => {
                                match value {
                                    Some(value) => match value {
                                        Label::$label_type(v) => col.array.push_and_get_id(Some(v)),
                                        v @ _ => panic!(
                                            "type of pushing value does not match column, expect {} found {}",
                                            LabelType::from(Label::$label_type(())),
                                            v.r#type()
                                        ),
                                    },
                                    None => col.array.push_and_get_id(None),
                                }
                            }
                        )*
                    }
                }
            };
        }

        push!(String, IPv4, IPv6, Int, Bool)
    }

    pub fn lookup_value_id(
        &self,
        matcher: &Option<LabelValue>,
    ) -> Result<Option<usize>, FilterError> {
        use common::column::label::TryAsRef;
        use paste::paste;

        macro_rules! lookup_value_id {
            ($($label_type:ident), * => $op:expr, $matcher:expr) => {
                paste! {
                    match &self.0 {
                        $(
                            Label::$label_type(col) => {
                                let matcher = match $matcher {
                                    Some(matcher) => Some(matcher.try_as_ref()?),
                                    None => None,
                                };
                                matcher
                                    .as_ref()
                                    .map(|v| col.array.lookup_id(v))
                                    .unwrap_or(Some(0))
                            }
                        )*
                    }
                }
            };
        }

        Ok(lookup_value_id!(String, IPv4, IPv6, Int, Bool => op, matcher))
    }

    pub fn len(&self) -> usize {
        use paste::paste;

        macro_rules! len {
            ($($label_type:ident), *) => {
                paste! {
                        match &self.0 {
                        $(
                            Label::$label_type(col) => col.array.len(),
                        )*
                    }
                }
            };
        }

        len!(String, IPv4, IPv6, Int, Bool)
    }

    pub fn filter<'s>(
        &'s self,
        matcher: &'s MatcherOp,
        superset: &'s mut Bitmap,
    ) -> Result<
        FilterGen<
            impl 's + Generator<Yield = (), Return = ()> + Unpin,
            impl 's + Generator<Yield = (), Return = ()> + Unpin,
            impl 's + Generator<Yield = (), Return = ()> + Unpin,
            impl 's + Generator<Yield = (), Return = ()> + Unpin,
            impl 's + Generator<Yield = (), Return = ()> + Unpin,
            impl 's + Generator<Yield = (), Return = ()> + Unpin,
        >,
        FilterError,
    > {
        use common::column::label::TryAsRef;
        use paste::paste;

        macro_rules! label_lookup {
            ($($label_type:ident), * => $op:expr, $matcher:expr) => {
                paste! {
                    match &self.0 {
                        $(
                            Label::$label_type(col) => {
                                let matcher = match $matcher {
                                    Some(matcher) => Some(matcher.try_as_ref()?),
                                    None => None,
                                };

                                Ok(FilterGen::Lookup(Label::$label_type(col.lookup(
                                    $op.positive(),
                                    matcher,
                                    superset,
                                ))))
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
                    Label::String(col) => Ok(FilterGen::Regex(col.regex_match(
                        op.positive(),
                        matcher,
                        superset,
                    )?)),
                    _ => Err(FilterError::RegexStringOnly),
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum FilterGen<I1, I2, I3, I4, I5, I6> {
    Regex(I1),
    Lookup(Label<I2, I3, I4, I5, I6>),
}

impl<I1, I2, I3, I4, I5, I6> Generator for FilterGen<I1, I2, I3, I4, I5, I6>
where
    I1: Generator<Yield = (), Return = ()> + Unpin,
    I2: Generator<Yield = (), Return = ()> + Unpin,
    I3: Generator<Yield = (), Return = ()> + Unpin,
    I4: Generator<Yield = (), Return = ()> + Unpin,
    I5: Generator<Yield = (), Return = ()> + Unpin,
    I6: Generator<Yield = (), Return = ()> + Unpin,
{
    type Yield = ();
    type Return = ();

    fn resume(self: Pin<&mut Self>, arg: ()) -> GeneratorState<Self::Yield, Self::Return> {
        match self.get_mut() {
            FilterGen::Regex(iter) => Pin::new(iter).resume(arg),
            FilterGen::Lookup(iter) => match iter {
                Label::String(iter) => match Pin::new(iter).resume(arg) {
                    GeneratorState::Yielded(yielded) => GeneratorState::Yielded(yielded),
                    GeneratorState::Complete(_) => GeneratorState::Complete(()),
                },
                Label::IPv4(iter) => match Pin::new(iter).resume(arg) {
                    GeneratorState::Yielded(yielded) => GeneratorState::Yielded(yielded),
                    GeneratorState::Complete(_) => GeneratorState::Complete(()),
                },
                Label::IPv6(iter) => match Pin::new(iter).resume(arg) {
                    GeneratorState::Yielded(yielded) => GeneratorState::Yielded(yielded),
                    GeneratorState::Complete(_) => GeneratorState::Complete(()),
                },
                Label::Int(iter) => match Pin::new(iter).resume(arg) {
                    GeneratorState::Yielded(yielded) => GeneratorState::Yielded(yielded),
                    GeneratorState::Complete(_) => GeneratorState::Complete(()),
                },
                Label::Bool(iter) => match Pin::new(iter).resume(arg) {
                    GeneratorState::Yielded(yielded) => GeneratorState::Yielded(yielded),
                    GeneratorState::Complete(_) => GeneratorState::Complete(()),
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        ops::{Generator, GeneratorState},
        pin::Pin,
    };

    use common::array::ListArray;
    use croaring::Bitmap;

    use super::{LabelColumn, StringLabel};

    fn test_string_label() -> LabelColumn<ListArray<u8>> {
        let mut column = LabelColumn::<StringLabel>::new();
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

        let mut result = Bitmap::from_range(0..5);
        {
            let mut i = column.lookup(true, Some("hello".as_ref()), &mut result);
            loop {
                if let GeneratorState::Complete(_) = Pin::new(&mut i).resume(()) {
                    break;
                }
            }
        }
        assert_eq!(result, Bitmap::from_iter([2_u32, 4_u32]));

        let mut result = Bitmap::from_range(0..5);
        {
            let mut i = column.lookup(true, Some("universe".as_ref()), &mut result);
            loop {
                if let GeneratorState::Complete(_) = Pin::new(&mut i).resume(()) {
                    break;
                }
            }
        }
        assert_eq!(result, Bitmap::create());

        let mut result = Bitmap::from_range(0..5);
        {
            let mut i = column.lookup(true, None, &mut result);
            loop {
                if let GeneratorState::Complete(_) = Pin::new(&mut i).resume(()) {
                    break;
                }
            }
        }
        assert_eq!(result, Bitmap::from_iter([1_u32]));

        let mut result = Bitmap::from_range(0..5);
        {
            let mut i = column.regex_match(true, "\\w+?", &mut result).unwrap();
            loop {
                if let GeneratorState::Complete(_) = Pin::new(&mut i).resume(()) {
                    break;
                }
            }
        }
        assert_eq!(result, Bitmap::from_iter([0, 2, 3, 4]));
    }
}
