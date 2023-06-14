use common::{column::ColumnType, query::MatcherOp};

use super::{Rule, TypeMismatch};
use crate::{check::Env, plan::physical::Scan};

impl Rule for Scan {
    fn check(&self, env: &Env) -> Result<(), TypeMismatch> {
        let columns = &env.table.as_ref().unwrap().meta.schema.labels;
        for (matcher, column) in self.matcher.iter().zip(columns.iter()) {
            if let Some(op) = matcher {
                match op {
                    MatcherOp::LiteralEqual(Some(matcher)) => {
                        let expect = column.r#type.r#type();
                        let found = matcher.r#type();
                        if expect != found {
                            return Err(TypeMismatch {
                                place: column.name.clone(),
                                expect: ColumnType::Label(expect),
                                found: ColumnType::Label(found),
                            });
                        }
                    }
                    MatcherOp::LiteralNotEqual(Some(matcher)) => {
                        let expect = column.r#type.r#type();
                        let found = matcher.r#type();
                        if expect != found {
                            return Err(TypeMismatch {
                                place: column.name.clone(),
                                expect: ColumnType::Label(expect),
                                found: ColumnType::Label(found),
                            });
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}
