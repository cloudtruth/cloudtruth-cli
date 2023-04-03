/// Convenience module to re-export stuff that's commonly used in tests
///
// exports from harness
pub use crate::assert::AssertCmdExt;
pub use crate::command::*;
pub use crate::name::*;

// export macros
#[cfg(feature = "macros")]
pub use {
    super::{cli_bin_path, cloudtruth},
    integration_test_macros::integration_test,
};

// exports from dependencies
pub use assert_cmd::prelude::*;

pub use miette::{Context, IntoDiagnostic, Result};
pub use predicates::boolean::PredicateBooleanExt;
pub use predicates::prelude::*;
pub use predicates::str::*;

// private imports
use predicates::boolean::NotPredicate;

/// Helper to allow prefix form of not(predicate) instead of predicate.not()
pub fn not<Predicate, Item>(predicate: Predicate) -> NotPredicate<Predicate, Item>
where
    Predicate: PredicateBooleanExt<Item>,
    Item: ?Sized,
{
    predicate.not()
}
