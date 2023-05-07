use std::net::{Ipv4Addr, Ipv6Addr};

use common::{column::Label, time::Instant};

#[derive(Debug, PartialEq, Clone)]
pub struct Resource {
    pub catalog: Option<String>,
    pub namespace: Option<String>,
    pub table: String,
}

impl Resource {
    pub fn from_str(value: &str) -> Option<Self> {
        let slice: Vec<_> = value.split('.').collect();
        let mut catalog = None;
        let mut namespace = None;
        let table;
        match slice.len() {
            2 => {
                namespace = Some(slice[0].to_owned());
                table = slice[1].to_owned();
            }
            1 => {
                table = slice[0].to_owned();
            }
            0 => return None,
            _ => {
                catalog = Some(slice[0].to_owned());
                namespace = Some(slice[1].to_owned());
                table = slice[2..].join(".");
            }
        }

        Some(Resource {
            catalog,
            namespace,
            table,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AggregateAction {
    Without,
    With,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Aggregate {
    pub name: String,
    pub action: AggregateAction,
    pub by: Vec<String>,
    pub args: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub name: String,
    pub args: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct Scan {
    pub resource: Resource,
    pub matchers: Vec<Matcher>,
    pub range: Range,
}

impl PartialEq for Scan {
    fn eq(&self, other: &Self) -> bool {
        self.resource == other.resource && self.matchers == other.matchers
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Range {
    pub start: Option<Instant>,
    pub end: Option<Instant>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Aggregate(Aggregate),
    Call(Call),
    Scan(Scan),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Matcher<Name = String> {
    pub name: Name,
    pub op: MatcherOp,
    pub value: Label<String, Ipv4Addr, Ipv6Addr, i64, bool>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MatcherOp {
    LiteralEqual,
    LiteralNotEqual,
    RegexMatch,
    RegexNotMatch,
}
