fn main() {
    let json: serde_json::Value = serde_json::from_str(SERIALIZED_JSON).unwrap();

    use system2json::Resolver;
    let resolved = json.resolve().unwrap();

    let deserialized: MyComplicatedStructure = serde_json::from_value(resolved).unwrap();

    dbg!(deserialized);
}

const SERIALIZED_JSON: &str = r#"{
  "aaa": "file://test-files/greeting.txt",
  "bbb": "file-json://test-files/sketchy.json",
  "nested": {
    "many": [
      "file-json://./test-files/sketchy.json",
      "file-json://test-files/sketchy.json"
    ]
  },
  "ccc": 1,
  "ddd": "file://test-files/sketchy.json"
}"#;

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct MyComplicatedStructure {
    aaa: String,
    bbb: MyNestedThingB,
    ccc: u32,
    ddd: String,
    nested: MyNestedThingA,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct MyNestedThingA {
    many: Vec<MyNestedThingB>,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct MyNestedThingB {
    should_not_be_resolved: String,
}
