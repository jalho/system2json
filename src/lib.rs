//! Resolve string values with specific prefixes in a JSON structure from
//! environment variables and file system.
//!
//! | head            | tail               | pipeline                                      | feature |
//! | --------------- | ------------------ | --------------------------------------------- | ------- |
//! | `env://`        | name of an env var | as UTF-8 string                               |         |
//! | `file://`       | file system path   | as UTF-8 string                               |         |
//! | `file-json://`  | file system path   | as UTF-8 string → JSON → `serde_json::Value`  |         |
//! | `file-json5://` | file system path   | as UTF-8 string → JSON5 → `serde_json::Value` | `json5` |
//! | `file-toml://`  | file system path   | as UTF-8 string → TOML → `serde_json::Value`  | `toml`  |
//! | `file-yaml://`  | file system path   | as UTF-8 string → YAML → `serde_json::Value`  | `yaml`  |
//!
//! ### Design
//!
//! - **Recursivity:** The resolvable structure may be deeply nested and complex,
//!   and still all leaf string values are resolved. That is, the resolvable
//!   structure is _traversed recursively_. However, the _resolving_ is not
//!   recursive: A value that resolves to a string with one of the resolving
//!   prefixes is not attempted to be resolved, but is kept as string instead.
//!
//! - **Gated Features:** Some formats are gated behind crate features, as per the
//!   above table. The trait method [Resolver::resolve] returns a [Result::Err] when
//!   resolving a gated format is attempted.
//!
//! - **Notation and Networks:** Despite the notation resembling that used in the
//!   web (e.g. `https://whatever.internal:443`), This library does not directly
//!   intend to support resolving values over a network.
//!
//! ## Example: Resolvable Content in a Static Buffer
//!
//! After deserializing a resolvable JSON structure into [serde_json::Value], and
//! then using the [Resolver] trait to get a resolved [serde_json::Value], you may
//! deserialize it into some structure of your own.
//!
//! ```rust
//! fn main() {
//!     let json: serde_json::Value = serde_json::from_str(STATIC_BUFFER).unwrap();
//!
//!     let resolved: serde_json::Value = system2json::Resolver::resolve(json).unwrap();
//!
//!     let deserialized: MyComplicatedStructure = serde_json::from_value(resolved).unwrap();
//! }
//!
//! const STATIC_BUFFER: &str = r#"{
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
//! #[derive(serde::Deserialize)]
//! struct MyComplicatedStructure {
//!     aaa: String,
//!     bbb: MyNestedThingB,
//!     ccc: u32,
//!     ddd: String,
//!     nested: MyNestedThingA,
//!     cargo_binary_path: std::path::PathBuf,
//! }
//!
//! #[derive(serde::Deserialize)]
//! struct MyNestedThingA {
//!     many: Vec<MyNestedThingB>,
//! }
//!
//! #[derive(serde::Deserialize)]
//! struct MyNestedThingB {
//!     should_not_be_resolved: String,
//! }
//! ```
//!
//! Note that the `CARGO` env var read in the example is automatically
//! set by `cargo` for example when you run these tests: See
//! [docs](https://doc.rust-lang.org/cargo/reference/environment-variables.html)
//! (accessed 2026-03-15).
//!
//! For the referenced test files, see the repository.
//!
//! ## Example: Resolve Values in a TOML File
//!
//! ```rust
//! fn main() {
//!     let toml_serialized: String =
//!         std::fs::read_to_string("test-files/has-resolvable-values.toml").unwrap();
//!
//!     let json: serde_json::Value = toml::from_str(&toml_serialized).unwrap();
//!
//!     use system2json::Resolver;
//!     let resolved = json.resolve().unwrap();
//!
//!     let deserialized: MyComplicatedStructure = serde_json::from_value(resolved).unwrap();
//! }
//!
//! #[derive(serde::Deserialize)]
//! struct MyComplicatedStructure {
//!     foo: MyNestedThing,
//!     baz: MyNestedThing,
//! }
//!
//! #[derive(serde::Deserialize)]
//! struct MyNestedThing {
//!     spam: String,
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
                        "file-json5" => {
                            #[cfg(not(feature = "json5"))]
                            {
                                Err(Box::new(Error::ResolvingPrefixNotSupported {
                                    prefix_attempted: format!("{head}{DELIMITER}"),
                                }))
                            }

                            #[cfg(feature = "json5")]
                            {
                                let content: String = std::fs::read_to_string(tail)?;
                                let json: serde_json::Value = serde_json5::from_str(&content)?;
                                Ok(json)
                            }
                        }

                        "file-toml" => {
                            #[cfg(not(feature = "toml"))]
                            {
                                Err(Box::new(Error::ResolvingPrefixNotSupported {
                                    prefix_attempted: format!("{head}{DELIMITER}"),
                                }))
                            }

                            #[cfg(feature = "toml")]
                            {
                                let content: String = std::fs::read_to_string(tail)?;
                                let json: serde_json::Value = toml::from_str(&content)?;
                                Ok(json)
                            }
                        }

                        "file-yaml" => {
                            #[cfg(not(feature = "yaml"))]
                            {
                                Err(Box::new(Error::ResolvingPrefixNotSupported {
                                    prefix_attempted: format!("{head}{DELIMITER}"),
                                }))
                            }

                            #[cfg(feature = "yaml")]
                            {
                                let content: String = std::fs::read_to_string(tail)?;
                                let json: serde_json::Value = serde_yaml::from_str(&content)?;
                                Ok(json)
                            }
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
    fn json_nested_not_resolved_recursively() {
        let deserialized: serde_json::Value = serde_json::from_str(
            r#"{"not_resolved_recursively":"file-json://test-files/sketchy-nested.json"}"#,
        )
        .unwrap();

        let resolved: serde_json::Value = crate::Resolver::resolve(deserialized).unwrap();

        let serialized: String = serde_json::to_string_pretty(&resolved).unwrap();
        println!("{serialized}");

        assert_eq!(
            serialized,
            r#"
{
  "not_resolved_recursively": {
    "arr": [
      {
        "should_not_be_resolved": "file://test-files/greeting.txt"
      }
    ],
    "record": {
      "should_not_be_resolved": "file://test-files/greeting.txt"
    }
  }
}
"#
            .trim()
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
#[cfg(feature = "json5")]
mod test_json5 {
    #[test]
    fn comments() {
        let deserialized: serde_json::Value =
            serde_json::from_str(r#"{"content":"file-json5://./test-files/has-comments.jsonc"}"#)
                .unwrap();

        let resolved: serde_json::Value = crate::Resolver::resolve(deserialized).unwrap();

        let serialized: String = serde_json::to_string(&resolved).unwrap();

        assert_eq!(
            serialized,
            r#"{"content":{"foo":"bar","baz":"spam","trailing_comma":1}}"#
        );
    }
}

#[cfg(test)]
#[cfg(feature = "toml")]
mod test_toml {
    #[test]
    fn comments() {
        let deserialized: serde_json::Value =
            serde_json::from_str(r#"{"content":"file-toml://test-files/has-comments.toml"}"#)
                .unwrap();

        let resolved: serde_json::Value = crate::Resolver::resolve(deserialized).unwrap();

        let serialized: String = serde_json::to_string_pretty(&resolved).unwrap();

        assert_eq!(
            serialized,
            r#"
{
  "content": {
    "foo": {
      "bar": {
        "baz": 1
      }
    }
  }
}
"#
            .trim()
        );
    }
}

#[cfg(test)]
#[cfg(feature = "yaml")]
mod test_yaml {
    #[test]
    fn comments() {
        let deserialized: serde_json::Value =
            serde_json::from_str(r#"{"content":"file-yaml://test-files/has-comments.yaml"}"#)
                .unwrap();

        let resolved: serde_json::Value = crate::Resolver::resolve(deserialized).unwrap();

        let serialized: String = serde_json::to_string_pretty(&resolved).unwrap();

        assert_eq!(
            serialized,
            r#"
{
  "content": {
    "foo": {
      "bar": {
        "spam": null
      },
      "baz": [
        "aaa",
        "bb",
        3
      ]
    }
  }
}
"#
            .trim()
        );
    }
}

#[cfg(test)]
mod test_reserved {
    /*
     * tfwhne = This Format Will Hopefully Never Exist
     */
    #[test]
    fn tfwhne() {
        let deserialized: serde_json::Value =
            serde_json::from_str(r#"{"reserved":"file-tfwhne:///etc/hosts.tfwhne"}"#).unwrap();

        let attempt = crate::Resolver::resolve(deserialized);
        assert!(matches!(attempt, Err(..)));

        let err: Box<dyn std::error::Error> = match attempt {
            Ok(_) => panic!(),
            Err(err) => err,
        };

        let err_display: String = format!("{err}");
        assert_eq!(
            err_display,
            r#"resolving prefix not supported: "file-tfwhne://""#
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

#[cfg(test)]
mod test_ignored {
    #[test]
    fn http() {
        let deserialized: serde_json::Value =
            serde_json::from_str(r#"{"not-resolved":"http://127.0.0.1:8080"}"#).unwrap();

        let resolved: serde_json::Value = crate::Resolver::resolve(deserialized).unwrap();

        let serialized: String = serde_json::to_string(&resolved).unwrap();

        assert_eq!(serialized, r#"{"not-resolved":"http://127.0.0.1:8080"}"#);
    }

    #[test]
    fn https() {
        let deserialized: serde_json::Value =
            serde_json::from_str(r#"{"not-resolved":"https://never.internal:443"}"#).unwrap();

        let resolved: serde_json::Value = crate::Resolver::resolve(deserialized).unwrap();

        let serialized: String = serde_json::to_string(&resolved).unwrap();

        assert_eq!(
            serialized,
            r#"{"not-resolved":"https://never.internal:443"}"#
        );
    }
}

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
