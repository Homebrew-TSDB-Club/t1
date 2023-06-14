#![allow(clippy::len_without_is_empty)]
#![feature(split_array)]
#![feature(portable_simd)]

pub mod array;
pub mod column;
pub mod context;
pub mod either;
pub mod index;
pub mod primitive;
pub mod query;
pub mod scalar;
pub mod schema;
pub mod time;

use std::error::Error;

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

pub type DynError = Box<dyn 'static + Error + Send + Sync>;
