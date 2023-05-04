use std::error::Error;

use common::{query::Projection, Set};

use super::{scan::ScanPlanner, ExecutionImpl, Planner};
use crate::checker::mir::Mir;

#[derive(Debug)]
pub struct Context {
    limit: Option<usize>,
    projection: Projection<usize>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            limit: Default::default(),
            projection: Projection {
                labels: Set::Universe,
                fields: Set::Universe,
            },
        }
    }
}

pub fn plan(cx: &mut Context, expr: Mir) -> Result<ExecutionImpl, Box<dyn Error>> {
    match expr {
        Mir::Scan(scan) => {
            cx.projection.append(scan.projection);
            Ok(ExecutionImpl::Scan(
                ScanPlanner {
                    resource: scan.resource,
                    matcher: scan.matcher,
                    limit: cx.limit,
                    projection: cx.projection.clone(),
                    range: scan.range,
                }
                .plan(ExecutionImpl::Id(()))
                .map_err(|e| Box::new(e) as Box<_>)?,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use resource::db::tests::test_db;

    use super::{plan, Context};
    use crate::{checker::Checker, parser::Parser, Layer, Pass};

    #[test]
    fn plan_scan() {
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
                let mut cx = Context::default();
                let plan = plan(&mut cx, mir).unwrap();
                println!("plan {:?}", plan);
            });
    }
}
