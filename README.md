# `system2json`

From some given (JSON mappable) structure, resolve string values with specific
prefixes from environment variables and file system.

## Example

```rust
fn main() {
  let serialized: String =
    std::fs::read_to_string("test-files/has-resolvable-values.toml").unwrap();

  let intermediate: serde_json::Value = toml::from_str(&serialized).unwrap();

  let resolved: serde_json::Value = system2json::Resolver::resolve(intermediate).unwrap();

  let deserialized: MyComplicatedStructure = serde_json::from_value(resolved).unwrap();

  assert_eq!(deserialized.foo.spam, "Hello world!\n");
  assert_eq!(deserialized.asd.trailing_comma, 1);
}

#[derive(serde::Deserialize)]
struct MyComplicatedStructure {
  foo: MyNestedThing,
  baz: MyNestedThing,
  asd: AnotherNestedThing,
}

#[derive(serde::Deserialize)]
struct MyNestedThing {
  spam: String,
}

#[derive(serde::Deserialize)]
struct AnotherNestedThing {
  foo: String,
  baz: String,
  trailing_comma: u16,
}
```

`has-resolvable-values.toml`:

```toml
asd = "file-json5://test-files/has-comments.jsonc"

[foo]
spam = "file://test-files/greeting.txt"

[baz]
spam = "file://test-files/greeting.txt"
```

`greeting.txt`:

```txt
Hello world!
```

`has-comments.jsonc`:

```json5
{
  /*
   * Multi-line
   * comment.
   */
   "foo": "bar",

   "baz": "spam", // single-line comment

   "trailing_comma": 1,
}
```
