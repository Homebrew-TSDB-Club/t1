use common::{
    column::label::Label,
    query::{MatcherOp, Projection},
    time::{Duration, Instant, Range},
    Set,
};
use regex::Regex;

use super::Error;
use crate::plan::logical::{
    Aggregate, AggregateAction, Call, Logical, Matcher, Scan, Window, WindowSize,
};

pub fn parse(literal: &str) -> Result<Logical, Error> {
    let expr = promql::parse(literal.as_bytes(), true).map_err(|e| Error::ParsingWrong {
        err: format!("{}", e),
    })?;
    translate(expr)
}

fn translate(expr: promql::Node) -> Result<Logical, Error> {
    use promql::{LabelMatchOp, Node};

    match expr {
        Node::Vector(vector) => {
            let mut name = None;
            let mut matcher = Vec::with_capacity(vector.labels.len() - 1);

            for label in vector.labels {
                if label.name == "__name__" {
                    name = Some(label.value);
                } else {
                    let op = match label.op {
                        LabelMatchOp::Eq => {
                            MatcherOp::LiteralEqual(Some(Label::String(label.value.into())))
                        }
                        LabelMatchOp::Ne => {
                            MatcherOp::LiteralNotEqual(Some(Label::String(label.value.into())))
                        }
                        LabelMatchOp::REq => MatcherOp::RegexMatch(Regex::new(&label.value)?),
                        LabelMatchOp::RNe => MatcherOp::RegexNotMatch(Regex::new(&label.value)?),
                    };
                    matcher.push(Matcher {
                        name: label.name,
                        op,
                    });
                }
            }

            let mut end = Instant::now();
            if let Some(offset) = vector.offset {
                end = end - Duration::from_secs(offset as i64);
            }
            let start = vector
                .range
                .map(|range| end - Duration::from_secs(range as i64));

            Ok(Logical::Scan(Scan {
                resource: name.ok_or(Error::NoName)?,
                matcher,
                range: Range {
                    start,
                    end: Some(end),
                },
                projection: Projection {
                    labels: Set::Universe,
                    fields: Set::Some(vec!["value".into()]),
                },
            }))
        }
        Node::Function {
            name,
            args: arg_nodes,
            aggregation,
        } => match aggregation {
            Some(aggr) => {
                let action = match aggr.action {
                    promql::AggregationAction::Without => AggregateAction::Without,
                    promql::AggregationAction::By => AggregateAction::With,
                };
                let mut args = Vec::with_capacity(arg_nodes.len());
                for arg in arg_nodes {
                    args.push(translate(arg)?);
                }
                args.push(Logical::Literal("value".into()));
                Ok(Logical::Aggregate(Aggregate {
                    name,
                    action,
                    by: aggr.labels,
                    args,
                    window: Window {
                        op: "first".into(),
                        size: WindowSize::Depends,
                    },
                }))
            }
            None => {
                let mut args = Vec::with_capacity(arg_nodes.len());
                for arg in arg_nodes {
                    args.push(translate(arg)?);
                }
                args.push(Logical::Literal("value".into()));
                Ok(Logical::Call(Call { name, args }))
            }
        },
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {
    use common::{
        column::label::Label,
        query::{MatcherOp, Projection},
        time::{Instant, Range},
        Set,
    };
    use regex::Regex;

    use super::parse;
    use crate::plan::logical::{
        Aggregate, AggregateAction, Call, Logical, Matcher, Scan, Window, WindowSize,
    };

    #[test]
    fn it_works() {
        let query = r#"sum (rate(foo.bar.something_used{env="production", status!~"4.."}[5m] offset 1w)) by (test)"#;

        let expr = parse(query).unwrap();

        let expected = Logical::Aggregate(Aggregate {
            name: "sum".into(),
            action: AggregateAction::With,
            by: vec!["test".into()],
            args: vec![
                Logical::Call(Call {
                    name: "rate".into(),
                    args: vec![
                        Logical::Scan(Scan {
                            resource: "foo.bar.something_used".into(),
                            matcher: vec![
                                Matcher {
                                    name: "env".into(),
                                    op: MatcherOp::LiteralEqual(Some(Label::String(
                                        "production".into(),
                                    ))),
                                },
                                Matcher {
                                    name: "status".into(),
                                    op: MatcherOp::RegexNotMatch(Regex::new("4..").unwrap()),
                                },
                            ],
                            range: Range {
                                start: Some(Instant::from_millis(1682752166643)),
                                end: Some(Instant::from_millis(1682752466643)),
                            },
                            projection: Projection {
                                labels: Set::Universe,
                                fields: Set::Some(vec!["value".into()]),
                            },
                        }),
                        Logical::Literal("value".into()),
                    ],
                }),
                Logical::Literal("value".into()),
            ],
            window: Window {
                op: "first".into(),
                size: WindowSize::Depends,
            },
        });

        assert_eq!(expr, expected);
    }
}
