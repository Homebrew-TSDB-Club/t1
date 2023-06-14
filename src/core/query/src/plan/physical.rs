use std::sync::Arc;

use common::{
    query::{MatcherOp, Projection},
    time::Range,
};
use resource::table::Table;

use crate::execute::function::Function;

#[derive(Debug, Clone)]
pub enum Physical {
    Scan(Scan),
    Call(Call),
}

#[derive(Debug, Clone)]
pub struct Scan {
    pub resource: Arc<Table>,
    pub matcher: Vec<Option<MatcherOp>>,
    pub range: Range,
    pub projection: Projection,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub args: Vec<Physical>,
    pub name: String,
    pub function: Function,
}
