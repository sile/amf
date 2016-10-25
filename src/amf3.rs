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
    Null,
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
            MARKER_UNDEFINED => unimplemented!(),
            MARKER_NULL => unimplemented!(),
            MARKER_FALSE => unimplemented!(),
            MARKER_TRUE => unimplemented!(),
            MARKER_INTEGER => unimplemented!(),
            MARKER_DOUBLE => unimplemented!(),
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {}
}
