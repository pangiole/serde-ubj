# serde-ubj
[Universal Binary JSON](https://github.com/ubjson/universal-binary-json) format for Rust with [Serde](https://github.com/serde-rs/serde), fully compliant and fully tested. No bad surprises.

> [!WARNING]  
> This project is in an early stage of development, and <strong>not</strong> production ready yet.
> Use with caution!
> 

## tl;dr
For the impatient.

### serialization
With any Rust standard writer you like, serialize your user-defined model to Universal Binary JSON in a few instructions: 

```rust
use serde_ubj;
use sdt::{error, io};

fn main() -> Result<(), Box<dyn error::Error>> {

  // With any writer you like  
  let mut w: io::Write = ...;
  // for example stdout, buffer, file, socket, etc.

  // Create your user-defined data model
  let model: MyModel = ...; 
  // for which you have derived the serde::Serialize implementation   
    
  // And then write it to Universal Binary JSON
  serde_ubj::to_writer(&w, &model)?;
}
```

### deserialization
Coming soon.


## exceptions
This implementation provides all the features of the Universal Binary JSON specification that make sense for the Rust type system, with the following few exceptions:

* Upon serialization

  * Rust `&[u8]` (byte slices)
  * Rust `u64` values greater than `i64::MAX`
  * Rust `alloc::string::String` (or `&str` slices) with len greater `i64::MAX`,
  * Rust `std::collections::HashMap`

* Upon deserialization
  * ???

## book
Coming soon.

## crate
Coming soon.

## build
To build this project, you must have [rustup](https://rustup.rs/) preinstalled, and run our preliminary setup script (only once):

```
./setup.sh
```

Then run our build script (which goes through all the steps to build the project), or the usual cargo commands:

```sh
./build.sh
```