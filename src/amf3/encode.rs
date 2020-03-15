use super::marker;
use super::Value;
use crate::Pair;
use byteorder::{BigEndian, WriteBytesExt};
use std::io;
use std::time;

/// AMF3 encoder.
#[derive(Debug)]
pub struct Encoder<W> {
    inner: W,
}
impl<W> Encoder<W> {
    /// Unwraps this `Encoder`, returning the underlying writer.
    pub fn into_inner(self) -> W {
        self.inner
    }
    /// Returns a reference to the underlying writer.
    pub fn inner(&mut self) -> &mut W {
        &mut self.inner
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

    /// Encodes a AMF3 value.
    pub fn encode(&mut self, value: &Value) -> io::Result<()> {
        match *value {
            Value::Undefined => self.encode_undefined(),
            Value::Null => self.encode_null(),
            Value::Boolean(x) => self.encode_boolean(x),
            Value::Integer(x) => self.encode_integer(x),
            Value::Double(x) => self.encode_double(x),
            Value::String(ref x) => self.encode_string(x),
            Value::XmlDocument(ref x) => self.encode_xml_document(x),
            Value::Date { unix_time } => self.encode_date(unix_time),
            Value::Array {
                ref assoc_entries,
                ref dense_entries,
            } => self.encode_array(assoc_entries, dense_entries),
            Value::Object {
                ref class_name,
                sealed_count,
                ref entries,
            } => self.encode_object(class_name, sealed_count, entries),
            Value::Xml(ref x) => self.encode_xml(x),
            Value::ByteArray(ref x) => self.encode_byte_array(x),
            Value::IntVector {
                is_fixed,
                ref entries,
            } => self.encode_int_vector(is_fixed, entries),
            Value::UintVector {
                is_fixed,
                ref entries,
            } => self.encode_uint_vector(is_fixed, entries),
            Value::DoubleVector {
                is_fixed,
                ref entries,
            } => self.encode_double_vector(is_fixed, entries),
            Value::ObjectVector {
                ref class_name,
                is_fixed,
                ref entries,
            } => self.encode_object_vector(class_name, is_fixed, entries),
            Value::Dictionary {
                is_weak,
                ref entries,
            } => self.encode_dictionary(is_weak, entries),
        }
    }

    fn encode_undefined(&mut self) -> io::Result<()> {
        self.inner.write_u8(marker::UNDEFINED)?;
        Ok(())
    }
    fn encode_null(&mut self) -> io::Result<()> {
        self.inner.write_u8(marker::NULL)?;
        Ok(())
    }
    fn encode_boolean(&mut self, b: bool) -> io::Result<()> {
        if b {
            self.inner.write_u8(marker::TRUE)?;
        } else {
            self.inner.write_u8(marker::FALSE)?;
        }
        Ok(())
    }
    fn encode_integer(&mut self, i: i32) -> io::Result<()> {
        self.inner.write_u8(marker::INTEGER)?;
        let u29 = if i >= 0 {
            i as u32
        } else {
            ((1 << 29) + i) as u32
        };
        self.encode_u29(u29)?;
        Ok(())
    }
    fn encode_double(&mut self, d: f64) -> io::Result<()> {
        self.inner.write_u8(marker::DOUBLE)?;
        self.inner.write_f64::<BigEndian>(d)?;
        Ok(())
    }
    fn encode_string(&mut self, s: &str) -> io::Result<()> {
        self.inner.write_u8(marker::STRING)?;
        self.encode_utf8(s)?;
        Ok(())
    }
    fn encode_xml_document(&mut self, xml: &str) -> io::Result<()> {
        self.inner.write_u8(marker::XML_DOC)?;
        self.encode_utf8(xml)?;
        Ok(())
    }
    fn encode_date(&mut self, unix_time: time::Duration) -> io::Result<()> {
        let millis = unix_time.as_secs() * 1000 + (unix_time.subsec_nanos() as u64) / 1_000_000;
        self.inner.write_u8(marker::DATE)?;
        self.encode_size(0)?;
        self.inner.write_f64::<BigEndian>(millis as f64)?;
        Ok(())
    }
    fn encode_array(&mut self, assoc: &[Pair<String, Value>], dense: &[Value]) -> io::Result<()> {
        self.inner.write_u8(marker::ARRAY)?;
        self.encode_size(dense.len())?;
        self.encode_pairs(assoc)?;
        dense
            .iter()
            .map(|v| self.encode(v))
            .collect::<io::Result<Vec<_>>>()?;
        Ok(())
    }
    fn encode_object(
        &mut self,
        class_name: &Option<String>,
        sealed_count: usize,
        entries: &[Pair<String, Value>],
    ) -> io::Result<()> {
        self.inner.write_u8(marker::OBJECT)?;
        self.encode_trait(class_name, sealed_count, entries)?;
        for e in entries.iter().take(sealed_count) {
            self.encode(&e.value)?;
        }
        if entries.len() > sealed_count {
            self.encode_pairs(&entries[sealed_count..])?;
        }
        Ok(())
    }
    fn encode_xml(&mut self, xml: &str) -> io::Result<()> {
        self.inner.write_u8(marker::XML)?;
        self.encode_utf8(xml)?;
        Ok(())
    }
    fn encode_byte_array(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.inner.write_u8(marker::BYTE_ARRAY)?;
        self.encode_size(bytes.len())?;
        self.inner.write_all(bytes)?;
        Ok(())
    }
    fn encode_int_vector(&mut self, is_fixed: bool, vec: &[i32]) -> io::Result<()> {
        self.inner.write_u8(marker::VECTOR_INT)?;
        self.encode_size(vec.len())?;
        self.inner.write_u8(is_fixed as u8)?;
        for &x in vec {
            self.inner.write_i32::<BigEndian>(x)?;
        }
        Ok(())
    }
    fn encode_uint_vector(&mut self, is_fixed: bool, vec: &[u32]) -> io::Result<()> {
        self.inner.write_u8(marker::VECTOR_UINT)?;
        self.encode_size(vec.len())?;
        self.inner.write_u8(is_fixed as u8)?;
        for &x in vec {
            self.inner.write_u32::<BigEndian>(x)?;
        }
        Ok(())
    }
    fn encode_double_vector(&mut self, is_fixed: bool, vec: &[f64]) -> io::Result<()> {
        self.inner.write_u8(marker::VECTOR_DOUBLE)?;
        self.encode_size(vec.len())?;
        self.inner.write_u8(is_fixed as u8)?;
        for &x in vec {
            self.inner.write_f64::<BigEndian>(x)?;
        }
        Ok(())
    }
    fn encode_object_vector(
        &mut self,
        class_name: &Option<String>,
        is_fixed: bool,
        vec: &[Value],
    ) -> io::Result<()> {
        self.inner.write_u8(marker::VECTOR_OBJECT)?;
        self.encode_size(vec.len())?;
        self.inner.write_u8(is_fixed as u8)?;
        self.encode_utf8(class_name.as_ref().map_or("*", |s| &s))?;
        for x in vec {
            self.encode(x)?;
        }
        Ok(())
    }
    fn encode_dictionary(
        &mut self,
        is_weak: bool,
        entries: &[Pair<Value, Value>],
    ) -> io::Result<()> {
        self.inner.write_u8(marker::DICTIONARY)?;
        self.encode_size(entries.len())?;
        self.inner.write_u8(is_weak as u8)?;
        for e in entries {
            self.encode(&e.key)?;
            self.encode(&e.value)?;
        }
        Ok(())
    }
    fn encode_trait(
        &mut self,
        class_name: &Option<String>,
        sealed_count: usize,
        entries: &[Pair<String, Value>],
    ) -> io::Result<()> {
        assert!(sealed_count <= entries.len());
        let not_reference = 1;
        let is_externalizable = false as usize;
        let is_dynamic = (sealed_count < entries.len()) as usize;
        let u28 =
            (sealed_count << 3) | (is_dynamic << 2) | (is_externalizable << 1) | not_reference;
        self.encode_size(u28)?;

        let class_name = class_name.as_ref().map_or("", |s| &s);
        self.encode_utf8(class_name)?;
        for e in entries.iter().take(sealed_count) {
            self.encode_utf8(&e.key)?;
        }
        Ok(())
    }
    fn encode_size(&mut self, size: usize) -> io::Result<()> {
        assert!(size < (1 << 28));
        let not_reference = 1;
        self.encode_u29(((size << 1) | not_reference) as u32)
    }
    #[allow(clippy::zero_prefixed_literal, clippy::identity_op)]
    fn encode_u29(&mut self, u29: u32) -> io::Result<()> {
        if u29 < 0x80 {
            self.inner.write_u8(u29 as u8)?;
        } else if u29 < 0x4000 {
            let b1 = ((u29 >> 0) & 0b0111_1111) as u8;
            let b2 = ((u29 >> 7) | 0b1000_0000) as u8;
            for b in &[b2, b1] {
                self.inner.write_u8(*b)?;
            }
        } else if u29 < 0x20_0000 {
            let b1 = ((u29 >> 00) & 0b0111_1111) as u8;
            let b2 = ((u29 >> 07) | 0b1000_0000) as u8;
            let b3 = ((u29 >> 14) | 0b1000_0000) as u8;
            for b in &[b3, b2, b1] {
                self.inner.write_u8(*b)?;
            }
        } else if u29 < 0x4000_0000 {
            let b1 = ((u29 >> 00) & 0b1111_1111) as u8;
            let b2 = ((u29 >> 08) | 0b1000_0000) as u8;
            let b3 = ((u29 >> 15) | 0b1000_0000) as u8;
            let b4 = ((u29 >> 22) | 0b1000_0000) as u8;
            for b in &[b4, b3, b2, b1] {
                self.inner.write_u8(*b)?;
            }
        } else {
            panic!("Too large number: {}", u29);
        }
        Ok(())
    }
    /// Encode an AMF3 string.
    ///
    /// Use this if you need to decode an AMF3 string outside of value context.
    /// An example of this is writing keys in Local Shared Object file.
    pub fn encode_utf8(&mut self, s: &str) -> io::Result<()> {
        self.encode_size(s.len())?;
        self.inner.write_all(s.as_bytes())?;
        Ok(())
    }
    fn encode_pairs(&mut self, pairs: &[Pair<String, Value>]) -> io::Result<()> {
        for p in pairs {
            self.encode_utf8(&p.key)?;
            self.encode(&p.value)?;
        }
        self.encode_utf8("")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::Value;
    use crate::Pair;
    use std::time;

    macro_rules! encode_eq {
        ($value:expr, $file:expr) => {{
            let expected = include_bytes!(concat!("../testdata/", $file));
            let mut buf = Vec::new();
            $value.write_to(&mut buf).unwrap();
            assert_eq!(buf, &expected[..]);
        }};
    }
    macro_rules! encode_and_decode {
        ($value:expr) => {{
            let v = $value;
            let mut buf = Vec::new();
            v.write_to(&mut buf).unwrap();
            assert_eq!(v, Value::read_from(&mut &buf[..]).unwrap());
        }};
    }

    #[test]
    fn encodes_undefined() {
        encode_eq!(Value::Undefined, "amf3-undefined.bin");
    }
    #[test]
    fn encodes_null() {
        encode_eq!(Value::Null, "amf3-null.bin");
    }
    #[test]
    fn encodes_boolean() {
        encode_eq!(Value::Boolean(true), "amf3-true.bin");
        encode_eq!(Value::Boolean(false), "amf3-false.bin");
    }
    #[test]
    fn encodes_integer() {
        encode_eq!(Value::Integer(0), "amf3-0.bin");
        encode_eq!(Value::Integer(0b1000_0000), "amf3-integer-2byte.bin");
        encode_eq!(
            Value::Integer(0b100_0000_0000_0000),
            "amf3-integer-3byte.bin"
        );
        encode_eq!(Value::Integer(-0x1000_0000), "amf3-min.bin");
        encode_eq!(Value::Integer(0xFFF_FFFF), "amf3-max.bin");
    }
    #[test]
    fn encodes_double() {
        encode_eq!(Value::Double(3.5), "amf3-float.bin");
        encode_eq!(Value::Double(2f64.powf(1000f64)), "amf3-bignum.bin");
        encode_eq!(Value::Double(-0x1000_0001 as f64), "amf3-large-min.bin");
        encode_eq!(Value::Double(268_435_456_f64), "amf3-large-max.bin");
    }
    #[test]
    fn encodes_string() {
        encode_eq!(s("String . String"), "amf3-string.bin");
        encode_eq!(
            dense_array(&[i(5), s("Shift テスト"), s("UTF テスト"), i(5)][..]),
            "amf3-complex-encoded-string-array.bin"
        );
    }
    #[test]
    fn encodes_array() {
        encode_eq!(
            dense_array(&[i(1), i(2), i(3), i(4), i(5)][..]),
            "amf3-primitive-array.bin"
        );
        encode_and_decode!(Value::Array {
            assoc_entries: [("2", s("bar3")), ("foo", s("bar")), ("asdf", s("fdsa"))]
                .iter()
                .map(|e| pair(e.0, e.1.clone()))
                .collect(),
            dense_entries: vec![s("bar"), s("bar1"), s("bar2")],
        });
    }
    #[test]
    fn encodes_object() {
        encode_eq!(
            typed_obj(
                "org.amf.ASClass",
                &[("foo", s("bar")), ("baz", Value::Null)][..]
            ),
            "amf3-typed-object.bin"
        );
        encode_eq!(
            obj(&[("foo", s("bar")), ("answer", i(42))][..]),
            "amf3-hash.bin"
        );
    }
    #[test]
    fn encodes_xml_doc() {
        encode_eq!(
            Value::XmlDocument("<parent><child prop=\"test\" /></parent>".to_string()),
            "amf3-xml-doc.bin"
        );
    }
    #[test]
    fn encodes_xml() {
        let xml = Value::Xml("<parent><child prop=\"test\"/></parent>".to_string());
        encode_eq!(xml, "amf3-xml.bin");
    }
    #[test]
    fn encodes_byte_array() {
        encode_eq!(
            Value::ByteArray(vec![
                0, 3, 227, 129, 147, 227, 130, 140, 116, 101, 115, 116, 64
            ]),
            "amf3-byte-array.bin"
        );
    }
    #[test]
    fn encodes_date() {
        let d = Value::Date {
            unix_time: time::Duration::from_secs(0),
        };
        encode_eq!(d, "amf3-date.bin");
    }
    #[test]
    fn encodes_dictionary() {
        let entries = vec![
            (s("bar"), s("asdf1")),
            (
                typed_obj(
                    "org.amf.ASClass",
                    &[("foo", s("baz")), ("baz", Value::Null)][..],
                ),
                s("asdf2"),
            ),
        ];
        encode_and_decode!(dic(&entries));
        encode_eq!(dic(&[][..]), "amf3-empty-dictionary.bin");
    }
    #[test]
    fn encodes_vector() {
        encode_eq!(
            Value::IntVector {
                is_fixed: false,
                entries: vec![4, -20, 12],
            },
            "amf3-vector-int.bin"
        );

        encode_eq!(
            Value::UintVector {
                is_fixed: false,
                entries: vec![4, 20, 12],
            },
            "amf3-vector-uint.bin"
        );

        encode_eq!(
            Value::DoubleVector {
                is_fixed: false,
                entries: vec![4.3, -20.6],
            },
            "amf3-vector-double.bin"
        );

        let objects = vec![
            typed_obj(
                "org.amf.ASClass",
                &[("foo", s("foo")), ("baz", Value::Null)][..],
            ),
            typed_obj(
                "org.amf.ASClass",
                &[("foo", s("bar")), ("baz", Value::Null)][..],
            ),
            typed_obj(
                "org.amf.ASClass",
                &[("foo", s("baz")), ("baz", Value::Null)][..],
            ),
        ];
        encode_and_decode!(Value::ObjectVector {
            class_name: Some("org.amf.ASClass".to_string()),
            is_fixed: false,
            entries: objects,
        });
    }

    fn i(i: i32) -> Value {
        Value::Integer(i)
    }
    fn s(s: &str) -> Value {
        Value::String(s.to_string())
    }
    fn pair(key: &str, value: Value) -> Pair<String, Value> {
        Pair {
            key: key.to_string(),
            value,
        }
    }
    fn dense_array(entries: &[Value]) -> Value {
        Value::Array {
            assoc_entries: Vec::new(),
            dense_entries: entries.to_vec(),
        }
    }
    fn dic(entries: &[(Value, Value)]) -> Value {
        Value::Dictionary {
            is_weak: false,
            entries: entries
                .iter()
                .map(|e| Pair {
                    key: e.0.clone(),
                    value: e.1.clone(),
                })
                .collect(),
        }
    }
    fn obj(entries: &[(&str, Value)]) -> Value {
        Value::Object {
            class_name: None,
            sealed_count: 0,
            entries: entries.iter().map(|e| pair(e.0, e.1.clone())).collect(),
        }
    }
    fn typed_obj(class: &str, entries: &[(&str, Value)]) -> Value {
        Value::Object {
            class_name: Some(class.to_string()),
            sealed_count: entries.len(),
            entries: entries.iter().map(|e| pair(e.0, e.1.clone())).collect(),
        }
    }
}
