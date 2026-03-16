# Plan for Rewrite

- Expose deserialization and system access functionalities separately. Make
  use of builder pattern to allow library users to optionally provide their own
  implementation of system access. Ideally, for example, library users could
  make use of only the deserialization part in e.g. a WASM target build, and
  provide their own system access implementation if necessary.

- Support asynchronous programs optionally via a `::crate::async::tokio::`
  module behind a feature gate. For symmetry, provide synchronous API by default
  via `::crate::sync::` module.

- Pick a more sensible default format than JSON: Consider JSON5. Why not!
  JSON sucks for the main imaginable use case of this library, i.e. resolving
  configuration files! Keep other formats feature gated.

- Write tests, examples and docs only after the main scaffolding of the library
  is stable.

- Rename the library from `system2json` to `desys`, meaning "deserialize"
  and "system". Define an internal intermediate type that encapsulates
  `serde_json::Value`. The library is really about dealing with any structures
  that are mappable to JSON, and not specifically only JSON, hence the renaming.
