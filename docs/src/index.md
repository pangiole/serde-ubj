# Walkthrough a Serde format in Rust
_by Paolo Angioletti, Software Engineering Series, Year 2026_

> [!WARNING]
> This book is **under development**. Some chapters are still incomplete and
> the content may change without notice.

Welcome! In this book we take a guided tour throughout the implementation of a
<a href="https://serde.rs" target="_blank" rel="noopener noreferrer">Serde</a>
data format in Rust language. 

Instead of treating serialization as a black box, we open up a real crate and
follow the data from a Rust value, to bytes on the wire, and all the way back 
again. The crate we walk through is <a href="https://crates.io/crates/serde-ubj" 
target="_blank" rel="noopener noreferrer"><code>serde_ubj</code></a>, a small
implementation of <a href="https://github.com/ubjson/universal-binary-json" 
target="_blank" rel="noopener noreferrer">UBJ - Universal Binary JSON</a>:
a compact, binary cousin of JSON that keeps the same conceptual model while
using far fewer bytes.