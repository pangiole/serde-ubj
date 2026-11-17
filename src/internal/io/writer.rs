use crate::internal::error::UbjError;
use crate::internal::io::marker::UbjMarker;
use crate::internal::result::UbjResult;
use embedded_io::{Write as Sink};


/// UBJ-aware writer wrapping any [`embedded_io::Write`] sink.
pub struct UbjWriter<S>
where
    S: Sink
{
    // The embedded sink this writer writes to
    sink: S
}

// ---------------------------------------------------------------------------------
//  F A C T O R I E S
// ---------------------------------------------------------------------------------

#[cfg(feature = "std")]
impl<W> UbjWriter<embedded_io_adapters::std::FromStd<W>>
where
    W: std::io::Write
{
    /// Construct a new UBJ writer over the given `std::io::Write` writer,
    /// by internally adapting it through the `embedded-io-adapters` `FromStd` wrapper.
    pub fn new(writer: W) -> Self {
        Self {
            sink: embedded_io_adapters::std::FromStd::new(writer)
        }
    }
}

#[cfg(not(feature = "std"))]
impl<S> UbjWriter<S>
where
    S: Sink
{
    /// Construct a new UBJ writer over the given [`embedded_io::Write`] sink.
    pub fn new(sink: S) -> Self {
        Self { sink }
    }
}


impl<S> UbjWriter<S>
where S: Sink
{
    // ---------------------------------------------------------------------------------
    //  B A S I C   B U I L D I N G   B L O C K S
    // ---------------------------------------------------------------------------------

    fn write_all(&mut self, buf: &[u8]) -> UbjResult<()> {
        use embedded_io::Error;
        self.sink
            .write_all(buf)
            .map_err(|e| e.kind().into())
    }

    fn write_marker(&mut self, marker: UbjMarker) -> UbjResult<()> {
        self.write_all(&[u8::from(marker)])
    }

    fn write_payload(&mut self, payload: &[u8]) -> UbjResult<()> {
        self.write_all(payload)
    }

    fn write_sized_packet<const N: usize>(&mut self, marker: UbjMarker, payload: [u8; N]) -> UbjResult<()> {
        self.write_marker(marker)?;
        self.write_payload(&payload)
    }


    // ---------------------------------------------------------------------------------
    //  W R I T I N G    S C A L A R    V A L U E s
    // ---------------------------------------------------------------------------------

    pub fn write_null(&mut self) -> UbjResult<()> {
        self.write_marker(UbjMarker::Null)
    }

    pub fn write_bool(&mut self, v: bool) -> UbjResult<()> {
        self.write_marker(if v { UbjMarker::True } else { UbjMarker::False })
    }

    pub fn write_int8(&mut self, v: i8) -> UbjResult<()> {
        self.write_sized_packet(UbjMarker::Int8, v.to_be_bytes())
    }

    pub fn write_uint8(&mut self, v: u8) -> UbjResult<()> {
        // Note that there's no benefit in attempting a conversion to int8
        // as we would end up writing 1 single byte anyway
        self.write_sized_packet(UbjMarker::Uint8, v.to_be_bytes())
    }

    pub fn write_int16(&mut self, v: i16) -> UbjResult<()> {
        if v >= i8::MIN as i16 && v < 0 {
            self.write_int8(v as i8)
        } else if v >= 0 && v <= u8::MAX as i16 {
            self.write_uint8(v as u8)
        } else {
            self.write_sized_packet(UbjMarker::Int16, v.to_be_bytes())
        }
    }

    pub fn write_int32(&mut self, v: i32) -> UbjResult<()> {
        if v >= i16::MIN as i32 && v <= i16::MAX as i32 {
            self.write_int16(v as i16)
        } else {
            self.write_sized_packet(UbjMarker::Int32, v.to_be_bytes())
        }
    }

    pub fn write_int64(&mut self, v: i64) -> UbjResult<()> {
        if v >= i32::MIN as i64 && v <= i32::MAX as i64 {
            self.write_int32(v as i32)
        } else {
            self.write_sized_packet(UbjMarker::Int64, v.to_be_bytes())
        }
    }

    // TODO pub fn write_int128(&mut self, v: i128) -> UbjResult<()>

    pub fn write_float32(&mut self, v: f32) -> UbjResult<()> {
        self.write_sized_packet(UbjMarker::Float32, v.to_be_bytes())
    }

    pub fn write_float64(&mut self, v: f64) -> UbjResult<()> {
        self.write_sized_packet(UbjMarker::Float64, v.to_be_bytes())
    }

    pub fn write_char(&mut self, v: char) -> UbjResult<()> {
        // In Rust, a 'char' type represents a Unicode Scalar Value (exactly 4 bytes),
        // while UTF-8 is a variable-width encoding (from 1 to a maximum of 4 bytes).
        // Our goal is to get the code point (integer value) representing the
        // character's position in the Unicode standard table (which extends the ASCII
        // standard table beyond the first 0-127 range).
        let utf8_code = u32::from(v);

        // This conversion has not to be confused with turning the given character into
        // the corresponding UTF-8 encoded, variable width, sequence of bytes which,
        // instead, we would get as follows:
        //
        //      let utf8_bytes = v.encode_utf8(&mut [0; 4]);
        //
        // Now, according to the official specification, the char type in UBJ is an
        // unsigned integer byte meant to represent a single ASCII character whose position
        // lays within the 0..127 range of the Unicode standard table.
        //
        // See https://github.com/ubjson/universal-binary-json/issues/56
        if utf8_code > 127 {
            Err(UbjError::NotAsciiChar(utf8_code))
        } else {
            self.write_sized_packet(UbjMarker::Char, [utf8_code as u8])
        }
    }


    pub fn write_marked_string(&mut self, v: &str) -> UbjResult<()> {
        self.write_marker(UbjMarker::String)?;
        self.write_unmarked_string(v)
    }

    pub fn write_unmarked_string(&mut self, v: &str) -> UbjResult<()> {
        let payload = v.as_bytes();
        let len = payload.len();
        if len <= (i64::MAX as usize) {
            self.write_int64(len as i64)?;
            self.write_payload(payload)
        } else {
            Err(UbjError::Unsupported(
                "Rust String values with length greater than i64::MAX",
            ))
        }
    }


    // ---------------------------------------------------------------------------------
    //  W R I T I N G    C O N T A I N E R     M A R K E R s
    // ---------------------------------------------------------------------------------

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
}

#[cfg(test)]
mod tests {
    use super::UbjWriter;
    use crate::internal::error::UbjError;
    use crate::internal::result::UbjResult;
    use embedded_io::ErrorKind;

    // ---------------------------------------------------------------------------------
    //  T E S T   H E L P E R S
    // ---------------------------------------------------------------------------------

    /// Runs the closure body against a `UbjWriter` backed by a fixed 64-byte buffer and
    /// asserts the exact sequence of bytes that was produced. Being a macro, the body is
    /// expanded in place, so a failing assertion points at the call site.
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

    /// Runs `write` against a `UbjWriter` backed by a buffer that is too small for the
    /// payload and asserts that the sink failure is surfaced as an I/O error.
    fn assert_io_error<const N: usize>(
        mut buf: [u8; N],
        write: impl FnOnce(&mut UbjWriter<&mut [u8]>) -> UbjResult<()>,
    ) {
        let mut writer = UbjWriter { sink: &mut buf[..] };
        let err = write(&mut writer).expect_err("the write operation should fail");
        assert!(
            matches!(err, UbjError::IO(ErrorKind::WriteZero)),
            "expected UbjError::IO(ErrorKind::WriteZero), got {err:?}"
        );
    }

    // ---------------------------------------------------------------------------------
    //  S C A L A R S
    // ---------------------------------------------------------------------------------

    #[test]
    fn should_write_null_emits_the_null_marker() {
        assert_encodes!(|w| w.write_null(), &[0x5A]);
    }

    #[test]
    fn should_write_bool_emits_true_and_false_markers() {
        assert_encodes!(|w| w.write_bool(true), &[0x54]);
        assert_encodes!(|w| w.write_bool(false), &[0x46]);
    }

    #[test]
    fn should_write_int8_always_uses_the_int8_marker() {
        assert_encodes!(|w| w.write_int8(i8::MIN), &[0x69, 0x80]);
        assert_encodes!(|w| w.write_int8(0), &[0x69, 0x00]);
        assert_encodes!(|w| w.write_int8(i8::MAX), &[0x69, 0x7F]);
        assert_encodes!(|w| w.write_int8(-123), &[0x69, 0x85]);
    }

    #[test]
    fn should_write_uint8_always_uses_the_uint8_marker() {
        assert_encodes!(|w| w.write_uint8(0), &[0x55, 0x00]);
        assert_encodes!(|w| w.write_uint8(127), &[0x55, 0x7F]);
        assert_encodes!(|w| w.write_uint8(254), &[0x55, 0xFE]);
        assert_encodes!(|w| w.write_uint8(u8::MAX), &[0x55, 0xFF]);
    }

    #[test]
    fn should_write_int16_narrows_to_int8_when_negative_and_fitting() {
        assert_encodes!(|w| w.write_int16(i8::MIN as i16), &[0x69, 0x80]);
        assert_encodes!(|w| w.write_int16(-123), &[0x69, 0x85]);
        assert_encodes!(|w| w.write_int16(-1), &[0x69, 0xFF]);
    }

    #[test]
    fn should_write_int16_narrows_to_uint8_when_non_negative_and_fitting() {
        assert_encodes!(|w| w.write_int16(0), &[0x55, 0x00]);
        assert_encodes!(|w| w.write_int16(123), &[0x55, 0x7B]);
        assert_encodes!(|w| w.write_int16(u8::MAX as i16), &[0x55, 0xFF]);
    }

    #[test]
    fn should_write_int16_uses_int16_outside_the_narrow_ranges() {
        // -129 is one below i8::MIN, 256 is one above u8::MAX.
        assert_encodes!(|w| w.write_int16(-129), &[0x49, 0xFF, 0x7F]);
        assert_encodes!(|w| w.write_int16(256), &[0x49, 0x01, 0x00]);
        assert_encodes!(|w| w.write_int16(i16::MIN), &[0x49, 0x80, 0x00]);
        assert_encodes!(|w| w.write_int16(i16::MAX), &[0x49, 0x7F, 0xFF]);
    }

    #[test]
    fn should_write_int32_narrows_when_fitting_in_i16() {
        // Delegates to write_int16, which narrows even further when possible.
        assert_encodes!(|w| w.write_int32(123), &[0x55, 0x7B]);
        assert_encodes!(|w| w.write_int32(-32700), &[0x49, 0x80, 0x44]);
        assert_encodes!(|w| w.write_int32(32700), &[0x49, 0x7F, 0xBC]);
        assert_encodes!(|w| w.write_int32(i16::MIN as i32), &[0x49, 0x80, 0x00]);
        assert_encodes!(|w| w.write_int32(i16::MAX as i32), &[0x49, 0x7F, 0xFF]);
    }

    #[test]
    fn should_write_int32_uses_int32_outside_the_i16_range() {
        // -32769 and 32768 sit just outside the i16 range.
        assert_encodes!(|w| w.write_int32(-32769), &[0x6C, 0xFF, 0xFF, 0x7F, 0xFF]);
        assert_encodes!(|w| w.write_int32(32768), &[0x6C, 0x00, 0x00, 0x80, 0x00]);
        assert_encodes!(|w| w.write_int32(1_247_483_648), &[0x6C, 0x4A, 0x5B, 0x17, 0x00]);
        assert_encodes!(|w| w.write_int32(i32::MIN), &[0x6C, 0x80, 0x00, 0x00, 0x00]);
        assert_encodes!(|w| w.write_int32(i32::MAX), &[0x6C, 0x7F, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn should_write_int64_narrows_when_fitting_in_i32() {
        // Delegates to write_int32, which narrows even further when possible.
        assert_encodes!(|w| w.write_int64(123), &[0x55, 0x7B]);
        assert_encodes!(|w| w.write_int64(1_247_483_648), &[0x6C, 0x4A, 0x5B, 0x17, 0x00]);
        assert_encodes!(|w| w.write_int64(i32::MIN as i64), &[0x6C, 0x80, 0x00, 0x00, 0x00]);
        assert_encodes!(|w| w.write_int64(i32::MAX as i64), &[0x6C, 0x7F, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn should_write_int64_uses_int64_outside_the_i32_range() {
        assert_encodes!(
            |w| w.write_int64((i32::MIN as i64) - 1),
            &[0x4C, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F, 0xFF, 0xFF, 0xFF],
        );
        assert_encodes!(
            |w| w.write_int64((i32::MAX as i64) + 1),
            &[0x4C, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00],
        );
        assert_encodes!(
            |w| w.write_int64(i64::MIN),
            &[0x4C, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        );
        assert_encodes!(
            |w| w.write_int64(i64::MAX),
            &[0x4C, 0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        );
    }

    #[test]
    fn should_write_float32_emits_the_marker_and_a_big_endian_payload() {
        assert_encodes!(|w| w.write_float32(1.0), &[0x64, 0x3F, 0x80, 0x00, 0x00]);
        assert_encodes!(|w| w.write_float32(-2.5), &[0x64, 0xC0, 0x20, 0x00, 0x00]);
    }

    #[test]
    fn should_write_float64_emits_the_marker_and_a_big_endian_payload() {
        assert_encodes!(
            |w| w.write_float64(1.0),
            &[0x44, 0x3F, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        );
        assert_encodes!(
            |w| w.write_float64(-2.5),
            &[0x44, 0xC0, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        );
    }

    #[test]
    fn should_write_char_emits_the_ascii_code_point() {
        assert_encodes!(|w| w.write_char('A'), &[0x43, 0x41]);
        assert_encodes!(|w| w.write_char('\0'), &[0x43, 0x00]);
        // 0x7F is the largest code point that is still ASCII.
        assert_encodes!(|w| w.write_char('\u{7F}'), &[0x43, 0x7F]);
    }

    #[test]
    fn should_write_char_rejects_non_ascii_code_points() {
        let mut buf = [0u8; 8];
        let mut writer = UbjWriter { sink: &mut buf[..] };
        let err = writer.write_char('é').expect_err("non-ASCII chars must be rejected");
        assert!(matches!(err, UbjError::NotAsciiChar(0xE9)));
    }

    // ---------------------------------------------------------------------------------
    //  S T R I N G S
    // ---------------------------------------------------------------------------------

    #[test]
    fn should_write_marked_string_emits_marker_length_and_payload() {
        assert_encodes!(|w| w.write_marked_string(""), &[0x53, 0x55, 0x00]);
        assert_encodes!(|w| w.write_marked_string("hi"), &[0x53, 0x55, 0x02, 0x68, 0x69]);
    }

    #[test]
    fn should_write_unmarked_string_emits_length_and_payload_only() {
        assert_encodes!(|w| w.write_unmarked_string(""), &[0x55, 0x00]);
        assert_encodes!(|w| w.write_unmarked_string("hi"), &[0x55, 0x02, 0x68, 0x69]);
    }

    #[test]
    fn should_write_unmarked_string_measures_length_in_bytes_not_characters() {
        // "héllo" is 5 characters but 6 UTF-8 bytes (é encodes to 0xC3 0xA9).
        assert_encodes!(
            |w| w.write_unmarked_string("héllo"),
            &[0x55, 0x06, 0x68, 0xC3, 0xA9, 0x6C, 0x6C, 0x6F],
        );
    }

    #[test]
    fn should_write_unmarked_string_narrows_the_length_prefix() {
        // The length is written through write_int64, so it narrows like an integer.
        static A_255: [u8; 255] = [b'a'; 255];
        static A_256: [u8; 256] = [b'a'; 256];

        // 255 bytes still fit a Uint8 length prefix.
        let mut buf = [0u8; 260];
        {
            let mut writer = UbjWriter { sink: &mut buf[..] };
            writer
                .write_unmarked_string(core::str::from_utf8(&A_255).unwrap())
                .unwrap();
        }
        assert_eq!(&buf[..2], &[0x55, 0xFF]);
        assert!(buf[2..257].iter().all(|&b| b == b'a'));

        // 256 bytes no longer fit, so an Int16 length prefix is used.
        let mut buf = [0u8; 260];
        {
            let mut writer = UbjWriter { sink: &mut buf[..] };
            writer
                .write_unmarked_string(core::str::from_utf8(&A_256).unwrap())
                .unwrap();
        }
        assert_eq!(&buf[..3], &[0x49, 0x01, 0x00]);
        assert!(buf[3..259].iter().all(|&b| b == b'a'));
    }

    // ---------------------------------------------------------------------------------
    //  C O N T A I N E R S
    // ---------------------------------------------------------------------------------

    #[test]
    fn should_write_containers_emit_their_markers() {
        assert_encodes!(|w| w.write_start_array(), &[0x5B]);
        assert_encodes!(|w| w.write_end_array(), &[0x5D]);
        assert_encodes!(|w| w.write_start_object(), &[0x7B]);
        assert_encodes!(|w| w.write_end_object(), &[0x7D]);
    }

    #[test]
    fn should_write_a_heterogeneous_array() {
        // UBJ arrays are not homogeneous and carry no element count: each value is
        // written back to back, and the array is closed by the EndArray marker.
        assert_encodes!(
            |w| {
                w.write_start_array()?;
                w.write_null()?;
                w.write_bool(true)?;
                w.write_bool(false)?;
                w.write_int8(-5)?;
                w.write_uint8(200)?;
                w.write_int16(1_000)?;
                w.write_int32(100_000)?;
                w.write_int64(5_000_000_000)?;
                w.write_float32(1.5)?;
                w.write_float64(-2.5)?;
                w.write_char('A')?;
                w.write_marked_string("hi")?;
                w.write_marked_string("héllo")?;
                w.write_end_array()
            },
            &[
                0x5B,                                                  // StartArray
                0x5A,                                                  // null
                0x54,                                                  // true
                0x46,                                                  // false
                0x69, 0xFB,                                            // Int8  -5
                0x55, 0xC8,                                            // Uint8 200
                0x49, 0x03, 0xE8,                                      // Int16 1_000
                0x6C, 0x00, 0x01, 0x86, 0xA0,                          // Int32 100_000
                0x4C, 0x00, 0x00, 0x00, 0x01, 0x2A, 0x05, 0xF2, 0x00,  // Int64 5_000_000_000
                0x64, 0x3F, 0xC0, 0x00, 0x00,                          // Float32 1.5
                0x44, 0xC0, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // Float64 -2.5
                0x43, 0x41,                                            // Char 'A'
                0x53, 0x55, 0x02, 0x68, 0x69,                          // marked string "hi"
                0x53, 0x55, 0x06, 0x68, 0xC3, 0xA9, 0x6C, 0x6C, 0x6F,  // marked string "héllo"
                0x5D,                                                  // EndArray
            ],
        );
    }

    #[test]
    fn should_write_a_heterogeneous_object() {
        // Object keys are unmarked strings (marker-less length + UTF-8 payload),
        // while values keep their own markers, whatever their kind.
        assert_encodes!(
            |w| {
                w.write_start_object()?;
                w.write_unmarked_string("id")?;
                w.write_uint8(7)?;
                w.write_unmarked_string("ok")?;
                w.write_bool(true)?;
                w.write_unmarked_string("name")?;
                w.write_marked_string("hi")?;
                w.write_unmarked_string("tag")?;
                w.write_marked_string("xy")?;
                w.write_end_object()
            },
            &[
                0x7B,                                 // StartObject
                0x55, 0x02, 0x69, 0x64,               // key "id"
                0x55, 0x07,                           // Uint8 7
                0x55, 0x02, 0x6F, 0x6B,               // key "ok"
                0x54,                                 // true
                0x55, 0x04, 0x6E, 0x61, 0x6D, 0x65,   // key "name"
                0x53, 0x55, 0x02, 0x68, 0x69,         // marked string "hi"
                0x55, 0x03, 0x74, 0x61, 0x67,         // key "tag"
                0x53, 0x55, 0x02, 0x78, 0x79,         // marked string "xy"
                0x7D,                                 // EndObject
            ],
        );
    }

    #[test]
    fn should_write_nested_heterogeneous_containers() {
        // Containers nest and mix freely; each one is self-delimited by its own
        // End* marker, so no separator is needed between elements.
        assert_encodes!(
            |w| {
                w.write_start_array()?;
                w.write_start_array()?;
                w.write_uint8(1)?;
                w.write_marked_string("a")?;
                w.write_end_array()?;
                w.write_start_object()?;
                w.write_unmarked_string("k")?;
                w.write_char('z')?;
                w.write_end_object()?;
                w.write_end_array()
            },
            &[
                0x5B,                    // outer StartArray
                0x5B,                    // inner StartArray
                0x55, 0x01,              // Uint8 1
                0x53, 0x55, 0x01, 0x61,  // marked string "a"
                0x5D,                    // inner EndArray
                0x7B,                    // StartObject
                0x55, 0x01, 0x6B,        // key "k"
                0x43, 0x7A,              // Char 'z'
                0x7D,                    // EndObject
                0x5D,                    // outer EndArray
            ],
        );
    }

    #[test]
    fn should_write_consecutive_containers_are_self_delimiting() {
        // Nothing separates one top-level container from the next one.
        assert_encodes!(
            |w| {
                w.write_start_array()?;
                w.write_uint8(1)?;
                w.write_end_array()?;
                w.write_start_object()?;
                w.write_unmarked_string("k")?;
                w.write_bool(true)?;
                w.write_end_object()?;
                w.write_uint8(9)
            },
            &[
                0x5B, 0x55, 0x01, 0x5D,              // array [1]
                0x7B, 0x55, 0x01, 0x6B, 0x54, 0x7D,  // object {"k": true}
                0x55, 0x09,                          // Uint8 9
            ],
        );
    }

    #[test]
    fn should_write_a_sequence_failure_keeps_the_already_written_prefix() {
        // The values written before the failure stay in the sink; the string that
        // does not fit only contributes its String marker.
        let mut buf = [0u8; 6];
        {
            let mut writer = UbjWriter { sink: &mut buf[..] };
            writer.write_start_array().unwrap();
            writer.write_uint8(1).unwrap();
            writer.write_uint8(2).unwrap();
            let err = writer
                .write_marked_string("hello")
                .expect_err("the string payload must not fit");
            assert!(matches!(err, UbjError::IO(ErrorKind::WriteZero)));
        }
        assert_eq!(buf, [0x5B, 0x55, 0x01, 0x55, 0x02, 0x53]);
    }

    // ---------------------------------------------------------------------------------
    //  E R R O R   P R O P A G A T I O N
    // ---------------------------------------------------------------------------------

    #[test]
    fn should_write_propagates_sink_errors_as_io_errors() {
        // Each buffer below is too small to hold the full packet.
        assert_io_error([0u8; 1], |w| w.write_int16(256));
        assert_io_error([0u8; 2], |w| w.write_int32(32768));
        assert_io_error([0u8; 1], |w| w.write_marked_string("hi"));
    }

    #[test]
    fn should_write_leaves_a_prefix_in_the_sink_when_it_runs_out_of_space() {
        let mut buf = [0u8; 1];
        {
            let mut writer = UbjWriter { sink: &mut buf[..] };
            let err = writer.write_int16(256).expect_err("the payload must not fit");
            assert!(matches!(err, UbjError::IO(ErrorKind::WriteZero)));
        }
        // The Int16 marker was already flushed before the payload write failed.
        assert_eq!(buf, [0x49]);
    }
}

