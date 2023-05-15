pub mod value;

use std::fmt::Display;

pub use self::value::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Label<S, IP4, IP6, I, B> {
    String(S),
    IPv4(IP4),
    IPv6(IP6),
    Int(I),
    Bool(B),
}

impl<S, IP4, IP6, I, B> Label<S, IP4, IP6, I, B> {
    pub fn r#type(&self) -> LabelType {
        match self {
            Label::String(_) => Label::String(()).into(),
            Label::IPv4(_) => Label::IPv4(()).into(),
            Label::IPv6(_) => Label::IPv6(()).into(),
            Label::Int(_) => Label::Int(()).into(),
            Label::Bool(_) => Label::Bool(()).into(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct LabelType(pub Label<(), (), (), (), ()>);

impl From<Label<(), (), (), (), ()>> for LabelType {
    fn from(value: Label<(), (), (), (), ()>) -> Self {
        Self(value)
    }
}

impl Display for LabelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Label::String(_) => f.write_str("String"),
            Label::IPv4(_) => f.write_str("IPv4"),
            Label::IPv6(_) => f.write_str("IPv6"),
            Label::Int(_) => f.write_str("Int"),
            Label::Bool(_) => f.write_str("Bool"),
        }
    }
}
