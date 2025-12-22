// See http://dmitry-ra.github.io/ubjson-test-suite/json-converter.html
use serde_ubj::*;

#[test]
fn serialize_bool() {
    assert_ok(true , &[0x54]);
    assert_ok(false, &[0x46]);
}

#[test]
fn serialize_i8() {
    // MIN                            MAX
    // -128             0             127
    // |----------------|---------------|
    //
    assert_ok(-123_i8, &[0x69, 0x85]);
    assert_ok(123_i8 , &[0x69, 0x7B]);
}

#[test]
fn serialize_u8() {
    // MIN                            MAX
    // 0                127           255
    // |----------------|--------------|
    //
    assert_ok(123_u8    , &[ 0x55, 0x7B ]);
    assert_ok(254_u8    , &[ 0x55, 0xFE ]);
}

#[test]
fn serialize_i16() {
    // MIN                                                                  MAX
    // -32768                  -128   0   127  255                        32767
    // |---------- . ------------|====|~~~~|~~~~|----------- . ------------|
    //
    assert_ok(-32700_i16, &[ 0x49, 0x80, 0x44 ]);
    assert_ok(-123_i16  , &[ 0x69, 0x85       ]); // ==> int8
    assert_ok(123_i16   , &[ 0x55, 0x7B       ]); // ~~> uint8
    assert_ok(254_i16   , &[ 0x55, 0xFE       ]); // ~~> uint8
    assert_ok(32700_i16 , &[ 0x49, 0x7F, 0xBC ]);
}


#[test]
fn serialize_u16() {
    // MIN                                                                                  MAX
    // 0   127  255                                  32767                                 65535
    // |~~~~|~~~~|***************** . ****************|----------------- . -----------------|
    //
    assert_ok(123_u16   , &[ 0x55, 0x7B                   ]); // ~~> uint8
    assert_ok(254_u16   , &[ 0x55, 0xFE                   ]); // ~~> uint8
    assert_ok(32700_u16 , &[ 0x49, 0x7F, 0xBC             ]); // **> int16
    assert_ok(65000_u16 , &[ 0x6C, 0x00, 0x00, 0xFD, 0xE8 ]); // ``> int32
}


#[test]
fn serialize_i32() {
    // MIN                                                                                                                      MAX
    // -2147483648             -65538        -32768        -128   0   127   255         32767         65535                2147483647
    // |-------------------------|----- . -----|***** . *****|====|~~~~|~~~~|***** . *****|----- .-----|-------------------------|
    //
    assert_ok(-1247483648_i32     , &[ 0x6C, 0xB5, 0xA4, 0xE9, 0x00 ]);
    assert_ok(-65000_i32          , &[ 0x6C, 0xFF, 0xFF, 0x02, 0x18 ]);
    assert_ok(-32700_i32          , &[ 0x49, 0x80, 0x44             ]); // **> int16
    assert_ok(-123_i32            , &[ 0x69, 0x85                   ]); // ==> int8
    assert_ok(123_i32             , &[ 0x55, 0x7B                   ]); // ~~> uint8
    assert_ok(254_i32             , &[ 0x55, 0xFE                   ]); // ~~> uint8
    assert_ok(32700_i32           , &[ 0x49, 0x7F, 0xBC             ]); // **> int16
    assert_ok(65000_i32           , &[ 0x6C, 0x00, 0x00, 0xFD, 0xE8 ]);
    assert_ok(1247483648_i32      , &[ 0x6C, 0x4A, 0x5B, 0x17, 0x00 ]);
}

#[test]
fn serialize_u32() {
    assert_ok(123_u32             , &[ 0x55, 0x7B                                           ]); // ~~> uint8
    assert_ok(254_u32             , &[ 0x55, 0xFE                                           ]); // ~~> uint8
    assert_ok(32700_u32           , &[ 0x49, 0x7F, 0xBC                                     ]); // **> int16
    assert_ok(65000_u32           , &[ 0x6C, 0x00, 0x00, 0xFD, 0xE8                         ]); // ``> int32
    assert_ok(1247483648_u32      , &[ 0x6C, 0x4A, 0x5B, 0x17, 0x00                         ]); // ``> int32
    assert_ok(4294967290_u32      , &[ 0x4C, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFA ]); //     int64
}

#[test]
fn serialize_i64() {
    assert_ok(-922337203685477_i64, &[ 0x4C, 0xFF, 0xFC, 0xB9, 0x23, 0xA2, 0x9C, 0x77, 0x9B ]);
    assert_ok(-4294967290_i64     , &[ 0x4C, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x06 ]);
    assert_ok(-1247483648_i64     , &[ 0x6C, 0xB5, 0xA4, 0xE9, 0x00                         ]); // ``> int32
    assert_ok(-65000_i64          , &[ 0x6C, 0xFF, 0xFF, 0x02, 0x18                         ]); // ``> int32
    assert_ok(-32700_i64          , &[ 0x49, 0x80, 0x44                                     ]); // **> int16
    assert_ok(-123_i64            , &[ 0x69, 0x85                                           ]); // ==> int8
    assert_ok(123_i64             , &[ 0x55, 0x7B                                           ]); // ~~> uint8
    assert_ok(254_i64             , &[ 0x55, 0xFE                                           ]); // ~~> uint8
    assert_ok(32700_i64           , &[ 0x49, 0x7F, 0xBC                                     ]); // **> int16
    assert_ok(65000_i64           , &[ 0x6C, 0x00, 0x00, 0xFD, 0xE8                         ]); // ``> int32
    assert_ok(1247483648_i64      , &[ 0x6C, 0x4A, 0x5B, 0x17, 0x00                         ]); // ``> int32
    assert_ok(4294967290_i64      , &[ 0x4C, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFA ]);
    assert_ok(922337203685477_i64 , &[ 0x4C, 0x00, 0x03, 0x46, 0xDC, 0x5D, 0x63, 0x88, 0x65 ]);
}

#[test]
#[ignore]
fn serialize_huge() {
    todo!()
}

#[test]
fn serialize_u64() {
    assert_ok(922337203685477_u64 , &[0x4C, 0x00, 0x03, 0x46, 0xDC, 0x5D, 0x63, 0x88, 0x65]);
    assert_err(u64::MAX, serde_ubj::UbjError::UnsupportedType("u64"));
}

#[test]
fn serialize_float_f32() {
    assert_ok(0.15625_f32, &[0x64, 0x3E, 0x20, 0x00, 0x00]);
}

#[test]
fn serialize_float_f64() {
    assert_ok(
        16777216.125_f64,
        &[0x44, 0x41, 0x70, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00],
    );
}

// TODO Test when serializing a char that happend to be out of the 0..127 range
#[test]
fn serialize_char_ok() {
    assert_ok('H', &[0x43, 0x48]);
}

#[test]
fn serialize_char_out_of_range() {
    assert_err('ü', UbjError::CharOutOfRange('ü'));
}


#[test]
fn serialize_str_45_long() {
    let text = generate_multilingual_text(45);
    let expected = build_expected_bytes(text.as_bytes());
    assert_eq!(expected[1], 0x55);
    assert_ok(text.as_str(), expected.as_slice());
}

#[test]
fn serialize_str_230_long() {
    let text = generate_multilingual_text(230);
    let expected = build_expected_bytes(text.as_bytes());
    assert_eq!(expected[1], 0x55); // u8
    assert_ok(text.as_str(), expected.as_slice());
}

#[test]
fn serialize_str_15300_long() {
    let s = generate_multilingual_text(15300);
    let expected = build_expected_bytes(s.as_bytes());
    assert_eq!(expected[1], 0x49); // i16
    assert_ok(s.as_str(), expected.as_slice());
}

#[test]
fn serialize_str_7483648_long() {
    let text = generate_multilingual_text(7483648);
    let expected = build_expected_bytes(text.as_bytes());
    assert_eq!(expected[1], 0x6C); // i32
    assert_ok(text.as_str(), expected.as_slice());
}

#[test]
#[ignore]
// WARNING this test requires 2.15 GB of RAM (too expensive for CI)
fn serialize_str_2147483648_long() {
    let text = generate_multilingual_text(2147483648);
    let expected = build_expected_bytes(text.as_bytes());
    assert_eq!(expected[1], 0x4C); // i64
    assert_ok(text.as_str(), expected.as_slice());
}

#[test]
fn serialize_unit() {
    assert_ok((), &[0x5A]);
}

#[test]
fn serialize_none() {
    assert_ok(None as Option<bool>, &[0x5A]);
}

#[test]
#[ignore]
fn serialize_bytes() {
    todo!()
}

// TODO Add more test cases for most of the std (standard) types, such as std::time::Instant and similar


// ---------------------------------------------------------------------------------

#[test]
fn serialize_some() {
    assert_ok(Some(true), &[0x54]);
}

#[derive(serde::Serialize)]
struct UnitStruct;

#[test]
fn serialize_unit_struct() {
    assert_ok(UnitStruct {}, &[0x7B, 0x7D]);
}

#[derive(serde::Serialize)]
enum Enum1 {
    Variant1,
    _Variant2,
}

#[test]
fn serialize_unit_variant() {
    assert_ok(
        Enum1::Variant1,
        &[
            0x53, 0x55, 0x08, 0x56, 0x61, 0x72, 0x69, 0x61, 0x6E, 0x74, 0x31,
        ],
    );
}

#[derive(serde::Serialize)]
struct NewtypeStruct(i8);

#[test]
fn serialize_newtype_struct() {
    assert_ok(NewtypeStruct(123_i8), &[0x69, 0x7B]);
}

#[derive(serde::Serialize)]
enum Enum2 {
    Variant1(&'static str),
    _Variant2(f32),
}
#[test]
fn serialize_newtype_variant() {
    assert_ok(
        Enum2::Variant1("value1"),
        &[
            0x7B, 0x55, 0x08, 0x56, 0x61, 0x72, 0x69, 0x61, 0x6E, 0x74, 0x31, 0x53, 0x55, 0x06,
            0x76, 0x61, 0x6C, 0x75, 0x65, 0x31, 0x7D,
        ],
    );
}

#[test]
fn serialize_sequence() {
    assert_ok(vec![12_i8, 64_i8], &[0x5B, 0x69, 0x0C, 0x69, 0x40, 0x5D]);
}

#[test]
fn serialize_tuple() {
    assert_ok([12_i8, 64_i8], &[0x5B, 0x69, 0x0C, 0x69, 0x40, 0x5D]);
}

#[derive(serde::Serialize)]
struct TupleStruct(i8, i16, i32);

#[test]
fn serialize_tuple_struct() {
    assert_ok(
        TupleStruct(123_i8, 32700_i16, 1247483648_i32),
        &[
            0x5B, 0x69, 0x7B, 0x49, 0x7F, 0xBC, 0x6C, 0x4A, 0x5B, 0x17, 0x00, 0x5D,
        ],
    );
}

#[derive(serde::Serialize)]
enum Enum3 {
    Variant1(i8, i16, i32),
    _Variant2(String),
}

#[test]
fn serialize_tuple_variant() {
    assert_ok(
        Enum3::Variant1(123_i8, 32700_i16, 1247483648_i32),
        &[
            0x5B, 0x69, 0x7B, 0x49, 0x7F, 0xBC, 0x6C, 0x4A, 0x5B, 0x17, 0x00, 0x5D,
        ],
    )
}

#[derive(serde::Serialize)]
enum Enum4 {
    Variant1 { x: i8, y: bool },
    _Variant2 { z: String },
}

#[test]
fn serialize_struct_variant() {
    assert_ok(
        Enum4::Variant1 {
            x: 123_i8,
            y: false,
        },
        &[
            0x7B, 0x55, 0x08, 0x56, 0x61, 0x72, 0x69, 0x61, 0x6E, 0x74, 0x31, 0x7B, 0x55, 0x01,
            0x78, 0x69, 0x7B, 0x55, 0x01, 0x79, 0x46, 0x7D, 0x7D,
        ],
    );
}

#[derive(serde::Serialize)]
struct Struct {
    x: i8,
    y: bool,
}

#[test]
fn serialize_struct() {
    assert_ok(
        Struct {
            x: 123_i8,
            y: false,
        },
        &[
            0x7B, 0x55, 0x01, 0x78, 0x69, 0x7B, 0x55, 0x01, 0x79, 0x46, 0x7D,
        ],
    );
}


#[test]
#[ignore]
fn serialize_map() {
    todo!();
    // use std::collections::HashMap;
    // let mut map = HashMap::new();
    // map.insert("key1", 123_i8);
    // map.insert("key2", 45_i8);
    // map.insert("key3", 67_i8);
    //
    // assert_ok(
    //     map,
    //     &[
    //         0x7B, // {
    //         0x69, 0x04, 0x6B, 0x65, 0x79, 0x31, 0x69, 0x7B, // "key1": 123
    //         0x69, 0x04, 0x6B, 0x65, 0x79, 0x32, 0x69, 0x2D, // "key2": 45
    //         0x69, 0x04, 0x6B, 0x65, 0x79, 0x33, 0x69, 0x43, // "key3": 67
    //         0x7D, // }
    //     ],
    // );
}


// ================================================================================================
//  TEST  HELPERS

/// Generates a multilingual text string that, when UTF-8 encoded, has exactly `byte_count` bytes.
/// Uses a mix of ASCII (1 byte), Latin Extended (2 bytes), CJK (3 bytes), and Emoji (4 bytes).
fn generate_multilingual_text(byte_count: usize) -> String {
    let mut result = String::new();
    let mut bytes_added = 0;

    // Character sets with their UTF-8 byte sizes
    let chars_1byte = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j']; // ASCII
    let chars_2byte = ['á', 'é', 'ñ', 'ü', 'ø', 'ć', 'ž', 'ł', 'ş', 'ğ']; // Latin Extended
    let chars_3byte = ['中', '日', '한', '語', '文', '字', '本', '国', '語', '言']; // CJK
    let chars_4byte = ['😀', '🌍', '🚀', '🎨', '🔥', '💻', '🎵', '🌟', '⚡', '🎯']; // Emoji

    let mut idx = 0;
    while bytes_added < byte_count {
        let remaining = byte_count - bytes_added;

        // Choose character based on remaining bytes and cycling pattern
        let ch = match (remaining, idx % 4) {
            (1, _) => chars_1byte[idx % chars_1byte.len()],
            (2, _) => chars_2byte[idx % chars_2byte.len()],
            (3, _) => chars_3byte[idx % chars_3byte.len()],
            (_, 0) if remaining >= 4 => chars_4byte[idx % chars_4byte.len()],
            (_, 1) if remaining >= 3 => chars_3byte[idx % chars_3byte.len()],
            (_, 2) if remaining >= 2 => chars_2byte[idx % chars_2byte.len()],
            _ => chars_1byte[idx % chars_1byte.len()],
        };

        result.push(ch);
        bytes_added += ch.len_utf8();
        idx += 1;
    }

    result
}

fn build_expected_bytes(bytes: &[u8]) -> Vec<u8> {
    let len = bytes.len();
    let mut expected = Vec::with_capacity(1 + 5 + len); // worst-case for i32 length
    expected.push(0x53); // 'S'
    if len <= u8::MAX as usize {
        expected.push(0x55); // 'U'
        expected.push(len as u8);
    } else if len <= i16::MAX as usize {
        expected.push(0x49); // 'I'
        expected.push(((len >> 8) & 0xFF) as u8);
        expected.push((len & 0xFF) as u8);
    } else if len <= i32::MAX as usize {
        expected.push(0x6C); // 'l'
        expected.push(((len >> 24) & 0xFF) as u8);
        expected.push(((len >> 16) & 0xFF) as u8);
        expected.push(((len >> 8) & 0xFF) as u8);
        expected.push((len & 0xFF) as u8);
    } else {
        assert!(
            len <= i64::MAX as usize,
            "test string too long for i64 length"
        );
        expected.push(0x4C); // 'L'
        expected.push(((len >> 56) & 0xFF) as u8);
        expected.push(((len >> 48) & 0xFF) as u8);
        expected.push(((len >> 40) & 0xFF) as u8);
        expected.push(((len >> 32) & 0xFF) as u8);
        expected.push(((len >> 24) & 0xFF) as u8);
        expected.push(((len >> 16) & 0xFF) as u8);
        expected.push(((len >> 8) & 0xFF) as u8);
        expected.push((len & 0xFF) as u8);
    }
    expected.extend_from_slice(bytes);
    expected
}

fn assert_ok<T>(value: T, expected: &[u8]) where T: serde::Serialize
{
    let mut buffer = Vec::new();
    let result = serde_ubj::to_writer(&mut buffer, &value);
    assert!(result.is_ok());
    assert_eq!(buffer.as_slice(), expected);
}

fn assert_err<T>(value: T, err: serde_ubj::UbjError) where T: serde::Serialize {
    let mut buffer = Vec::new();
    let result = serde_ubj::to_writer(&mut buffer, &value);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), err.to_string());
}

