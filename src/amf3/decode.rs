use std::io;
use std::time;
use byteorder::BigEndian;
use byteorder::ReadBytesExt;

use Pair;
use DecodeResult;
use error::DecodeError;

use super::Value;
use super::marker;

#[derive(Debug, Clone)]
struct Trait {
    class_name: Option<String>,
    is_dynamic: bool,
    fields: Vec<String>,
}

#[derive(Debug)]
enum SizeOrIndex {
    Size(usize),
    Index(usize),
}

#[derive(Debug)]
pub struct Decoder<R> {
    inner: R,
    traits: Vec<Trait>,
    strings: Vec<String>,
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
            traits: Vec::new(),
            strings: Vec::new(),
            complexes: Vec::new(),
        }
    }
    pub fn decode(&mut self) -> DecodeResult<Value> {
        self.traits.clear();
        self.strings.clear();
        self.complexes.clear();
        self.decode_value()
    }

    fn decode_value(&mut self) -> DecodeResult<Value> {
        let marker = try!(self.inner.read_u8());
        match marker {
            marker::UNDEFINED => Ok(Value::Undefined),
            marker::NULL => Ok(Value::Null),
            marker::FALSE => Ok(Value::Boolean(false)),
            marker::TRUE => Ok(Value::Boolean(true)),
            marker::INTEGER => self.decode_integer(),
            marker::DOUBLE => self.decode_double(),
            marker::STRING => self.decode_string(),
            marker::XML_DOC => self.decode_xml_doc(),
            marker::DATE => self.decode_date(),
            marker::ARRAY => self.decode_array(),
            marker::OBJECT => self.decode_object(),
            marker::XML => self.decode_xml(),
            marker::BYTE_ARRAY => self.decode_byte_array(),
            marker::VECTOR_INT => self.decode_vector_int(),
            marker::VECTOR_UINT => self.decode_vector_uint(),
            marker::VECTOR_DOUBLE => self.decode_vector_double(),
            marker::VECTOR_OBJECT => self.decode_vector_object(),
            marker::DICTIONARY => self.decode_dictionary(),
            _ => Err(DecodeError::Unknown { marker: marker }),
        }
    }

    fn decode_integer(&mut self) -> DecodeResult<Value> {
        let n = try!(self.decode_u29()) as i32;
        let n = if n >= (1 << 28) { n - (1 << 29) } else { n };
        Ok(Value::Integer(n))
    }
    fn decode_double(&mut self) -> DecodeResult<Value> {
        let n = try!(self.inner.read_f64::<BigEndian>());
        Ok(Value::Double(n))
    }
    fn decode_string(&mut self) -> DecodeResult<Value> {
        let s = try!(self.decode_utf8());
        Ok(Value::String(s))
    }
    fn decode_xml_doc(&mut self) -> DecodeResult<Value> {
        self.decode_complex_type(|this, len| this.read_utf8(len).map(Value::XmlDocument))
    }
    fn decode_date(&mut self) -> DecodeResult<Value> {
        self.decode_complex_type(|this, _| {
            let millis = try!(this.inner.read_f64::<BigEndian>());
            if !(millis.is_finite() && millis.is_sign_positive()) {
                Err(DecodeError::InvalidDate { millis: millis })
            } else {
                Ok(Value::Date { unix_time: time::Duration::from_millis(millis as u64) })
            }
        })
    }
    fn decode_array(&mut self) -> DecodeResult<Value> {
        self.decode_complex_type(|this, count| {
            let assoc = try!(this.decode_pairs());
            let dense = try!((0..count).map(|_| this.decode_value()).collect());
            Ok(Value::Array {
                assoc_entries: assoc,
                dense_entries: dense,
            })
        })
    }
    fn decode_object(&mut self) -> DecodeResult<Value> {
        self.decode_complex_type(|this, u28| {
            let amf_trait = try!(this.decode_trait(u28));
            let mut entries = try!(amf_trait.fields
                .iter()
                .map(|k| {
                    Ok(Pair {
                        key: k.clone(),
                        value: try!(this.decode_value()),
                    })
                })
                .collect::<DecodeResult<Vec<_>>>());
            if amf_trait.is_dynamic {
                entries.extend(try!(this.decode_pairs()));
            }
            Ok(Value::Object {
                class_name: amf_trait.class_name,
                sealed_count: amf_trait.fields.len(),
                entries: entries,
            })
        })
    }
    fn decode_xml(&mut self) -> DecodeResult<Value> {
        self.decode_complex_type(|this, len| this.read_utf8(len).map(Value::Xml))
    }
    fn decode_byte_array(&mut self) -> DecodeResult<Value> {
        self.decode_complex_type(|this, len| this.read_bytes(len).map(Value::ByteArray))
    }
    fn decode_vector_int(&mut self) -> DecodeResult<Value> {
        self.decode_complex_type(|this, count| {
            let is_fixed = try!(this.inner.read_u8()) != 0;
            let entries = try!((0..count).map(|_| this.inner.read_i32::<BigEndian>()).collect());
            Ok(Value::IntVector {
                is_fixed: is_fixed,
                entries: entries,
            })
        })
    }
    fn decode_vector_uint(&mut self) -> DecodeResult<Value> {
        self.decode_complex_type(|this, count| {
            let is_fixed = try!(this.inner.read_u8()) != 0;
            let entries = try!((0..count).map(|_| this.inner.read_u32::<BigEndian>()).collect());
            Ok(Value::UintVector {
                is_fixed: is_fixed,
                entries: entries,
            })
        })
    }
    fn decode_vector_double(&mut self) -> DecodeResult<Value> {
        self.decode_complex_type(|this, count| {
            let is_fixed = try!(this.inner.read_u8()) != 0;
            let entries = try!((0..count).map(|_| this.inner.read_f64::<BigEndian>()).collect());
            Ok(Value::DoubleVector {
                is_fixed: is_fixed,
                entries: entries,
            })
        })
    }
    fn decode_vector_object(&mut self) -> DecodeResult<Value> {
        self.decode_complex_type(|this, count| {
            let is_fixed = try!(this.inner.read_u8()) != 0;
            let class_name = try!(this.decode_utf8());
            let entries = try!((0..count).map(|_| this.decode_value()).collect());
            Ok(Value::ObjectVector {
                class_name: if class_name == "*" {
                    None
                } else {
                    Some(class_name)
                },
                is_fixed: is_fixed,
                entries: entries,
            })
        })
    }
    fn decode_dictionary(&mut self) -> DecodeResult<Value> {
        self.decode_complex_type(|this, count| {
            let is_weak = try!(this.inner.read_u8()) == 1;
            let entries = try!((0..count)
                .map(|_| {
                    Ok(Pair {
                        key: try!(this.decode_value()),
                        value: try!(this.decode_value()),
                    })
                })
                .collect::<DecodeResult<_>>());
            Ok(Value::Dictionary {
                is_weak: is_weak,
                entries: entries,
            })
        })
    }

    fn decode_utf8(&mut self) -> DecodeResult<String> {
        match try!(self.decode_size_or_index()) {
            SizeOrIndex::Size(len) => {
                let bytes = try!(self.read_bytes(len));
                let s = try!(String::from_utf8(bytes));
                if !s.is_empty() {
                    self.strings.push(s.clone());
                }
                Ok(s)
            }
            SizeOrIndex::Index(index) => {
                let s = try!(self.strings
                    .get(index)
                    .ok_or(DecodeError::OutOfRangeRference { index: index }));
                Ok(s.clone())
            }
        }
    }
    fn decode_u29(&mut self) -> DecodeResult<u32> {
        let mut n = 0;
        for _ in 0..3 {
            let b = try!(self.inner.read_u8()) as u32;
            n = (n << 7) | (b & 0b0111_1111);
            if (b & 0b1000_0000) == 0 {
                return Ok(n);
            }
        }
        let b = try!(self.inner.read_u8()) as u32;
        n = (n << 8) | b;
        Ok(n)
    }
    fn decode_size_or_index(&mut self) -> DecodeResult<SizeOrIndex> {
        let u29 = try!(self.decode_u29()) as usize;
        let is_reference = (u29 & 0b01) == 0;
        let value = u29 >> 1;
        if is_reference {
            Ok(SizeOrIndex::Index(value))
        } else {
            Ok(SizeOrIndex::Size(value))
        }
    }
    fn decode_complex_type<F>(&mut self, f: F) -> DecodeResult<Value>
        where F: FnOnce(&mut Self, usize) -> DecodeResult<Value>
    {
        match try!(self.decode_size_or_index()) {
            SizeOrIndex::Index(index) => {
                self.complexes
                    .get(index)
                    .ok_or(DecodeError::OutOfRangeRference { index: index })
                    .and_then(|v| if *v == Value::Null {
                        Err(DecodeError::CircularReference { index: index })
                    } else {
                        Ok(v.clone())
                    })
            }
            SizeOrIndex::Size(u28) => {
                let index = self.complexes.len();
                self.complexes.push(Value::Null);
                let value = try!(f(self, u28));
                self.complexes[index] = value.clone();
                Ok(value)
            }
        }
    }
    fn decode_pairs(&mut self) -> DecodeResult<Vec<Pair<String, Value>>> {
        let mut pairs = Vec::new();
        loop {
            let key = try!(self.decode_utf8());
            if key.is_empty() {
                return Ok(pairs);
            }
            let value = try!(self.decode_value());
            pairs.push(Pair {
                key: key,
                value: value,
            });
        }
    }
    fn decode_trait(&mut self, u28: usize) -> DecodeResult<Trait> {
        if (u28 & 0b1) == 0 {
            let i = (u28 >> 1) as usize;
            let t = try!(self.traits.get(i).ok_or(DecodeError::OutOfRangeRference { index: i }));
            Ok(t.clone())
        } else if (u28 & 0b10) != 0 {
            let class_name = try!(self.decode_utf8());
            Err(DecodeError::ExternalizableType { name: class_name })
        } else {
            let is_dynamic = (u28 & 0b100) != 0;
            let field_num = u28 >> 3;
            let class_name = try!(self.decode_utf8());
            let fields = try!((0..field_num).map(|_| self.decode_utf8()).collect());

            let t = Trait {
                class_name: if class_name.is_empty() {
                    None
                } else {
                    Some(class_name)
                },
                is_dynamic: is_dynamic,
                fields: fields,
            };
            self.traits.push(t.clone());
            Ok(t)
        }
    }
    fn read_bytes(&mut self, len: usize) -> DecodeResult<Vec<u8>> {
        let mut buf = vec![0; len];
        try!(self.inner.read_exact(&mut buf));
        Ok(buf)
    }
    fn read_utf8(&mut self, len: usize) -> DecodeResult<String> {
        self.read_bytes(len).and_then(|b| Ok(try!(String::from_utf8(b))))
    }
}

#[cfg(test)]
mod test {
    use std::io;
    use std::f64;
    use std::time;
    use Pair;
    use error::DecodeError;
    use super::super::Value;

    macro_rules! decode {
        ($file:expr) => {
            {
                let input = include_bytes!(concat!("../testdata/", $file));
                Value::read_from(&mut &input[..])
            }
        }
    }
    macro_rules! decode_eq {
        ($file:expr, $expected: expr) => {
            {
                let value = decode!($file).unwrap();
                assert_eq!(value, $expected)
            }
        }
    }
    macro_rules! decode_unexpected_eof {
        ($file:expr) => {
            {
                let result = decode!($file);
                match result {
                    Err(DecodeError::Io(e)) => assert_eq!(e.kind(), io::ErrorKind::UnexpectedEof),
                    _ => assert!(false),
                }
            }
        }
    }

    #[test]
    fn decodes_undefined() {
        decode_eq!("amf3-undefined.bin", Value::Undefined);
    }
    #[test]
    fn decodes_null() {
        decode_eq!("amf3-null.bin", Value::Null);
    }
    #[test]
    fn decodes_boolean() {
        decode_eq!("amf3-true.bin", Value::Boolean(true));
        decode_eq!("amf3-false.bin", Value::Boolean(false));
    }
    #[test]
    fn decodes_integer() {
        decode_eq!("amf3-0.bin", Value::Integer(0));
        decode_eq!("amf3-min.bin", Value::Integer(-0x1000_0000));
        decode_eq!("amf3-max.bin", Value::Integer(0x0FFF_FFFF));
        decode_eq!("amf3-integer-2byte.bin", Value::Integer(0b10000000));
        decode_eq!("amf3-integer-3byte.bin", Value::Integer(0b100000000000000));
    }
    #[test]
    fn decodes_double() {
        decode_eq!("amf3-float.bin", Value::Double(3.5));
        decode_eq!("amf3-bignum.bin", Value::Double(2f64.powf(1000f64)));
        decode_eq!("amf3-large-min.bin", Value::Double(-0x1000_0001 as f64));
        decode_eq!("amf3-large-max.bin", Value::Double(0x1000_0000 as f64));
        decode_eq!("amf3-double-positive-infinity.bin",
                   Value::Double(f64::INFINITY));
    }
    #[test]
    fn decodes_string() {
        decode_eq!("amf3-string.bin", s("String . String"));
        decode_eq!("amf3-symbol.bin", s("foo"));
        decode_eq!("amf3-string-ref.bin",
                   dense_array(&[s("foo"),
                                 s("str"),
                                 s("foo"),
                                 s("str"),
                                 s("foo"),
                                 obj(&[("str", s("foo"))][..])][..]));
        decode_eq!("amf3-encoded-string-ref.bin",
                   dense_array(&[s("this is a テスト"), s("this is a テスト")][..]));
        decode_eq!("amf3-complex-encoded-string-array.bin",
                   dense_array(&[i(5), s("Shift テスト"), s("UTF テスト"), i(5)][..]));
        decode_eq!("amf3-empty-string-ref.bin",
                   dense_array(&[s(""), s("")][..]));
    }
    #[test]
    fn decodes_array() {
        decode_eq!("amf3-primitive-array.bin",
                   dense_array(&[i(1), i(2), i(3), i(4), i(5)][..]));
        decode_eq!("amf3-empty-array-ref.bin",
                   dense_array(&[dense_array(&[][..]),
                                 dense_array(&[][..]),
                                 dense_array(&[][..]),
                                 dense_array(&[][..])][..]));
        decode_eq!("amf3-array-ref.bin",
                   dense_array(&[dense_array(&[i(1),i(2),i(3)][..]),
                                 dense_array(&[s("a"),s("b"),s("c")][..]),
                                 dense_array(&[i(1),i(2),i(3)][..]),
                                 dense_array(&[s("a"),s("b"),s("c")][..])][..]));
        decode_eq!("amf3-associative-array.bin",
                   Value::Array {
                       assoc_entries: [("2", s("bar3")), ("foo", s("bar")), ("asdf", s("fdsa"))]
                           .iter()
                           .map(|e| pair(e.0, e.1.clone()))
                           .collect(),
                       dense_entries: vec![s("bar"), s("bar1"), s("bar2")],
                   });

        let o1 = obj(&[("foo_one", s("bar_one"))][..]);
        let o2 = obj(&[("foo_two", s(""))][..]);
        let o3 = obj(&[("foo_three", i(42))][..]);
        let empty = obj(&[][..]);
        decode_eq!("amf3-mixed-array.bin",
                   dense_array(&[o1.clone(),
                                 o2.clone(),
                                 o3.clone(),
                                 empty.clone(),
                                 dense_array(&[o1, o2, o3.clone()][..]),
                                 dense_array(&[][..]),
                                 i(42),
                                 s(""),
                                 dense_array(&[][..]),
                                 s(""),
                                 empty,
                                 s("bar_one"),
                                 o3][..]));
    }
    #[test]
    fn decodes_object() {
        let o = obj(&[("foo", s("bar"))][..]);
        decode_eq!("amf3-object-ref.bin",
                   dense_array(&[dense_array(&[o.clone(), o.clone()][..]),
                                 s("bar"),
                                 dense_array(&[o.clone(), o.clone()][..])][..]));

        decode_eq!("amf3-dynamic-object.bin",
                   obj(&[("property_one", s("foo")),
                         ("another_public_property", s("a_public_value")),
                         ("nil_property", Value::Null)][..]));

        decode_eq!("amf3-typed-object.bin",
                   typed_obj("org.amf.ASClass",
                             &[("foo", s("bar")), ("baz", Value::Null)][..]));

        let o = [typed_obj("org.amf.ASClass",
                           &[("foo", s("foo")), ("baz", Value::Null)]),
                 typed_obj("org.amf.ASClass",
                           &[("foo", s("bar")), ("baz", Value::Null)])];
        decode_eq!("amf3-trait-ref.bin", dense_array(&o[..]));

        decode_eq!("amf3-hash.bin",
                   obj(&[("foo", s("bar")), ("answer", i(42))][..]));

        assert_eq!(decode!("amf3-externalizable.bin"),
                   Err(DecodeError::ExternalizableType { name: "ExternalizableTest".to_string() }));
        assert_eq!(decode!("amf3-array-collection.bin"),
                   Err(DecodeError::ExternalizableType {
                       name: "flex.messaging.io.ArrayCollection".to_string(),
                   }));
    }
    #[test]
    fn decodes_xml_doc() {
        decode_eq!("amf3-xml-doc.bin",
                   Value::XmlDocument("<parent><child prop=\"test\" /></parent>".to_string()));
    }
    #[test]
    fn decodes_xml() {
        let xml = Value::Xml("<parent><child prop=\"test\"/></parent>".to_string());
        decode_eq!("amf3-xml.bin", xml);
        decode_eq!("amf3-xml-ref.bin", dense_array(&[xml.clone(), xml][..]));
    }
    #[test]
    fn decodes_byte_array() {
        decode_eq!("amf3-byte-array.bin",
                   Value::ByteArray(vec![0, 3, 227, 129, 147, 227, 130, 140, 116, 101, 115, 116,
                                         64]));

        let b = Value::ByteArray("ASDF".as_bytes().iter().cloned().collect());
        decode_eq!("amf3-byte-array-ref.bin", dense_array(&[b.clone(), b][..]));
    }
    #[test]
    fn decodes_date() {
        let d = Value::Date { unix_time: time::Duration::from_secs(0) };
        decode_eq!("amf3-date.bin", d);
        decode_eq!("amf3-date-ref.bin", dense_array(&[d.clone(), d][..]));
    }
    #[test]
    fn decodes_dictionary() {
        let entries = vec![(s("bar"), s("asdf1")),
                           (typed_obj("org.amf.ASClass",
                                      &[("foo", s("baz")), ("baz", Value::Null)][..]),
                            s("asdf2"))];
        decode_eq!("amf3-dictionary.bin", dic(&entries));
        decode_eq!("amf3-empty-dictionary.bin", dic(&[][..]));
    }
    #[test]
    fn decodes_vector() {
        decode_eq!("amf3-vector-int.bin",
                   Value::IntVector {
                       is_fixed: false,
                       entries: vec![4, -20, 12],
                   });

        decode_eq!("amf3-vector-uint.bin",
                   Value::UintVector {
                       is_fixed: false,
                       entries: vec![4, 20, 12],
                   });

        decode_eq!("amf3-vector-double.bin",
                   Value::DoubleVector {
                       is_fixed: false,
                       entries: vec![4.3, -20.6],
                   });

        let objects = vec![
            typed_obj("org.amf.ASClass", &[("foo", s("foo")), ("baz", Value::Null)][..]),
            typed_obj("org.amf.ASClass", &[("foo", s("bar")), ("baz", Value::Null)][..]),
            typed_obj("org.amf.ASClass", &[("foo", s("baz")), ("baz", Value::Null)][..]),
        ];
        decode_eq!("amf3-vector-object.bin",
                   Value::ObjectVector {
                       class_name: Some("org.amf.ASClass".to_string()),
                       is_fixed: false,
                       entries: objects,
                   });
    }
    #[test]
    fn other_errors() {
        assert_eq!(decode!("amf3-graph-member.bin"),
                   Err(DecodeError::CircularReference { index: 0 }));
        assert_eq!(decode!("amf3-bad-object-ref.bin"),
                   Err(DecodeError::OutOfRangeRference { index: 10 }));
        assert_eq!(decode!("amf3-bad-trait-ref.bin"),
                   Err(DecodeError::OutOfRangeRference { index: 4 }));
        assert_eq!(decode!("amf3-bad-string-ref.bin"),
                   Err(DecodeError::OutOfRangeRference { index: 8 }));
        assert_eq!(decode!("amf3-unknown-marker.bin"),
                   Err(DecodeError::Unknown { marker: 123 }));
        assert_eq!(decode!("amf3-date-invalid-millis.bin"),
                   Err(DecodeError::InvalidDate { millis: f64::INFINITY }));
        assert_eq!(decode!("amf3-date-minus-millis.bin"),
                   Err(DecodeError::InvalidDate { millis: -1.0 }));
        decode_unexpected_eof!("amf3-empty.bin");
        decode_unexpected_eof!("amf3-double-partial.bin");
        decode_unexpected_eof!("amf3-date-partial.bin");
        decode_unexpected_eof!("amf3-dictionary-partial.bin");
        decode_unexpected_eof!("amf3-vector-partial.bin");
        decode_unexpected_eof!("amf3-vector-int-partial.bin");
        decode_unexpected_eof!("amf3-vector-uint-partial.bin");
        decode_unexpected_eof!("amf3-xml-partial.bin");
        decode_unexpected_eof!("amf3-string-partial.bin");
        decode_unexpected_eof!("amf3-u29-partial.bin");
    }

    fn i(i: i32) -> Value {
        Value::Integer(i)
    }
    fn s(s: &str) -> Value {
        Value::String(s.to_string())
    }
    fn dense_array(entries: &[Value]) -> Value {
        Value::Array {
            assoc_entries: Vec::new(),
            dense_entries: entries.iter().cloned().collect(),
        }
    }
    fn pair(key: &str, value: Value) -> Pair<String, Value> {
        Pair {
            key: key.to_string(),
            value: value,
        }
    }
    fn dic(entries: &[(Value, Value)]) -> Value {
        Value::Dictionary {
            is_weak: false,
            entries: entries.iter()
                .map(|e| {
                    Pair {
                        key: e.0.clone(),
                        value: e.1.clone(),
                    }
                })
                .collect(),
        }
    }
    fn obj(entries: &[(&str, Value)]) -> Value {
        Value::Object {
            class_name: None,
            sealed_count: 0,
            entries: entries.iter()
                .map(|e| {
                    Pair {
                        key: e.0.to_string(),
                        value: e.1.clone(),
                    }
                })
                .collect(),
        }
    }
    fn typed_obj(class: &str, entries: &[(&str, Value)]) -> Value {
        Value::Object {
            class_name: Some(class.to_string()),
            sealed_count: entries.len(),
            entries: entries.iter()
                .map(|e| {
                    Pair {
                        key: e.0.to_string(),
                        value: e.1.clone(),
                    }
                })
                .collect(),
        }
    }
}
