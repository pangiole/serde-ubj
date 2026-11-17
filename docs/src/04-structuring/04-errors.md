# Errors

> [!WARNING]
> This book is **under development**. Some chapters are still incomplete and
> the content may change without notice.

In this section we look at the custom error type that our library crate reports 
back to our users through the API Layer. We assume sufficient familiarity with 
Rust enumerative types, error handling, try operator, core formatters and core 
converters.


## Decoupling Principle

By following common software engineering practices, we should always 
**decouple** our functions' signatures from 3rd party dependencies. 

Our _fallible_ functions should be defined as returning the 
`core::result::Result` type, but with its recoverable `Err` variant conveying 
our own custom error type, instead of any type defined by other libraries. An 
example of usage will help to better understand this decoupling principle.  

Our internal Serde layer's serializer module (`ser`) defines the following 
`to_vec` fallible function:

```rust
// file: src/internal/serde/ser.rs
use crate::internal::result::UbjResult;

// A public fallible function
pub fn to_vec<T>(value: &T) -> UbjResult<Vec<u8>> {
  // Serialize the given value to a heap-allocated vector
  todo!()
}
```

Ignore the body function for now (as it will be implemented later on), and 
focus on its result type. The `UbjResult` is our **synonym** for the 
`core::result::Result` type, and it is defined by our internal `result.rs` 
module:

```rust
// file: src/internal/result.rs
use crate::internal::error::UbjError;

/// A convenient type alias for a result able to wrap a custom UBJ error
pub type UbjResult<T> = core::result::Result<T, UbjError>;
```

Our synonym sets our custom `UbjError` **enumerative** type as the one 
implementing the `core::error::Error` trait. Look at how our internal 
`error.rs` module defines it:

```rust
//  file: src/internal/error.rs

/// An error encountered while serializing to (or deserializing from) UBJ
#[non_exhaustive]
pub enum UbjError {

  /// Non ASCII character was encountered
  NotAsciiChar(u32),

  /// Invalid byte marker was encountered
  IllegalMarker(u8),
    
  // more variants here ...
}
```

The `UbjError` enumeration will evolve over the course of this book, with more 
and more variants being added as we understand the many ways a serializer 
(writer) and a deserializer (reader) can possibly fail. 

## Non Exhaustiveness
Our `UbjError` type is annotated with the `#[non_exhaustive]` attribute, which
allows us to add new error variants in the future without causing a breaking
change for callers. In Rust, adding a variant to a standard public enum normally
breaks any caller that matches on all variants exhaustively.

For example, attempting serialization could make the caller pattern match 
over all variants of our `UbjError` enumerative type, as follows:

```rust
fn attempt_serialization() 
{
  use UbjError::*;
  
  // the caller who pattern matches over our custom error variants
  match to_vec(&employee1) {
    Ok(_)                 => todo!(),
    Err(NotAsciiChar(c))  => todo!(),
    Err(IllegalMarker(m)) => todo!(),
    
    // Wildcard required by #[non_exhaustive]
    Err(_)                => todo!()
  }
}
```

Focus on the list of arms: forcing callers to include a **wildcard arm** (the 
last at the bottom) ensures their code continues to compile even when our 
library adds new error variants later on.


## Core Traits
Our `UbjError` type should implement the `core::error::Error` trait, which in
turn requires both the `core::fmt::Debug` and `core::fmt::Display` traits to 
be implemented as well.


### `Debug`
Let's implement the `core::fmt::Debug` trait first:

```rust
// file: src/internal/error.rs
use core::fmt;

impl fmt::Debug for UbjError 
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
  {
    match self {
        
      Self::NotAsciiChar(c) => {
          f.debug_tuple("NotAsciiChar")
              .field(c)
              .finish()
      }
      // more variants here ...
      _ => todo!()  
    }
  }
}
```

It's just a matter of pattern matching over all possible variants and then
invoking what the formatter has to offer, to produce a nicely formatted
debug message.


### `Display`
Now time to implement the `core::fmt::Display` trait too. 

```rust
// file: src/internal/error.rs
use core::fmt;

impl fmt::Display for UbjError 
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result 
  {
    match self {
        Self::NotAsciiChar(c) => {
            write!(f, "Char not within ASCII range: {c:#X}")
        }
        // more variants here ...
        _ => todo!()
    }
  }
}
```

### `Error`
Finally, let's implement the `core::error::Error` trait, which may turn into a 
pretty trivial job!

```rust
// file: src/internal/error.rs
use core::error;

impl error::Error for UbjError {
    // nothing to define
}
```

The core `Error` trait defines methods like `source()`, but none of them are 
mandatory. It is essentially a **marker trait** with a couple of optional 
inspection hooks. Its primary purpose in the Rust type system is to say: 
_"This type is an official error and can even be treated as dynamic  
(`dyn Error`)."_

The actual hard work of being an error is delegated to two prerequisite traits 
that `Error` requires we had to implement first:

* `core::fmt::Debug` (how it looks when printed for developers)   
* `core::fmt::Display` (how its user-facing error message is rendered)

Since our custom `UbjError` type already implements `Debug` and `Display`, 
implementing the core `Error` trait is just a final stamp of approval, which 
is why an empty implementation may be completely sufficient.

Nonetheless, a common practice for custom errors, such as our `UbjError` type, 
is to define newtype variants wrapping around other types also implementing 
the `core:error:Error` trait. For example: 

```rust
// file: src/internal/error.rs
use core::str;

pub enum UbjError {

  /// A bad string error caused by a UTF-8 encoding error
  BadString(str::Utf8Error),

  /// IO error caused by embedded read/write operations
  IO(embedded_io::ErrorKind),

  // more variants here ...
}
```

Since the additional `BadString` and `IO` newtype variant, we're now in need 
to  implement the `core:error::Error::source` function, as we want the Rust's 
error-chaining system to recognize the inner source error wrapped within them:

```rust
// file: src/internal/error.rs
use core::error;

impl error::Error for UbjError 
{
  fn source(&self) -> Option<&(dyn error::Error + 'static)> 
  {
    match self {
        
        // Expose the inner error
        UbjError::BadString(e) => Some(e),
        UbjError::IO(e) => Some(e),

        // No inner error to expose
        _ => None
    } 
  }
}
```


### `From<T>`

While implementing the core `Debug`, `Display` and `Error` traits was 
necessary, implementing the `core::convert::From<T>` trait is purely optional, 
yet preferable.

Implementing it unlocks ergonomic error propagation across our Rust code using 
the `?` (try) operator. In Rust, the `?` operator doesn't just return early on 
an `Err`, it also automatically calls `From::from()` on the error to convert it 
into the return type expected by the enclosing function, which in our case is 
the `UbjResult` type. Look at the following example, with and without an 
implementation of the `From<T>` trait:

```rust
// file: src/internal/io/reader.rs
use alloc::string::String;
use core::str::from_utf8;

fn read_utf8_bytes(&mut self, len: usize) -> UbjResult<&str> 
{
    // Read bytes supposed to represent UTF-8 characters
    let bytes = ...;

    // Without From<T>, we have to manually map (convert) the error type
    let chars = from_utf8(&bytes[..])
        /*<highlight>*/.map_err(|e| UbjError::BadString(e))/*</highlight>*/?;
    
    // With From<T>, the `?` operator does the conversion implicitly for us!
    let chars = from_utf8(&bytes[..])?;

    Ok(chars)
}
```

Therefore, although fully optional, we'd be better off at implementing it. This 
is how we provide an implicit converter from the Rust `core::str::Utf8Error` 
type to our custom `UbjError` enum type:

```rust
// file: src/internal/error.rs
use core::convert;
use core::str;

impl convert::From<str::Utf8Error> for UbjError 
{
  fn from(e: str::Utf8Error) -> Self 
  {
      UbjError::BadString(e)
  }
}
```

A second conversion is what lets the writer collapse any sink error into our
own type. The sink reports a fixed `embedded_io::ErrorKind`, which we store in
the `IO` variant:

```rust
// file: src/internal/error.rs
use core::convert;

impl convert::From<embedded_io::ErrorKind> for UbjError
{
  fn from(e: embedded_io::ErrorKind) -> Self 
  {
      UbjError::IO(e)
  }
}
```

In similar ways, we could code more and more `From<T>` implementations, where
`T` is any type we can possibly convert our `UbjError` from.


## Macros
For this particular implementation we adopt a 3rd party library to
nicely reduce the amount of boilerplate (repetitive) code. It's the popular
<a href="https://crates.io/crates/thiserror" target="_blank"
rel="noopener noreferrer"><code>thiserror</code></a> crate, which provides 
useful code generator macros, and which we add to our Cargo build as follows:

```toml
# file: Cargo.toml

[dependencies]
  # other dependencies here ...
  thiserror = {
    version = "2.0.21",
    default-features = false,
    features = []
  }

[features]
  # other features here ...
  std = [
    "alloc",
    "serde/std",
    "thiserror/std",
    "embedded-io-adapters/std"
  ]
```

Now, instead of manually implementing the core `Display`, `Error` and `From` 
traits (as verbosely shown above), we succinctly annotate our custom 
error variants with a few macro attributes. These useful macros
automatically generate the most of the Rust code able to implement the above 
core traits.

```rust
//  file: src/internal/error.rs
use thiserror::Error;

/// An error encountered while serializing or deserializing
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum UbjError {

  /// Invalid byte marker was encountered
  #[error("Illegal marker: {0:#X}")]
  IllegalMarker(u8),
    
  /// Non ASCII character was encountered
  #[error("Char not within ASCII range: {0:#X}")]
  NotAsciiChar(u32),

  /// Bad String caused by UTF-8 encoding error
  #[error("Bad String: {0}")]
  BadString(#[from] core::str::Utf8Error),

  /// IO error caused by embedded read/write operations
  #[error("I/O error occurred: {0}")]
  IO(#[from] embedded_io::ErrorKind),

  // more variants here ...
}
```

Notice the `#[derive(Debug, Error)]` line: it replaces *both* implementations
we hand-wrote earlier in this chapter.

* `Debug` *does* have a derive in the standard library. `#[derive(Debug)]`
  generates exactly the `f.debug_tuple()` / `f.debug_struct()` match arms we
  wrote by hand, which format directly into the formatter's byte buffer without
  ever allocating a temporary `String` on the heap.

* `Display` has **no** derive in the standard library: a user-facing message has
  no single mechanically-correct default to derive. So we delegate it to
  `thiserror`, whose `#[derive(Error)]` expands to the very same
  `write!` / `format_args!` calls we wrote by hand — again allocation-free and
  living entirely in `core::fmt`.

In both cases the generated code is pure `core::fmt`, so it is equally efficient
and equally suitable for **no_std** environments, while sparing us from writing
and maintaining one arm per variant. Our hand-rolled `Debug` and `Display`
implementations were therefore purely pedagogical: they showed *what* these
derives expand to, not what we'd maintain in production.

After Rust 1.81+, thanks to the stabilization of the `core::error::Error` trait
into the `core` module, the `thiserror` crate version 2.0+, officially supports
native **no_std** environments out of the box when you disable its default
features. It eliminates massive amounts of boilerplate code for core `Display`
(and `From` conversions), and error wrapping without forcing standard library
dependencies onto your users.

Another point worth noticing is that, while it is true that `thiserror`
transitively pulls in heavy metaprogramming crates like `syn` (for parsing Rust
syntax) and `quote` (for generating Rust code), however:

* these additional dependencies only execute on our build machine to parse the
  syntax of our code while compiling,

* they are never compiled for the target machine (microcontroller such as
  ARM, Cortex-M or RISC-V). They add zero instructions, zero data structures,
  and zero bytes to our target binary footprint.


## Serde Traits

So far we have implemented the core traits to describe how our custom `UbjError`
type is formatted and recognized as a valid Rust error. There are a few more 
traits we're due to implement:

* the `serde::ser::Error`,
* and the `serde::de::Error` trait.

Both of the above Serde error traits require the implementation of the 
`custom()` function only:

```rust
fn custom(msg: T) -> UbjError where T: Display;
```

which works as a **bridge** through which all arbitrary error messages 
must pass when Serde doesn't have a more specialized error method to call. Serde 
internally calls the `custom()` function, passing a displayable message, 
whenever deserialization or serialization fails unexpectedly, for reasons 
beyond our data format implementation.

The best approach is to provide our own `UbjError` enum type with an additional 
`Custom` variant, conveying either a heap-allocated `String` when the Cargo 
feature `alloc` is active, or conveying no message otherwise.

```rust
// file: src/internal/error.rs

pub enum UbjError {
  // more variants here ...

  #[cfg(feature = "alloc")]
  #[error("{0}")]
  /// Custom error  as defined by the user of this crate
  Custom(alloc::string::String),

  #[cfg(not(feature = "alloc"))]
  #[error("")]
  /// Custom error  as defined by the user of this crate
  Custom
}
```

[//]: # (TODO Tell something about conditional compilation)

As Serde data format authors, we implement the Serde error `custom()` function 
so that Serde's framework internal code, and its deriving macros, have a way to 
hand error messages back to our `UbjError::Custom` enum variant:

```rust
//  file: src/internal/error.rs
use core::fmt;
use serde::ser;


impl ser::Error for UbjError 
{
  fn custom<T: fmt::Display>(msg: T) -> UbjError 
  {
      #[cfg(feature = "alloc")] {
        UbjError::Custom(alloc::string::ToString::to_string(&msg))
      }
      #[cfg(not(feature = "alloc"))] {
        let _ = msg;
        UbjError::Custom
      }
  }
}
```

and similarly for the `serde::de::Error::custom()` function. 

Finally, note how we're using the `alloc::string::ToString::to_string` as that
is the best function to invoke in **no_std** environments (with heap 
allocation enabled) to convert any displayable type into an owned 
heap-allocated string.
 