//! Resolve string values with specific prefixes in a JSON structure from
//! environment variables and file system.
//!
//! | head            | tail               | pipeline                              |
//! | --------------- | ------------------ | ------------------------------------- |
//! | `env-plain://`  | name of an env var | as UTF-8 string                       |
//! | `env-hex://`    | name of an env var | as UTF-8 string → as hex → `Vec<u8>`  |
//! | `file-plain://` | file system path   | as UTF-8 string                       |
//! | `file-hex://`   | file system path   | as UTF-8 string → as hex → `Vec<u8>`  |
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
//!   "database_connection_string": "env-plain://FOO_BAR",
//!   "procedural_gen_seed": "file-hex:///opt/seed.hex",
//!   "player_to_privileges_mapping": "file-json:///opt/player-privileges.json",
//!   "global_greeting": "file-plain:///opt/greeting.txt",
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

pub trait Resolver {
    fn resolve(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>>;
}

impl Resolver for serde_json::Value {
    fn resolve(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        Ok(self.clone())
    }
}

#[cfg(test)]
mod test_keep_intact {
    #[test]
    fn record_simple() {
        let deserialized: serde_json::Value = serde_json::from_str(r#"{"foo":"bar"}"#).unwrap();

        let resolved: serde_json::Value = crate::Resolver::resolve(&deserialized).unwrap();

        let serialized: String = serde_json::to_string(&resolved).unwrap();

        assert_eq!(serialized, r#"{"foo":"bar"}"#);
    }

    #[test]
    fn nested_complex() {
        let deserialized: serde_json::Value = serde_json::from_str(
            r#"[{"foo":{"bar":"baz","array":[{"aaa":{"num":-1}},1,"asd"]}},{"foo":{"bar":"baz"}}]"#,
        )
        .unwrap();

        let resolved: serde_json::Value = crate::Resolver::resolve(&deserialized).unwrap();

        let serialized: String = serde_json::to_string(&resolved).unwrap();

        assert_eq!(
            serialized,
            r#"[{"foo":{"bar":"baz","array":[{"aaa":{"num":-1}},1,"asd"]}},{"foo":{"bar":"baz"}}]"#
        );
    }
}
