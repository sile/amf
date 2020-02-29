//! An [AMF3](https://www.adobe.com/content/dam/acom/en/devnet/pdf/amf-file-format-spec.pdf) implementation.
//!
//! # Examples
//! ```
//! use amf::amf3::Value;
//!
//! // Encodes a AMF3's integer
//! let integer = Value::from(Value::Integer(123));
//! let mut buf = Vec::new();
//! integer.write_to(&mut buf).unwrap();
//!
//! // Decodes above integer
//! let decoded = Value::read_from(&mut &buf[..]).unwrap();
//! assert_eq!(integer, decoded);
//! ```
use crate::{DecodeResult, Pair};
use std::io;
use std::time;

pub use self::decode::Decoder;
pub use self::encode::Encoder;

mod decode;
mod encode;

mod marker {
    pub const UNDEFINED: u8 = 0x00;
    pub const NULL: u8 = 0x01;
    pub const FALSE: u8 = 0x02;
    pub const TRUE: u8 = 0x03;
    pub const INTEGER: u8 = 0x04;
    pub const DOUBLE: u8 = 0x05;
    pub const STRING: u8 = 0x06;
    pub const XML_DOC: u8 = 0x07;
    pub const DATE: u8 = 0x08;
    pub const ARRAY: u8 = 0x09;
    pub const OBJECT: u8 = 0x0A;
    pub const XML: u8 = 0x0B;
    pub const BYTE_ARRAY: u8 = 0x0C;
    pub const VECTOR_INT: u8 = 0x0D;
    pub const VECTOR_UINT: u8 = 0xE;
    pub const VECTOR_DOUBLE: u8 = 0x0F;
    pub const VECTOR_OBJECT: u8 = 0x10;
    pub const DICTIONARY: u8 = 0x11;
}

/// AMF3 value.
///
/// # Examples
/// ```
/// use amf::amf3::Value;
///
/// // Encodes a AMF3's integer
/// let integer = Value::from(Value::Integer(123));
/// let mut buf = Vec::new();
/// integer.write_to(&mut buf).unwrap();
///
/// // Decodes above integer
/// let decoded = Value::read_from(&mut &buf[..]).unwrap();
/// assert_eq!(integer, decoded);
/// ```
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    /// See [3.2 undefined Type]
    /// (https://www.adobe.com/content/dam/acom/en/devnet/pdf/amf-file-format-spec.pdf#page=6&zoom=auto,88,264).
    Undefined,

    /// See [3.3 null Type]
    /// (https://www.adobe.com/content/dam/acom/en/devnet/pdf/amf-file-format-spec.pdf#page=6&zoom=auto,88,139).
    Null,

    /// See [3.4 false Type]
    /// (https://www.adobe.com/content/dam/acom/en/devnet/pdf/amf-file-format-spec.pdf#page=7&zoom=auto,88,694)
    /// and
    /// [3.5 true Type]
    /// (https://www.adobe.com/content/dam/acom/en/devnet/pdf/amf-file-format-spec.pdf#page=7&zoom=auto,88,596).
    Boolean(bool),

    /// See [3.6 integer Type]
    /// (https://www.adobe.com/content/dam/acom/en/devnet/pdf/amf-file-format-spec.pdf#page=7&zoom=auto,88,499).
    Integer(i32),

    /// See [3.7 double Type]
    /// (https://www.adobe.com/content/dam/acom/en/devnet/pdf/amf-file-format-spec.pdf#page=7&zoom=auto,88,321).
    Double(f64),

    /// See [3.8 String Type]
    /// (https://www.adobe.com/content/dam/acom/en/devnet/pdf/amf-file-format-spec.pdf#page=7&zoom=auto,88,196).
    String(String),

    /// See [3.9 XMLDocument Type]
    /// (https://www.adobe.com/content/dam/acom/en/devnet/pdf/amf-file-format-spec.pdf#page=8&zoom=auto,88,639).
    XmlDocument(String),

    /// See [3.10 Date Type]
    /// (https://www.adobe.com/content/dam/acom/en/devnet/pdf/amf-file-format-spec.pdf#page=8&zoom=auto,88,316).
    Date {
        /// Unix timestamp with milliseconds precision.
        unix_time: time::Duration,
    },

    /// See [3.11 Array Type]
    /// (https://www.adobe.com/content/dam/acom/en/devnet/pdf/amf-file-format-spec.pdf#page=9&zoom=auto,88,720).
    Array {
        /// Entries of the associative part of the array.
        assoc_entries: Vec<Pair<String, Value>>,

        /// Entries of the dense part of the array.
        dense_entries: Vec<Value>,
    },

    /// See [3.12 Object Type]
    /// (https://www.adobe.com/content/dam/acom/en/devnet/pdf/amf-file-format-spec.pdf#page=9&zoom=auto,88,275).
    Object {
        /// The class name of the object.
        /// `None` means it is an anonymous object.
        class_name: Option<String>,

        /// Sealed member count of the object.
        ///
        /// Sealed members are located in front of the `entries`.
        sealed_count: usize,

        /// Members of the object.
        entries: Vec<Pair<String, Value>>,
    },

    /// See [3.13 XML Type]
    /// (https://www.adobe.com/content/dam/acom/en/devnet/pdf/amf-file-format-spec.pdf#page=11&zoom=auto,88,360).
    Xml(String),

    /// See [3.14 ByteArray Type]
    /// (https://www.adobe.com/content/dam/acom/en/devnet/pdf/amf-file-format-spec.pdf#page=11&zoom=auto,88,167).
    ByteArray(Vec<u8>),

    /// See [3.15 Vector Type]
    /// (https://www.adobe.com/content/dam/acom/en/devnet/pdf/amf-file-format-spec.pdf#page=12&zoom=auto,88,534).
    IntVector {
        /// If `true`, this is a fixed-length vector.
        is_fixed: bool,

        /// The entries of the vector.
        entries: Vec<i32>,
    },

    /// See [3.15 Vector Type]
    /// (https://www.adobe.com/content/dam/acom/en/devnet/pdf/amf-file-format-spec.pdf#page=12&zoom=auto,88,534).
    UintVector {
        /// If `true`, this is a fixed-length vector.
        is_fixed: bool,

        /// The entries of the vector.
        entries: Vec<u32>,
    },

    /// See [3.15 Vector Type]
    /// (https://www.adobe.com/content/dam/acom/en/devnet/pdf/amf-file-format-spec.pdf#page=12&zoom=auto,88,534).
    DoubleVector {
        /// If `true`, this is a fixed-length vector.
        is_fixed: bool,

        /// The entries of the vector.
        entries: Vec<f64>,
    },

    /// See [3.15 Vector Type]
    /// (https://www.adobe.com/content/dam/acom/en/devnet/pdf/amf-file-format-spec.pdf#page=12&zoom=auto,88,534).
    ObjectVector {
        /// The base type name of entries in the vector.
        /// `None` means it is the ANY type.
        class_name: Option<String>,

        /// If `true`, this is a fixed-length vector.
        is_fixed: bool,

        /// The entries of the vector.
        entries: Vec<Value>,
    },

    /// See [3.16 Dictionary Type]
    /// (https://www.adobe.com/content/dam/acom/en/devnet/pdf/amf-file-format-spec.pdf#page=13&zoom=auto,88,601).
    Dictionary {
        /// If `true`, the keys of `entries` are weakly referenced.
        is_weak: bool,

        /// The entries of the dictionary.
        entries: Vec<Pair<Value, Value>>,
    },
}
impl Value {
    /// Reads an AMF3 encoded `Value` from `reader`.
    ///
    /// Note that reference objects are copied in the decoding phase
    /// for the sake of simplicity of the resulting value representation.
    /// And circular reference are unsupported (i.e., those are treated as errors).
    pub fn read_from<R>(reader: R) -> DecodeResult<Self>
    where
        R: io::Read,
    {
        Decoder::new(reader).decode()
    }

    /// Writes the AMF3 encoded bytes of this value to `writer`.
    pub fn write_to<W>(&self, writer: W) -> io::Result<()>
    where
        W: io::Write,
    {
        Encoder::new(writer).encode(self)
    }

    /// Tries to convert the value as a `str` reference.
    pub fn try_as_str(&self) -> Option<&str> {
        match *self {
            Value::String(ref x) => Some(x.as_str()),
            Value::XmlDocument(ref x) => Some(x.as_str()),
            Value::Xml(ref x) => Some(x.as_str()),
            _ => None,
        }
    }

    /// Tries to convert the value as a `f64`.
    pub fn try_as_f64(&self) -> Option<f64> {
        match *self {
            Value::Integer(x) => Some(x as f64),
            Value::Double(x) => Some(x),
            _ => None,
        }
    }

    /// Tries to convert the value as an iterator of the contained values.
    pub fn try_into_values(self) -> Result<Box<dyn Iterator<Item = Value>>, Self> {
        match self {
            Value::Array { dense_entries, .. } => Ok(Box::new(dense_entries.into_iter())),
            Value::IntVector { entries, .. } => {
                Ok(Box::new(entries.into_iter().map(Value::Integer)))
            }
            Value::UintVector { entries, .. } => Ok(Box::new(
                entries.into_iter().map(|n| Value::Double(n as f64)),
            )),
            Value::DoubleVector { entries, .. } => {
                Ok(Box::new(entries.into_iter().map(Value::Double)))
            }
            Value::ObjectVector { entries, .. } => Ok(Box::new(entries.into_iter())),
            _ => Err(self),
        }
    }

    /// Tries to convert the value as an iterator of the contained pairs.
    pub fn try_into_pairs(self) -> Result<Box<dyn Iterator<Item = (String, Value)>>, Self> {
        match self {
            Value::Array { assoc_entries, .. } => Ok(Box::new(
                assoc_entries.into_iter().map(|p| (p.key, p.value)),
            )),
            Value::Object { entries, .. } => {
                Ok(Box::new(entries.into_iter().map(|p| (p.key, p.value))))
            }
            _ => Err(self),
        }
    }
}
