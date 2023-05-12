use std::{hash::Hash, ops::Generator};

use common::{
    array::{Array, ConstFixedSizedListArray, IdArray, ListArray, PrimitiveArray},
    expression::MatcherOp,
};
use croaring::Bitmap;
use executor::iter::{Iterator, Step};
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
    ) -> Result<impl 's + Generator<(), Yield = (), Return = ()>, FilterError> {
        let regex = Regex::new(pattern)?;
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
            .map(|(id, _)| id as u32);

        Ok(move || {
            let mut set: Bitmap = Bitmap::create();
            loop {
                match iter.next() {
                    Step::Ready(row_id) => {
                        yield set.add(row_id);
                    }
                    Step::NotYet => yield,
                    Step::Done => {
                        return superset.and_inplace(&set);
                    }
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
    pub(crate) fn lookup<'s>(
        &'s self,
        positive: bool,
        value: Option<A::ItemRef<'s>>,
        superset: &'s mut Bitmap,
    ) -> impl 's + Generator<(), Yield = (), Return = ()> {
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

        move || {
            if !self.index.exactly() {
                let mut iter = superset.iter().filter(move |row_id| {
                    match self.array.get_unchecked(*row_id as usize) {
                        Some(item) => match &value {
                            Some(value) => !(positive ^ (*value == item)),
                            None => false,
                        },
                        None => match value {
                            Some(_) => false,
                            None => true,
                        },
                    }
                });
                let mut set = Bitmap::create();
                loop {
                    match iter.next() {
                        Some(row_id) => {
                            yield set.add(row_id);
                        }
                        None => {
                            return *superset = set;
                        }
                    }
                }
            }
        }
    }

    pub fn push(&mut self, value: Option<A::ItemRef<'_>>) {
        let id = self.array.push_and_get_id(value);
        self.index.insert(self.array.len() - 1, id);
    }
}

pub trait Filter {
    fn lookup<'s>(
        &'s self,
        matcher: &'s MatcherOp,
        superset: &'s mut Bitmap,
    ) -> Result<Box<dyn 's + Generator<(), Yield = (), Return = ()> + Unpin>, FilterError>;
}

// impl Filter for LabelColumn<StringLabel> {
//     fn lookup<'s>(
//         &'s self,
//         matcher: &'s MatcherOp,
//         superset: &'s mut Bitmap,
//     ) -> Result<Box<dyn 's + Generator<(), Yield = (), Return = ()> + Unpin>, FilterError> {
//         match matcher {
//             op @ (MatcherOp::LiteralEqual(matcher) | MatcherOp::LiteralNotEqual(matcher)) => {
//                 let matcher = match matcher {
//                     Some(matcher) => match matcher {
//                         Label::String(s) => Some(s.as_ref()),
//                         m @ _ => {
//                             return Err(FilterError::MismatchType {
//                                 expect: LabelType(Label::String(())),
//                                 found: m.r#type(),
//                             });
//                         }
//                     },
//                     None => None,
//                 };
//                 Ok(Box::new(self.lookup(op.positive(), matcher, superset)))
//             }
//             op @ (MatcherOp::RegexMatch(matcher) | MatcherOp::RegexNotMatch(matcher)) => self
//                 .regex_match(op.positive(), matcher, superset)
//                 .map(|g| Box::new(g) as Box<_>),
//         }
//     }
// }
