/// Convenience module to re-export stuff that's commonly used in tests
///
// exports from harness
pub use crate::assert::AssertCmdExt;
pub use crate::command::*;
pub use crate::data::*;
pub use crate::predicates::*;
pub use crate::util::*;

// export macros
#[cfg(feature = "macros")]
pub use {
    super::{all, any, cli_bin_path, cloudtruth, contains, contains_all, contains_any, diff},
    cloudtruth_test_macros::use_harness,
};

// exports from dependencies
pub use assert_cmd::prelude::*;

pub use miette::{Context, IntoDiagnostic};

pub use anyhow::Result;
