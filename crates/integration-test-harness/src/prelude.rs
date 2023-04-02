/// Convenience module to re-export stuff that's commonly used in tests
///
// public re-exports
pub use super::*;
pub use crate::assert::AssertCmdExt;
pub use crate::command::*;
pub use crate::scopes::*;
pub use assert_cmd::prelude::*;
pub use integration_test_macros::integration_test;
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
