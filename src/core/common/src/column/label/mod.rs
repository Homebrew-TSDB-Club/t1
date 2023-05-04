pub mod value;

use std::fmt::Display;

pub use self::value::*;

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
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
            Label::String(_) => Label::String(()),
            Label::IPv4(_) => Label::IPv4(()),
            Label::IPv6(_) => Label::IPv6(()),
            Label::Int(_) => Label::Int(()),
            Label::Bool(_) => Label::Bool(()),
        }
    }
}

pub type LabelType = Label<(), (), (), (), ()>;

impl Display for Label<(), (), (), (), ()> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Label::String(_) => f.write_str("String"),
            Label::IPv4(_) => f.write_str("IPv4"),
            Label::IPv6(_) => f.write_str("IPv6"),
            Label::Int(_) => f.write_str("Int"),
            Label::Bool(_) => f.write_str("Bool"),
        }
    }
}
