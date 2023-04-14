use core::{fmt, hash::Hash};
use predicates::{
    ord::eq,
    reflection::{self, PredicateReflection},
    Predicate,
};
use serde_json::Value;
use std::borrow::Borrow;

#[derive(Clone, Debug)]
struct JsonPredicate<P>(P)
where
    P: Predicate<Value>;

impl<P> Predicate<[u8]> for JsonPredicate<P>
where
    P: Predicate<Value>,
{
    fn eval(&self, bytes: &[u8]) -> bool {
        match serde_json::from_slice(bytes) {
            Ok(value) => self.0.eval(&value),
            Err(err) => {
                eprintln!("{}", err);
                false
            }
        }
    }
    fn find_case<'a>(&'a self, expected: bool, variable: &[u8]) -> Option<reflection::Case<'a>> {
        let case = reflection::Case::new(Some(self), expected);
        match (expected, serde_json::from_slice(variable)) {
            (_, Ok(value)) => self
                .0
                .find_case(expected, &value)
                .map(|child| case.add_child(child)),
            (false, Err(err)) => {
                Some(case.add_product(reflection::Product::new("serde_error", err)))
            }
            _ => None,
        }
    }
}

impl<P> PredicateReflection for JsonPredicate<P>
where
    P: Predicate<Value>,
{
    fn children<'a>(&'a self) -> Box<dyn Iterator<Item = predicates::reflection::Child<'a>> + 'a> {
        let children = vec![reflection::Child::new("predicate", &self.0)];
        Box::new(children.into_iter())
    }
}

impl<P> fmt::Display for JsonPredicate<P>
where
    P: Predicate<Value>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "json(predicate)")
    }
}

/// Given a predicate over a serde_json value, produces a predicate over bytes by deserializing the input
/// and then applying the given predicate
pub fn json<P>(predicate: P) -> impl Predicate<[u8]>
where
    P: Predicate<serde_json::Value>,
{
    JsonPredicate(predicate)
}

#[derive(Clone, Debug)]
pub struct JsonPropPredicate<'a, K, P>
where
    String: Borrow<K>,
    K: Ord + Eq + Hash + fmt::Debug + fmt::Display + ?Sized,
    P: Predicate<Value> + 'a,
{
    key: &'a K,
    predicate: P,
}

impl<'a, K, P> Predicate<Value> for JsonPropPredicate<'a, K, P>
where
    String: Borrow<K>,
    K: Ord + Eq + Hash + fmt::Debug + fmt::Display + ?Sized,
    P: Predicate<Value> + 'a,
{
    fn eval(&self, value: &Value) -> bool {
        value
            .as_object()
            .and_then(|obj| obj.get(self.key).map(|v| self.predicate.eval(v)))
            .unwrap_or(false)
    }
    fn find_case<'b>(&'b self, expected: bool, variable: &Value) -> Option<reflection::Case<'b>> {
        let obj = variable.as_object();
        if !expected && obj.is_none() {
            return Some(
                reflection::Case::new(Some(self), expected)
                    .add_product(reflection::Product::new("var.is_object", expected)),
            );
        }
        let val = obj?.get(self.key);
        if !expected && val.is_none() {
            return Some(reflection::Case::new(Some(self), expected).add_product(
                reflection::Product::new(format!("var.contains({})", self.key), expected),
            ));
        }
        self.predicate
            .find_case(expected, val?)
            .map(|child| reflection::Case::new(Some(self), expected).add_child(child))
    }
}

impl<'a, K, P> PredicateReflection for JsonPropPredicate<'a, K, P>
where
    String: Borrow<K>,
    K: Ord + Eq + Hash + fmt::Debug + fmt::Display + ?Sized,
    P: Predicate<Value> + 'a,
{
    fn children<'b>(&'b self) -> Box<dyn Iterator<Item = reflection::Child<'b>> + 'b> {
        let children = vec![reflection::Child::new("predicate", &self.predicate)];
        Box::new(children.into_iter())
    }
}

impl<'a, K, P> fmt::Display for JsonPropPredicate<'a, K, P>
where
    String: Borrow<K>,
    K: Ord + Eq + Hash + fmt::Debug + fmt::Display + ?Sized,
    P: Predicate<Value> + 'a,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "prop({:?}, predicate)", self.key)
    }
}

/// Predicate over a property of a JSON object
pub fn prop<'a, K, P>(key: &'a K, predicate: P) -> impl Predicate<Value> + 'a
where
    String: Borrow<K>,
    K: Ord + Eq + Hash + fmt::Debug + fmt::Display + ?Sized,
    P: Predicate<Value> + 'a,
{
    JsonPropPredicate { key, predicate }
}

/// Predicate that checks equality of two JSON values
pub fn value<V: Into<Value>>(value: V) -> impl Predicate<Value> {
    eq(value.into())
}

#[derive(Clone, Debug)]
pub struct FindEntryPredicate<KP, VP>
where
    KP: Predicate<Value>,
    VP: Predicate<Value>,
{
    key_predicate: KP,
    value_predicate: VP,
}

impl<KP, VP> Predicate<Value> for FindEntryPredicate<KP, VP>
where
    KP: Predicate<Value>,
    VP: Predicate<Value>,
{
    fn eval(&self, value: &Value) -> bool {
        value
            .as_array()
            .and_then(|arr| {
                arr.iter()
                    .find(|e| self.key_predicate.eval(e))
                    .map(|e| self.value_predicate.eval(e))
            })
            .unwrap_or(false)
    }
    fn find_case<'a>(&'a self, expected: bool, variable: &Value) -> Option<reflection::Case<'a>> {
        let arr = variable.as_array();
        if !expected && arr.is_none() {
            return Some(
                reflection::Case::new(Some(self), expected)
                    .add_product(reflection::Product::new("var.is_array()", expected)),
            );
        }
        let entry = arr?.iter().find_map(|val| {
            self.key_predicate
                .find_case(true, val)
                .map(|key_case| (val, key_case))
        });
        if !expected && entry.is_none() {
            let mut case = reflection::Case::new(Some(self), expected);
            let children = arr?
                .iter()
                .filter_map(|val| self.key_predicate.find_case(expected, val));
            for child in children {
                case = case.add_child(child);
            }
            return Some(case);
        }
        let (val, key_case) = entry?;
        let val_case = self.value_predicate.find_case(expected, val)?;
        Some(
            reflection::Case::new(Some(self), expected)
                .add_child(key_case)
                .add_child(val_case),
        )
    }
}

impl<KP, VP> PredicateReflection for FindEntryPredicate<KP, VP>
where
    KP: Predicate<Value>,
    VP: Predicate<Value>,
{
    fn children<'a>(&'a self) -> Box<dyn Iterator<Item = reflection::Child<'a>> + 'a> {
        let children = vec![
            reflection::Child::new("key_predicate", &self.key_predicate),
            reflection::Child::new("value_predicate", &self.value_predicate),
        ];
        Box::new(children.into_iter())
    }
}

impl<KP, VP> fmt::Display for FindEntryPredicate<KP, VP>
where
    KP: Predicate<Value>,
    VP: Predicate<Value>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "find_entry(key, value)")
    }
}

/// Iterates over JSON array elements and looks for one value to match the predicate
pub fn find_entry<KP, VP>(key_predicate: KP, value_predicate: VP) -> impl Predicate<Value>
where
    KP: Predicate<Value>,
    VP: Predicate<Value>,
{
    FindEntryPredicate {
        key_predicate,
        value_predicate,
    }
}
