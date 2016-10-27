use std::io;
use std::time;
use byteorder::BigEndian;
use byteorder::ReadBytesExt;

use amf3;
use Pair;
use DecodeResult;
use error::DecodeError;
use super::Value;
use super::marker;

#[derive(Debug)]
pub struct Decoder<R> {
    inner: R,
    complexes: Vec<Value>,
}
impl<R> Decoder<R> {
    pub fn into_inner(self) -> R {
        self.inner
    }
}
impl<R> Decoder<R>
    where R: io::Read
{
    pub fn new(inner: R) -> Self {
        Decoder {
            inner: inner,
            complexes: Vec::new(),
        }
    }
    pub fn decode(&mut self) -> DecodeResult<Value> {
        self.complexes.clear();
        self.decode_value()
    }

    fn decode_value(&mut self) -> DecodeResult<Value> {
        let marker = try!(self.inner.read_u8());
        match marker {
            marker::NUMBER => self.decode_number(),
            marker::BOOLEAN => self.decode_boolean(),
            marker::STRING => self.decode_string(),
            marker::OBJECT => self.decode_object(),
            marker::MOVIECLIP => Err(DecodeError::Unsupported { marker: marker }),
            marker::NULL => Ok(Value::Null),
            marker::UNDEFINED => Ok(Value::Undefined),
            marker::REFERENCE => self.decode_reference(),
            marker::ECMA_ARRAY => self.decode_ecma_array(),
            marker::OBJECT_END_MARKER => Err(DecodeError::UnexpectedObjectEnd),
            marker::STRICT_ARRAY => self.decode_strict_array(),
            marker::DATE => self.decode_date(),
            marker::LONG_STRING => self.decode_long_string(),
            marker::UNSUPPORTED => Err(DecodeError::Unsupported { marker: marker }),
            marker::RECORDSET => Err(DecodeError::Unsupported { marker: marker }),
            marker::XML_DOCUMENT => self.decode_xml_document(),
            marker::TYPED_OBJECT => self.decode_typed_object(),
            marker::AVMPLUS_OBJECT => self.decode_avmplus(),
            _ => Err(DecodeError::Unknown { marker: marker }),
        }
    }
    fn decode_number(&mut self) -> DecodeResult<Value> {
        let n = try!(self.inner.read_f64::<BigEndian>());
        Ok(Value::Number(n))
    }
    fn decode_boolean(&mut self) -> DecodeResult<Value> {
        let b = try!(self.inner.read_u8()) != 0;
        Ok(Value::Boolean(b))
    }
    fn decode_string(&mut self) -> DecodeResult<Value> {
        let len = try!(self.inner.read_u16::<BigEndian>()) as usize;
        self.read_utf8(len).map(Value::String)
    }
    fn decode_object(&mut self) -> DecodeResult<Value> {
        self.decode_complex_type(|this| {
            let entries = try!(this.decode_pairs());
            Ok(Value::Object {
                class_name: None,
                entries: entries,
            })
        })
    }
    fn decode_reference(&mut self) -> DecodeResult<Value> {
        let index = try!(self.inner.read_u16::<BigEndian>()) as usize;
        self.complexes
            .get(index)
            .ok_or(DecodeError::OutOfRangeRference { index: index })
            .and_then(|v| if *v == Value::Null {
                Err(DecodeError::CircularReference { index: index })
            } else {
                Ok(v.clone())
            })
    }
    fn decode_ecma_array(&mut self) -> DecodeResult<Value> {
        self.decode_complex_type(|this| {
            let count = try!(this.inner.read_u32::<BigEndian>()) as usize;
            let entries = try!(this.decode_pairs());
            debug_assert_eq!(entries.len(), count);
            Ok(Value::EcmaArray { entries: entries })
        })
    }
    fn decode_strict_array(&mut self) -> DecodeResult<Value> {
        self.decode_complex_type(|this| {
            let count = try!(this.inner.read_u32::<BigEndian>()) as usize;
            let entries = try!((0..count).map(|_| this.decode_value()).collect());
            Ok(Value::Array { entries: entries })
        })
    }
    fn decode_date(&mut self) -> DecodeResult<Value> {
        let millis = try!(self.inner.read_f64::<BigEndian>());
        let time_zone = try!(self.inner.read_i16::<BigEndian>());
        if time_zone != 0 {
            Err(DecodeError::NonZeroTimeZone { offset: time_zone })
        } else if !(millis.is_finite() && millis.is_sign_positive()) {
            Err(DecodeError::InvalidDate { millis: millis })
        } else {
            Ok(Value::Date { unix_time: time::Duration::from_millis(millis as u64) })
        }
    }
    fn decode_long_string(&mut self) -> DecodeResult<Value> {
        let len = try!(self.inner.read_u32::<BigEndian>()) as usize;
        self.read_utf8(len).map(Value::String)
    }
    fn decode_xml_document(&mut self) -> DecodeResult<Value> {
        let len = try!(self.inner.read_u32::<BigEndian>()) as usize;
        self.read_utf8(len).map(Value::XmlDocument)
    }
    fn decode_typed_object(&mut self) -> DecodeResult<Value> {
        self.decode_complex_type(|this| {
            let len = try!(this.inner.read_u16::<BigEndian>()) as usize;
            let class_name = try!(this.read_utf8(len));
            let entries = try!(this.decode_pairs());
            Ok(Value::Object {
                class_name: Some(class_name),
                entries: entries,
            })
        })
    }
    fn decode_avmplus(&mut self) -> DecodeResult<Value> {
        let value = try!(amf3::Decoder::new(&mut self.inner).decode());
        Ok(Value::AvmPlus(value))
    }

    fn read_utf8(&mut self, len: usize) -> DecodeResult<String> {
        let mut buf = vec![0; len];
        try!(self.inner.read_exact(&mut buf));
        let utf8 = try!(String::from_utf8(buf));
        Ok(utf8)
    }
    fn decode_pairs(&mut self) -> DecodeResult<Vec<Pair<String, Value>>> {
        let mut entries = Vec::new();
        loop {
            let len = try!(self.inner.read_u16::<BigEndian>()) as usize;
            let key = try!(self.read_utf8(len));
            match self.decode_value() {
                Ok(value) => {
                    entries.push(Pair {
                        key: key,
                        value: value,
                    });
                }
                Err(DecodeError::UnexpectedObjectEnd) if key.is_empty() => break,
                Err(e) => return Err(e),
            }
        }
        Ok(entries)
    }
    fn decode_complex_type<F>(&mut self, f: F) -> DecodeResult<Value>
        where F: FnOnce(&mut Self) -> DecodeResult<Value>
    {
        let index = self.complexes.len();
        self.complexes.push(Value::Null);
        let value = try!(f(self));
        self.complexes[index] = value.clone();
        Ok(value)
    }
}

#[cfg(test)]
mod test {
    use std::iter;
    use std::time;
    use Pair;
    use super::*;
    use super::super::Value;

    macro_rules! decode_eq {
        ($file:expr, $expected: expr) => {
            {
                let input = include_bytes!(concat!("../testdata/", $file));
                let value = Decoder::new(&mut &input[..]).decode().unwrap();
                assert_eq!(value, $expected)
            }
        }
    }

    #[test]
    fn decodes_boolean() {
        decode_eq!("amf0-boolean-true.bin", Value::Boolean(true));
        decode_eq!("amf0-boolean-false.bin", Value::Boolean(false));
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
    }
    #[test]
    fn decodes_string() {
        decode_eq!("amf0-string.bin",
                   Value::String("this is a テスト".to_string()));
    }
    #[test]
    fn decodes_long_string() {
        decode_eq!("amf0-long-string.bin",
                   Value::String(iter::repeat('a').take(0x10013).collect()));
    }
    #[test]
    fn decodes_xml_document() {
        decode_eq!("amf0-xml-doc.bin",
                   Value::XmlDocument("<parent><child prop=\"test\" /></parent>".to_string()));
    }
    #[test]
    fn decodes_object() {
        decode_eq!("amf0-object.bin",
                   obj(None,
                       &[("", s("")), ("foo", s("baz")), ("bar", n(3.14))][..]));
    }
    #[test]
    fn decodes_typed_object() {
        decode_eq!("amf0-typed-object.bin",
                   obj(Some("org.amf.ASClass"),
                       &[("foo", s("bar")), ("baz", Value::Null)]));
    }
    #[test]
    fn decodes_ecma_array() {
        let entries = es(&[("0", s("a")), ("1", s("b")), ("2", s("c")), ("3", s("d"))][..]);
        decode_eq!("amf0-ecma-ordinal-array.bin",
                   Value::EcmaArray { entries: entries });
    }
    #[test]
    fn decodes_strict_array() {
        decode_eq!("amf0-strict-array.bin",
                   Value::Array { entries: vec![n(1.0), s("2"), n(3.0)] });
    }
    #[test]
    fn decodes_reference() {
        let object = obj(None, &[("foo", s("baz")), ("bar", n(3.14))][..]);
        let expected = obj(None, &[("0", object.clone()), ("1", object.clone())][..]);
        decode_eq!("amf0-ref-test.bin", expected);
    }
    #[test]
    fn decodes_date() {
        decode_eq!("amf0-time.bin",
                   Value::Date { unix_time: time::Duration::from_millis(1045112400000) });
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
        entries.iter()
            .map(|e| {
                Pair {
                    key: e.0.to_string(),
                    value: e.1.clone(),
                }
            })
            .collect()
    }
}
