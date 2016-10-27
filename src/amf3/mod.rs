use std::time;

use Pair;

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

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Undefined,
    Null,
    Boolean(bool),
    Integer(i32),
    Double(f64),
    String(String),
    XmlDocument(String),
    Date { unix_time: time::Duration },
    Array {
        assoc_entries: Vec<Pair<String, Value>>,
        dense_entries: Vec<Value>,
    },
    Object {
        class_name: Option<String>,
        sealed_count: usize,
        entries: Vec<Pair<String, Value>>,
    },
    Xml(String),
    ByteArray(Vec<u8>),
    IntVector { is_fixed: bool, entries: Vec<i32> },
    UintVector { is_fixed: bool, entries: Vec<u32> },
    DoubleVector { is_fixed: bool, entries: Vec<f64> },
    ObjectVector {
        class_name: Option<String>,
        is_fixed: bool,
        entries: Vec<Value>,
    },
    Dictionary {
        is_weak: bool,
        entries: Vec<Pair<Value, Value>>,
    },
}
