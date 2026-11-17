# Serde Model

> [!WARNING]
> This book is **under development**. Some chapters are still incomplete and
> the content may change without notice.

In this section we do ...


## What is Serde
Although we went this far through the book, we may still wonder: _"But, what is Serde?"_. 

To be clear, Serde is not a data format neither a data model. Rather, it is 
a **serialization framework** implemented as a set of traits enforcing 
interactions (function calls) aimed to serialize (and deserialize) any 
application data model to (and from) any external data format.

In on order to achieve such an ambitious goal, the software engineers who 
designed the Serde framework needed a way to abstract over the Rust type 
system, at least for serialization purpose. Hence, the so-called
__"Serde Data Model"__ was conceived as an intermediate abstraction able to
represents all serializable Rust types by classifying them into a universal set
of categories: primitives, options, unit / newtype / tuple / struct / enum 
variants, strings, sequences, and maps.

This Serde data model sits in between our application data model, expressed
by combining Rust types, and the many external data formats we wish to
serialize to (or deserialize from). This intermediate model is what allows
serializers (and deserializers) to be implemented without tight coupling
them to the specific rules of the external data formats.

![The Serde data model](../images/serde_approach.webp)

As already explained above, without such the Serde data model, if we have
$N$ Rust types and $M$ external data formats (JSON, Protobuf, YAML, TOML,
MessagePack, etc.), we would  theoretically need to write $N \times M$
implementations. As soon as we introduce the Serde data model, the equation
changes completely:

1. a Rust type implements the serialization traits by breaking itself down
   into the Serde data model (e.g., a Rust struct becomes a Serde map or a
   Serde sequence of fields);

2. the serialization implementation knows how to write the Serde Data Model
   out to (and/or in from) a specific data format (e.g., UBJ  structures or
   JSON object syntax).

This reduces the implementation overhead from $N \times M$ custom serializers
down to $N + M$ modular integrations.

## Types

...


| Rust types                           | Serde model         |
|--------------------------------------|---------------------|
| `()`                                 | **unit**            |
| `core::Option<T>::None`              | **none**            |
| `bool`                               | **bool**            |
| `char`                               | **char**            |
| `i8`, `i16`, `i32`, `i64`, etc.      | *integers*          |
| `f32`, `f64`                         | *floats*            |
| `&str`                               | **str**             |
| `alloc::string::String`              | **str**             |
| `&[u8]`                              | **bytes**           |
| `core::Option<T>::Some()`            | **some**            |
| `alloc::vec::Vec<T>`                 | **seq**             |
| `[T; N]`                             | **tuple**           |
| `(A, B)   `                          | **tuple**           |
| `struct Unit`                        | **unit_struct**     |
| `struct Newtype(T)`                  | **newtype_struct**  |
| `struct Tuple(A, B)`                 | **tuple_struct**    |
| `struct WithFields {a: A}`           | **struct**          |
| `enum Unit { Variant }`              | **unit_variant**    |
| `enum Newtype { Variant(T) }`        | **newtype_variant** |
| `enum Tuple { Variant(A, B) }`       | **tuple_variant**   |
| `enum WithFields { Variant {a: A} }` | **struct_variant**  |
| `alloc::collections::BTreeMap<K, V>` | **map**             |



