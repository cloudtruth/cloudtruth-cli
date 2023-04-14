use core::hash::Hash;
use predicates::{ord::eq, Predicate};
use serde_json::Value;
use std::borrow::Borrow;

/// Given a predicate over a serde_json value, produces a predicate over bytes by deserializing the input
/// and then applying the given predicate
pub fn json<P>(predicate: P) -> impl Predicate<[u8]>
where
    P: Predicate<serde_json::Value>,
{
    predicates::function::function(move |bytes| match serde_json::from_slice(bytes) {
        Ok(value) => predicate.eval(&value),
        Err(err) => {
            eprintln!("{}", err);
            false
        }
    })
    .fn_name("json")
}

/// Predicate over a property of a JSON object
pub fn prop<'a, K, P>(key: &'a K, predicate: P) -> impl Predicate<Value> + 'a
where
    String: Borrow<K>,
    K: Ord + Eq + Hash + ?Sized + core::fmt::Debug,
    P: Predicate<Value> + 'a,
{
    predicates::function::function(move |json: &Value| {
        json.as_object()
            .and_then(|obj| obj.get(key).map(|v| predicate.eval(v)))
            .unwrap_or(false)
    })
    .fn_name("prop")
}

/// Predicate that checks equality of two JSON values
pub fn value<V: Into<Value>>(value: V) -> impl Predicate<Value> {
    eq(value.into())
}

/// Iterates over JSON array elements and looks for one value to match the predicate
pub fn array_with_element<P>(predicate: P) -> impl Predicate<Value>
where
    P: Predicate<Value>,
{
    predicates::function::function(move |json: &Value| {
        json.as_array()
            .map(|arr| arr.iter().any(|e| predicate.eval(e)))
            .unwrap_or(false)
    })
    .fn_name("array_with_element")
}
