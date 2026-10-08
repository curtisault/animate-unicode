//! A piece's option overrides, as JSON from the tag's `options` attribute, the
//! React prop or the CLI flag. Pieces read them with typed getters that fall
//! back to a default, so a wrong type is never an error, just the default.
//!
//! Every key a getter is asked for is recorded, so the contract test can check
//! that a piece reads only options its `Meta::options` gives a default for
//! (the site builds its option controls from those defaults).

use serde_json::{Map, Value};
use std::cell::RefCell;
use std::collections::BTreeSet;

/// Option values by name, with typed getters that fall back to a default.
#[derive(Clone, Debug, Default)]
pub struct Options {
    values: Map<String, Value>,
    read: RefCell<BTreeSet<String>>,
}

impl Options {
    /// Parses a JSON object. Anything else, including the empty string, is no options.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        if json.trim().is_empty() {
            return Ok(Self::default());
        }
        match serde_json::from_str::<Value>(json)? {
            Value::Object(values) => Ok(Self { values, read: RefCell::default() }),
            _ => Ok(Self::default()),
        }
    }

    /// Overrides layered on a piece's `Meta::options` defaults.
    pub fn with_defaults(mut self, defaults: Option<&str>) -> Self {
        if let Some(d) = defaults {
            if let Ok(base) = Self::from_json(d) {
                for (k, v) in base.values {
                    self.values.entry(k).or_insert(v);
                }
            }
        }
        self
    }

    /// The value under `key`, noting that the key was read.
    fn get(&self, key: &str) -> Option<&Value> {
        self.read.borrow_mut().insert(key.to_owned());
        self.values.get(key)
    }

    /// A number, or `default` if missing or not a number.
    pub fn f64(&self, key: &str, default: f64) -> f64 {
        self.get(key).and_then(Value::as_f64).unwrap_or(default)
    }

    /// An integer, or `default` if missing or not an integer.
    pub fn i64(&self, key: &str, default: i64) -> i64 {
        self.get(key).and_then(Value::as_i64).unwrap_or(default)
    }

    /// A boolean, or `default` if missing or not a boolean.
    pub fn bool(&self, key: &str, default: bool) -> bool {
        self.get(key).and_then(Value::as_bool).unwrap_or(default)
    }

    /// A string borrowed from the options, or `default` if missing or not a string.
    pub fn str<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get(key).and_then(Value::as_str).unwrap_or(default)
    }

    /// An owned string, for a piece that keeps it past `make`: `str`, copied.
    pub fn string(&self, key: &str, default: &str) -> String {
        self.str(key, default).to_owned()
    }

    /// A list of numbers, or `default` if missing, not a list, or holding
    /// anything that is not a number.
    pub fn list_f64(&self, key: &str, default: &[f64]) -> Vec<f64> {
        self.get(key).and_then(Value::as_array).and_then(|a| a.iter().map(Value::as_f64).collect::<Option<Vec<_>>>()).unwrap_or_else(|| default.to_vec())
    }

    /// True if there are no options at all.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Every option name present, defaults included, sorted.
    pub fn keys(&self) -> Vec<&str> {
        self.values.keys().map(String::as_str).collect()
    }

    /// Every key a getter has been asked for so far, sorted, present or not.
    pub fn read_keys(&self) -> Vec<String> {
        self.read.borrow().iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_typed_values_with_defaults() {
        let o = Options::from_json(r#"{"speed": 2.5, "text": "hi", "on": true}"#).unwrap();
        assert_eq!(o.f64("speed", 1.0), 2.5);
        assert_eq!(o.f64("missing", 1.0), 1.0);
        assert_eq!(o.str("text", ""), "hi");
        assert_eq!(o.str("speed", "x"), "x"); // wrong type falls back
        assert!(o.bool("on", false));
    }

    #[test]
    fn defaults_fill_only_the_gaps() {
        let o = Options::from_json(r#"{"a": 1}"#).unwrap().with_defaults(Some(r#"{"a": 9, "b": 2}"#));
        assert_eq!(o.i64("a", 0), 1);
        assert_eq!(o.i64("b", 0), 2);
    }

    #[test]
    fn empty_and_non_object_are_no_options() {
        assert!(Options::from_json("").unwrap().is_empty());
        assert!(Options::from_json("[1,2]").unwrap().is_empty());
        assert!(Options::from_json("{").is_err());
    }

    #[test]
    fn owned_strings_and_number_lists() {
        let o = Options::from_json(r#"{"name": "ok", "xs": [1, 2.5], "mixed": [1, "a"], "n": 3}"#).unwrap();
        assert_eq!(o.string("name", "x"), "ok");
        assert_eq!(o.string("n", "x"), "x");
        assert_eq!(o.list_f64("xs", &[]), vec![1.0, 2.5]);
        assert_eq!(o.list_f64("mixed", &[9.0]), vec![9.0], "one bad element falls back whole");
        assert_eq!(o.list_f64("n", &[9.0]), vec![9.0]);
        assert_eq!(o.list_f64("missing", &[]), Vec::<f64>::new());
    }

    #[test]
    fn records_every_key_read() {
        let o = Options::from_json(r#"{"a": 1, "b": 2}"#).unwrap();
        assert!(o.read_keys().is_empty());
        o.f64("a", 0.0);
        o.bool("typo", false);
        o.f64("a", 0.0);
        assert_eq!(o.read_keys(), vec!["a", "typo"]);
        assert_eq!(o.keys(), vec!["a", "b"]);
    }
}
