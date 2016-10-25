use std::io;
use byteorder::BigEndian;
use byteorder::ReadBytesExt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Undefined,
    Bool(bool),
    Str(Vec<u8>),
    Xml(Vec<u8>),
    Number(f64),
    Object(Vec<(Vec<u8>, Value)>),
    Array(Vec<Value>),
    EcmaArray(Vec<(Vec<u8>, Value)>),
    TypedObject(TypedObject),

    // TODO: remove
    ObjectEnd,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypedObject {
    pub type_name: Vec<u8>,
    pub members: Vec<(Vec<u8>, Value)>,
}

const MARKER_NUMBER: u8 = 0x00;
const MARKER_BOOLEAN: u8 = 0x01;
const MARKER_STRING: u8 = 0x02;
const MARKER_OBJECT: u8 = 0x03;
const MARKER_MOVIECLIP: u8 = 0x04; // reserved, not supported
const MARKER_NULL: u8 = 0x05;
const MARKER_UNDEFINED: u8 = 0x06;
const MARKER_REFERENCE: u8 = 0x07;
const MARKER_ECMA_ARRAY: u8 = 0x08;
const MARKER_OBJECT_END_MARKER: u8 = 0x09;
const MARKER_STRICT_ARRAY: u8 = 0x0A;
const MARKER_DATE: u8 = 0x0B;
const MARKER_LONG_STRING: u8 = 0x0C;
const MARKER_UNSUPPORTED: u8 = 0x0D;
const MARKER_RECORDSET: u8 = 0x0E; // reserved, not supported
const MARKER_XML_DOCUMENT: u8 = 0x0F;
const MARKER_TYPED_OBJECT: u8 = 0x10;
const MARKER_AVMPLUS_OBJECT: u8 = 0x11;

#[derive(Debug)]
pub struct Decoder<R> {
    inner: R,
}
impl<R> Decoder<R>
    where R: io::Read
{
    pub fn new(inner: R) -> Self {
        Decoder { inner: inner }
    }
    pub fn into_inner(self) -> R {
        self.inner
    }
    pub fn decode(&mut self) -> io::Result<Value> {
        let marker = try!(self.inner.read_u8());
        match marker {
            MARKER_NUMBER => self.decode_number(),
            MARKER_BOOLEAN => self.decode_boolean(),
            MARKER_STRING => self.decode_string(),
            MARKER_OBJECT => self.decode_object(),
            MARKER_MOVIECLIP => unimplemented!(),
            MARKER_NULL => Ok(Value::Null),
            MARKER_UNDEFINED => Ok(Value::Undefined),
            MARKER_REFERENCE => unimplemented!(),
            MARKER_ECMA_ARRAY => self.decode_ecma_array(),
            MARKER_OBJECT_END_MARKER => Ok(Value::ObjectEnd),
            MARKER_STRICT_ARRAY => self.decode_strict_array(),
            MARKER_DATE => unimplemented!(),
            MARKER_LONG_STRING => self.decode_long_string(),
            MARKER_UNSUPPORTED => unimplemented!(),
            MARKER_RECORDSET => unimplemented!(),
            MARKER_XML_DOCUMENT => self.decode_xml_document(),
            MARKER_TYPED_OBJECT => self.decode_typed_object(),
            MARKER_AVMPLUS_OBJECT => unimplemented!(),
            _ => panic!("Unknown marker: {}", marker),
        }
    }
    fn decode_boolean(&mut self) -> io::Result<Value> {
        match try!(self.inner.read_u8()) {
            0 => Ok(Value::Bool(false)),
            1 => Ok(Value::Bool(true)),
            _ => panic!(),
        }
    }
    fn decode_number(&mut self) -> io::Result<Value> {
        let n = try!(self.inner.read_f64::<BigEndian>());
        Ok(Value::Number(n))
    }
    fn decode_string(&mut self) -> io::Result<Value> {
        let len = try!(self.inner.read_u16::<BigEndian>()) as usize;
        let mut buf = vec![0; len];
        try!(self.inner.read_exact(&mut buf));
        Ok(Value::Str(buf))
    }
    fn decode_long_string(&mut self) -> io::Result<Value> {
        let len = try!(self.inner.read_u32::<BigEndian>()) as usize;
        let mut buf = vec![0; len];
        try!(self.inner.read_exact(&mut buf));
        Ok(Value::Str(buf))
    }
    fn decode_xml_document(&mut self) -> io::Result<Value> {
        let len = try!(self.inner.read_u32::<BigEndian>()) as usize;
        let mut buf = vec![0; len];
        try!(self.inner.read_exact(&mut buf));
        Ok(Value::Xml(buf))
    }
    fn decode_object(&mut self) -> io::Result<Value> {
        let pairs = try!(self.decode_pairs());
        Ok(Value::Object(pairs))
    }
    fn decode_typed_object(&mut self) -> io::Result<Value> {
        let len = try!(self.inner.read_u16::<BigEndian>()) as usize;
        let mut type_name = vec![0; len];
        try!(self.inner.read_exact(&mut type_name));
        let pairs = try!(self.decode_pairs());
        Ok(Value::TypedObject(TypedObject {
            type_name: type_name,
            members: pairs,
        }))
    }
    fn decode_ecma_array(&mut self) -> io::Result<Value> {
        let _count = try!(self.inner.read_u32::<BigEndian>());
        let pairs = try!(self.decode_pairs());
        Ok(Value::EcmaArray(pairs))
    }
    fn decode_strict_array(&mut self) -> io::Result<Value> {
        let count = try!(self.inner.read_u32::<BigEndian>()) as usize;
        let mut v = Vec::with_capacity(count);
        for _ in 0..count {
            v.push(try!(self.decode()));
        }
        Ok(Value::Array(v))
    }
    fn decode_pairs(&mut self) -> io::Result<Vec<(Vec<u8>, Value)>> {
        let mut pairs = Vec::new();
        loop {
            let len = try!(self.inner.read_u16::<BigEndian>()) as usize;
            let mut key = vec![0; len];
            try!(self.inner.read_exact(&mut key));
            let value = try!(self.decode());
            if value == Value::ObjectEnd {
                break;
            }
            pairs.push((key, value));
        }
        Ok(pairs)
    }
}

pub fn decode_bytes(mut bytes: &[u8]) -> io::Result<Value> {
    let mut decoder = Decoder::new(&mut bytes);
    decoder.decode()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn decodes_boolean() {
        let input = include_bytes!("testdata/amf0-boolean-true.bin");
        assert_eq!(decode_bytes(&input[..]).unwrap(), Value::Bool(true));

        let input = include_bytes!("testdata/amf0-boolean-false.bin");
        assert_eq!(decode_bytes(&input[..]).unwrap(), Value::Bool(false));
    }
    #[test]
    fn decodes_null() {
        let input = include_bytes!("testdata/amf0-null.bin");
        assert_eq!(decode_bytes(&input[..]).unwrap(), Value::Null);
    }
    #[test]
    fn decodes_undefined() {
        let input = include_bytes!("testdata/amf0-undefined.bin");
        assert_eq!(decode_bytes(&input[..]).unwrap(), Value::Undefined);
    }
    #[test]
    fn decodes_number() {
        let input = include_bytes!("testdata/amf0-number.bin");
        assert_eq!(decode_bytes(&input[..]).unwrap(), Value::Number(3.5));
    }
    #[test]
    fn decodes_string() {
        let input = include_bytes!("testdata/amf0-string.bin");
        assert_eq!(decode_bytes(&input[..]).unwrap(),
                   Value::Str("this is a テスト".as_bytes().iter().cloned().collect()));
    }
    #[test]
    fn decodes_long_string() {
        let input = include_bytes!("testdata/amf0-long-string.bin");
        assert_eq!(decode_bytes(&input[..]).unwrap(),
                   Value::Str(vec![b'a'; 0x10013]));
    }
    #[test]
    fn decodes_xml_document() {
        let input = include_bytes!("testdata/amf0-xml-doc.bin");
        assert_eq!(decode_bytes(&input[..]).unwrap(),
                   Value::Xml(to_vec(b"<parent><child prop=\"test\" /></parent>")));
    }
    #[test]
    fn decodes_object() {
        let input = include_bytes!("testdata/amf0-object.bin");
        assert_eq!(decode_bytes(&input[..]).unwrap(),
                   Value::Object([(vec![], Value::Str(vec![])),
                                  (to_vec(b"foo"), Value::Str(to_vec(b"baz"))),
                                  (to_vec(b"bar"), Value::Number(3.14))]
                       .iter()
                       .cloned()
                       .collect()));
    }
    #[test]
    fn decodes_typed_object() {
        let input = include_bytes!("testdata/amf0-typed-object.bin");
        assert_eq!(decode_bytes(&input[..]).unwrap(),
                   Value::TypedObject(TypedObject {
                       type_name: to_vec(b"org.amf.ASClass"),
                       members:
                           Vec::from(&[(to_vec(b"foo"),
                                        Value::Str(to_vec(b"bar"))),
                                       (to_vec(b"baz"),
                                        Value::Null)][..]),
                   }));
    }
    #[test]
    fn decodes_ecma_array() {
        let input = include_bytes!("testdata/amf0-ecma-ordinal-array.bin");
        assert_eq!(decode_bytes(&input[..]).unwrap(),
                   Value::EcmaArray([(to_vec(b"0"), Value::Str(to_vec(b"a"))),
                                     (to_vec(b"1"), Value::Str(to_vec(b"b"))),
                                     (to_vec(b"2"), Value::Str(to_vec(b"c"))),
                                     (to_vec(b"3"), Value::Str(to_vec(b"d")))]
                       .iter()
                       .cloned()
                       .collect()));
    }
    #[test]
    fn decodes_strict_array() {
        let input = include_bytes!("testdata/amf0-strict-array.bin");
        assert_eq!(decode_bytes(&input[..]).unwrap(),
                   Value::Array(Vec::from(&[Value::Number(1.0),
                                            Value::Str(to_vec(b"2")),
                                            Value::Number(3.0)][..])));
    }

    fn to_vec(bytes: &[u8]) -> Vec<u8> {
        Vec::from(bytes)
    }
}
