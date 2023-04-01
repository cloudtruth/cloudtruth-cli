/// Convenience module to re-export stuff that's commonly used in tests
///
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
