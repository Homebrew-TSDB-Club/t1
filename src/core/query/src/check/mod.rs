mod normalize;
mod rules;

use std::sync::{Arc, RwLock};

use normalize::{Normalize, NormalizeError};
use resource::{db::DB, table::Table};
use thiserror::Error;

use self::rules::{Rule, TypeMismatch};
use crate::{
    execute::function::rate,
    plan::{
        logical::Logical,
        physical::{Call, Physical},
    },
    Layer, Pass,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    NormalizeError(#[from] NormalizeError),
    #[error(transparent)]
    TypeError(#[from] TypeMismatch),
    #[error(transparent)]
    UpStream(#[from] Box<dyn std::error::Error>),
}

pub struct Env<'db> {
    db: &'db DB,
    table: Option<Arc<Table>>,
}

pub struct Check<P> {
    db: Arc<RwLock<DB>>,
    inner: P,
}

impl<Input, P> Pass<Input> for Check<P>
where
    P: Pass<Input, Output = Logical>,
{
    type Output = Physical;
    type Error = Error;

    fn apply(&self, input: Input) -> Result<Self::Output, Self::Error> {
        let mut env = Env {
            db: &self.db.read().unwrap(),
            table: None,
        };
        check(
            &mut env,
            self.inner
                .apply(input)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?,
        )
    }
}

fn check(env: &mut Env, expression: Logical) -> Result<Physical, Error> {
    match expression {
        Logical::Call(call) => {
            let mut args = Vec::with_capacity(call.args.len());
            for arg in call.args {
                args.push(check(env, arg)?);
            }
            let func = match call.name.as_str() {
                "rate" => rate,
                _ => unimplemented!(),
            };
            Ok(Physical::Call(Call {
                args,
                name: call.name,
                function: func,
            }))
        }
        Logical::Scan(scan) => {
            let scan = scan.normalize(env)?;
            scan.check(env)?;
            Ok(Physical::Scan(scan))
        }
        _ => unimplemented!(),
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

impl<Input, Inner: Pass<Input, Output = Logical>> Layer<Input, Inner> for Checker {
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
    use crate::{parse::Parser, Layer, Pass};

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
