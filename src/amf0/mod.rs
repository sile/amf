use std::io;
use std::time;

use amf3;
use Pair;
use DecodeResult;

pub use self::decode::Decoder;
pub use self::encode::Encoder;

mod decode;
mod encode;

mod marker {
    pub const NUMBER: u8 = 0x00;
    pub const BOOLEAN: u8 = 0x01;
    pub const STRING: u8 = 0x02;
    pub const OBJECT: u8 = 0x03;
    pub const MOVIECLIP: u8 = 0x04; // reserved, not supported
    pub const NULL: u8 = 0x05;
    pub const UNDEFINED: u8 = 0x06;
    pub const REFERENCE: u8 = 0x07;
    pub const ECMA_ARRAY: u8 = 0x08;
    pub const OBJECT_END_MARKER: u8 = 0x09;
    pub const STRICT_ARRAY: u8 = 0x0A;
    pub const DATE: u8 = 0x0B;
    pub const LONG_STRING: u8 = 0x0C;
    pub const UNSUPPORTED: u8 = 0x0D;
    pub const RECORDSET: u8 = 0x0E; // reserved, not supported
    pub const XML_DOCUMENT: u8 = 0x0F;
    pub const TYPED_OBJECT: u8 = 0x10;
    pub const AVMPLUS_OBJECT: u8 = 0x11;
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Null,
    Undefined,
    Boolean(bool),
    String(String),
    XmlDocument(String),
    Number(f64),
    Date { unix_time: time::Duration },
    Array { entries: Vec<Value> },
    EcmaArray { entries: Vec<Pair<String, Value>> },
    Object {
        class_name: Option<String>,
        entries: Vec<Pair<String, Value>>,
    },
    AvmPlus(amf3::Value),
}
impl Value {
    pub fn read_from<R>(reader: R) -> DecodeResult<Self>
        where R: io::Read
    {
        Decoder::new(reader).decode()
    }
    pub fn write_to<W>(&self, writer: W) -> io::Result<()>
        where W: io::Write
    {
        Encoder::new(writer).encode(self)
    }
}
