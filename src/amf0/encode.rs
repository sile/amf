use super::marker;
use super::Value;
use crate::amf3;
use crate::Pair;
use byteorder::{BigEndian, WriteBytesExt};
use std::io;
use std::time;

/// AMF0 encoder.
#[derive(Debug)]
pub struct Encoder<W> {
    inner: W,
}
impl<W> Encoder<W> {
    /// Unwraps this `Encoder`, returning the underlying writer.
    pub fn into_inner(self) -> W {
        self.inner
    }
}
impl<W> Encoder<W>
where
    W: io::Write,
{
    /// Makes a new instance.
    pub fn new(inner: W) -> Self {
        Encoder { inner }
    }
    /// Encodes a AMF0 value.
    pub fn encode(&mut self, value: &Value) -> io::Result<()> {
        match *value {
            Value::Number(x) => self.encode_number(x),
            Value::Boolean(x) => self.encode_boolean(x),
            Value::String(ref x) => self.encode_string(x),
            Value::Object {
                ref class_name,
                ref entries,
            } => self.encode_object(class_name, entries),
            Value::Null => self.encode_null(),
            Value::Undefined => self.encode_undefined(),
            Value::EcmaArray { ref entries } => self.encode_ecma_array(entries),
            Value::Array { ref entries } => self.encode_strict_array(entries),
            Value::Date { unix_time } => self.encode_date(unix_time),
            Value::XmlDocument(ref x) => self.encode_xml_document(x),
            Value::AvmPlus(ref x) => self.encode_avmplus(x),
        }
    }

    fn encode_number(&mut self, n: f64) -> io::Result<()> {
        self.inner.write_u8(marker::NUMBER)?;
        self.inner.write_f64::<BigEndian>(n)?;
        Ok(())
    }
    fn encode_boolean(&mut self, b: bool) -> io::Result<()> {
        self.inner.write_u8(marker::BOOLEAN)?;
        self.inner.write_u8(b as u8)?;
        Ok(())
    }
    fn encode_string(&mut self, s: &str) -> io::Result<()> {
        if s.len() <= 0xFFFF {
            self.inner.write_u8(marker::STRING)?;
            self.write_str_u16(&s)?;
        } else {
            self.inner.write_u8(marker::LONG_STRING)?;
            self.write_str_u32(&s)?;
        }
        Ok(())
    }
    fn encode_object(
        &mut self,
        class_name: &Option<String>,
        entries: &[Pair<String, Value>],
    ) -> io::Result<()> {
        assert!(entries.len() <= 0xFFFF_FFFF);
        if let Some(class_name) = class_name.as_ref() {
            self.inner.write_u8(marker::TYPED_OBJECT)?;
            self.write_str_u16(class_name)?;
        } else {
            self.inner.write_u8(marker::OBJECT)?;
        }
        self.encode_pairs(entries)?;
        Ok(())
    }
    fn encode_null(&mut self) -> io::Result<()> {
        self.inner.write_u8(marker::NULL)?;
        Ok(())
    }
    fn encode_undefined(&mut self) -> io::Result<()> {
        self.inner.write_u8(marker::UNDEFINED)?;
        Ok(())
    }
    fn encode_ecma_array(&mut self, entries: &[Pair<String, Value>]) -> io::Result<()> {
        assert!(entries.len() <= 0xFFFF_FFFF);
        self.inner.write_u8(marker::ECMA_ARRAY)?;
        self.inner.write_u32::<BigEndian>(entries.len() as u32)?;
        self.encode_pairs(entries)?;
        Ok(())
    }
    fn encode_strict_array(&mut self, entries: &[Value]) -> io::Result<()> {
        assert!(entries.len() <= 0xFFFF_FFFF);
        self.inner.write_u8(marker::STRICT_ARRAY)?;
        self.inner.write_u32::<BigEndian>(entries.len() as u32)?;
        for e in entries {
            self.encode(e)?;
        }
        Ok(())
    }
    fn encode_date(&mut self, unix_time: time::Duration) -> io::Result<()> {
        let millis = unix_time.as_secs() * 1000 + (unix_time.subsec_nanos() as u64) / 1_000_000;

        self.inner.write_u8(marker::DATE)?;
        self.inner.write_f64::<BigEndian>(millis as f64)?;
        self.inner.write_i16::<BigEndian>(0)?;
        Ok(())
    }
    fn encode_xml_document(&mut self, xml: &str) -> io::Result<()> {
        self.inner.write_u8(marker::XML_DOCUMENT)?;
        self.write_str_u32(xml)?;
        Ok(())
    }
    fn encode_avmplus(&mut self, value: &amf3::Value) -> io::Result<()> {
        self.inner.write_u8(marker::AVMPLUS_OBJECT)?;
        amf3::Encoder::new(&mut self.inner).encode(value)?;
        Ok(())
    }

    fn write_str_u32(&mut self, s: &str) -> io::Result<()> {
        assert!(s.len() <= 0xFFFF_FFFF);
        self.inner.write_u32::<BigEndian>(s.len() as u32)?;
        self.inner.write_all(s.as_bytes())?;
        Ok(())
    }
    fn write_str_u16(&mut self, s: &str) -> io::Result<()> {
        assert!(s.len() <= 0xFFFF);
        self.inner.write_u16::<BigEndian>(s.len() as u16)?;
        self.inner.write_all(s.as_bytes())?;
        Ok(())
    }
    fn encode_pairs(&mut self, pairs: &[Pair<String, Value>]) -> io::Result<()> {
        for p in pairs {
            self.write_str_u16(&p.key)?;
            self.encode(&p.value)?;
        }
        self.inner.write_u16::<BigEndian>(0)?;
        self.inner.write_u8(marker::OBJECT_END_MARKER)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::approx_constant)]
    use super::super::Value;
    use crate::amf3;
    use crate::Pair;
    use std::iter;
    use std::time;

    macro_rules! encode_eq {
        ($value:expr, $file:expr) => {{
            let expected = include_bytes!(concat!("../testdata/", $file));
            let mut buf = Vec::new();
            $value.write_to(&mut buf).unwrap();
            assert_eq!(buf, &expected[..]);
        }};
    }

    #[test]
    fn encodes_number() {
        encode_eq!(Value::Number(3.5), "amf0-number.bin");
    }
    #[test]
    fn encodes_boolean() {
        encode_eq!(Value::Boolean(true), "amf0-boolean-true.bin");
        encode_eq!(Value::Boolean(false), "amf0-boolean-false.bin");
    }
    #[test]
    fn encodes_string() {
        encode_eq!(
            Value::String("this is a テスト".to_string()),
            "amf0-string.bin"
        );
        encode_eq!(
            obj(
                None,
                &[
                    ("utf", s("UTF テスト")),
                    ("zed", n(5.0)),
                    ("shift", s("Shift テスト"))
                ][..]
            ),
            "amf0-complex-encoded-string.bin"
        );
    }
    #[test]
    fn encodes_long_string() {
        encode_eq!(
            Value::String(iter::repeat('a').take(0x10013).collect()),
            "amf0-long-string.bin"
        );
    }
    #[test]
    fn encodes_object() {
        encode_eq!(
            obj(
                None,
                &[("", s("")), ("foo", s("baz")), ("bar", n(3.14))][..]
            ),
            "amf0-object.bin"
        );
        encode_eq!(
            obj(None, &[("foo", s("bar")), ("baz", Value::Null)][..]),
            "amf0-untyped-object.bin"
        );
    }
    #[test]
    fn encodes_null() {
        encode_eq!(Value::Null, "amf0-null.bin");
    }
    #[test]
    fn encodes_undefined() {
        encode_eq!(Value::Undefined, "amf0-undefined.bin");
    }
    #[test]
    fn encodes_ecma_array() {
        let entries = es(&[("0", s("a")), ("1", s("b")), ("2", s("c")), ("3", s("d"))][..]);
        encode_eq!(
            Value::EcmaArray { entries: entries },
            "amf0-ecma-ordinal-array.bin"
        );
    }
    #[test]
    fn encodes_string_array() {
        encode_eq!(
            Value::Array {
                entries: vec![n(1.0), s("2"), n(3.0)]
            },
            "amf0-strict-array.bin"
        );
    }
    #[test]
    fn encodes_date() {
        encode_eq!(
            Value::Date {
                unix_time: time::Duration::from_millis(1590796800_000)
            },
            "amf0-date.bin"
        );
        encode_eq!(
            Value::Date {
                unix_time: time::Duration::from_millis(1045112400_000)
            },
            "amf0-time.bin"
        );
    }
    #[test]
    fn encodes_xml_document() {
        encode_eq!(
            Value::XmlDocument("<parent><child prop=\"test\" /></parent>".to_string()),
            "amf0-xml-doc.bin"
        );
    }
    #[test]
    fn encodes_typed_object() {
        encode_eq!(
            obj(
                Some("org.amf.ASClass"),
                &[("foo", s("bar")), ("baz", Value::Null)]
            ),
            "amf0-typed-object.bin"
        );
    }
    #[test]
    fn encodes_avmplus() {
        let value = amf3::Value::Array {
            assoc_entries: vec![],
            dense_entries: (1..4).map(amf3::Value::Integer).collect(),
        };
        encode_eq!(Value::AvmPlus(value), "amf0-avmplus-object.bin");
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
