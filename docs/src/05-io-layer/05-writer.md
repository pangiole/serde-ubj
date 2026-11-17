# Writing

In this section we build the writing side of the I/O Layer: the `UbjWriter` type.
We define the struct and its two `new` factories — one for `no_std` sinks and one
that adapts a `std::io::Write` through `FromStd` — then implement its methods,
from the building blocks (`write_all`, `write_marker`, `write_payload`, plus the
`write_sized_packet` helper) up through the scalar writers that narrow integers
to the smallest width that fits and the four container markers.

## Struct
As already noted earlier in the 
[Dependencies](../04-structuring/04-crate.html#dependencies) section of this 
book, we code our solution against the `embedded-io` crate, and we use the 
embedded I/O replacement traits instead of the standard I/O traits.

Just to ease our understanding, let's introduce the `Sink` synonym for the
`embedded_io::Write` trait, and let's define our public `UbjWriter` struct type
as something wrapping an underlying sink (instead of wrapping an 
underlying standard I/O writer).

```rust
// file: src/internal/io/writer.rs
use embedded_io::{Write as Sink};

/// UBJ-aware writer wrapping any [`embedded_io::Write`] sink.
pub struct UbjWriter<S> where S: Sink 
{
    sink: S
}
```

## Factories
And now, as we usually do in Rust idiomatic code, we implement one or more 
publicly associated `new` factory functions, each one able to construct such a 
`UbjWriter` thing, and each one taking an embedded `sink` as input argument. 
Let's begin with targeting `no_std` environments first (either with 
heap-allocators or more strictly without):

```rust
// file: src/internal/io/writer.rs
use embedded_io::{Write as Sink};

#[cfg(not(feature = "std"))]
impl<S> /*<highlight>*/UbjWriter<S>/*</highlight>*/
where S: Sink 
{
    /// Construct a new UBJ writer over the given [`embedded_io::Write`] sink.
    pub fn new(sink: S) -> Self {
        Self { /*<highlight>*/sink/*</highlight>*/ }
    }
}
```

Note how we _"gated"_ the `new` factory function behind the `#[cfg]` attribute 
because we want it to be conditionally compiled only when targeting `no_std` 
environments.

#### IO Traits
The keystone of this design is the decision to express all input/output through 
the `embedded-io` crate rather than through the `std::io` built-in module (as 
someone would instinctively adopt at first).

The `embedded-io` create defines a `no_std`-compatible set of I/O traits — most 
importantly the `embedded_io::Write` — that are intentionally dependency-free 
and heap-allocation-free. A type only has to promise _"I can accept bytes and I 
can report a failure kind"_ to act as a sink; it does not have to know anything 
about the standard library. This single trait is what lets the same writer 
target a fixed `&mut [u8]` on a microcontroller, a growable `Vec<u8>` under 
`no_std + alloc`, or a `std::io::Write` on a desktop machine.


#### Adapting
So far so good! Let's move on and implement another `new` function, this time 
targeting standard environments for desktop machines. 

For that we need a slightly different factory function, able to _"adapt"_ a 
standard I/O writer so it can be used as if it were an embedded I/O sink. The 
solution is using the `FromStd` wrapper of the`embedded-io-adapters` crate, 
of which we reveal a few inner details:

```rust
// file: embedded-io-adapters-0.7.0/src/std.rs

pub struct FromStd<W> 
{
    inner: W,
}


impl<W> FromStd<W> 
{
    pub fn new(inner: W) -> Self { Self { inner } }
}


impl<W> embedded_io::Write for FromStd<W> 
where W: std::io::Write
{
    // writes by delegating to self.inner.write(...)
}
```

The important fact is that the `FromStd<W>` adapter _implements the embedded 
I/O trait_ whenever `W` implements the standard I/O trait. Therefore, 
`FromStd<W>` is special kind of embedded sink whose methods simply delegate 
any write operation to the inner standard I/O writer. Therefore, an 
instance of `FromStd<W>` can be passed upon construction of our `UbjWriter` 
struct.

Let's finally implement the `UbjWriter::new` factory function able to target 
the standard environment, as follows:

```rust
// file: src/internal/io/writer.rs
use embedded_io::{Write as Sink};
use embedded_io_adapters::std::FromStd;

#[cfg(feature = "std")]
impl<W> /*<highlight>*/UbjWriter<FromStd<W>>/*</highlight>*/
where W: std::io::Write 
{
    /// Construct a new UBJ writer over the given `std::io::Write` writer
    pub fn new(writer: W) -> Self 
    {
        Self { 
            // Adapt the standard writer to make it usable as an embedded sink
            sink: /*<highlight>*/FromStd::new(writer)/*</highlight>*/
        }
    }
}
```

This time we have not implemented it for the `UbjWriter<S>` (which is the most 
_"generic"_ shape for any possible embedded sinks). Instead, have implemented 
it for the `UbjWriter` **at a specific instantiation**: the UBJ writer whose 
embedded sink is of type `FromStd<W>`.

We chose `W` and `S` as type parameters because they help to visualize the 
difference. Our mental model is: `W` is the standard I/O type as seen from the 
outside world, while `S` is the embedded I/O type as seen from the inside of 
our `serde_ubj` crate. The `FromStd` adapter is the bridge that turns a `W` 
into an `S`.


## Methods

The `UbjWriter`'s public surface is a thin layer over a handful of private
helpers. Three of them — `write_all`, `write_marker` and `write_payload` — are
the **basic building blocks**: every byte that reaches the sink passes through
`write_all`, and every public `write_*` method ultimately calls `write_marker` or
`write_payload`. A fourth helper, `write_sized_packet`, is a convenience form
built on top of the other two and used by the fixed-width scalars.

### Basics

A **UBJ packet** — the unit this writer emits — is exactly the structure defined
in the [Format - Structure](./05-format.md#structure) section: a one-byte **marker**, optionally 
followed by a **payload**, and, for containers, optionally followed by a 
**closing marker**. The three methods below are the basic building blocks from 
which every packet is assembled; everything else in this chapter is built 
on them.

#### `write_all`
This is the most foundational method of our `UbjWriter`, that every other 
method ends up calling either directly or indirectly.

```rust
// file: src/internal/io/writer.rs

impl<S> UbjWriter<S> where S: Sink
{
    /// Writes the whole buffer, collapsing the sink's associated `S::Error` through
    /// [`to_ubj_error`].
    fn write_all(&mut self, buf: &[u8]) -> UbjResult<()> {
        use embedded_io::Error;
        self.sink
            .write_all(buf)
            .map_err(/*<highlight>*/e.kind().into()/*</highlight>*/)
    }
    
    // ... more methods here 
}
```

`write_all` is the only method that talks to the sink, so it is also the only 
one that has to deal with the sink's error type. `Sink::write_all` behaves like 
the familiar method of the same name: it 
keeps  writing until the whole buffer has been consumed, looping over partial 
writes internally. Our callers therefore never observe a short write — either 
the full `buf` is written, or the call fails once.

Another interesting fragment is the error mapping. The sink's error is an 
*associated* type (`S::Error`): the writer cannot name it, so it cannot store 
it in `UbjError`. It can, however, ask for its `ErrorKind` through the 
`embedded_io::Error` bound, and then convert that fixed enum into `UbjError::IO`
through the `From` implementation (which is what makes the `into()` method 
available). Because this collapse lives in exactly one place, every other 
method can simply use `?` and never mention the associated `S::Error` again: 
the concrete sink error never leaks into the rest of the crate.


Finally, another important matter deserving an explanation is **flushing** the 
sink. Our writer deliberately does **not** expose a `flush` method, and it does
**not** take the initiative to flush on the caller's behalf. Flushing is a 
property of the sink's buffer, and that buffer belongs to whoever created it: 
only the caller knows whether the bytes should be pushed out now or accumulated 
across several writes, and only the caller can decide how to handle a flush 
failure. The `UbjWriter` therefore just writes; a caller that wraps its writer 
in a `std::io::BufWriter` flushes that buffer itself once it is done, typically
after `to_writer` returns.


#### `write_marker` 

```rust
// file: src/internal/io/writer.rs

impl<S> UbjWriter<S> where S: Sink 
{
    fn write_marker(&mut self, marker: UbjMarker) -> UbjResult<()> 
    {
        self.write_all(&[/*<highlight>*/u8::from(marker)/*</highlight>*/])
    }
    
    // ... more methods here 
}
```

`UbjMarker` is the `#[repr(u8)]` enum we met in the
[Format - Markers](./05-format.md#markers) section, so it already knows its own wire byte. Rather
than casting at every call site, the enum implements `From<UbjMarker> for u8`, 
the single place that cast lives:  `u8::from(UbjMarker::Int8)` is `0x69`, 
`u8::from(UbjMarker::StartArray)` is `0x5B`, and so on.

`write_marker` wraps that byte in a one-element slice, `&[u8::from(marker)]`,
and hands it to `write_all`. Notice what a packet is made of here. `write_marker` 
emits the leading marker — and, for containers, the *closing* marker as well, 
since closing is just another marker. The `write_payload` (see below) emits the 
payload. Fixed-width scalars, however, repeat the same two-step
dance — marker, then big-endian payload — often enough that it is worth naming,
so the next section introduces `write_sized_packet`: a small helper built
directly from `write_marker` and `write_payload`, used for packets whose payload
size is known at compile time.

#### `write_payload`

```rust
// file: src/internal/io/writer.rs

impl<S> UbjWriter<S> where S: Sink 
{
    fn write_payload(&mut self, payload: &[u8]) -> UbjResult<()> 
    {
        self.write_all(payload)
    }
}
```

The `write_payload` method is simply a named alias for `write_all`: a payload 
is the *data* that follows a marker, as opposed to the marker itself, and having
a separate method name keeps the call sites self-documenting.

The `write_marker` and `write_payload` methods, together with `write_all`, are 
the only primitives. 


#### `write_sized_packet`
Before explaining what this method does, we need to revise some basic Rust 
types having to do with arrays: `&[u8]` and `[u8; N]`. These two closely related
types carry bytes through our `UbjWriter`, and it is worth telling them apart.

  * The first is `&[u8]`, a **borrowed slice of bytes** — the type `write_all` 
     and `write_payload` accept as input argument. A slice is a *wide pointer*: 
    two  machine words holding the address of the first byte and the number of 
    bytes:
  
    <pre style="width: fit-content; margin-left: auto; margin-right: auto;"><code class="language-text">&amp;[u8]  —  a borrowed, contiguous run of bytes
  
          address length
         ┌───────┬──────┐
         │   ●   │   4  │
         └───────┴──────┘
             │
             ▼
         ┌───────────┬───────────┬───────────┬───────────┐
         │   0xFF    │   0xFF    │   0x02    │   0x18    │      
         └───────────┴───────────┴───────────┴───────────┘
             [0]         [1]         [2]         [3]
    </code></pre>
  
    A `&[u8]` byte slice does not own the bytes, and it does not allocate: it 
    is no more than a temporary view into memory that belongs to someone else. 
    And that's exactly what  `write_payload` needs — read the bytes just long 
    enough to push them to the sink. 

  * The second is `[u8; N]`, a **fixed-size array whose length is part of its 
    type**. The Rust core built-in crate provides the `to_be_bytes()` method 
    (_"big-endian ordering"_), which we can invoke on any Rust primitive value, 
    and which returns one of these — `[u8; 1]`, `[u8; 2]`, `[u8; 4]` or `[u8; 8]` 
    depending on the value — so a fixed-width scalar produces a fixed-size 
    payload by value:
    
    ```rust
    // file: src/internal/io/writer.rs

    impl<S> UbjWriter<S> where S: Sink 
    {
      pub fn write_int8(&mut self, v: i8) -> UbjResult<()> 
      {
          self.write_sized_packet(UbjMarker::Int8, /*<highlight>*/v.to_be_bytes()/*</highlight>*/)
      }
    }  
    ```

Therefore, the `write_sized_packet` method accepts both the marker and the 
payload as fixed-size array, passed in **by value**, so that it forwards them 
through `write_marker` and `write_payload` calls, as follows:

```rust
// file: src/internal/io/writer.rs

impl<S> UbjWriter<S> where S: Sink
{
  fn write_sized_packet<const N: usize>(
      &mut self,
      marker: UbjMarker,
      payload: [u8; N]

  ) -> UbjResult<()> 
  {
      self.write_marker(marker)?;
      self.write_payload(&payload)   // [u8; N] coerces to &[u8] here
  }
```

The `const N: usize` type parameter makes this helper generic over the array  
length, so one method serves every scalar width: no `&` at the call site, no
allocation, and the same `&[u8]` that ultimately reaches `write_all`. Because the
payload size is part of the array's type, `write_sized_packet` covers only the
*sized* case — a marker plus a compile-time-sized payload — and never writes a
closing marker, since only the container methods need one.

> [!NOTE]
> Do not confuse the `&[u8]` (a borrowed byte slice) and the `[u8; N]` (a 
> fixed-size array) types with `Vec<u8>` (a growable, heap-allocated buffer). 
> Nothing in the `UbjWriter` ever allocates to the heap —  every payload is 
> either a borrowed slice of bytes or a fixed-size array on the stack.

### Scalars

Most of the writer's intelligence lives in the scalar methods. A few values are
marker-only, the fixed-width ones are a marker plus a big-endian payload, and the
integer methods try to pick the cheapest representation that still holds the
value.

#### `null` and `bool`
While the UBJ `bool` type is pretty much the same as the Rust `bool` type, the 
UBJ `null` does not correspond to any of the primitive Rust values. The closest 
Rust equivalent for UBJ `null` is the Rust `Option::None` variant.

```rust
// file: src/internal/io/writer.rs

impl<S> UbjWriter<S> where S: Sink
{
    pub fn write_null(&mut self) -> UbjResult<()> 
    {
        self.write_marker(UbjMarker::Null)
    }

    pub fn write_bool(&mut self, v: bool) -> UbjResult<()> 
    {
        self.write_marker(
            if v { UbjMarker::True } 
            else { UbjMarker::False }
        )
    }
}
```

The UBJ `null` and the two booleans carry no data at all: the marker *is* the 
value (`[Z]`, `[T]`, `[F]`). There is nothing to narrow and nothing to encode.


#### `integers`
The UBJ `int8`, `uint8`, `int16`, `int32` and `int64` types are pretty much the 
same as the Rust `i8`, `u8`, `i16`, `i32` and `i64` types.

The [Format - Optmizations](./05-format.md#optimizations) section gives an important rule: *always use 
the smallest numeric type that fits*. Our `UbjWriter` does not trust its caller 
to have picked the right width: each widening method first checks whether the 
value would fit in a narrower type and, when it does, delegates to that narrower
writer. The `int16` method is where the rule is most visible:

```rust
// file: src/internal/io/writer.rs

impl<S> UbjWriter<S> where S: Sink
{
    pub fn write_int16(&mut self, v: i16) -> UbjResult<()> 
    {
        if v >= i8::MIN as i16 && v < 0 {
            self.write_int8(v as i8)
        } 
        else if v >= 0 && v <= u8::MAX as i16 {
            self.write_uint8(v as u8)
        } 
        else {
            self.write_sized_packet(UbjMarker::Int16, v.to_be_bytes())
        }
    }
}
```

The three arms are a small decision table:

- a **negative** value that fits `i8` is written as `int8` — one payload byte;
- a **non-negative** value that fits `u8` is written as `uint8` — also one byte,
  but with twice the range, `0…255` instead of `-128…127`. That is why `100`
  and `200` both cost two bytes, while `-300` needs the full `Int16`
  (`[I][-300]`, three bytes);
- anything else keeps the `Int16` marker and its two-byte payload.

`write_int32` and `write_int64` apply the same idea one level up: they test the
next narrower *signed* range and delegate, so a value can descend the whole
chain instead of stopping halfway.

```rust
// file: src/internal/io/writer.rs

impl<S> UbjWriter<S> where S: Sink
{
    pub fn write_int32(&mut self, v: i32) -> UbjResult<()> 
    {
        if v >= i16::MIN as i32 && v <= i16::MAX as i32 {
            self.write_int16(v as i16)
        } 
        else {
            self.write_sized_packet(UbjMarker::Int32, v.to_be_bytes())
        }
    }

    pub fn write_int64(&mut self, v: i64) -> UbjResult<()> 
    {
        if v >= i32::MIN as i64 && v <= i32::MAX as i64 
        {
            self.write_int32(v as i32)
        } 
        else {
            self.write_sized_packet(UbjMarker::Int64, v.to_be_bytes())
        }
    }
}
```

Because each step delegates, the choice *cascades*. A `100` passed to
`write_int64` walks all the way down to `write_uint8` and costs two bytes
instead of the nine an `Int64` would take:

<pre style="width: fit-content; margin-left: auto; margin-right: auto;"><code class="language-text">write_int64(100)
    -> write_int32(100)   fits i32
    -> write_int16(100)   fits i16
    -> write_uint8(100)   fits u8
    -> [U][100]           2 bytes</code></pre>

A couple of edges are worth remembering. UBJ has a single unsigned marker (`U`)
for the whole `0…255` byte range, and every wider non-negative value is written
with a *signed* marker — so `write_int32(65000)` cannot shrink to an unsigned
16-bit form and stays `[l][65000]` (five bytes), exactly as the
[Format - Values](./05-format.md#values) section shows.
And the two smallest writers cannot narrow at all: `write_int8` is
already minimal, while `write_uint8` gains nothing from turning into `int8`
because either way the payload is a single byte. Every payload, finally, is
produced with `to_be_bytes()`, which yields the big-endian array the
specification demands.

#### `floats`
The UBJ `float32` and `float64` types are pretty much the same as the Rust 
`f32` and `f64` types.

```rust
// file: src/internal/io/writer.rs

impl<S> UbjWriter<S> where S: Sink
{
    pub fn write_float32(&mut self, v: f32) -> UbjResult<()> 
    {
        self.write_sized_packet(UbjMarker::Float32, v.to_be_bytes())
    }
    
    pub fn write_float64(&mut self, v: f64) -> UbjResult<()> 
    {
        self.write_sized_packet(UbjMarker::Float64, v.to_be_bytes())
    }
}
```

There is no narrowing here, and none is possible: a float is not an integer, so
`float32` and `float64` always write their fixed four or eight bytes.
`to_be_bytes()` gives the IEEE 754 layout in the same most-significant-byte-first
order the integers use.


#### `char`
First of all, an UBJ `char` is an ASCII character, that's *not* the same 
as a Rust `char`.

```rust
// file: src/internal/io/writer.rs

impl<S> UbjWriter<S> where S: Sink
{
    pub fn write_char(&mut self, v: char) -> UbjResult<()> 
    {
        let utf8_code = u32::from(v);
        if utf8_code > 127 {
            Err(UbjError::NotAsciiChar(utf8_code))
        } 
        else {
            self.write_sized_packet(UbjMarker::Char, [utf8_code as u8])
        }
    }
}
```

The two `char` notions do not line up, and mixing them up is an easy bug:

- a Rust `char` is a **Unicode scalar value**: it takes exactly four bytes in 
  memory (sharing the same memory layout of Rust `u32`), and it represents 
  every code point from `U+0000` to `U+10FFFF` (`'A'` is `U+0041`, `'é'` 
  is `U+00E9`);
- on the other hand, a UBJ `char` (`C`) is a **single byte** holding an ASCII 
  code in the `0…127` range.

The Rust core `u32::from(v)` method extracts the code point; it is *not* the 
same as calling `v.encode_utf8(...)`, which would produce a variable-width 
sequence of one to four UTF-8 bytes. For a UBJ char we want the code point 
(which is scalar value), and then a range check. Anything above `127` is 
rejected with `UbjError::NotAsciiChar`, so `'A'` becomes `[C][A]` (`43 41`) 
while `'é'` is an error rather than a two-byte sequence.

#### `Strings`

UBJ strings come in two flavours. `write_marked_string` writes a complete value;
`write_unmarked_string` writes only the length-plus-bytes part, which is what an
object name needs.

```rust
// file: src/internal/io/writer.rs

impl<S> UbjWriter<S> where S: Sink
{
  pub fn write_marked_string(&mut self, v: &str) -> UbjResult<()> {
    self.write_marker(UbjMarker::String)?;
    self.write_unmarked_string(v)
  }
    
  pub fn write_unmarked_string(&mut self, v: &str) -> UbjResult<()>
  {
      let payload: &[u8] = v.as_bytes();
      let len = payload.len();
      if len <= (i64::MAX as usize) {
          self.write_int64(len as i64)?;
          self.write_payload(payload)
      } 
      else {
          Err(UbjError::Unsupported(
              "string value with length greater than i64::MAX",
          ))
      }
  }
}
```

`write_unmarked_string` is just the marker-less part of a string: the length,
then the UTF-8 bytes. The length is written with `write_int64`, which means it
benefits from the narrowing above for free — a two-byte name gets a one-byte
`U` length, not an eight-byte `L`. 

`write_marked_string` only prepends the `S` marker and defers the rest.

So `"hi"` is `[S][U][2][hi]` — `53 55 02 68 69` — and the same pair of bytes
appears, without the `53`, as an object name. The `i64::MAX` guard exists only
because the length must fit the `L` payload; on the platforms our `serde_ubj` 
crate targets the length always fits, so the branch is effectively unreachable 
and is there for completeness.

### Containers
The four container methods hold no surprises: each writes a single marker and
nothing else.

```rust
// file: src/internal/io/writer.rs
pub fn write_start_array(&mut self) -> UbjResult<()> {
    self.write_marker(UbjMarker::StartArray)
}

pub fn write_end_array(&mut self) -> UbjResult<()> {
    self.write_marker(UbjMarker::EndArray)
}

pub fn write_start_object(&mut self) -> UbjResult<()> {
    self.write_marker(UbjMarker::StartObject)
}

pub fn write_end_object(&mut self) -> UbjResult<()> {
    self.write_marker(UbjMarker::EndObject)
}
```

There is no length, no count, and no payload: the streaming form is delimited by
the open and close markers alone. The writer does not track nesting depth or
balance either — opening and closing are just two calls, and it is the caller's
responsibility to pair them. A serializer writes the start marker, then any 
number of child packets, then the matching end marker:

```rust
w.write_start_array()?;
w.write_uint8(1)?;
w.write_uint8(2)?;
w.write_end_array()?;
```

That emits `5B 55 01 55 02 5D` — exactly the array from the
[Format - Containers](./05-format.md#containers) section. Objects work the same 
way, with each name written by `write_unmarked_string` and each value by one of 
the scalar or container methods: `{"hi": 1}` is `7B 55 02 68 69 55 01 7D`.


## Unit Test

Following the recipe from the 
[Structuring - Tests](../04-structuring/04-tests.md) section, the `UbjWriter`
unit tests live in a `#[cfg(test)] mod tests` block at the bottom  of the 
`src/internal/io/writer.rs` file. Being nested inside the file, that module can
reach the private `sink` field, so each test can drive the writer over a plain
`&mut [u8]` sink. This works in every configuration — `embedded_io::Write` is
implemented for `&mut [u8]` even under strict `no_std`, and nothing ever touches
the heap — so the very same test cases run under `std`, `no_std + alloc` and
strict `no_std`.

> [!NOTE]
> A Rust **macro** is a rule that rewrites a short piece of syntax into a 
> larger piece of code *before* the compiler type-checks it. 

We already met macros in the 
[Structuring - Errors](../04-structuring/04-errors.md#macros) section, where 
`thiserror` expands a few attribute lines into whole trait  implementations. 
Here we use a `macro_rules!` *macro by example*: the name is followed by `!`, 
and each > `(pattern) => { body }` arm describes how a call is rewritten. A 
macro can do two things a plain function cannot — a failing assertion reports 
the *call site* file and line, and the captured tokens can  be quoted with 
`stringify!` — and that is exactly what makes it attractive for a test helper. 
For everything else, prefer a function.

### `assert_encodes!`

Almost every test asks the same question: *"if I call these methods, do I get
exactly these bytes?"* Instead of repeating the setup, we package it once into 
a reusable macro rule:

```rust
// file: src/internal/io/writer.rs

#[cfg(test)]
mod tests {
    use super::UbjWriter;
    use crate::internal::result::UbjResult;

    macro_rules! assert_encodes {
        (|$writer:ident| $body:expr, $expected:expr $(,)?) => {{
            let mut buf = [0u8; 64];
            assert!(
                $expected.len() <= 64,
                "the expected encoding ({} bytes) exceeds the 64-byte helper buffer",
                $expected.len()
            );
            let mut writer = UbjWriter { sink: &mut buf[..] };
            let run = |$writer: &mut UbjWriter<&mut [u8]>| -> UbjResult<()> { $body };
            run(&mut writer).expect(concat!(
                "the write operation should succeed (",
                stringify!($body),
                ")"
            ));
            let written = 64 - writer.sink.len();
            assert_eq!(&buf[..written], &$expected[..]);
        }};
    }

    // ... test cases here ...
}
```

The closure comes first and the expected bytes second. The helper keeps a
64-byte buffer on the stack — comfortably above the longest encoding our tests
produce, but fixed, so nothing is allocated. The `$expected.len() <= 64` guard
turns a future, too-long expectation into a clear panic instead of a misleading
`WriteZero`, while `stringify!` quotes the body in the failure message. The
trailing `$(,)?` accepts an optional comma, so multi-line invocations may end
with a trailing comma like ordinary function calls. Note also the closure
parameter: the `$writer:ident` in the pattern is what keeps `w` hygienic, so the
same name can be reused by every test without clashing.

### Examples

Scalars assert the marker and the payload together:

```rust
#[test]
fn write_int16_narrows_when_fitting() {
    assert_encodes!(|w| w.write_int16(-123), &[0x69, 0x85]);
    assert_encodes!(|w| w.write_int16(123), &[0x55, 0x7B]);
    assert_encodes!(|w| w.write_int16(256), &[0x49, 0x01, 0x00]);
}
```

Containers use the closure body to run several method calls in sequence, and
assert the whole self-delimiting stream at once:

```rust
#[test]
fn write_a_heterogeneous_array() {
    assert_encodes!(
        |w| {
            w.write_start_array()?;
            w.write_null()?;
            w.write_bool(true)?;
            w.write_int8(-5)?;
            w.write_char('A')?;
            w.write_end_array()
        },
        &[
            0x5B,        // StartArray
            0x5A,        // null
            0x54,        // true
            0x69, 0xFB,  // Int8  -5
            0x43, 0x41,  // Char 'A'
            0x5D,        // EndArray
        ],
    );
}
```

Objects work the same way: each name goes through `write_unmarked_string` and
each value through one of the scalar or container methods. Here is `{"id": 7}`:

```rust
#[test]
fn write_an_object() {
    assert_encodes!(
        |w| {
            w.write_start_object()?;
            w.write_unmarked_string("id")?;
            w.write_uint8(7)?;
            w.write_end_object()
        },
        &[0x7B, 0x55, 0x02, 0x69, 0x64, 0x55, 0x07, 0x7D],
    );
}
```

Error cases deliberately keep a *function* helper instead of the macro, because
the test has to pick the buffer size: a sink that is one byte too small is
exactly what forces the failure.

```rust
#[test]
fn write_propagates_sink_errors_as_io_errors() {
    assert_io_error([0u8; 1], |w| w.write_int16(256));
    assert_io_error([0u8; 2], |w| w.write_int32(32768));
}
```

That `assert_io_error` takes the buffer by value, runs the closure, and asserts
the result is `UbjError::IO(ErrorKind::WriteZero)`. Leaving the implementation 
of the `assert_io_error` helper as exercise to the reader.

