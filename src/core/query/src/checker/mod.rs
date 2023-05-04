mod function;
pub mod mir;

use std::{
    fmt::Display,
    mem::MaybeUninit,
    sync::{Arc, RwLock},
};

use common::{
    column::{
        field::FieldType,
        label::{AnyValue, LabelType},
    },
    query::{MatcherOp, Projection},
    Set,
};
use resource::{db::DB, table::Table};
use thiserror::Error;

use self::mir::{Mir, Scan};
use crate::{
    parser::hir::{Hir, Matcher},
    Layer, Pass,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("resource: {name} not exists")]
    ResourceNotExists { name: String },
    #[error(transparent)]
    NoColumn(#[from] NoColumn),
    #[error(transparent)]
    MatchError(#[from] MatchError),
    #[error(transparent)]
    TypeError(#[from] TypeError),
    #[error(transparent)]
    UpStream(#[from] Box<dyn std::error::Error>),
}

pub struct Env<'db> {
    db: &'db DB,
    table: MaybeUninit<Arc<Table>>,
}

pub fn check(env: &mut Env, expression: Hir) -> Result<Mir, Error> {
    match expression {
        Hir::Aggregate(_) => unimplemented!(),
        Hir::Call(_call) => {
            todo!()
        }
        Hir::Scan(scan) => {
            let resource = env
                .db
                .get(&scan.resource)
                .ok_or_else(|| Error::ResourceNotExists {
                    name: scan.resource.clone(),
                })?
                .clone();
            env.table.write(resource.clone());
            let projection = resource.normalize(scan.projection)?;
            let matcher = resource.normalize(scan.matcher)?.check(env)?;
            Ok(Mir::Scan(Scan {
                resource,
                matcher,
                range: scan.range,
                projection,
            }))
        }
        _ => unreachable!(),
    }
}

trait Normalize<Input> {
    type Output;
    type Error: std::error::Error;

    fn normalize(&self, input: Input) -> Result<Self::Output, Self::Error>;
}

#[derive(Error, Debug)]
#[error("{op} failed: table {table} does not have column {name}")]
pub struct NoColumn {
    op: &'static str,
    table: String,
    name: String,
}

impl Normalize<Projection<String>> for Table {
    type Output = Projection;
    type Error = NoColumn;

    fn normalize(&self, projection: Projection<String>) -> Result<Self::Output, Self::Error> {
        macro_rules! normalize {
            ($column:ident) => {
                if let Set::Some(p) = &projection.$column {
                    let mut $column = Vec::with_capacity(p.len());
                    for p in p {
                        let (id, _) = self
                            .meta
                            .schema
                            .$column
                            .iter()
                            .map(|label| &label.name)
                            .enumerate()
                            .find(|(_, name)| *name == p)
                            .ok_or_else(|| NoColumn {
                                op: "project",
                                table: self.name.to_string(),
                                name: p.clone(),
                            })?;
                        $column.push(id);
                    }
                    Set::Some($column)
                } else {
                    Set::Universe
                }
            };
        }

        Ok(Projection {
            labels: normalize!(labels),
            fields: normalize!(fields),
        })
    }
}

#[derive(Error, Debug)]
pub enum MatchError {
    #[error(transparent)]
    NoColumn(#[from] NoColumn),
    #[error(
        "{label} type of label column {name} in table: {table} does not support regex matching"
    )]
    NoSupportRegex {
        label: LabelType,
        name: String,
        table: String,
    },
}

impl Normalize<Vec<Matcher>> for Table {
    type Output = Vec<Option<MatcherOp>>;
    type Error = MatchError;

    fn normalize(&self, matcher: Vec<Matcher>) -> Result<Self::Output, Self::Error> {
        let mut op = vec![None; self.meta.schema.labels.len()];
        for m in &matcher {
            let (id, meta) = self
                .meta
                .schema
                .labels
                .iter()
                .enumerate()
                .find(|(_, label)| label.name == m.name)
                .ok_or_else(|| NoColumn {
                    op: "match",
                    table: self.name.to_string(),
                    name: m.name.clone(),
                })?;

            match m.op {
                MatcherOp::RegexMatch(_) | MatcherOp::RegexNotMatch(_) => match &meta.r#type {
                    LabelType::String(_) => {}
                    other => {
                        return Err(MatchError::NoSupportRegex {
                            label: other.clone(),
                            name: meta.name.clone(),
                            table: self.name.to_string(),
                        });
                    }
                },
                _ => {}
            }

            op[id] = Some(m.op.clone());
        }
        Ok(op)
    }
}

#[derive(Debug)]
pub enum Type {
    Label(LabelType),
    Field(FieldType),
    Literal,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

#[derive(Error, Debug)]
#[error("{place} type mismatch, expect {expect} found {found}")]
pub struct TypeError {
    place: &'static str,
    expect: Type,
    found: Type,
}

trait TypeCheck {
    type Output;

    fn check(self, env: &mut Env) -> Result<Self::Output, TypeError>;
}

impl TypeCheck for Vec<Option<MatcherOp>> {
    type Output = Vec<Option<MatcherOp<AnyValue>>>;

    fn check(self, env: &mut Env) -> Result<Self::Output, TypeError> {
        let mut output = Vec::with_capacity(self.len());

        let columns = &unsafe { env.table.assume_init_ref() }.meta.schema.labels;
        for (matcher, column) in self.into_iter().zip(columns.iter()) {
            let m = if let Some(op) = matcher {
                match op {
                    MatcherOp::LiteralEqual(matcher) => match matcher {
                        Some(matcher) => {
                            let expect = column.r#type.r#type();
                            let found = matcher.r#type();
                            if expect == found {
                                Some(MatcherOp::LiteralEqual(Some(AnyValue::from(matcher))))
                            } else {
                                return Err(TypeError {
                                    place: "matcher",
                                    expect: Type::Label(expect),
                                    found: Type::Label(found),
                                });
                            }
                        }
                        None => Some(MatcherOp::LiteralEqual(None)),
                    },
                    MatcherOp::LiteralNotEqual(matcher) => match matcher {
                        Some(matcher) => {
                            let expect = column.r#type.r#type();
                            let found = matcher.r#type();
                            if expect == found {
                                Some(MatcherOp::LiteralEqual(Some(AnyValue::from(matcher))))
                            } else {
                                return Err(TypeError {
                                    place: "matcher",
                                    expect: Type::Label(expect),
                                    found: Type::Label(found),
                                });
                            }
                        }
                        None => Some(MatcherOp::LiteralEqual(None)),
                    },
                    MatcherOp::RegexMatch(matcher) => Some(MatcherOp::RegexMatch(matcher)),
                    MatcherOp::RegexNotMatch(matcher) => Some(MatcherOp::RegexNotMatch(matcher)),
                }
            } else {
                None
            };

            output.push(m);
        }

        Ok(output)
    }
}

pub struct Check<P> {
    db: Arc<RwLock<DB>>,
    inner: P,
}

impl<Input, P> Pass<Input> for Check<P>
where
    P: Pass<Input, Output = Hir>,
{
    type Output = Mir;
    type Error = Error;

    fn apply(&self, input: Input) -> Result<Self::Output, Self::Error> {
        let mut env = Env {
            db: &*self.db.read().unwrap(),
            table: MaybeUninit::uninit(),
        };
        check(
            &mut env,
            self.inner
                .apply(input)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?,
        )
    }
}

pub struct Checker {
    db: Arc<RwLock<DB>>,
}

impl Checker {
    pub(crate) fn new(db: Arc<RwLock<DB>>) -> Self {
        Self { db }
    }
}

impl<Input, Inner: Pass<Input, Output = Hir>> Layer<Input, Inner> for Checker {
    type Pass = Check<Inner>;

    fn layer(&self, inner: Inner) -> Self::Pass {
        Check {
            db: self.db.clone(),
            inner,
        }
    }
}

#[cfg(test)]
mod tests {

    use resource::db::tests::test_db;

    use super::Checker;
    use crate::{parser::Parser, Layer, Pass};

    #[test]
    fn check_scan() {
        executor::ExecutorBuilder::new()
            .worker_num(1)
            .build()
            .unwrap()
            .run(|| async {
                let parser = Checker::new(test_db()).layer(Parser::new().layer(()));
                let mir = parser
                    .apply(
                        r#"foo.bar.something_used{env="production", status!~"4.."}[5m] offset 1w"#,
                    )
                    .unwrap();
                println!("{:?}", mir);
            });
    }
}
