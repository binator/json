#![doc = include_str!("../readme.md")]
// #![cfg_attr(not(test), no_std)]
#![feature(trait_alias)]
#![feature(test)]
#![warn(missing_docs)]
// #![feature(generic_const_exprs)]
#![deny(clippy::default_numeric_fallback)]

use std::{
  collections::HashMap,
  fmt::Debug,
};

use binator::{
  base::{
    float,
    is,
    one_of,
    tag,
    utf8,
    BaseAtom,
    FloatParse,
    IntRadixAtom,
  },
  utils::{
    Acc,
    Utils,
    UtilsAtom,
  },
  Contexting,
  CoreAtom,
  Parse,
  Parsed,
  Streaming,
};

/// Generic Json Value
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
  /// Null
  Null,
  /// Book
  Bool(bool),
  /// String
  String(String),
  /// Number
  Number(f64),
  /// Array
  Array(Vec<Value>),
  /// Object
  Object(HashMap<String, Value>),
}

/// Meta trait for Json combinator
pub trait JsonParse<Stream, Context> = FloatParse<Stream, Context>
where
  Stream: Streaming + Clone + Eq,
  <Stream as Streaming>::Item: Into<u8> + Clone,
  <Stream as Streaming>::Item: PartialEq<<Stream as Streaming>::Item>,
  <Stream as Streaming>::Span: AsRef<[u8]>,
  Context: Contexting<UtilsAtom<Stream>>,
  Context: Contexting<BaseAtom<u8>>,
  Context: Contexting<IntRadixAtom<u8>>,
  Context: Contexting<CoreAtom<Stream>>,
  u8: Into<<Stream as Streaming>::Item>;

// ws = *(
//   %x20 /              ; Space
//   %x09 /              ; Horizontal tab
//   %x0A /              ; Line feed or New line
//   %x0D )              ; Carriage return
#[cfg_attr(
  feature = "tracing",
  tracing::instrument(level = "trace", skip_all, ret(Display))
)]
fn ws<Stream, Context>(stream: Stream) -> Parsed<(), Stream, Context>
where
  (): JsonParse<Stream, Context>,
{
  one_of(&[b' ', b'\t', b'\n', b'\r'])
    .drop()
    .fold_bounds(.., || (), Acc::acc)
    .parse(stream)
}

// begin-array     = ws %x5B ws  ; [ left square bracket
#[cfg_attr(
  feature = "tracing",
  tracing::instrument(level = "trace", skip_all, ret(Display))
)]
fn begin_array<Stream, Context>(stream: Stream) -> Parsed<(), Stream, Context>
where
  (): JsonParse<Stream, Context>,
{
  (ws, is(b'['), ws).drop().parse(stream)
}

// begin-object    = ws %x7B ws  ; { left curly bracket
#[cfg_attr(
  feature = "tracing",
  tracing::instrument(level = "trace", skip_all, ret(Display))
)]
fn begin_object<Stream, Context>(stream: Stream) -> Parsed<(), Stream, Context>
where
  (): JsonParse<Stream, Context>,
{
  (ws, is(b'{'), ws).drop().parse(stream)
}

// end-array       = ws %x5D ws  ; ] right square bracket
#[cfg_attr(
  feature = "tracing",
  tracing::instrument(level = "trace", skip_all, ret(Display))
)]
fn end_array<Stream, Context>(stream: Stream) -> Parsed<(), Stream, Context>
where
  (): JsonParse<Stream, Context>,
{
  (ws, is(b']'), ws).drop().parse(stream)
}

// end-object      = ws %x7D ws  ; } right curly bracket
#[cfg_attr(
  feature = "tracing",
  tracing::instrument(level = "trace", skip_all, ret(Display))
)]
fn end_object<Stream, Context>(stream: Stream) -> Parsed<(), Stream, Context>
where
  (): JsonParse<Stream, Context>,
{
  (ws, is(b'}'), ws).drop().parse(stream)
}

// name-separator  = ws %x3A ws  ; : colon
#[cfg_attr(
  feature = "tracing",
  tracing::instrument(level = "trace", skip_all, ret(Display))
)]
fn name_separator<Stream, Context>(stream: Stream) -> Parsed<(), Stream, Context>
where
  (): JsonParse<Stream, Context>,
{
  (ws, is(b':'), ws).drop().parse(stream)
}

// value-separator = ws %x2C ws  ; , comma
#[cfg_attr(
  feature = "tracing",
  tracing::instrument(level = "trace", skip_all, ret(Display))
)]
fn value_separator<Stream, Context>(stream: Stream) -> Parsed<(), Stream, Context>
where
  (): JsonParse<Stream, Context>,
{
  (ws, is(b','), ws).drop().parse(stream)
}

/// Parser that parse json
// JSON-text = ws value ws
#[cfg_attr(
  feature = "tracing",
  tracing::instrument(level = "trace", skip_all, ret(Display))
)]
pub fn json_text<Stream, Context>(stream: Stream) -> Parsed<Value, Stream, Context>
where
  (): JsonParse<Stream, Context>,
{
  (ws, value, ws).map(|(_, value, _)| value).parse(stream)
}

// value = false / null / true / object / array / number / string
// false = %x66.61.6c.73.65   ; false
// null  = %x6e.75.6c.6c      ; null
// true  = %x74.72.75.65      ; true
#[cfg_attr(
  feature = "tracing",
  tracing::instrument(level = "trace", skip_all, ret(Display))
)]
fn value<Stream, Context>(stream: Stream) -> Parsed<Value, Stream, Context>
where
  (): JsonParse<Stream, Context>,
{
  string
    .map(|string| Value::String(string))
    .or(float.map(|float| Value::Number(float)))
    .or(tag("true").to(Value::Bool(true)))
    .or(tag("false").to(Value::Bool(false)))
    .or(object.map(|object| Value::Object(object)))
    .or(array.map(|array| Value::Array(array)))
    .or(tag("null").to(Value::Null))
    .parse(stream)
}

// string = quotation-mark *char quotation-mark
#[cfg_attr(
  feature = "tracing",
  tracing::instrument(level = "trace", skip_all, ret(Display))
)]
fn string<Stream, Context>(stream: Stream) -> Parsed<String, Stream, Context>
where
  (): JsonParse<Stream, Context>,
{
  (
    is(b'"'),
    json_char.fold_bounds(.., || String::new(), Acc::acc),
    is(b'"'),
  )
    .map(|(_, string, _)| string)
    .parse(stream)
}

//       char = unescaped /
//           escape (
//               %x22 /          ; "    quotation mark  U+0022
//               %x5C /          ; \    reverse solidus U+005C
//               %x2F /          ; /    solidus         U+002F
//               %x62 /          ; b    backspace       U+0008
//               %x66 /          ; f    form feed       U+000C
//               %x6E /          ; n    line feed       U+000A
//               %x72 /          ; r    carriage return U+000D
//               %x74 /          ; t    tab             U+0009
//               %x75 4HEXDIG )  ; uXXXX                U+XXXX
//       escape = %x5C              ; \
//       quotation-mark = %x22      ; "
//       unescaped = %x20-21 / %x23-5B / %x5D-10FFFF
#[cfg_attr(
  feature = "tracing",
  tracing::instrument(level = "trace", skip_all, ret(Display))
)]
fn json_char<Stream, Context>(stream: Stream) -> Parsed<char, Stream, Context>
where
  (): JsonParse<Stream, Context>,
{
  utf8
    .filter(|&c| c != '"' && c != '\\')
    .or(is(b'\\').drop_and([
      is(b'"').to('"'),
      is(b'\\').to('\\'),
      is(b'/').to('/'),
      is(b'b').to('\x08'),
      is(b'f').to('\x0C'),
      is(b'n').to('\n'),
      is(b'r').to('\r'),
      is(b't').to('\t'),
      //      is(b'u').and_then(|_| ) decode_utf16 ?
    ]))
    .parse(stream)

  // let Success { token: c, stream } = utf8.parse(stream)?;

  // //  let c = c as char;

  // match c {
  //   '"' => Parsed::Failure(Context::new(UtilsAtom::Filter)),
  //   '\\' => {
  //     let Success { token: c, stream } = utf8.parse(stream)?;
  //     match c {
  //       c => Parsed::Success { token: c, stream },
  //     }
  //   }
  //   c => Parsed::Success { token: c, stream },
  // }
}

// array = begin-array [ value *( value-separator value ) ] end-array
#[cfg_attr(
  feature = "tracing",
  tracing::instrument(level = "trace", skip_all, ret(Display))
)]
fn array<Stream, Context>(stream: Stream) -> Parsed<Vec<Value>, Stream, Context>
where
  (): JsonParse<Stream, Context>,
{
  (
    begin_array,
    value_separator
      .opt()
      .drop_and(value)
      .fold_bounds(.., Vec::new, Acc::acc),
    end_array,
  )
    .map(|(_, array, _)| array)
    .parse(stream)
}

// object = begin-object [ member *( value-separator member ) ] end-object
// member = string name-separator value
#[cfg_attr(
  feature = "tracing",
  tracing::instrument(level = "trace", skip_all, ret(Display))
)]
fn object<Stream, Context>(stream: Stream) -> Parsed<HashMap<String, Value>, Stream, Context>
where
  (): JsonParse<Stream, Context>,
{
  let member = (string, name_separator, value).map(|(string, _, value)| (string, value));
  (
    begin_object,
    value_separator
      .opt()
      .drop_and(member)
      .fold_bounds(.., || HashMap::new(), Acc::acc),
    end_object,
  )
    .map(|(_, object, _)| object)
    .parse(stream)
}

#[cfg(test)]
mod tests {
  extern crate test;

  use binator::context::Ignore;
  use test::{
    black_box,
    Bencher,
  };

  use super::*;

  static SAMPLE: &'static [u8] = include_bytes!("sample.json");

  #[bench]
  fn bench_json(b: &mut Bencher) {
    b.iter(|| black_box(json_text::<_, Ignore>.parse(SAMPLE).unwrap()));
  }

  #[test_log::test]
  fn test_json() {
    json_text::<_, Ignore>.parse(SAMPLE).unwrap();
    //    assert!(false);
  }
}
