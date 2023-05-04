use std::sync::Arc;

use common::{
    column::label::AnyValue,
    query::{MatcherOp, Projection},
    time::Range,
};
use resource::table::Table;

#[derive(Debug)]
pub enum Mir {
    Scan(Scan),
}

#[derive(Debug, Clone)]
pub struct Scan {
    pub resource: Arc<Table>,
    pub matcher: Vec<Option<MatcherOp<AnyValue>>>,
    pub range: Range,
    pub projection: Projection,
}
