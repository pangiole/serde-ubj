# Introduction

In this chapter we set the stage, clarifying who this book is for, how to read
it, and to expect. 

## Who this book is for

We assume that we already have some basic Rust coding skills: for example, what 
are traits and how to implement them for our types; what are generic types, and 
how to handle results representing either success or failures. We do *not* 
assume any prior knowledge of Serde internals, neither we assume knowledge of 
the UBJ - Universal Binary JSON format. We introduce concepts from first 
principles, and we grow the complexity step by step.

## How to read this book

We recommend reading the chapters in order. The first three chapters set up the
problem and the architecture. The two middle parts build the format layer and
then the Serde layer, mirroring the order in which the real crate is written.
The final part covers the practical concerns that make the implementation
usable in the real world.

## What we are going to build

By the end of the book we will understand, layer by layer, how a Serde format
can be implemented in Rust language. We will be implementing a Rust library 
crate, named `serde-ubj`, targeting one particular data format, named **UBJ - 
Universal Binary JSON** (see <a href="https://ubjson.org/" target="_blank" 
rel="noopener noreferrer">https://ubjson.org/</a>).

Our Rust crate is going to be layered as follows:

- **the wire format**  
  the binary markers and payloads that define UBJ on disk or on the network,
  as defined by the official format specification (see 
  <a href="https://ubjson.org" target="_blank" rel="noopener noreferrer"></a>);

- **the format layer**  
  small reader and writer Rust types that know the _"on-the-wire"_ bytes but 
  nothing about Serde;

- **the Serde layer**  
  the `Serializer`/`Deserializer` adapters that translate the
  <a href="https://serde.rs/data-model.html" target="_blank"
  rel="noopener noreferrer">Serde data model</a> into/from those bytes;

- **the edge cases**  
  enums, options, `no_std` support, buffering, and testing.

