use common::{
    column::label::LabelType,
    query::{MatcherOp, Projection},
    Set,
};
use thiserror::Error;

use super::Env;
use crate::plan::{logical, logical::Matcher, physical};

#[derive(Error, Debug)]
pub enum NormalizeError {
    #[error("{op} failed: table {table} does not have column {name}")]
    NoColumn {
        op: &'static str,
        table: String,
        name: String,
    },
    #[error(
        "{label} type of label column {name} in table: {table} does not support regex matching"
    )]
    NoSupportRegex {
        label: LabelType,
        name: String,
        table: String,
    },
    #[error("resource: {name} not exists")]
    ResourceNotExists { name: String },
}

pub(super) trait Normalize {
    type Output;

    fn normalize(self, env: &mut Env<'_>) -> Result<Self::Output, NormalizeError>;
}

impl Normalize for logical::Scan {
    type Output = physical::Scan;

    fn normalize(self, env: &mut Env<'_>) -> Result<Self::Output, NormalizeError> {
        let resource = env
            .db
            .get(&self.resource)
            .ok_or_else(|| NormalizeError::ResourceNotExists {
                name: self.resource.clone(),
            })?
            .clone();
        let _ = env.table.insert(resource.clone());
        Ok(physical::Scan {
            resource,
            matcher: self.matcher.normalize(env)?,
            range: self.range,
            projection: self.projection.normalize(env)?,
        })
    }
}

impl Normalize for Projection<String> {
    type Output = Projection;

    fn normalize(self, env: &mut Env<'_>) -> Result<Self::Output, NormalizeError> {
        macro_rules! normalize {
            ($column:ident) => {{
                let table = env.table.as_ref().unwrap();
                if let Set::Some(p) = &self.$column {
                    let mut $column = Vec::with_capacity(p.len());
                    for p in p {
                        let (id, _) = table
                            .meta
                            .schema
                            .$column
                            .iter()
                            .map(|label| &label.name)
                            .enumerate()
                            .find(|(_, name)| *name == p)
                            .ok_or_else(|| NormalizeError::NoColumn {
                                op: "project",
                                table: table.name.to_string(),
                                name: p.clone(),
                            })?;
                        $column.push(id);
                    }
                    Set::Some($column)
                } else {
                    Set::Universe
                }
            }};
        }

        Ok(Projection {
            labels: normalize!(labels),
            fields: normalize!(fields),
        })
    }
}

impl Normalize for Vec<Matcher> {
    type Output = Vec<Option<MatcherOp>>;

    fn normalize(self, env: &mut Env<'_>) -> Result<Self::Output, NormalizeError> {
        let table = env.table.as_ref().unwrap();
        let mut op = vec![None; table.meta.schema.labels.len()];
        for m in self {
            let (id, meta) = table
                .meta
                .schema
                .labels
                .iter()
                .enumerate()
                .find(|(_, label)| label.name == m.name)
                .ok_or_else(|| NormalizeError::NoColumn {
                    op: "match",
                    table: table.name.to_string(),
                    name: m.name.clone(),
                })?;

            match m.op {
                MatcherOp::RegexMatch(_) | MatcherOp::RegexNotMatch(_) => match &meta.r#type {
                    LabelType::String(_) => {}
                    other => {
                        return Err(NormalizeError::NoSupportRegex {
                            label: other.clone(),
                            name: meta.name.clone(),
                            table: table.name.to_string(),
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
