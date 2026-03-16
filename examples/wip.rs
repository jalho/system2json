const SERIALIZED_JSON5: &str = r#"
{
  /*
   * Multi-line comment.
   */
  "foo": "/etc/hostname", // single-line comment
  "bar": "/etc/hostname", // Note the trailing comma: Not normal JSON!
}
"#;

const SERIALIZED_TOML: &str = r#"
bar = "/etc/hostname" # comment
foo = "/etc/hostname"
"#;

fn main() {
    let example_file_json5: &std::path::Path = std::path::Path::new("examples/flat.json5");
    std::fs::write(example_file_json5, SERIALIZED_JSON5.trim()).unwrap();
    let buffer_json5: String = std::fs::read_to_string(example_file_json5).unwrap();
    /*
     * API for deserializing from an in-mem buffer without file system access: JSON5.
     */
    {
        let resolvable: desys::Resolvable = desys::Resolvable::parse_json5(&buffer_json5).unwrap();
        let deserialized: MyStruct = resolvable.deserialize().unwrap();
        dbg!(deserialized.foo, deserialized.bar);
    }

    let example_file_toml: &std::path::Path = std::path::Path::new("examples/flat.toml");
    std::fs::write(example_file_toml, SERIALIZED_TOML.trim()).unwrap();
    let buffer_toml: String = std::fs::read_to_string(example_file_toml).unwrap();
    /*
     * API for deserializing from an in-mem buffer without file system access: TOML.
     */
    {
        let resolvable: desys::Resolvable = desys::Resolvable::parse_toml(&buffer_toml).unwrap();
        let deserialized: MyStruct = resolvable.deserialize().unwrap();
        dbg!(deserialized.foo, deserialized.bar);
    }

    /*
     * API for resolving from system, blocking.
     */
    {
        let resolvable: desys::Resolvable = desys::Resolvable::parse_json5(&buffer_json5).unwrap();

        use desys::blocking::Resolve;
        let resolved: desys::Resolved = resolvable.resolve().unwrap();

        let deserialized: MyStruct = resolved.deserialize().unwrap();
        dbg!(deserialized.foo, deserialized.bar);
    }

    /*
     * API for resolving from system, in tokio async runtime.
     */
    {
        let resolvable: desys::Resolvable = desys::Resolvable::parse_json5(&buffer_json5).unwrap();

        let runtime: tokio::runtime::Runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();

        let resolved: desys::Resolved = runtime
            .block_on(async {
                use desys::tokio::Resolve;
                resolvable.resolve().await
            })
            .unwrap();

        let deserialized: MyStruct = resolved.deserialize().unwrap();
        dbg!(deserialized.foo, deserialized.bar);
    }
}

#[derive(Debug, serde::Deserialize)]
struct MyStruct {
    foo: String,
    bar: String,
}
