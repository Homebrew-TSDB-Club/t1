use common::{column::Label, time::Instant};

#[derive(Debug)]
pub struct Resource {
    pub catalog: Option<String>,
    pub namespace: Option<String>,
    pub resource: String,
}

#[derive(Debug, Clone)]
pub enum AggregateAction {
    Without,
    With,
}

#[derive(Debug, Clone)]
pub struct Aggregation {
    pub action: AggregateAction,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
}

#[derive(Debug)]
pub struct Pipeline {
    pub functions: Vec<Function>,
    pub breaker: Option<Aggregation>,
}

#[derive(Debug)]
pub struct Projection {
    pub name: String,
    pub pipeline: Pipeline,
}

#[derive(Debug, Copy, Clone)]
pub struct Range {
    pub start: Option<Instant>,
    pub end: Option<Instant>,
}

#[derive(Debug)]
pub struct Expression {
    pub resource: Resource,
    pub filters: Vec<Matcher>,
    pub range: Range,
    pub projection: Vec<Projection>,
    pub aggregation: Option<Aggregation>,
}

#[derive(Debug, Clone)]
pub struct Matcher {
    pub name: String,
    pub op: MatcherOp,
    pub value: Option<Label<String, String, String, String, String>>,
}

#[derive(Debug, Copy, Clone)]
pub enum MatcherOp {
    LiteralEqual,
    LiteralNotEqual,
    RegexMatch,
    RegexNotMatch,
}
