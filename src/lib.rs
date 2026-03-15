//! Resolve string values with specific prefixes in a JSON structure from
//! environment variables and file system.
//!
//! | head            | tail               | pipeline                              |
//! | --------------- | ------------------ | ------------------------------------- |
//! | `env://`        | name of an env var | as UTF-8 string                       |
//! | `file://`       | file system path   | as UTF-8 string                       |
//! | `file-json://`  | file system path   | as UTF-8 string → `serde_json::Value` |
//!
//! The resolvable JSON structure may be deeply nested and complex, and still
//! all leaf string values are resolved. That is, the JSON structure is traversed
//! recursively. However, the resolving is not recursive: A value that resolves to
//! a string with one of the resolving prefixes is not attempted to be resolved, but
//! is kept as string instead.
//!
//! ## Example
//!
//! After deserializing a resolvable JSON structure into [serde_json::Value], and
//! then using the [Resolver] trait to get a resolved [serde_json::Value], you may
//! deserialize it into some structure of your own.
//!
//! ```rust
//! fn main() {
//!     let json: serde_json::Value = serde_json::from_str(SERIALIZED_JSON).unwrap();
//!
//!     let resolved: serde_json::Value = system2json::Resolver::resolve(json).unwrap();
//!
//!     let deserialized: MyComplicatedStructure = serde_json::from_value(resolved).unwrap();
//! }
//!
//! const SERIALIZED_JSON: &str = r#"{
//!   "aaa": "file://test-files/greeting.txt",
//!   "bbb": "file-json://test-files/sketchy.json",
//!   "nested": {
//!     "many": [
//!       "file-json://./test-files/sketchy.json",
//!       "file-json://test-files/sketchy.json"
//!     ]
//!   },
//!   "ccc": 1,
//!   "ddd": "file://test-files/sketchy.json",
//!   "cargo_binary_path": "env://CARGO"
//! }"#;
//!
//! #[derive(Debug, serde::Deserialize)]
//! struct MyComplicatedStructure {
//!     aaa: String,
//!     bbb: MyNestedThingB,
//!     ccc: u32,
//!     ddd: String,
//!     nested: MyNestedThingA,
//!     cargo_binary_path: std::path::PathBuf,
//! }
//!
//! #[derive(Debug, serde::Deserialize)]
//! struct MyNestedThingA {
//!     many: Vec<MyNestedThingB>,
//! }
//!
//! #[derive(Debug, serde::Deserialize)]
//! struct MyNestedThingB {
//!     should_not_be_resolved: String,
//! }
//! ```
//!
//! ## What is _serde_?
//!
//! Refer to the _serde ecosystem_ for more information on how to move
//! between its representations and your arbitrary structures.
//!
//! Possibly useful learning material:
//!
//! - [_Decrusting the serde crate_ on YouTube by Jon Gjengset](https://youtu.be/BI_bHCGRgMY)
//!   (accessed 2026-03-15)
//!
//! - [serde on docs.rs](https://docs.rs/serde/latest/serde/)
//!   (accessed 2026-03-15)

use std::str::FromStr;

pub trait Resolver {
    fn resolve(self) -> Result<serde_json::Value, Box<dyn std::error::Error>>;
}

impl Resolver for serde_json::Value {
    fn resolve(self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        match self {
            serde_json::Value::Null => Ok(self),
            serde_json::Value::Bool(_) => Ok(self),
            serde_json::Value::Number(_) => Ok(self),

            serde_json::Value::Array(items) => {
                let mut resolved: Vec<serde_json::Value> = Vec::new();
                for item in items {
                    let n: serde_json::Value = item.resolve()?;
                    resolved.push(n);
                }
                Ok(serde_json::Value::Array(resolved))
            }

            serde_json::Value::Object(entries) => {
                let mut map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
                for (key, value) in entries.into_iter() {
                    let resolved: serde_json::Value = value.resolve()?;
                    map.insert(key, resolved);
                }
                Ok(serde_json::Value::Object(map))
            }

            serde_json::Value::String(ref resolvable) => {
                if let Some((head, tail)) = resolvable.split_once(DELIMITER) {
                    match head {
                        "env" => {
                            let content: String = std::env::var(tail)?;
                            Ok(serde_json::Value::String(content))
                        }

                        "file" => {
                            let content: String = std::fs::read_to_string(tail)?;
                            Ok(serde_json::Value::String(content))
                        }
                        "file-json" => {
                            let content: String = std::fs::read_to_string(tail)?;
                            let json: serde_json::Value = serde_json::Value::from_str(&content)?;
                            Ok(json)
                        }

                        /*
                         * Ignore most, e.g. "https://" etc., but check some:
                         * If starts with "env" or "file", then return Err to
                         * reserve for future extension.
                         */
                        _ => {
                            if head.starts_with("env") || head.starts_with("file") {
                                Err(Box::new(Error::ResolvingPrefixNotSupported {
                                    prefix_attempted: format!("{head}{DELIMITER}"),
                                }))
                            } else {
                                Ok(self)
                            }
                        }
                    }
                } else {
                    Ok(self)
                }
            }
        }
    }
}

#[derive(Debug)]
enum Error {
    ResolvingPrefixNotSupported { prefix_attempted: String },
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix_attempted: &str = match self {
            Error::ResolvingPrefixNotSupported { prefix_attempted } => prefix_attempted,
        };
        write!(f, r#"resolving prefix not supported: "{prefix_attempted}""#)
    }
}

const DELIMITER: &str = "://";

#[cfg(test)]
mod test_keep_intact {
    #[test]
    fn record_simple() {
        let deserialized: serde_json::Value = serde_json::from_str(r#"{"foo":"bar"}"#).unwrap();

        let resolved: serde_json::Value = crate::Resolver::resolve(deserialized).unwrap();

        let serialized: String = serde_json::to_string(&resolved).unwrap();

        assert_eq!(serialized, r#"{"foo":"bar"}"#);
    }

    #[test]
    fn nested_complex() {
        let deserialized: serde_json::Value = serde_json::from_str(
            r#"[{"foo":{"bar":"baz","array":[{"aaa":{"num":-1}},1,"asd"]}},{"foo":{"bar":"baz"}}]"#,
        )
        .unwrap();

        let resolved: serde_json::Value = crate::Resolver::resolve(deserialized).unwrap();

        let serialized: String = serde_json::to_string(&resolved).unwrap();

        assert_eq!(
            serialized,
            r#"[{"foo":{"bar":"baz","array":[{"aaa":{"num":-1}},1,"asd"]}},{"foo":{"bar":"baz"}}]"#
        );
    }
}

#[cfg(test)]
mod test_file_system {
    #[test]
    fn relative_path_with_dot() {
        let deserialized: serde_json::Value =
            serde_json::from_str(r#"{"greeting":"file://./test-files/greeting.txt"}"#).unwrap();

        let resolved: serde_json::Value = crate::Resolver::resolve(deserialized).unwrap();

        let serialized: String = serde_json::to_string(&resolved).unwrap();

        assert_eq!(serialized, r#"{"greeting":"Hello world!\n"}"#);
    }

    #[test]
    fn relative_path_without_dot() {
        let deserialized: serde_json::Value =
            serde_json::from_str(r#"{"greeting":"file://test-files/greeting.txt"}"#).unwrap();

        let resolved: serde_json::Value = crate::Resolver::resolve(deserialized).unwrap();

        let serialized: String = serde_json::to_string(&resolved).unwrap();

        assert_eq!(serialized, r#"{"greeting":"Hello world!\n"}"#);
    }

    #[test]
    fn json_not_resolved_recursively() {
        let deserialized: serde_json::Value = serde_json::from_str(
            r#"{"not_resolved_recursively":"file-json://test-files/sketchy.json"}"#,
        )
        .unwrap();

        let resolved: serde_json::Value = crate::Resolver::resolve(deserialized).unwrap();

        let serialized: String = serde_json::to_string(&resolved).unwrap();

        assert_eq!(
            serialized,
            r#"{"not_resolved_recursively":{"should_not_be_resolved":"file://test-files/greeting.txt"}}"#
        );
    }

    #[test]
    fn nested_complex() {
        let deserialized: serde_json::Value = serde_json::from_str(
            r#"[
  {
    "aaa": "file://test-files/greeting.txt",
    "bbb": "file://test-files/greeting.txt",
    "ccc": "file://test-files/sketchy.json",
    "nested": {
      "arr": [
        -1,
        "file://test-files/greeting.txt",
        null,
        "file-json://test-files/sketchy.json",
        1
      ]
    },
    "ddd": "file://test-files/greeting.txt"
  }
]"#,
        )
        .unwrap();

        let resolved: serde_json::Value = crate::Resolver::resolve(deserialized).unwrap();

        let serialized: String = serde_json::to_string_pretty(&resolved).unwrap();

        assert_eq!(
            serialized,
            r#"[
  {
    "aaa": "Hello world!\n",
    "bbb": "Hello world!\n",
    "ccc": "{\"should_not_be_resolved\":\"file://test-files/greeting.txt\"}\n",
    "nested": {
      "arr": [
        -1,
        "Hello world!\n",
        null,
        {
          "should_not_be_resolved": "file://test-files/greeting.txt"
        },
        1
      ]
    },
    "ddd": "Hello world!\n"
  }
]"#
        );
    }
}

#[cfg(test)]
mod test_reserved {
    #[test]
    fn toml() {
        let deserialized: serde_json::Value =
            serde_json::from_str(r#"{"reserved":"file-toml:///etc/hosts.toml"}"#).unwrap();

        let attempt = crate::Resolver::resolve(deserialized);
        assert!(matches!(attempt, Err(..)));

        let err: Box<dyn std::error::Error> = match attempt {
            Ok(_) => panic!(),
            Err(err) => err,
        };

        let err_display: String = format!("{err}");
        assert_eq!(
            err_display,
            r#"resolving prefix not supported: "file-toml://""#
        );
    }

    #[test]
    fn env_hex() {
        let deserialized: serde_json::Value =
            serde_json::from_str(r#"{"reserved":"env-hex://FOO_BAR"}"#).unwrap();

        let attempt = crate::Resolver::resolve(deserialized);
        assert!(matches!(attempt, Err(..)));

        let err: Box<dyn std::error::Error> = match attempt {
            Ok(_) => panic!(),
            Err(err) => err,
        };

        let err_display: String = format!("{err}");
        assert_eq!(
            err_display,
            r#"resolving prefix not supported: "env-hex://""#
        );
    }
}
