# The API Layer

> [!WARNING]
> This book is **under development**. Some chapters are still incomplete and
> the content may change without notice.

In this chapter we do ...

## Modules
So far we have structured our `serde_ubj` crate into separate modules, some of 
which dealing with error handling, some forming the IO Layer, and others forming
the Serde Layer. 

It's now time to implement the public-facing API Layer (Application Programming 
interface) as the set of items we wish to export to our final users. 
Implementing this layer is as simple as re-exporting some items from the internal 
modules to the external world, and documenting them with example of usages.

```rust
// file: src/lib.rs

mod internal {

    // Error handling matters
    pub mod error;
    pub mod result;

    // I/O layer, items fully hidden inside
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

#[cfg(feature = "std")]
pub use internal::serde::ser::to_writer;

#[cfg(feature = "alloc")]
pub use internal::serde::ser::to_vec;

pub use internal::serde::ser::to_slice;

#[cfg(feature = "std")]
pub use internal::serde::de::from_buf_reader;

#[cfg(feature = "alloc")]
pub use internal::serde::de::from_vec;

#[cfg(feature = "alloc")]
pub use internal::serde::de::from_slice;
```


## Re-exports

A **re-export** in Rust is the practice of taking an item, such as a struct,
enum, or function defined deep within a private internal module and making it 
publicly available at a cleaner, shallower path — usually directly at the crate 
root — using the `pub use` keyword. We also use the `as` keyword during 
re-exporting to rename some types (for example `UbjResult` and `UbjError`) as 
we wish to provide a more ergonomic public name.

Seen from the outside (which is the client point of view), our `serde_ubj` 
crate is going to be used as follows:

```rust
use app::{Employee, Group};

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let employee1 = Employee {
        name: String::from("John Brown"), age: 12, groups: vec![Group::Leader]
    };
    
    // Serialize an employee
    
    let mut file = std::fs::File::create("emp1.ubj")?;
    let result_1: /*<highlight>*/serde_ubj::Result<()>/*</highlight>*/ = 
        /*<highlight>*/serde_ubj::to_writer(&mut file, &employee1)/*</highlight>*/;
    
    match result_1 {
        Err(/*<highlight>*/serde_ubj::Error::NotAsciiChar(c)/*</highlight>*/) => println!("Aaargh! {c:#X}"),
        _ =>  ()
    }
    
    // Deserialize an employee
    
    let bytes: Vec<u8> = std::fs::read("emp2.ubj")?;
    let employee_2 = 
        /*<highlight>*/serde_ubj::from_vec::<Employee>(bytes)/*</highlight>*/?;
    
    // do something with employee2 ...
}
```

Note how the users can refer to our result type by simply typing 
`serde_ubj::Result`, or to our error type as `serde_ubj::Error`. They can follow 
a short path starting from the root of our crate, and ending at a much 
shallower location than where the type actually is. That's the magic of Rust 
re-exports. 

Similarly, users can refer to our public-facing interface by simply typing 
`serde_ubj::to_writer` to invoke a serialization function, or typing 
`serde_ubj::from_vec` to invoke a deserialization function. Still Rust 
re-export magic here!

## Documentation
...