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
//! See below example of a resolvable JSON structure and how it might be resolved
//! from the system.
//!
//! ```json
//! {
//!   "database_connection_string": "env://FOO_BAR",
//!   "player_to_privileges_mapping": "file-json:///opt/player-privileges.json",
//!   "global_greeting": "file:///opt/greeting.txt",
//!   "nesting": {
//!     "many": ["file-json:///opt/foo.json", "file-json:///opt/sketchy.json"]
//!   }
//! }
//! ```
//!
//! After deserializing the resolvable JSON structure into [serde_json::Value], and
//! then using the [Resolver] trait to get a resolved [serde_json::Value], you may
//! deserialize it into some structure of your own. Below is an example of what kind
//! of structure you might expect from your JSON:
//!
//! ```rust
//! struct MyResolvedStructure {
//!   database_connection_string: String,
//!   procedural_gen_seed: Vec<u8>,
//!   player_to_privileges_mapping: serde_json::Value,
//!   global_greeting: String,
//!   nesting: MyNestedThingA,
//! }
//!
//! struct MyNestedThingA {
//!   many: Vec<MyNestedThingB>,
//! }
//!
//! struct MyNestedThingB {
//!   whatever: u32,
//! }
//! ```
//!
//! Refer to the _serde ecosystem_ for how to move between its representations and
//! your arbitrary structures. Possibly useful learning material: [_Decrusting the
//! serde crate_ on YouTube by Jon Gjengset](https://youtu.be/BI_bHCGRgMY) (accessed
//! 2026-03-15).

use std::str::FromStr;

pub trait Resolver {
    fn resolve(self) -> Result<serde_json::Value, Box<dyn std::error::Error>>;
}

impl Resolver for serde_json::Value {
    fn resolve(self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        match self {
            serde_json::Value::Null => return Ok(self),
            serde_json::Value::Bool(_) => return Ok(self),
            serde_json::Value::Number(_) => return Ok(self),

            serde_json::Value::Array(items) => {
                let mut resolved: Vec<serde_json::Value> = Vec::new();
                for item in items {
                    let n: serde_json::Value = item.resolve()?;
                    resolved.push(n);
                }
                return Ok(serde_json::Value::Array(resolved));
            }

            serde_json::Value::Object(entries) => {
                let mut map: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
                for (key, value) in entries.into_iter() {
                    let resolved: serde_json::Value = value.resolve()?;
                    map.insert(key, resolved);
                }
                return Ok(serde_json::Value::Object(map));
            }

            serde_json::Value::String(ref resolvable) => {
                if let Some((head, tail)) = resolvable.split_once(DELIMITER) {
                    match head {
                        "env" => {
                            let content: String = std::env::var(tail)?;
                            return Ok(serde_json::Value::String(content));
                        }

                        "file" => {
                            let content: String = std::fs::read_to_string(tail)?;
                            return Ok(serde_json::Value::String(content));
                        }
                        "file-json" => {
                            let content: String = std::fs::read_to_string(tail)?;
                            let json: serde_json::Value = serde_json::Value::from_str(&content)?;
                            return Ok(json);
                        }

                        _ => {
                            /*
                             * TODO: Ignore most, e.g. "https://" etc., but check some.
                             *       If starts with "env" or "file", then return some
                             *       special error (for reserving for future extension).
                             */
                            todo!();
                        }
                    }
                } else {
                    return Ok(self);
                }
            }
        }
    }
}

const DELIMITER: &'static str = "://";

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
}
