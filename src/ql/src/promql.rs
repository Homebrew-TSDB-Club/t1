use common::{
    column::Label,
    time::{Duration, Instant},
};

use crate::{
    expression::{
        Aggregate, AggregateAction, Call, Expression, Matcher, MatcherOp, Project, Range, Resource,
    },
    Error,
};

pub fn parse(literal: &str) -> Result<Expression, Error> {
    let expr = promql::parse(literal.as_bytes(), true).map_err(|e| Error::ParsingWrong {
        err: format!("{}", e),
    })?;
    translate(expr)
}

fn translate(expr: promql::Node) -> Result<Expression, Error> {
    use promql::{LabelMatchOp, Node};

    match expr {
        Node::Vector(vector) => {
            let mut name = None;
            let mut matchers = Vec::with_capacity(vector.labels.len() - 1);

            for label in vector.labels {
                if label.name == "__name__" {
                    name = Resource::from_str(&label.value);
                } else {
                    let op = match label.op {
                        LabelMatchOp::Eq => MatcherOp::LiteralEqual,
                        LabelMatchOp::Ne => MatcherOp::LiteralNotEqual,
                        LabelMatchOp::REq => MatcherOp::RegexMatch,
                        LabelMatchOp::RNe => MatcherOp::RegexNotMatch,
                    };
                    matchers.push(Matcher {
                        name: label.name,
                        op,
                        value: Label::String(label.value),
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

            Ok(Expression::Project(Project {
                resource: name.ok_or(Error::NoName)?,
                matchers,
                range: Range {
                    start: start.map(|start| start.into()),
                    end: Some(end),
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

                Ok(Expression::Aggregate(Aggregate {
                    name,
                    action,
                    by: aggr.labels,
                    args,
                }))
            }
            None => {
                let mut args = Vec::with_capacity(arg_nodes.len());
                for arg in arg_nodes {
                    args.push(translate(arg)?);
                }
                Ok(Expression::Call(Call { name, args }))
            }
        },
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {
    use common::{column::Label, time::Instant};

    use super::parse;
    use crate::expression::{
        Aggregate, AggregateAction, Call, Expression, Matcher, MatcherOp, Project, Range, Resource,
    };

    #[test]
    fn it_works() {
        let query = "sum by (test) (rate(foo.bar.something_used{env=\"production\", \
                     status!~\"4..\"}[5m] offset 1w))";

        let expr = parse(query).unwrap();

        let expected = Expression::Aggregate(Aggregate {
            name: "sum".into(),
            action: AggregateAction::With,
            by: vec!["test".into()],
            args: vec![Expression::Call(Call {
                name: "rate".into(),
                args: vec![Expression::Project(Project {
                    resource: Resource {
                        catalog: Some("foo".into()),
                        namespace: Some("bar".into()),
                        table: "something_used".into(),
                    },
                    matchers: vec![
                        Matcher {
                            name: "env".into(),
                            op: MatcherOp::LiteralEqual,
                            value: Label::String("production".into()),
                        },
                        Matcher {
                            name: "status".into(),
                            op: MatcherOp::RegexNotMatch,
                            value: Label::String("4..".into()),
                        },
                    ],
                    range: Range {
                        start: Some(Instant::from_millis(1682752166643)),
                        end: Some(Instant::from_millis(1682752466643)),
                    },
                })],
            })],
        });

        assert_eq!(expr, expected);
    }
}
