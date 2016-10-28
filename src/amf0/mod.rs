//! An [AMF0](http://download.macromedia.com/pub/labs/amf/amf0_spec_121207.pdf) implementation.
//!
//! # Examples
//! ```
//! use amf::amf0::Value;
//!
//! // Encodes a AMF3's number
//! let number = Value::from(Value::Number(12.3));
//! let mut buf = Vec::new();
//! number.write_to(&mut buf).unwrap();
//!
//! // Decodes above number
//! let decoded = Value::read_from(&mut &buf[..]).unwrap();
//! assert_eq!(number, decoded);
//! ```
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

/// AMF0 value.
///
/// # Examples
/// ```
/// use amf::amf0::Value;
///
/// // Encodes a AMF3's number
/// let number = Value::from(Value::Number(12.3));
/// let mut buf = Vec::new();
/// number.write_to(&mut buf).unwrap();
///
/// // Decodes above number
/// let decoded = Value::read_from(&mut &buf[..]).unwrap();
/// assert_eq!(number, decoded);
/// ```
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    /// See [2.2 Number Type]
    /// (http://download.macromedia.com/pub/labs/amf/amf0_spec_121207.pdf#page=5&zoom=auto,90,667).
    Number(f64),

    /// See [2.3 Boolean Type]
    /// (http://download.macromedia.com/pub/labs/amf/amf0_spec_121207.pdf#page=5&zoom=auto,90,569).
    Boolean(bool),

    /// See [2.4 String Type]
    /// (http://download.macromedia.com/pub/labs/amf/amf0_spec_121207.pdf#page=5&zoom=auto,90,432)
    /// and
    /// [2.14 Long String Type]
    /// (http://download.macromedia.com/pub/labs/amf/amf0_spec_121207.pdf#page=7&zoom=auto,90,360).
    String(String),

    /// See [2.5 Object Type]
    /// (http://download.macromedia.com/pub/labs/amf/amf0_spec_121207.pdf#page=5&zoom=auto,90,320)
    /// and
    /// [2.18 Typed Object Type]
    /// (http://download.macromedia.com/pub/labs/amf/amf0_spec_121207.pdf#page=8&zoom=auto,90,682).
    Object {
        /// The class name of the object.
        /// `None` means it is an anonymous object.
        class_name: Option<String>,

        /// Properties of the object.
        entries: Vec<Pair<String, Value>>,
    },

    /// See [2.7 null Type]
    /// (http://download.macromedia.com/pub/labs/amf/amf0_spec_121207.pdf#page=6&zoom=auto,90,720).
    Null,

    /// See [2.8 undefined Type]
    /// (http://download.macromedia.com/pub/labs/amf/amf0_spec_121207.pdf#page=6&zoom=auto,90,637).
    Undefined,

    /// See [2.10 ECMA Array Type]
    /// (http://download.macromedia.com/pub/labs/amf/amf0_spec_121207.pdf#page=6&zoom=auto,90,349).
    EcmaArray {
        /// Entries of the associative array.
        entries: Vec<Pair<String, Value>>,
    },

    /// [2.12 Strict Array Type]
    /// (http://download.macromedia.com/pub/labs/amf/amf0_spec_121207.pdf#page=7&zoom=auto,90,684)
    Array {
        /// Entries of the array.
        entries: Vec<Value>,
    },

    /// See [2.13 Date Type]
    /// (http://download.macromedia.com/pub/labs/amf/amf0_spec_121207.pdf#page=7&zoom=auto,90,546).
    Date {
        /// Unix timestamp with milliseconds precision.
        unix_time: time::Duration,
    },

    /// See [2.17 XML Document Type]
    /// (http://download.macromedia.com/pub/labs/amf/amf0_spec_121207.pdf#page=7&zoom=auto,90,147).
    XmlDocument(String),

    /// See [3.1 AVM+ Type Marker]
    /// (http://download.macromedia.com/pub/labs/amf/amf0_spec_121207.pdf#page=8&zoom=auto,90,518).
    AvmPlus(amf3::Value),
}
impl Value {
    /// Reads an AMF0 encoded `Value` from `reader`.
    ///
    /// Note that reference objects are copied in the decoding phase
    /// for the sake of simplicity of the resulting value representation.
    /// And circular reference are unsupported (i.e., those are treated as errors).
    pub fn read_from<R>(reader: R) -> DecodeResult<Self>
        where R: io::Read
    {
        Decoder::new(reader).decode()
    }

    /// Writes the AMF0 encoded bytes of this value to `writer`.
    pub fn write_to<W>(&self, writer: W) -> io::Result<()>
        where W: io::Write
    {
        Encoder::new(writer).encode(self)
    }
}
