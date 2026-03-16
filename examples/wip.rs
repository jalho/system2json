const SERIALIZED_JSON5: &str = r#"
{
  /*
   * Multi-line comment.
   */
  "foo": "file:///etc/hostname", // single-line comment
  "bar": "trailing comma", // Note the trailing comma: Not normal JSON!
}
"#;

fn main() {
    let example_file: &std::path::Path = std::path::Path::new("examples/flat.json5");
    std::fs::write(example_file, SERIALIZED_JSON5.trim()).unwrap();
    let buffer: String = std::fs::read_to_string(example_file).unwrap();

    /*
     * API for deserializing from an in-mem buffer without file system access.
     */
    {
        let resolvable: desys::Resolvable = desys::Resolvable::parse_json5(&buffer).unwrap();
        let deserialized: MyStruct = resolvable.deserialize().unwrap();
        dbg!(deserialized.foo, deserialized.bar);
    }

    /*
     * API for resolving from system, blocking.
     */
    {
        let resolvable: desys::Resolvable = desys::Resolvable::parse_json5(&buffer).unwrap();

        use desys::blocking::Resolve;
        let resolved: desys::Resolved = resolvable.resolve().unwrap();

        let deserialized: MyStruct = resolved.deserialize().unwrap();
        dbg!(deserialized.foo, deserialized.bar);
    }

    /*
     * API for resolving from system, in tokio async runtime.
     */
    {
        let resolvable: desys::Resolvable = desys::Resolvable::parse_json5(&buffer).unwrap();

        let runtime: tokio::runtime::Runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();

        let resolved: desys::Resolved = runtime.block_on(async {
            use desys::tokio::Resolve;
            resolvable.resolve().await
        }).unwrap();

        let deserialized: MyStruct = resolved.deserialize().unwrap();
        dbg!(deserialized.foo, deserialized.bar);
    }
}

#[derive(Debug, serde::Deserialize)]
struct MyStruct {
    foo: String,
    bar: String,
}
