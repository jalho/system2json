fn main() {
    let toml_serialized: String =
        std::fs::read_to_string("test-files/has-resolvable-values.toml").unwrap();

    let json: serde_json::Value = toml::from_str(&toml_serialized).unwrap();

    use system2json::Resolver;
    let resolved = json.resolve().unwrap();

    let deserialized: MyComplicatedStructure = serde_json::from_value(resolved).unwrap();

    dbg!(deserialized);
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct MyComplicatedStructure {
    foo: MyNestedThing,
    baz: MyNestedThing,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct MyNestedThing {
    spam: String,
}
