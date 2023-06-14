use regex::Regex;

use crate::{
    column::{label::LabelValue, ColumnType},
    Set,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Projection<V = usize> {
    pub labels: Set<Vec<V>>,
    pub fields: Set<Vec<V>>,
}

impl<V: PartialEq> Projection<V> {
    #[inline]
    pub fn as_ref(&self) -> ProjectionRef<'_, V> {
        ProjectionRef {
            labels: self.labels.as_ref().map(|p| &p[..]),
            fields: self.fields.as_ref().map(|p| &p[..]),
        }
    }

    #[inline]
    pub fn insert(&mut self, r#type: ColumnType, p: V) {
        match r#type {
            ColumnType::Label(_) => {
                self.labels.as_mut().map(|labels| {
                    if !labels.contains(&p) {
                        labels.push(p);
                    }
                });
            }
            ColumnType::Field(_) => {
                self.labels.as_mut().map(|labels| {
                    if !labels.contains(&p) {
                        labels.push(p);
                    }
                });
            }
        }
    }

    #[inline]
    pub fn append(&mut self, another: Self) {
        match another.labels {
            Set::Universe => {
                self.labels = Set::Universe;
            }
            Set::Some(mut labels) => {
                self.labels.as_mut().map(|set| set.append(&mut labels));
            }
        }
    }
}

#[derive(Debug)]
pub struct ProjectionRef<'r, V = usize> {
    pub labels: Set<&'r [V]>,
    pub fields: Set<&'r [V]>,
}

#[derive(Debug, Clone)]
pub enum MatcherOp<V = LabelValue> {
    LiteralEqual(Option<V>),
    LiteralNotEqual(Option<V>),
    RegexMatch(Regex),
    RegexNotMatch(Regex),
}

impl<V: PartialEq> PartialEq for MatcherOp<V> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::LiteralEqual(l0), Self::LiteralEqual(r0)) => l0 == r0,
            (Self::LiteralNotEqual(l0), Self::LiteralNotEqual(r0)) => l0 == r0,
            (Self::RegexMatch(l0), Self::RegexMatch(r0)) => l0.as_str() == r0.as_str(),
            (Self::RegexNotMatch(l0), Self::RegexNotMatch(r0)) => l0.as_str() == r0.as_str(),
            _ => false,
        }
    }
}

impl<V> MatcherOp<V> {
    pub fn positive(&self) -> bool {
        match self {
            MatcherOp::LiteralEqual(_) => true,
            MatcherOp::LiteralNotEqual(_) => false,
            MatcherOp::RegexMatch(_) => true,
            MatcherOp::RegexNotMatch(_) => false,
        }
    }
}
