#![feature(async_fn_in_trait)]

pub mod db;
pub mod table;

use chunk::mutable::column::FilterError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TableScanError {
    #[error("filter column error {}", .source)]
    FilterError {
        #[from]
        source: FilterError,
    },
}
