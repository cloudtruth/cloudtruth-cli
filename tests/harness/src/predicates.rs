/// Custom predicates for easy assertions
///
use core::fmt;

use predicates::{constant::always, reflection::PredicateReflection};

// submodules
pub mod json;
pub use json::*;

// re-export commonly used predicates
pub use predicates::boolean::PredicateBooleanExt;
pub use predicates::ord::*;
pub use predicates::prelude::*;
pub use predicates::str::*;
pub use predicates::BoxPredicate;

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

/// Checks that length of input matches given number
pub fn len<T>(length: usize) -> impl Predicate<[T]> {
    LenPredicate(length)
}

/// Checks that all predicates in iterator are true
pub fn all<T, P, I>(predicates: I) -> impl Predicate<T>
where
    T: 'static + ?Sized,
    P: Predicate<T> + Send + Sync + 'static,
    I: IntoIterator<Item = P>,
{
    let mut predicates = predicates.into_iter();
    let first = match predicates.next() {
        Some(pred) => BoxPredicate::new(pred),
        None => BoxPredicate::new(always()),
    };
    predicates.fold(first, |accum, pred| BoxPredicate::new(accum.and(pred)))
}

/// Checks that any predicates in iterator are true
pub fn any<T, P, I>(predicates: I) -> impl Predicate<T>
where
    T: 'static + ?Sized,
    P: Predicate<T> + Send + Sync + 'static,
    I: IntoIterator<Item = P>,
{
    let mut predicates = predicates.into_iter();
    let first = match predicates.next() {
        Some(pred) => BoxPredicate::new(pred),
        None => BoxPredicate::new(always()),
    };
    predicates.fold(first, |accum, pred| BoxPredicate::new(accum.or(pred)))
}

/// Checks that variable contains all strings from an iterator
pub fn contains_all<S: AsRef<str>, I: IntoIterator<Item = S>>(iter: I) -> impl Predicate<str> {
    all(iter
        .into_iter()
        .map(|str| BoxPredicate::new(contains(str.as_ref()))))
}

/// Checks that variable contains any strings from an iterator
pub fn contains_any<S: AsRef<str>, I: IntoIterator<Item = S>>(iter: I) -> impl Predicate<str> {
    any(iter
        .into_iter()
        .map(|str| BoxPredicate::new(contains(str.as_ref()))))
}
