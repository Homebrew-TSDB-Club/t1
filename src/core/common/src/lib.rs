#![feature(split_array)]

pub mod array;
pub mod column;
pub mod context;
pub mod index;
pub mod primitive;
pub mod schema;
pub mod time;
pub mod uuid {
    pub use uuid::*;
}
pub mod query;
pub mod scalar;

#[derive(Debug, Clone, PartialEq)]
pub enum Set<S> {
    Universe,
    Some(S),
}

impl<S> Set<S> {
    #[inline]
    pub fn as_mut(&mut self) -> Set<&mut S> {
        match self {
            Set::Universe => Set::Universe,
            Set::Some(set) => Set::Some(set),
        }
    }

    #[inline]
    pub fn as_ref(&self) -> Set<&S> {
        match self {
            Set::Universe => Set::Universe,
            Set::Some(set) => Set::Some(set),
        }
    }

    #[inline]
    pub fn map<B, F: FnOnce(S) -> B>(self, f: F) -> Set<B> {
        match self {
            Set::Universe => Set::Universe,
            Set::Some(set) => Set::Some((f)(set)),
        }
    }
}
