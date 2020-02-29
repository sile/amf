use super::marker;
use super::Value;
use crate::amf3;
use crate::error::DecodeError;
use crate::{DecodeResult, Pair};
use byteorder::{BigEndian, ReadBytesExt};
use std::io;
use std::time;

/// AMF0 decoder.
#[derive(Debug)]
pub struct Decoder<R> {
    inner: R,
    complexes: Vec<Value>,
}
impl<R> Decoder<R> {
    /// Unwraps this `Decoder`, returning the underlying reader.
    pub fn into_inner(self) -> R {
        self.inner
    }
}
impl<R> Decoder<R>
where
    R: io::Read,
{
    /// Makes a new instance.
    pub fn new(inner: R) -> Self {
        Decoder {
            inner,
            complexes: Vec::new(),
        }
    }

    /// Decodes a AMF0 value.
    pub fn decode(&mut self) -> DecodeResult<Value> {
        self.decode_value()
    }

    /// Clear the reference table of this decoder.
    ///
    /// > Note that object reference indices are local to each message body.
    /// > Serializers and deserializers must reset reference indices to 0 each time a new message is processed.
    /// >
    /// > [AMF 0 Specification: 4.1.3 AMF Message](http://download.macromedia.com/pub/labs/amf/amf0_spec_121207.pdf)
    pub fn clear_reference_table(&mut self) {
        self.complexes.clear();
    }

    fn decode_value(&mut self) -> DecodeResult<Value> {
        let marker = self.inner.read_u8()?;
        match marker {
            marker::NUMBER => self.decode_number(),
            marker::BOOLEAN => self.decode_boolean(),
            marker::STRING => self.decode_string(),
            marker::OBJECT => self.decode_object(),
            marker::MOVIECLIP => Err(DecodeError::Unsupported { marker }),
            marker::NULL => Ok(Value::Null),
            marker::UNDEFINED => Ok(Value::Undefined),
            marker::REFERENCE => self.decode_reference(),
            marker::ECMA_ARRAY => self.decode_ecma_array(),
            marker::OBJECT_END_MARKER => Err(DecodeError::UnexpectedObjectEnd),
            marker::STRICT_ARRAY => self.decode_strict_array(),
            marker::DATE => self.decode_date(),
            marker::LONG_STRING => self.decode_long_string(),
            marker::UNSUPPORTED => Err(DecodeError::Unsupported { marker }),
            marker::RECORDSET => Err(DecodeError::Unsupported { marker }),
            marker::XML_DOCUMENT => self.decode_xml_document(),
            marker::TYPED_OBJECT => self.decode_typed_object(),
            marker::AVMPLUS_OBJECT => self.decode_avmplus(),
            _ => Err(DecodeError::Unknown { marker }),
        }
    }
    fn decode_number(&mut self) -> DecodeResult<Value> {
        let n = self.inner.read_f64::<BigEndian>()?;
        Ok(Value::Number(n))
    }
    fn decode_boolean(&mut self) -> DecodeResult<Value> {
        let b = self.inner.read_u8()? != 0;
        Ok(Value::Boolean(b))
    }
    fn decode_string(&mut self) -> DecodeResult<Value> {
        let len = self.inner.read_u16::<BigEndian>()? as usize;
        self.read_utf8(len).map(Value::String)
    }
    fn decode_object(&mut self) -> DecodeResult<Value> {
        self.decode_complex_type(|this| {
            let entries = this.decode_pairs()?;
            Ok(Value::Object {
                class_name: None,
                entries,
            })
        })
    }
    fn decode_reference(&mut self) -> DecodeResult<Value> {
        let index = self.inner.read_u16::<BigEndian>()? as usize;
        self.complexes
            .get(index)
            .ok_or(DecodeError::OutOfRangeReference { index })
            .and_then(|v| {
                if *v == Value::Null {
                    Err(DecodeError::CircularReference { index })
                } else {
                    Ok(v.clone())
                }
            })
    }
    fn decode_ecma_array(&mut self) -> DecodeResult<Value> {
        self.decode_complex_type(|this| {
            let _count = this.inner.read_u32::<BigEndian>()? as usize;
            let entries = this.decode_pairs()?;
            Ok(Value::EcmaArray { entries })
        })
    }
    fn decode_strict_array(&mut self) -> DecodeResult<Value> {
        self.decode_complex_type(|this| {
            let count = this.inner.read_u32::<BigEndian>()? as usize;
            let entries = (0..count)
                .map(|_| this.decode_value())
                .collect::<DecodeResult<_>>()?;
            Ok(Value::Array { entries })
        })
    }
    fn decode_date(&mut self) -> DecodeResult<Value> {
        let millis = self.inner.read_f64::<BigEndian>()?;
        let time_zone = self.inner.read_i16::<BigEndian>()?;
        if time_zone != 0 {
            Err(DecodeError::NonZeroTimeZone { offset: time_zone })
        } else if !(millis.is_finite() && millis.is_sign_positive()) {
            Err(DecodeError::InvalidDate { millis })
        } else {
            Ok(Value::Date {
                unix_time: time::Duration::from_millis(millis as u64),
            })
        }
    }
    fn decode_long_string(&mut self) -> DecodeResult<Value> {
        let len = self.inner.read_u32::<BigEndian>()? as usize;
        self.read_utf8(len).map(Value::String)
    }
    fn decode_xml_document(&mut self) -> DecodeResult<Value> {
        let len = self.inner.read_u32::<BigEndian>()? as usize;
        self.read_utf8(len).map(Value::XmlDocument)
    }
    fn decode_typed_object(&mut self) -> DecodeResult<Value> {
        self.decode_complex_type(|this| {
            let len = this.inner.read_u16::<BigEndian>()? as usize;
            let class_name = this.read_utf8(len)?;
            let entries = this.decode_pairs()?;
            Ok(Value::Object {
                class_name: Some(class_name),
                entries,
            })
        })
    }
    fn decode_avmplus(&mut self) -> DecodeResult<Value> {
        let value = amf3::Decoder::new(&mut self.inner).decode()?;
        Ok(Value::AvmPlus(value))
    }

    fn read_utf8(&mut self, len: usize) -> DecodeResult<String> {
        let mut buf = vec![0; len];
        self.inner.read_exact(&mut buf)?;
        let utf8 = String::from_utf8(buf)?;
        Ok(utf8)
    }
    fn decode_pairs(&mut self) -> DecodeResult<Vec<Pair<String, Value>>> {
        let mut entries = Vec::new();
        loop {
            let len = self.inner.read_u16::<BigEndian>()? as usize;
            let key = self.read_utf8(len)?;
            match self.decode_value() {
                Ok(value) => {
                    entries.push(Pair { key, value });
                }
                Err(DecodeError::UnexpectedObjectEnd) if key.is_empty() => break,
                Err(e) => return Err(e),
            }
        }
        Ok(entries)
    }
    fn decode_complex_type<F>(&mut self, f: F) -> DecodeResult<Value>
    where
        F: FnOnce(&mut Self) -> DecodeResult<Value>,
    {
        let index = self.complexes.len();
        self.complexes.push(Value::Null);
        let value = f(self)?;
        self.complexes[index] = value.clone();
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::approx_constant)]
    use super::super::marker;
    use super::super::Value;
    use crate::amf3;
    use crate::error::DecodeError;
    use crate::Pair;
    use std::f64;
    use std::io;
    use std::iter;
    use std::time;

    macro_rules! decode {
        ($file:expr) => {{
            let input = include_bytes!(concat!("../testdata/", $file));
            Value::read_from(&mut &input[..])
        }};
    }
    macro_rules! decode_eq {
        ($file:expr, $expected: expr) => {{
            let value = decode!($file).unwrap();
            assert_eq!(value, $expected)
        }};
    }
    macro_rules! decode_unexpected_eof {
        ($file:expr) => {{
            let result = decode!($file);
            match result {
                Err(DecodeError::Io(e)) => assert_eq!(e.kind(), io::ErrorKind::UnexpectedEof),
                _ => assert!(false),
            }
        }};
    }

    #[test]
    fn decodes_boolean() {
        decode_eq!("amf0-boolean-true.bin", Value::Boolean(true));
        decode_eq!("amf0-boolean-false.bin", Value::Boolean(false));
        decode_unexpected_eof!("amf0-boolean-partial.bin");
    }
    #[test]
    fn decodes_null() {
        decode_eq!("amf0-null.bin", Value::Null);
    }
    #[test]
    fn decodes_undefined() {
        decode_eq!("amf0-undefined.bin", Value::Undefined);
    }
    #[test]
    fn decodes_number() {
        decode_eq!("amf0-number.bin", Value::Number(3.5));
        decode_eq!(
            "amf0-number-positive-infinity.bin",
            Value::Number(f64::INFINITY)
        );
        decode_eq!(
            "amf0-number-negative-infinity.bin",
            Value::Number(f64::NEG_INFINITY)
        );

        let is_nan = |v| {
            if let Value::Number(n) = v {
                n.is_nan()
            } else {
                false
            }
        };
        assert!(is_nan(decode!("amf0-number-quiet-nan.bin").unwrap()));
        assert!(is_nan(decode!("amf0-number-signaling-nan.bin").unwrap()));

        decode_unexpected_eof!("amf0-number-partial.bin");
    }
    #[test]
    fn decodes_string() {
        decode_eq!(
            "amf0-string.bin",
            Value::String("this is a テスト".to_string())
        );
        decode_eq!(
            "amf0-complex-encoded-string.bin",
            obj(
                None,
                &[
                    ("utf", s("UTF テスト")),
                    ("zed", n(5.0)),
                    ("shift", s("Shift テスト"))
                ][..]
            )
        );
        decode_unexpected_eof!("amf0-string-partial.bin");
    }
    #[test]
    fn decodes_long_string() {
        decode_eq!(
            "amf0-long-string.bin",
            Value::String(iter::repeat('a').take(0x10013).collect())
        );
        decode_unexpected_eof!("amf0-long-string-partial.bin");
    }
    #[test]
    fn decodes_xml_document() {
        decode_eq!(
            "amf0-xml-doc.bin",
            Value::XmlDocument("<parent><child prop=\"test\" /></parent>".to_string())
        );
        decode_unexpected_eof!("amf0-xml-document-partial.bin");
    }
    #[test]
    fn decodes_object() {
        decode_eq!(
            "amf0-object.bin",
            obj(
                None,
                &[("", s("")), ("foo", s("baz")), ("bar", n(3.14))][..]
            )
        );
        decode_eq!(
            "amf0-untyped-object.bin",
            obj(None, &[("foo", s("bar")), ("baz", Value::Null)][..])
        );
        assert_eq!(
            decode!("amf0-bad-object-end.bin"),
            Err(DecodeError::UnexpectedObjectEnd)
        );
        decode_unexpected_eof!("amf0-object-partial.bin");
    }
    #[test]
    fn decodes_typed_object() {
        decode_eq!(
            "amf0-typed-object.bin",
            obj(
                Some("org.amf.ASClass"),
                &[("foo", s("bar")), ("baz", Value::Null)]
            )
        );
        decode_unexpected_eof!("amf0-typed-object-partial.bin");
    }
    #[test]
    fn decodes_unsupported() {
        assert_eq!(
            decode!("amf0-movieclip.bin"),
            Err(DecodeError::Unsupported {
                marker: marker::MOVIECLIP
            })
        );
        assert_eq!(
            decode!("amf0-recordset.bin"),
            Err(DecodeError::Unsupported {
                marker: marker::RECORDSET
            })
        );
        assert_eq!(
            decode!("amf0-unsupported.bin"),
            Err(DecodeError::Unsupported {
                marker: marker::UNSUPPORTED
            })
        );
    }
    #[test]
    fn decodes_ecma_array() {
        let entries = es(&[("0", s("a")), ("1", s("b")), ("2", s("c")), ("3", s("d"))][..]);
        decode_eq!(
            "amf0-ecma-ordinal-array.bin",
            Value::EcmaArray { entries: entries }
        );
        decode_unexpected_eof!("amf0-ecma-array-partial.bin");

        let entries = es(&[("c", s("d")), ("a", s("b"))][..]);
        decode_eq!("amf0-hash.bin", Value::EcmaArray { entries: entries });
    }
    #[test]
    fn decodes_strict_array() {
        decode_eq!(
            "amf0-strict-array.bin",
            Value::Array {
                entries: vec![n(1.0), s("2"), n(3.0)]
            }
        );
        decode_unexpected_eof!("amf0-strict-array-partial.bin");
    }
    #[test]
    fn decodes_reference() {
        let object = obj(None, &[("foo", s("baz")), ("bar", n(3.14))][..]);
        let expected = obj(None, &[("0", object.clone()), ("1", object)][..]);
        decode_eq!("amf0-ref-test.bin", expected);
        decode_unexpected_eof!("amf0-reference-partial.bin");

        assert_eq!(
            decode!("amf0-bad-reference.bin"),
            Err(DecodeError::OutOfRangeReference { index: 0 })
        );
        assert_eq!(
            decode!("amf0-circular-reference.bin"),
            Err(DecodeError::CircularReference { index: 0 })
        );
    }
    #[test]
    fn decodes_date() {
        decode_eq!(
            "amf0-date.bin",
            Value::Date {
                unix_time: time::Duration::from_millis(1_590_796_800_000)
            }
        );
        decode_eq!(
            "amf0-time.bin",
            Value::Date {
                unix_time: time::Duration::from_millis(1_045_112_400_000)
            }
        );
        decode_unexpected_eof!("amf0-date-partial.bin");
        assert_eq!(
            decode!("amf0-date-minus.bin"),
            Err(DecodeError::InvalidDate { millis: -1.0 })
        );
        assert_eq!(
            decode!("amf0-date-invalid.bin"),
            Err(DecodeError::InvalidDate {
                millis: f64::INFINITY
            })
        );
    }
    #[test]
    fn decodes_avmplus() {
        let expected = amf3::Value::Array {
            assoc_entries: vec![],
            dense_entries: (1..4).map(amf3::Value::Integer).collect(),
        };
        decode_eq!("amf0-avmplus-object.bin", Value::AvmPlus(expected));
    }
    #[test]
    fn other_errors() {
        decode_unexpected_eof!("amf0-empty.bin");
        assert_eq!(
            decode!("amf0-unknown-marker.bin"),
            Err(DecodeError::Unknown { marker: 97 })
        );
    }

    fn s(s: &str) -> Value {
        Value::String(s.to_string())
    }
    fn n(n: f64) -> Value {
        Value::Number(n)
    }
    fn obj(name: Option<&str>, entries: &[(&str, Value)]) -> Value {
        Value::Object {
            class_name: name.map(|s| s.to_string()),
            entries: es(entries),
        }
    }
    fn es(entries: &[(&str, Value)]) -> Vec<Pair<String, Value>> {
        entries
            .iter()
            .map(|e| Pair {
                key: e.0.to_string(),
                value: e.1.clone(),
            })
            .collect()
    }
}
