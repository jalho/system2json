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
