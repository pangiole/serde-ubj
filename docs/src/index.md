# Walkthrough a Serde format in Rust
_by Paolo Angioletti, Software Engineering Series, Year 2026_

> [!WARNING]
> This book is **under development**. Some chapters are still incomplete and
> the content may change without notice.

Welcome! In this book we take a guided tour through the implementation of a
<a href="https://serde.rs" target="_blank"
   rel="noopener noreferrer">Serde</a>
data format in Rust. Instead of treating serialization as a black box, we open
up a real crate and follow the data from a Rust value, to bytes on the wire,
and all the way back again.

The crate we walk through is
<a href="https://github.com/pangiole/serde-ubj" target="_blank"
   rel="noopener noreferrer"><code>serde-ubj</code></a>,
a small implementation of
<a href="https://github.com/ubjson/universal-binary-json" target="_blank"
   rel="noopener noreferrer">Universal Binary JSON</a> (UBJ):
a compact, binary cousin of JSON that keeps the same conceptual model while
using far fewer bytes.
