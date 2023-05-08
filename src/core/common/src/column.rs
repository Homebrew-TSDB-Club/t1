use std::{
    fmt::Display,
    net::{Ipv4Addr, Ipv6Addr},
};

#[derive(Debug)]
pub struct LabelType(pub Label<(), (), (), (), ()>);

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

#[derive(Debug)]
pub struct FieldType(Field<(), (), (), (), (), (), (), (), (), (), ()>);

pub type LabelValue = Label<String, Ipv4Addr, Ipv6Addr, i64, bool>;

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
            Label::String(_) => LabelType(Label::String(())),
            Label::IPv4(_) => LabelType(Label::IPv4(())),
            Label::IPv6(_) => LabelType(Label::IPv6(())),
            Label::Int(_) => LabelType(Label::Int(())),
            Label::Bool(_) => LabelType(Label::Bool(())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Field<U8, U16, U32, U64, I8, I16, I32, I64, F32, F64, B> {
    UInt8(U8),
    UInt16(U16),
    UInt32(U32),
    UInt64(U64),
    Int8(I8),
    Int16(I16),
    Int32(I32),
    Int64(I64),
    Float32(F32),
    Float64(F64),
    Bool(B),
}
