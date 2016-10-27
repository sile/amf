extern crate byteorder;

use std::io;

pub mod amf0;
pub mod amf3;
pub mod error;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum Version {
    Amf0,
    Amf3,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Amf0(amf0::Value),
    Amf3(amf3::Value),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pair<K, V> {
    pub key: K,
    pub value: V,
}

pub type DecodeResult<T> = Result<T, error::DecodeError>;

#[derive(Debug)]
pub enum Decoder<R> {
    Amf0(amf0::Decoder<R>),
    Amf3(amf3::Decoder<R>),
}
impl<R> Decoder<R>
    where R: io::Read
{
    pub fn new(inner: R, version: Version) -> Self {
        match version {
            Version::Amf0 => Decoder::Amf0(amf0::Decoder::new(inner)),
            Version::Amf3 => Decoder::Amf3(amf3::Decoder::new(inner)),
        }
    }
    pub fn decode(&mut self) -> DecodeResult<Value> {
        match *self {
            Decoder::Amf0(ref mut x) => x.decode().map(Value::Amf0),
            Decoder::Amf3(ref mut x) => x.decode().map(Value::Amf3),
        }
    }
}
impl<R> Decoder<R> {
    pub fn into_inner(self) -> R {
        match self {
            Decoder::Amf0(x) => x.into_inner(),
            Decoder::Amf3(x) => x.into_inner(),
        }
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
