//! A Rust Implementation of AMF (Action Media Format).
//!
//! # Examples
//! ```
//! use amf::{Value, Amf0Value, Version};
//!
//! // Encodes a AMF0's number
//! let number = Value::from(Amf0Value::Number(1.23));
//! let mut buf = Vec::new();
//! number.write_to(&mut buf).unwrap();
//!
//! // Decodes above number
//! let decoded = Value::read_from(&mut &buf[..], Version::Amf0).unwrap();
//! assert_eq!(number, decoded);
//! ```
//!
//! # References
//! - [AMF0 Specification](http://download.macromedia.com/pub/labs/amf/amf0_spec_121207.pdf)
//! - [AMF3 Specification](http://download.macromedia.com/pub/labs/amf/amf3_spec_121207.pdf)
#![warn(missing_docs)]
extern crate byteorder;

use std::io;

pub use amf0::Value as Amf0Value;
pub use amf3::Value as Amf3Value;

pub mod amf0;
pub mod amf3;
pub mod error;

/// AMF decoding result.
pub type DecodeResult<T> = Result<T, error::DecodeError>;

/// Format version.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum Version {
    /// Version 0.
    Amf0,

    /// Version 3.
    Amf3,
}

/// AMF value.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    /// AMF0 value.
    Amf0(Amf0Value),

    /// AMF3 value.
    Amf3(Amf3Value),
}
impl Value {
    /// Reads an AMF encoded `Value` from `reader`.
    ///
    /// Note that reference objects are copied in the decoding phase
    /// for the sake of simplicity of the resulting value representation.
    /// And circular reference are unsupported (i.e., those are treated as errors).
    pub fn read_from<R>(reader: R, version: Version) -> DecodeResult<Self>
        where R: io::Read
    {
        match version {
            Version::Amf0 => Amf0Value::read_from(reader).map(Value::Amf0),
            Version::Amf3 => Amf3Value::read_from(reader).map(Value::Amf3),
        }
    }

    /// Writes the AMF encoded bytes of this value to `writer`.
    pub fn write_to<W>(&self, writer: W) -> io::Result<()>
        where W: io::Write
    {
        match *self {
            Value::Amf0(ref x) => x.write_to(writer),
            Value::Amf3(ref x) => x.write_to(writer),
        }
    }
}
impl From<Amf0Value> for Value {
    fn from(f: Amf0Value) -> Value {
        Value::Amf0(f)
    }
}
impl From<Amf3Value> for Value {
    fn from(f: Amf3Value) -> Value {
        Value::Amf3(f)
    }
}

/// Key-value pair.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pair<K, V> {
    /// The key of the pair.
    pub key: K,

    /// The value of the pair.
    pub value: V,
}
