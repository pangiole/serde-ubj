use crate::internal::error::UbjError;
use crate::internal::io::writer::UbjWriter;
use embedded_io::Write as Sink;

/// UBJ-aware serializer for Serde
struct UbjSerializer<S>
where
    S: Sink
{
    /// The embedded writer
    writer: UbjWriter<S>,
}


// ---------------------------------------------------------------------------------
//  F A C T O R I E S
// ---------------------------------------------------------------------------------

#[cfg(feature = "std")]
impl<W> UbjSerializer<embedded_io_adapters::std::FromStd<W>>
where W: std::io::Write
{
    fn new(writer: W) -> Self {
        Self {
            writer: UbjWriter::new(writer)
        }
    }
}

#[cfg(not(feature = "std"))]
impl<S> UbjSerializer<S>
where S: Sink {
    fn new(sink: S) -> Self {
        Self {
            writer: UbjWriter::new(sink)
        }
    }
}


impl<S> serde::Serializer for &mut UbjSerializer<S> where S: Sink {
    type Ok = ();
    type Error = UbjError;

    // ---------------------------------------------------------------------------------
    //  S C A L A R    values
    // ---------------------------------------------------------------------------------

    // Invoked when serializing Rust `()`
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_null()
    }

    // Invoked when serializing Rust `Option::None`
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_null()
    }

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.writer.write_bool(v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.writer.write_int8(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.writer.write_uint8(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.writer.write_int16(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.writer.write_int32(v as i32)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.writer.write_int32(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.writer.write_int64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.writer.write_int64(v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        if v <= i64::MAX as u64 {
            return self.writer.write_int64(v as i64);
        }
        Err(UbjError::Unsupported(
            "Rust u64 values greater than i64::MAX",
        ))
    }

    fn serialize_i128(self, _v: i128) -> Result<Self::Ok, Self::Error> {
        Err(UbjError::Unsupported("Rust i128 values"))
    }

    fn serialize_u128(self, _v: u128) -> Result<Self::Ok, Self::Error> {
        Err(UbjError::Unsupported("Rust u128 values"))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.writer.write_float32(v)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.writer.write_float64(v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.writer.write_char(v)
    }

    // Invoke for serializing either Rust &str (string slices) or
    // heap-allocated String
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.writer.write_marked_string(v)
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        todo!("serialize_bytes is not yet implemented")
    }

    // ---------------------------------------------------------------------------------
    //  C O M P O U N D   values
    // ---------------------------------------------------------------------------------

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.writer.write_start_array()?;
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.writer.write_start_array()?;
        Ok(self)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.writer.write_null()
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut *self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.writer.write_start_array()?;
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.writer.write_start_object()?;
        Ok(self)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant_name: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.writer.write_start_object()?;
        self.writer.write_unmarked_string(variant_name)?;
        self.writer.write_null()?;
        self.writer.write_end_object()
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant_name: &'static str,
        variant_value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.writer.write_start_object()?;
        self.writer.write_unmarked_string(variant_name)?;
        variant_value.serialize(&mut *self)?;
        self.writer.write_end_object()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant_name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.writer.write_start_object()?;
        self.writer.write_unmarked_string(variant_name)?;
        self.writer.write_start_array()?;
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant_name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.writer.write_start_object()?;
        self.writer.write_unmarked_string(variant_name)?;
        self.writer.write_start_object()?;
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.writer.write_start_object()?;
        Ok(self)
    }

    #[cfg(not(feature = "std"))]
    fn collect_str<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + core::fmt::Display,
    {
        todo!("collect_str is not yet implemented")
    }
}

impl<W> serde::ser::SerializeSeq for &mut UbjSerializer<W>
where
    W: Sink,
{
    type Ok = ();
    type Error = UbjError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        // re-borrow itself
        value.serialize(&mut (**self))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_end_array()
    }
}

impl<W> serde::ser::SerializeTuple for &mut UbjSerializer<W>
where
    W: Sink,
{
    type Ok = ();
    type Error = UbjError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut (**self))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_end_array()
    }
}

impl<W> serde::ser::SerializeTupleStruct for &mut UbjSerializer<W>
where
    W: Sink,
{
    type Ok = ();
    type Error = UbjError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut (**self))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_end_array()
    }
}

impl<W> serde::ser::SerializeTupleVariant for &mut UbjSerializer<W>
where
    W: Sink,
{
    type Ok = ();
    type Error = UbjError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut (**self))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_end_array()?;
        self.writer.write_end_object()
    }
}

impl<W> serde::ser::SerializeStructVariant for &mut UbjSerializer<W>
where
    W: Sink,
{
    type Ok = ();
    type Error = UbjError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.writer.write_unmarked_string(key)?;
        value.serialize(&mut (**self))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_end_object()?;
        self.writer.write_end_object()
    }
}

impl<W> serde::ser::SerializeStruct for &mut UbjSerializer<W>
where
    W: Sink,
{
    type Ok = ();
    type Error = UbjError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.writer.write_unmarked_string(key)?;
        value.serialize(&mut (**self))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_end_object()
    }
}

// -------------------------------------------------------------------------------------------------

impl<W> serde::ser::SerializeMap for &mut UbjSerializer<W>
where
    W: Sink,
{
    type Ok = ();
    type Error = UbjError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        let mut map_key_serializer = MapKeySerializer {
            ubj_serializer: self,
        };
        key.serialize(&mut map_key_serializer)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut (**self))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_end_object()
    }
}

struct MapKeySerializer<'s, W>
where
    W: Sink,
{
    ubj_serializer: &'s mut UbjSerializer<W>,
}

impl<'s, W> serde::Serializer for &mut MapKeySerializer<'s, W>
where
    W: Sink,
{
    type Ok = ();
    type Error = UbjError;

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.ubj_serializer.writer.write_unmarked_string(v)
    }

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(UbjError::IllegalKeyType("bool"))
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(UbjError::IllegalKeyType("i8"))
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(UbjError::IllegalKeyType("i16"))
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(UbjError::IllegalKeyType("i32"))
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(UbjError::IllegalKeyType("i64"))
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(UbjError::IllegalKeyType("u8"))
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(UbjError::IllegalKeyType("u16"))
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(UbjError::IllegalKeyType("u32"))
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(UbjError::IllegalKeyType("u64"))
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(UbjError::IllegalKeyType("f32"))
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(UbjError::IllegalKeyType("f64"))
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(UbjError::IllegalKeyType("char"))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(UbjError::IllegalKeyType("&[u8]"))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(UbjError::IllegalKeyType("None"))
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(UbjError::IllegalKeyType("()"))
    }

    // ---------------------------------------------------------------------------------
    // C O M P O U N D   values
    // ---------------------------------------------------------------------------------

    type SerializeSeq = serde::ser::Impossible<(), UbjError>;
    type SerializeTuple = serde::ser::Impossible<(), UbjError>;
    type SerializeTupleStruct = serde::ser::Impossible<(), UbjError>;
    type SerializeStruct = serde::ser::Impossible<(), UbjError>;
    type SerializeTupleVariant = serde::ser::Impossible<(), UbjError>;
    type SerializeStructVariant = serde::ser::Impossible<(), UbjError>;
    type SerializeMap = serde::ser::Impossible<(), UbjError>;

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(UbjError::IllegalKeyType("seq"))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(UbjError::IllegalKeyType("tuple"))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(UbjError::IllegalKeyType("struct"))
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        Err(UbjError::IllegalKeyType("struct"))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(UbjError::IllegalKeyType("struct"))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(UbjError::IllegalKeyType("struct"))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant_name: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(UbjError::IllegalKeyType("enum"))
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        Err(UbjError::IllegalKeyType("enum"))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(UbjError::IllegalKeyType("enum"))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(UbjError::IllegalKeyType("enum"))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(UbjError::IllegalKeyType("map"))
    }

    #[cfg(not(feature = "std"))]
    fn collect_str<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + core::fmt::Display,
    {
        todo!("collect_str<T>: embedded-io feature is required for this method")
    }
}

// -------------------------------------------------------------------------------------------------

#[cfg(feature = "std")]
/// Serializes a Rust value of type `T` to a standard IO writer.
///
/// Be aware that, to avoid accidental double buffering, this function does **not** wrap the provided
/// writer into a buffered one. Therefore, the caller should provide such a buffered writer if desired.
///
/// This function also does **not** flush the provided writer. Flushing is a property of the buffer,
/// which belongs to the caller: only the caller knows whether the bytes should be pushed out
/// immediately or accumulated across several calls, and only the caller can decide how to react to a
/// flush failure. When the writer is buffered, such as an [`std::io::BufWriter`], flush it after this
/// function returns. Note that a `BufWriter` does flush when dropped, but any error raised at that
/// point is silently discarded.
///
/// # Examples
/// ```rust
/// use core::error;
/// use std::{fs, io, io::Write};
///
/// fn main() -> Result<(), Box<dyn error::Error>> {
///
///     // Create a serializable value, such as a simple number, or a complex value for which
///     // you have implemented (or derived an implementation for) the serde::Serialize trait.
///     let value = 65000_i32;
///
///     // Create a writer (wrapped into a buffering one)
///     let file = fs::File::create("file.ubj")?;
///     let mut writer = io::BufWriter::new(file);
///
///     // Serialize the value, then flush the buffer owned by the caller
///     serde_ubj::to_writer(&mut writer, &value)?;
///     writer.flush()?;
///     Ok(())
/// }
/// ```
pub fn to_writer<W, T>(write: &mut W, value: &T) -> Result<(), UbjError>
where
    W: std::io::Write,
    T: serde::Serialize
{
    let mut serializer = UbjSerializer::new(write);
    value.serialize(&mut serializer)
}

#[cfg(feature = "alloc")]
/// Serializes a Rust value of type `T` to a heap-allocated vector (in-memory buffer) of bytes.
/// # Examples
/// ```rust, ignore
/// fn main() -> Result<(), serde_ubj::UbjError> {
///     // Create a serializable value, such as a simple number, or a complex value for which
///     // you have implemented (or derived an implementation for) the serde::Serialize trait.
///     let value = 123_i8;
///
///     // Serialize the value to an in-memory vector of bytes
///     let ubj_bytes: Vec<u8> = serde_ubj::to_vec(&value)?;
///     // Do something with the bytes…
///
///     Ok(())
/// }
/// ```
pub fn to_vec<T>(value: &T) -> Result<alloc::vec::Vec<u8>, UbjError>
where
    T: serde::Serialize,
{
    let mut vec = alloc::vec::Vec::new();
    let mut serializer = UbjSerializer::new(&mut vec);
    value.serialize(&mut serializer)?;
    Ok(vec)
}

/// Serializes a Rust value of type `T` into the provided byte slice.
///
/// On success, returns the number of bytes written at the beginning of `buf`. The remaining
/// (unwritten) tail of `buf` is left untouched.
///
/// This is the `no_std`-friendly counterpart of [`to_vec`]: it writes into a caller-provided
/// buffer instead of allocating a new one, and therefore requires no heap allocator.
///
/// If `buf` is too small to hold the serialized value, the underlying sink reports a
/// `WriteZero` error which surfaces as [`UbjError::IO`].
///
/// # Examples
/// ```rust, ignore
/// fn main() -> Result<(), serde_ubj::UbjError> {
///     let value = 123_i8;
///     let mut buf = [0u8; 16];
///
///     let written = serde_ubj::to_slice(&value, &mut buf)?;
///     let ubj_bytes = &buf[..written];
///     // Do something with the bytes…
///
///     Ok(())
/// }
/// ```
pub fn to_slice<T>(value: &T, buf: &mut [u8]) -> Result<usize, UbjError>
where
    T: serde::Serialize,
{
    let original_len = buf.len();

    // Keep a local slice referring to the still-unwritten part of `buf`. The `&mut [u8]` sink
    // implementation advances it as bytes get written, so its final length tells us how many
    // bytes were consumed.
    let mut remaining: &mut [u8] = buf;
    {
        let mut serializer = UbjSerializer::new(&mut remaining);
        value.serialize(&mut serializer)?;
    }

    Ok(original_len - remaining.len())
}
