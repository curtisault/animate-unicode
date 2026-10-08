//! A piece's option overrides, as JSON from the tag's `options` attribute, the
//! React prop or the CLI flag. Pieces read them with typed getters that fall
//! back to a default, so a wrong type is never an error, just the default.

use serde_json::{Map, Value};

#[derive(Clone, Debug, Default)]
pub struct Options(Map<String, Value>);

impl Options {
    /// Parses a JSON object. Anything else, including the empty string, is no options.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        if json.trim().is_empty() {
            return Ok(Self::default());
        }
        match serde_json::from_str::<Value>(json)? {
            Value::Object(map) => Ok(Self(map)),
            _ => Ok(Self::default()),
        }
    }

    /// Overrides layered on a piece's `Meta::options` defaults.
    pub fn with_defaults(mut self, defaults: Option<&str>) -> Self {
        if let Some(d) = defaults {
            if let Ok(Self(base)) = Self::from_json(d) {
                for (k, v) in base {
                    self.0.entry(k).or_insert(v);
                }
            }
        }
        self
    }

    pub fn f64(&self, key: &str, default: f64) -> f64 {
        self.0.get(key).and_then(Value::as_f64).unwrap_or(default)
    }

    pub fn i64(&self, key: &str, default: i64) -> i64 {
        self.0.get(key).and_then(Value::as_i64).unwrap_or(default)
    }

    pub fn bool(&self, key: &str, default: bool) -> bool {
        self.0.get(key).and_then(Value::as_bool).unwrap_or(default)
    }

    pub fn str<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.0.get(key).and_then(Value::as_str).unwrap_or(default)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
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
}
