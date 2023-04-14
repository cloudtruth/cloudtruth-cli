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
    super::{cli_bin_path, cloudtruth, contains, diff},
    integration_test_macros::integration_test,
};

// exports from dependencies
pub use assert_cmd::prelude::*;

pub use miette::{Context, IntoDiagnostic, Result};
pub use predicates::boolean::PredicateBooleanExt;
pub use predicates::ord::*;
pub use predicates::prelude::*;
pub use predicates::str::*;

// private imports
