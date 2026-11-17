# Application Model

> [!WARNING]
> This book is **under development**. Some chapters are still incomplete and
> the content may change without notice.

From our Rust application's point of view, any custom type we define to
represent business concepts belongs to the so-called _"Application
Model"_. This data model is what we author and wish to serialize to (and
deserialize from) an external data format - such as UBJ (Universal Binary
JSON) or Protocol Buffer. Serialization is what bridges our internal memory
structures with the outside world, as allows exchanging data across
applications.

For example, Since they represent business concepts of our application, the
following custom types are to be considered part of our application data model:

```rust
enum Group {
  Leader,
  Staff,
  Intern
}

struct Employee {
  name: &str,
  age: u8,
  groups: Vec<Group>
}
```

To appreciate why Serde matters, let's imagine it did not exist. Every time we
wanted to exchange a `Employee` with the outside world, we would have to write the
serialization logic by hand, in Rust language, once for every data format we
care about.

Take the `Group` enum. In JSON each variant must be turned into a quoted string:

```rust
match group {
    Group::Leader => "\"Leader\"",
    Group::Staff => "\"Staff\"",
    Group::Intern => "\"Intern\"",
}
```

MessagePack can also represent the variants as strings but, since it is a
binary format, the writer must emit a type marker and a byte-length prefix that
JSON does not have:

```rust
// a byte marker that encodes both the type and the 5-byte length
writer.write_byte(0xA5).write_bytes(b"Leader");
```

Protocol Buffers cannot reuse either idea: there an enum is an integer defined
in a `.proto` schema, with an explicit numeric value attached to each variant:

```proto
enum Group {
  ADMIN = 0;
  STAFF = 1;
  GUEST = 2;
}
```

The above examples are making us understand how the very same Rust values demand
three unrelated handwritten translations, one for each external data format.

The `Employee` struct amplifies the problem. JSON wants it as an object with quoted
keys — `name`, `age`, `groups` — and the string fields must be escaped according
to JSON's own rules. MessagePack might encode the same struct as a map or as a
fixed-order array of values, each preceded by its special binary markers.
Protocol Buffers expects a message whose fields are addressed by numeric tags
rather than names, with integers written using its variable-length encoding.
Here again, three external data formats, three different sets of rules, and
none of them can be produced by the same handwritten code.


![Implementation overhead before Serde](../images/before_serde.webp)

The burden compounds quickly. Our two types and three formats already amount to
six separate encoding routines — and six more for decoding. Add a fourth type
and the count jumps again; add a fourth format and every existing type must be
revisited. Each new format multiplies the number of routines by the number of
types, and vice versa. This is the combinatorial explosion that a naive,
format-by-format approach generates, and it is exactly what the Serde framework
is designed to eliminate.



