# Format

In this section we lay out the Universal Binary JSON format itself. We begin with
the **packet** structure — a marker, an optional payload, and (for containers
only) a closing marker — then catalogue every marker, describe the wire layout of
each **value** (null, booleans, the fixed-width numerics, char and string), walk
through the two **containers**, and close with the **optimizations** the
specification recommends. The tables here are the reference that the writer and
reader chapters encode into their `write_*` and `read_*` methods.


## Structure

Every value in UBJ is encoded as a **packet**: a one-byte **marker**, followed by
an optional **data payload**, and — for containers only — an optional **closing
marker**.

<pre style="width: fit-content; margin-left: auto; margin-right: auto;"><code class="language-text">
    packet = marker + (payload) + (marker)
    
        +----------+  +-------------------+  +----------+
        |  MARKER  |  |   DATA PAYLOAD    |  |  MARKER  |
        | (1 byte) |  | (0 or more bytes) |  | (0 or 1) |
        +----------+  +-------------------+  +----------+
           always          optional             optional
                                         (closing marker)            
    

    scalar packet:     [S] [i] [5] [hello]      (no closing marker) 
    container packet:  [ [U][1] [U][2] ]        (closes with ])     

</code></pre>

The **marker** is a single byte — always one of a small set of printable ASCII
characters — that describes *what kind of value* comes next. For the fixed-width
types the marker doubles as a width descriptor: `i` announces a signed 8-bit
integer in the next byte, `l` announces a signed 32-bit integer in the next four
bytes, and so on. The **payload** carries the data, and its shape is decided
entirely by the marker:

- **no payload** — `null`, `true` and `false` are fully described by their marker;
- **a fixed-width payload** — integers, floats and chars occupy a known number of
  bytes right after the marker;
- **a length-prefixed payload** — a string is a byte length followed by that many
  UTF-8 bytes;
- **a nested, close-terminated payload** — arrays and objects hold further
  packets between an opening and a closing marker.

The third part is the **closing marker**, and only the two container types use
it: an array or object opens with `[` or `{`, holds zero or more child packets,
and terminates with `]` or `}`. A scalar never has one — its packet is just the
marker and, when the type carries data, its payload.

There is no other framing: the bytes are laid out back to back, and the
marker → payload → closing-marker rhythm is all the reader needs to walk them.
All fixed-width numeric payloads are written in **big-endian order** — most 
significant byte first — so the same bytes mean the same number on every 
machine (as long as both the writer and reader are implemented to comply with 
big-endianness).

The specification illustrates bytes in **block-notation**, where every bracketed
token is a marker or a literal value. `[S][i][5][hello]` therefore reads: string
marker `S`, then int8 marker `i` with length `5`, then the five bytes `hello`. We
use the same notation when a picture is worth a thousand words.


## Markers

UBJ divides its markers into **value types** and **container types**. The
tables below reproduce the official quick reference; the `Marker` column is the
byte you actually find on the wire.

| Type | Size | Marker | Length | Data Payload |
|------|------|--------|--------|--------------|
| null | 1 byte | `Z` | No | No |
| no-op | 1 byte | `N` | No | No |
| true | 1 byte | `T` | No | No |
| false | 1 byte | `F` | No | No |
| int8 | 2 bytes | `i` | No | Yes |
| uint8 | 2 bytes | `U` | No | Yes |
| int16 | 3 bytes | `I` | No | Yes |
| int32 | 5 bytes | `l` | No | Yes |
| int64 | 9 bytes | `L` | No | Yes |
| float32 | 5 bytes | `d` | No | Yes |
| float64 | 9 bytes | `D` | No | Yes |
| high-precision | 1 byte + int + bytes | `H` | Yes | Yes (if non-empty) |
| char | 2 bytes | `C` | No | Yes |
| string | 1 byte + int + bytes | `S` | Yes | Yes (if non-empty) |
| array | 2+ bytes | `[` … `]` | Optional | Yes (if non-empty) |
| object | 2+ bytes | `{` … `}` | Optional | Yes (if non-empty) |


Two value types — **no-op** (`N`) and **high-precision number** (`H`) — belong to
the specification but are not needed by the I/O Layer described in this book, so
we list them only for completeness. Everything else above is what our UBJ 
writer and reader implementations must handle.


The specification is simple enough to be directly translated into a Rust 
`markers` module, and an `UbjMarker` enumeration, as follows:

```rust
// file: src/internal/io/marker.rs

#[derive(PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum UbjMarker {
    Null        = 0x5A,  // Z
    True        = 0x54,  // T
    False       = 0x46,  // F
    Int8        = 0x69,  // i  [-128 .. 0 .. 127]
    Uint8       = 0x55,  // U  [0 .. 255]
    Int16       = 0x49,  // I  [-32768 .. 0 .. 32767]
    Int32       = 0x6C,  // l  [-2147483648 .. 0 .. 2147483647]
    Int64       = 0x4C,  // L  [-9223372036854775808 .. 0 .. 9223372036854775807]
    Float32     = 0x64,  // d  IEEE 754 single precision
    Float64     = 0x44,  // D  IEEE 754 double precision
    Char        = 0x43,  // C  ASCII range
    String      = 0x53,  // S
    
    StartArray  = 0x5B,  // [
    EndArray    = 0x5D,  // ]
    
    StartObject = 0x7B,  // {
    EndObject   = 0x7D,  // }
}
```

## Values

**null** (`Z`) has neither length nor payload: the marker *is* the whole value.
**true** (`T`) and **false** (`F`) are likewise marker-only — there is no payload
byte to interpret, because the marker already gave the answer.

The **eight numeric types** are where the payload begins to matter. Each is a
fixed-width, big-endian integer or an IEEE 754 floating-point number:

| Type | Marker | Width | Range |
|------|--------|-------|-------|
| int8 | `i` | 1 byte | −128 … 127 |
| uint8 | `U` | 1 byte | 0 … 255 |
| int16 | `I` | 2 bytes | −32,768 … 32,767 |
| int32 | `l` | 4 bytes | −2,147,483,648 … 2,147,483,647 |
| int64 | `L` | 8 bytes | −9,223,372,036,854,775,808 … 9,223,372,036,854,775,807 |
| float32 | `d` | 4 bytes | IEEE 754 single precision |
| float64 | `D` | 8 bytes | IEEE 754 double precision |

All integer numbers are written in **big-endian order** (a.k.a. network order), 
so an `int16` value of `1` is `00 01` and an `int32` value of `65000` is
`00 00 FD E8`, with the most significant byte placed on the right-hand side. 
The floating numbers keep the same big-endian order and use the standard 
IEEE 754 sign/exponent/fraction layout.

A **char** (`C`) is a single byte holding an unsigned integer in the `0…127`
range — one ASCII character. It is *not* a UTF-8 encoded character and must 
not be used outside that range.

A **string** (`S`) is length-prefixed: the `S` marker is followed by a length 
and then that many UTF-8 bytes. The length is itself one of the integer types 
above, so a writer may pick the cheapest one that fits and a reader must accept 
any of them. UTF-8 is the *required* string encoding, and a string's byte size 
depends on both its length prefix and how many bytes its characters need 
(non-ASCII text takes more than one byte per character).

One special case: inside an object, the **name** of a name/value pair is a string
*without* its `S` marker. JSON forbids non-string names, so the marker is
redundant there and the specification says it must be omitted — only the length
and the UTF-8 bytes appear. We call this an *unmarked string*.

Here is every value type written out in block notation, next to the bytes a
reader would see on the wire. Marker-only types have no payload; every other
type shows its marker followed by the payload bytes.

<pre style="width: fit-content; margin-left: auto; margin-right: auto;"><code class="language-text">type      value    block notation  bytes
--------  -------  --------------  --------------------------
null               [Z]             5A
true               [T]             54
false              [F]             46
int8      -2       [i][-2]         69 FE
uint8     200      [U][200]        55 C8
int16     -300     [I][-300]       49 FE D4
int32     65000    [l][65000]      6C 00 00 FD E8
int64     1000000  [L][1000000]    4C 00 00 00 00 00 0F 42 40
float32   1.5      [d][1.5]        64 3F C0 00 00
float64   -2.5     [D][-2.5]       44 C0 04 00 00 00 00 00 00
char      'A'      [C][A]          43 41
string    "hi"     [S][U][2][hi]   53 55 02 68 69</code></pre>


## Containers

UBJ has exactly two container types, mirroring JSON's **array** and
**object**. Both are streaming-friendly: a single marker opens the container,
its elements follow, and a single marker closes it, so a producer can begin
writing before it knows how large the container will be. The shape of the data is
therefore identical to JSON — only the syntax is binary (single-byte markers
instead of commas and brackets).

An **array** is `[`, then zero or more values, then `]`. `[1, 2]` becomes
`5B 55 01 55 02 5D`: start array `5B`, uint8 `1` as `55 01`, uint8 `2` as
`55 02`, end array `5D`.

<pre style="width: fit-content; margin-left: auto; margin-right: auto;"><code class="language-text">array  [1, 2]

    [                        <- start array marker
      [U][1]                 <- element
      [U][2]                 <- element
    ]                        <- end array marker

bytes: 5B 55 01 55 02 5D</code></pre>

An **object** is `{`, then zero or more name/value pairs, then `}`. Each name is
an *unmarked string* and each value is an ordinary marker-plus-payload. An object
whose only pair maps the name `hi` to the value `1` becomes
`7B 55 02 68 69 55 01 7D`: start object `7B`, name length `2` as `55 02`, the
bytes `h` `i` as `68 69`, uint8 `1` as `55 01`, end object `7D`.

<pre style="width: fit-content; margin-left: auto; margin-right: auto;"><code class="language-text">object  {"hi": 1}

    {                        <- start object marker
      [U][2][hi]  [U][1]     <- name "hi"  value 1
    }                        <- end object marker

bytes: 7B 55 02 68 69 55 01 7D</code></pre>

Notice the recursion: a container is a well-delimited run of values (or
name/value pairs), and every element is itself just a value. That regularity is
what lets one small dispatch routine handle the entire tree.

The specification also defines an optional **optimized format** for containers,
which we look at next.

## Optimizations

The specification's headline advice is blunt: *always use the smallest numeric
type that fits your needs.* Because there is a marker for every width, storing
`100` as an `int32` is pure waste — as a `uint8` it costs two bytes instead of
five. The writer implements exactly this rule, narrowing each integer to the
smallest type that can hold it before emitting the marker.

That optimization mirrors onto the reading side. Since a producer may
legitimately choose *any* width — including for a string's length prefix — a
consumer cannot assume a particular marker and must accept them all. The reading
chapter shows the reader trying each width in turn until one succeeds.

The specification also defines an optional **optimized format** for containers,
built from two parameters: a **count** (`#`, how many children follow) and a
**type** (`$`, one marker shared by every child, producing a *strongly typed
container*). This crate does not emit that form — it always writes the plain
streaming form above — but recognising the terms helps when reading the
specification, and explains why a container *could* declare its size even though
ours never does.


## Conclusion
That is everything we need to understand the next chapters: the I/O Layer will 
expose a `write_*` method for each marker above — from `write_null` and 
`write_int8` up through `write_marked_string` and `write_start_array` — and a 
matching `read_*` counterpart on the reading side.