pub mod execution;
pub mod language;

use std::{borrow::Borrow, future::Future};

use common::expression::Matcher;
// use execution::Execution;
use uuid::Uuid;

#[derive(Debug)]
pub struct Context {
    session_id: Uuid,
}

#[derive(Debug)]
pub struct Projection {
    pub label: Vec<usize>,
    pub field: Vec<usize>,
}

impl Projection {
    pub fn as_ref(&self) -> ProjectionRef<'_> {
        ProjectionRef {
            label: &self.label[..],
            field: &self.field[..],
        }
    }
}

#[derive(Debug)]
pub struct ProjectionRef<'r> {
    pub label: &'r [usize],
    pub field: &'r [usize],
}
