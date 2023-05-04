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
            ColumnType::Label => {
                self.labels.as_mut().map(|labels| {
                    if !labels.contains(&p) {
                        labels.push(p);
                    }
                });
            }
            ColumnType::Field => {
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

#[derive(Debug, Clone, PartialEq)]
pub enum MatcherOp<V = LabelValue> {
    LiteralEqual(Option<V>),
    LiteralNotEqual(Option<V>),
    RegexMatch(String),
    RegexNotMatch(String),
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
