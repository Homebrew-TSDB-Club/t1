#![feature(async_fn_in_trait)]
#![feature(impl_trait_projections)]
#![feature(get_mut_unchecked)]

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
