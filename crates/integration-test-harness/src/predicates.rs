/// Custom predicates for easier assertions
use predicates::{
    boolean::PredicateBooleanExt, constant::always, str::contains, BoxPredicate, Predicate,
};

pub mod json;

pub use json::*;

/// Helper to allow prefix form of not(predicate) instead of predicate.not()
pub fn not<P, I>(predicate: P) -> impl Predicate<I>
where
    P: PredicateBooleanExt<I>,
    I: ?Sized,
{
    predicate.not()
}

/// Predicate over the length of a slice
pub fn len<T>(length: usize) -> impl Predicate<[T]> {
    predicates::function::function(move |slice: &[T]| slice.len() == length).fn_name("len")
}

/// Checks that variable contains all strings from an iterator
pub fn contains_all<S: Into<String>, I: IntoIterator<Item = S>>(iter: I) -> impl Predicate<str> {
    iter.into_iter()
        .fold(BoxPredicate::new(always()), |accum, str| {
            BoxPredicate::new(accum.and(contains(str)))
        })
}
