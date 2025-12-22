# serde-ubj
[Universal Binary JSON](https://github.com/ubjson/universal-binary-json) format for Rust with [Serde](https://github.com/serde-rs/serde) for both `std` and `no_std` environments.

<div class="warning">
<p>This project is in an early stage of development, and <strong>not</strong> production ready yet.<br>Use with caution!</p>
</div>

This implementation does:

* provide serialization for most of the [Serde data model](https://serde.rs/data-model.html) types,
* support both `std` (standard) and `no_std` environments (with `alloc`)
* pass enough unit tests covering at least 85% of its code.


### exceptions
This implementation does **not** support the following Serde types:

* serialization
  * Serde byte array `u8`
  * Serde numeric `u64` values greater than Rust `i64::MAX`
  * Serde numeric `i128` and `u128`
  * Serde `map`
  * Serde `string` having length greater than Rust `i64::MAX`,

* deserialization
  * any Serde types


## tl;dr
For the impatient.

### serialization
With any Rust `std` writer of your choice (i.e., output console, memory buffer, file on disk, network socket, etc.), serialize your user-defined model to Universal Binary JSON in a few instructions: 

```rust,ignore
use sdt::{error, io};

fn main() -> Result<(), Box<dyn error::Error>> {

  // Create a "serializable" value for which you have either derived 
  // or provided a serde::Serialize implementation   
  let value = 123_i16; 
  
  // Create an IO writer and then serialize the value to it
  let mut w: io::Write = io::stdout();
  serde_ubj::to_writer(&w, &value)?;
}
```

### deserialization
Coming soon.

## book
Coming soon.


## no_std
This implementation can be adopted in `no_std` environments, as long as you disable the default features and, at the same time, enable the `alloc` feature, as per following `Cargo.toml` example:

```toml
[package]
name = "my_no_std_project"
version = "0.1.0"
edition = "2024"

[dependencies]
serde_ubj = { version = "0.2.0", default-features = false, features = ["alloc"] }
# ... add more Serde dependencies as needed ...
```
