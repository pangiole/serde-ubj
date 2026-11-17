# Tests

> [!WARNING]
> This book is **under development**. Some chapters are still incomplete and
> the content may change without notice.

In this section we show how to structure our `serde_ubj` crate for automated 
testing, where to place unit and integration tests, and give some advice on 
conditional feature flags.


### Unit Tests
We put the most of our internal functions under the scrutiny of unit tests, 
which see at our `serde_ubj` crate like a (transparent) white box, and have 
knowledge of the hidden details.

We do it by adding Rust `mod tests` directly inside each one of our internal
modules' source files, usually at the bottom. For example directly inside the 
`src/internal/error.rs` file, and also inside the `src/internal/io/reader.rs` 
file, and so on and so forth.

```txt
serde_ubj
├── Cargo.toml
├── src
│    ├── internal
│    │    ├── error.rs         <-- unit tests here
│    │    ├── ...              <-- unit tests here
│    │    └── io/reader.rs     <-- unit tests here
│    └── lib.rs
└── tests
     ├── from_vec.rs
     ├── ...
     └── to_writer.rs          
```

Each one of these inner `tests` module can be seen as a _"test suite"_, made
of a collection of related test cases. We annotate each of these test suites 
with the `#[cfg(test)]` attribute to make sure they get conditionally compiled 
only upon running `cargo test`.

```rust
// file: src/internal/error.rs

// This is a test suite
#[cfg(test)]
mod tests {
    
  // Add test cases here ...
}
```

Test cases are added the test suite as ordinary functions annotated with the 
`#[test]` attribute, so that they are invoked upon running `cargo test` (the 
test runner).

```rust
// file: src/internal/error.rs

#[cfg(test)]
mod tests {
  use super::UbjError;
  use crate::alloc::string::ToString;
    
  #[test]
  fn should_display_ubj_error_not_ascii_char() 
  {
      let e = UbjError::NotAsciiChar(0xFF);
      assert_eq!(e.to_string(), "Char not within ASCII range: 0xFF");
  } 

  // Add more test cases here ...
}
```

Note how the test suite is allowed to access the `super::UbjError` type of the 
enclosing module, and that would happen even though the type hadn't been defined
as `pub` (public). In fact, in Rust language, the inner modules have full 
visibility of the items defined by their enclosing module (and not the way 
round).

### Integration Tests
In addition to unit tests, we also develop integration tests. They are meant to
test the API layer from the final user point of view, which sees at our 
`serde_ubj` crate more like a black box, without any knowledge of the inside.

Therefore, we create a `tests/` folder at the crate's root, and we place some 
additional Rust files in it, one for each of the public-facing items we wish to 
test (such as `from_vec.rs` and `to_writer.rs`)


```txt
serde_ubj
├── Cargo.toml
├── src
│    ├── internal
│    │    ├── error.rs     
│    │    ├── ...           
│    │    └── io/reader.rs     
│    └── lib.rs
└── tests
     ├── from_vec.rs     <-- integration tests here
     ├── ...                <-- integration tests here
     └── to_writer.rs       <-- integration tests here
```

Cargo treats those additional Rust files as completely separate, external 
crates consuming our library from the outside world. It compiles each file as a 
separate integration test binary, running them all automatically when we 
execute `cargo test`.

In order to write good integration tests, we should refrain from poking the 
internals of our implementation, especially for those internal items not 
publicly accessible. Therefore, we should adopt our users point of view, and 
code the integration test cases by looking at public API layer only. 

As better explained in the [API Layer](../08-api-layer/08-api-layer.md) chapter 
of this book, our crate root file (which is the `src/lib.rs` file) re-exports 
the items supposed to be seen from the outside world:

```rust
// file: src/lib.rs

// Re-export some items to the public-facing interface
pub use internal::error::UbjError as Error;
pub use internal::result::UbjResult as Result;
```

The above re-exports allow us writing the following integration test:

```rust
// file: test/error.rs
use core::{fmt, error};

// A helper function that forces the compiler to check trait bounds
fn assert_trait_bounds<T: fmt::Debug + fmt::Display + error::Error>() {}

// An integration test case
#[test]
fn should_core_traits_been_implemented_for_serde_ubj_error() 
{
    assert_trait_bounds::<serde_ubj::Error>();
}
```

Note how the integration test sees the `serde_ubj::Error` (re-exported) item 
the same way our final users would see from the outside world once the 
`serde_ubj` crate is delivered. The same item would have been referred as 
`crate::internal::error::UbjError` from our unit tests, instead, as they see
items from the inside of the crate.