use common::{
    query::{MatcherOp, Projection},
    time::{Duration, Range},
};

#[derive(Debug, PartialEq, Clone)]
pub enum Logical {
    Aggregate(Aggregate),
    Call(Call),
    Scan(Scan),
    Literal(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AggregateAction {
    Without,
    With,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WindowSize {
    Exact(Duration),
    Depends,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Window {
    pub op: String,
    pub size: WindowSize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Aggregate {
    pub name: String,
    pub action: AggregateAction,
    pub by: Vec<String>,
    pub args: Vec<Logical>,
    pub window: Window,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub name: String,
    pub args: Vec<Logical>,
}

#[derive(Debug, Clone)]
pub struct Scan {
    pub resource: String,
    pub matcher: Vec<Matcher>,
    pub range: Range,
    pub projection: Projection<String>,
}

impl PartialEq for Scan {
    fn eq(&self, other: &Self) -> bool {
        self.resource == other.resource
            && self.matcher == other.matcher
            && self.projection == other.projection
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Matcher<Name = String> {
    pub name: Name,
    pub op: MatcherOp,
}
