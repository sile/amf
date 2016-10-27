extern crate byteorder;

use std::io;

pub mod amf0;
pub mod amf3;
pub mod error;

pub type DecodeResult<T> = Result<T, error::DecodeError>;
pub type Amf0Value = amf0::Value;
pub type Amf3Value = amf3::Value;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum Version {
    Amf0,
    Amf3,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Amf0(Amf0Value),
    Amf3(Amf3Value),
}
impl Value {
    pub fn read_from<R>(reader: R, version: Version) -> DecodeResult<Self>
        where R: io::Read
    {
        match version {
            Version::Amf0 => Amf0Value::read_from(reader).map(Value::Amf0),
            Version::Amf3 => Amf3Value::read_from(reader).map(Value::Amf3),
        }
    }
    pub fn write_to<W>(&self, writer: W) -> io::Result<()>
        where W: io::Write
    {
        match *self {
            Value::Amf0(ref x) => x.write_to(writer),
            Value::Amf3(ref x) => x.write_to(writer),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pair<K, V> {
    pub key: K,
    pub value: V,
}
