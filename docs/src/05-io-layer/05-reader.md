# Reading

> [!WARNING]
> This book is **under development**. Some chapters are still incomplete and
> the content may change without notice.

In this section we build the mirror image of the writing module: the `UbjReader`.
Where the writer pushes markers and payloads into a sink, the reader pulls
exactly the right number of bytes out of a buffered source — even when they
arrive in awkward chunks.

## Struct

As with the writer, we code against `embedded-io` instead of `std::io`. Let's
introduce the `Source` synonym for the `embedded_io::BufRead` trait, and define
our public `UbjReader` struct as something wrapping an underlying buffered
source.

```rust
// file: src/internal/io/reader.rs
use embedded_io::BufRead as Source;

pub struct UbjReader<S>
where
    S: Source,
{
    source: S,
    bytes_consumed: usize,
}
```

The extra `bytes_consumed` counter is not required by the decoder itself, but it
keeps track of how much input we have already touched.

## Factories

Just like the `UbjWriter`, we provide two `new` factory functions — one per
world.

The `no_std` one takes an embedded source directly:

```rust
// file: src/internal/io/reader.rs
#[cfg(not(feature = "std"))]
impl<S> /*<highlight>*/UbjReader<S>/*</highlight>*/
where
    S: Source,
{
    /// Creates a new UBJ reader instance that delegates all operations to the
    /// provided source.
    pub fn new(source: S) -> Self {
        Self { /*<highlight>*/source/*</highlight>*/, bytes_consumed: 0 }
    }
}
```

The `std` one adapts any `std::io::BufRead` through the `FromStd` wrapper, so the
deserializer can be built from an ordinary buffered reader:

```rust
// file: src/internal/io/reader.rs
use embedded_io_adapters::std::FromStd;

#[cfg(feature = "std")]
impl<S> /*<highlight>*/UbjReader<FromStd<S>>/*</highlight>*/
where
    S: std::io::BufRead,
{
    /// Creates a new UBJ reader instance that delegates all operations to the
    /// provided `std::io::BufRead` reader, adapting it through `FromStd`.
    pub fn new(source: S) -> Self {
        Self {
            /*<highlight>*/source: FromStd::new(source)/*</highlight>*/,
            bytes_consumed: 0,
        }
    }
}
```

Note that `&[u8]` implements *both* `std::io::BufRead` and
`embedded_io::BufRead`. That is the small but crucial detail which lets the
slice-based entry points work unchanged in either world.

## The three primitives

Almost everything is built from three small helpers:

- **`buf_refill`**  
  asks the source for a slice of buffered bytes and turns an empty slice into
  `UnexpectedEof`. Errors from the source are collapsed to an `ErrorKind` and
  converted through `From`:

  ```rust
  use embedded_io::Error;

  let buf = self.source.fill_buf().map_err(|e| UbjError::from(e.kind()))?;
  ```

- **`buf_consume`**  
  advances the source with `consume` and updates the byte counter.

- **`buf_consume_marker`**  
  looks at the next byte, checks that it matches the expected marker, and only
  then consumes it. A mismatch becomes `IllegalMarker`. Because the byte is *not*
  consumed on a mismatch, this also doubles as a one-byte peek — which is what
  makes optional values and container end-detection possible.

## Reading a fixed number of bytes

Many values need a fixed-width payload: an `i32` needs four bytes, an `f64`
needs eight. A single `fill_buf` call is not obliged to return them all at once,
so the reader loops, copying as much as it can on each iteration into a
stack-allocated array.

```rust
fn buf_consume_bytes<const LEN: usize>(&mut self) -> UbjResult<[u8; LEN]> {
    let mut joining_buffer = [0u8; LEN];
    let mut pending_bytes = LEN;
    let mut bytes_copied = 0;

    while pending_bytes > 0 {
        let source_buffer = self.buf_refill()?;
        let current_chunk = source_buffer.len().min(pending_bytes);
        // ... copy the chunk, consume it, update the counters ...
    }

    Ok(joining_buffer)
}
```

## Reading a length

Strings and other length-prefixed values need a `usize`. Because the writer may
have narrowed the length to any integer width, the reader simply tries them in
order until one succeeds:

```rust
fn buf_consume_usize(&mut self) -> UbjResult<usize> {
    self.read_uint8().map(|n| n as usize)
        .or_else(|_| self.read_int8().map(|n| n as usize))
        .or_else(|_| self.read_int16().map(|n| n as usize))
        .or_else(|_| self.read_int32().map(|n| n as usize))
        .or_else(|_| self.read_int64().map(|n| n as usize))
}
```

## The UTF-8 chunk boundary problem

Reading a string is where buffering really bites. We know the byte length, but a
multi-byte UTF-8 character may be split across two buffer chunks. The reader
inspects the prefix bits of the first trailing byte to learn how long the
character is meant to be, then reads the remaining bytes with
`buf_consume_bytes` and stitches the character back together.

```rust
fn bytes_count(first_byte: &u8) -> usize {
    if first_byte & 0b1000_0000 == 0 { 1 }                 // 0xxxxxxx
    else if first_byte & 0b1110_0000 == 0b1100_0000 { 2 }  // 110xxxxx
    else if first_byte & 0b1111_0000 == 0b1110_0000 { 3 }  // 1110xxxx
    else if first_byte & 0b1111_1000 == 0b1111_0000 { 4 }  // 11110xxx
    else { 0 }                                             // invalid
}
```

## Entry points

The deserializer layer builds on the reader. The crate exposes three free
functions that create it for you:

- **`from_buf_reader(&mut reader)`** (`std` only)  
  accepts any `std::io::BufRead` (already buffered) and adapts it via `FromStd`.
- **`from_vec(vec)`** (`alloc`)  
  borrows the vector as a slice and deserializes from it.
- **`from_slice(&[u8])`** (`alloc`)  
  the `no_std`-friendly counterpart: it borrows the input bytes without taking
  ownership.

Because `&[u8]` is itself a `BufRead` in both worlds, `from_vec` and `from_slice`
need no extra buffering layer.

