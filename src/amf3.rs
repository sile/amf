use std::io;
use std::rc::Rc;
use byteorder::BigEndian;
use byteorder::ReadBytesExt;

const MARKER_UNDEFINED: u8 = 0x00;
const MARKER_NULL: u8 = 0x01;
const MARKER_FALSE: u8 = 0x02;
const MARKER_TRUE: u8 = 0x03;
const MARKER_INTEGER: u8 = 0x04;
const MARKER_DOUBLE: u8 = 0x05;
const MARKER_STRING: u8 = 0x06;
const MARKER_XML_DOC: u8 = 0x07;
const MARKER_DATE: u8 = 0x08;
const MARKER_ARRAY: u8 = 0x09;
const MARKER_OBJECT: u8 = 0x0A;
const MARKER_XML: u8 = 0x0B;
const MARKER_BYTE_ARRAY: u8 = 0x0C;
const MARKER_VECTOR_INT: u8 = 0x0D;
const MARKER_VECTOR_UINT: u8 = 0xE;
const MARKER_VECTOR_DOUBLE: u8 = 0x0F;
const MARKER_VECTOR_OBJECT: u8 = 0x10;
const MARKER_DICTIONARY: u8 = 0x11;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Undefined,
    Null,
    Bool(bool),
    Float(f64),
}

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
        self.decode_value()
    }
    fn decode_value(&mut self) -> io::Result<Value> {
        let marker = try!(self.inner.read_u8());
        match marker {
            MARKER_UNDEFINED => Ok(Value::Undefined),
            MARKER_NULL => Ok(Value::Null),
            MARKER_FALSE => Ok(Value::Bool(false)),
            MARKER_TRUE => Ok(Value::Bool(true)),
            MARKER_INTEGER => unimplemented!(),
            MARKER_DOUBLE => self.decode_double(),
            MARKER_STRING => unimplemented!(),
            MARKER_XML_DOC => unimplemented!(),
            MARKER_DATE => unimplemented!(),
            MARKER_ARRAY => unimplemented!(),
            MARKER_OBJECT => unimplemented!(),
            MARKER_XML => unimplemented!(),
            MARKER_BYTE_ARRAY => unimplemented!(),
            MARKER_VECTOR_INT => unimplemented!(),
            MARKER_VECTOR_UINT => unimplemented!(),
            MARKER_VECTOR_DOUBLE => unimplemented!(),
            MARKER_VECTOR_OBJECT => unimplemented!(),
            MARKER_DICTIONARY => unimplemented!(),
            _ => panic!(),
        }
    }
    fn decode_double(&mut self) -> io::Result<Value> {
        let n = try!(self.inner.read_f64::<BigEndian>());
        Ok(Value::Float(n))
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
    fn decodes_undefined() {
        let input = include_bytes!("testdata/amf3-undefined.bin");
        assert_eq!(decode_bytes(&input[..]).unwrap(), Value::Undefined);
    }
    #[test]
    fn decodes_null() {
        let input = include_bytes!("testdata/amf3-null.bin");
        assert_eq!(decode_bytes(&input[..]).unwrap(), Value::Null);
    }
    #[test]
    fn decodes_true() {
        let input = include_bytes!("testdata/amf3-true.bin");
        assert_eq!(decode_bytes(&input[..]).unwrap(), Value::Bool(true));
    }
    #[test]
    fn decodes_false() {
        let input = include_bytes!("testdata/amf3-false.bin");
        assert_eq!(decode_bytes(&input[..]).unwrap(), Value::Bool(false));
    }
    #[test]
    fn decodes_float() {
        let input = include_bytes!("testdata/amf3-float.bin");
        assert_eq!(decode_bytes(&input[..]).unwrap(), Value::Float(3.5));
    }
}
