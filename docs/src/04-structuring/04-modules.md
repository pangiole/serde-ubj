# Modules

> [!WARNING]
> This book is **under development**. Some chapters are still incomplete and
> the content may change without notice.

Rust modules (`mod`) are the foundational building blocks for organizing code,
managing scope, and controlling visibility within a crate. They allow you to 
group related logic together into a hierarchical tree, mimicking a file and 
folder structure. The benefits of modularization are huge:

* **Namespacing**  
  they prevent naming collisions by scoping items (functions, structs, enums)
  under a module name.

* **Visibility Control**  
  By default, everything in Rust is private to its parent module. We must 
  explicitly use the `pub` keyword to expose items.

* **File-to-Code Mapping**  
  Modern Rust maps modules directly to files and directories (e.g., 
  declaring `mod error`; tells Rust to look for the `error.rs` file).

Library design best practices encourage breaking a large Rust codebase into 
as much modular as possible, while flattening the public-facing interface 
(the API layer's items) at the root of the `lib.rs` crate file.

Since wish to break down our Rust code into separate files, let's begin with 
creating a good directory layout for our Rust modules. Both the Serde layer and
the I/O layer are to be considered internal modules, whose members have to be 
hidden away from public use. The API layer instead, with the public-facing 
functions and types, is going to be exposed by using Rust re-exports of a few 
internal items.

```text
├── Cargo.toml
└── src/
    ├── lib.rs
    └── internal/
        ├── error.rs
        ├── result.rs
        ├── io/
        │   ├── marker.rs
        │   ├── reader.rs
        │   └── writer.rs
        └── serde/
            ├── de.rs
            └── ser.rs
```

For example the `internal` Rust module defines submodules such as:

* `error` and `result` 
  defines items (types and implementations) that deal with error
  handling best practices in Rust language;

* `io`  
  the lowermost I/O layer, fully hidden inside. It is a directory module
  gathering `marker` (the UBJ marker vocabulary), `reader` and `writer`, and
  dealing with the nitty-gritty of binary encoding and decoding of byte
  sequences;

* `serde`  
  the intermediate Serde layer, with partially exposed items. It is a
  directory module gathering `ser` and `de`, and dealing with serializing and
  deserializing of the Serde data model

```rust
// file: src/lib.rs

mod internal {

    // Error handling matters
    pub mod error;
    pub mod result;

    // I/O layer, fully hidden inside
    mod io {
        mod marker;
        pub mod reader;
        pub mod writer;
    }

    // Serde layer, with partially exposed items
    pub mod serde {
        pub mod de;
        pub mod ser;
    }
}

// Re-export some items to the public-facing interface

pub use internal::error::UbjError as Error;
pub use internal::result::UbjResult as Result;

pub use internal::serde::ser::to_slice;
pub use internal::serde::de::from_slice;

#[cfg(feature = "alloc")]
pub use internal::serde::ser::to_vec;
#[cfg(feature = "alloc")]
pub use internal::serde::de::from_vec;

#[cfg(feature = "std")]
pub use internal::serde::ser::to_writer;
#[cfg(feature = "std")]
pub use internal::serde::de::from_buf_reader;
```

Each re-export carries the same `#[cfg]` gate as the item it exposes, so the
public surface of the crate shrinks and grows with the active Cargo features:
`to_slice` is always available, while `to_vec`, `from_vec` and `from_slice` 
need at least the `alloc` feature activated, and `to_writer` and 
`from_buf_reader` need the superior `std` feature.

