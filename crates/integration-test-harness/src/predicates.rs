/// Custom predicates for easy assertions
///
use core::fmt;

use predicates::{constant::always, reflection::PredicateReflection, BoxPredicate, Predicate};

// submodules
pub mod json;
pub use json::*;

// re-export commonly used predicates
pub use predicates::boolean::PredicateBooleanExt;
pub use predicates::ord::*;
pub use predicates::prelude::*;
pub use predicates::str::*;

/// Helper to allow prefix form of not(predicate) instead of predicate.not()
pub fn not<P, I>(predicate: P) -> impl Predicate<I>
where
    P: PredicateBooleanExt<I>,
    I: ?Sized,
{
    predicate.not()
}

/// Predicate over the length of a slice
struct LenPredicate(usize);
impl<T> Predicate<[T]> for LenPredicate {
    fn eval(&self, slice: &[T]) -> bool {
        slice.len() == self.0
    }
}
impl PredicateReflection for LenPredicate {}
impl fmt::Display for LenPredicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "len({})", self.0)
    }
}

pub fn len<T>(length: usize) -> impl Predicate<[T]> {
    LenPredicate(length)
}

/// Checks that variable contains all strings from an iterator
pub fn contains_all<S: Into<String>, I: IntoIterator<Item = S>>(iter: I) -> impl Predicate<str> {
    iter.into_iter()
        .fold(BoxPredicate::new(always()), |accum, str| {
            BoxPredicate::new(accum.and(contains(str)))
        })
}
