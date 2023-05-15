use thiserror::Error;

use super::{Label, LabelType};

pub type LabelValue = Label<Vec<u8>, [u8; 4], [u8; 16], i64, bool>;

#[derive(Error, Debug)]
#[error("mismatch type of label value, expect {}, found: {}", .expect, .found)]
pub struct LabelTypeMismatch {
    pub expect: LabelType,
    pub found: LabelType,
}

pub trait TryAsRef<T: ?Sized> {
    type Error: std::error::Error;

    fn try_as_ref(&self) -> Result<&T, Self::Error>;
}

impl<IP4, IP6, I, B> TryAsRef<[u8]> for Label<Vec<u8>, IP4, IP6, I, B> {
    type Error = LabelTypeMismatch;

    fn try_as_ref(&self) -> Result<&[u8], Self::Error> {
        match self {
            Label::String(s) => Ok(s),
            m @ _ => Err(LabelTypeMismatch {
                expect: Label::String(()).into(),
                found: m.r#type(),
            }),
        }
    }
}

impl<S, IP6, I, B> TryAsRef<[u8; 4]> for Label<S, [u8; 4], IP6, I, B> {
    type Error = LabelTypeMismatch;

    fn try_as_ref(&self) -> Result<&[u8; 4], Self::Error> {
        match self {
            Label::IPv4(s) => Ok(s),
            m @ _ => Err(LabelTypeMismatch {
                expect: Label::IPv4(()).into(),
                found: m.r#type(),
            }),
        }
    }
}

impl<S, IP4, I, B> TryAsRef<[u8; 16]> for Label<S, IP4, [u8; 16], I, B> {
    type Error = LabelTypeMismatch;

    fn try_as_ref(&self) -> Result<&[u8; 16], Self::Error> {
        match self {
            Label::IPv6(s) => Ok(s),
            m @ _ => Err(LabelTypeMismatch {
                expect: Label::IPv4(()).into(),
                found: m.r#type(),
            }),
        }
    }
}

impl<S, IP4, IP6, B> TryAsRef<i64> for Label<S, IP4, IP6, i64, B> {
    type Error = LabelTypeMismatch;

    fn try_as_ref(&self) -> Result<&i64, Self::Error> {
        match self {
            Label::Int(s) => Ok(s),
            m @ _ => Err(LabelTypeMismatch {
                expect: Label::IPv4(()).into(),
                found: m.r#type(),
            }),
        }
    }
}

impl<S, IP4, IP6, I> TryAsRef<bool> for Label<S, IP4, IP6, I, bool> {
    type Error = LabelTypeMismatch;

    fn try_as_ref(&self) -> Result<&bool, Self::Error> {
        match self {
            Label::Bool(s) => Ok(s),
            m @ _ => Err(LabelTypeMismatch {
                expect: Label::IPv4(()).into(),
                found: m.r#type(),
            }),
        }
    }
}
