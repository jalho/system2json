# `system2json`

From some given (JSON mappable) structure, resolve string values with specific
prefixes from environment variables and file system.

## Example

```rust
fn main() {
  let serialized: String =
    std::fs::read_to_string("has-resolvable-values.toml").unwrap();

  let intermediate: serde_json::Value = toml::from_str(&serialized).unwrap();

  let resolved: serde_json::Value = system2json::Resolver.resolve(intermediate).unwrap();

  let deserialized: MyComplicatedStructure = serde_json::from_value(resolved).unwrap();
}

#[derive(serde::Deserialize)]
struct MyComplicatedStructure {
  foo: MyNestedThing,
  baz: MyNestedThing,
}

#[derive(serde::Deserialize)]
struct MyNestedThing {
  spam: String,
}
```

`has-resolvable-values.toml`:

```toml
[foo]
spam = "file://./greeting.txt"

[baz]
spam = "file://./greeting.txt"
```

`greeting.txt`:

```txt
Hello world!
```
