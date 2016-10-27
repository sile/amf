use std::io;
use std::fmt;
use std::error;
use std::string;

#[derive(Debug)]
pub enum DecodeError {
    Io(io::Error),
    String(string::FromUtf8Error),
    Unknown { marker: u8 },
    Unsupported { marker: u8 },
    UnexpectedObjectEnd,
    CircularReference { index: usize },
    OutOfRangeRference { index: usize },
    NonZeroTimeZone { offset: i16 },
    InvalidDate { millis: f64 },
    ExternalizableType { name: String },
}
impl error::Error for DecodeError {
    fn description(&self) -> &str {
        use self::DecodeError::*;
        match *self {
            Io(ref x) => x.description(),
            String(ref x) => x.description(),
            Unknown { .. } => "Unknown marker",
            Unsupported { .. } => "Unsupported type",
            UnexpectedObjectEnd => "Unexpected occurrence of object-end-marker",
            CircularReference { .. } => "Circular reference",
            OutOfRangeRference { .. } => "Out-of-range reference index",
            NonZeroTimeZone { .. } => "Non zero time zone",
            InvalidDate { .. } => "Invalid date",
            ExternalizableType { .. } => "Unsupported externalizable type",
        }
    }
    fn cause(&self) -> Option<&error::Error> {
        use self::DecodeError::*;
        match *self {
            Io(ref x) => x.cause(),
            String(ref x) => x.cause(),
            _ => None,
        }
    }
}
impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::DecodeError::*;
        match *self {
            Io(ref x) => write!(f, "I/O Error: {}", x),
            String(ref x) => write!(f, "Invalid String: {}", x),
            Unknown { marker } => write!(f, "Unknown marker: {}", marker),
            Unsupported { marker } => write!(f, "Unsupported type: maker={}", marker),
            UnexpectedObjectEnd => write!(f, "Unexpected occurrence of object-end-marker"),
            CircularReference { index } => {
                write!(f, "Circular references are unsupported: index={}", index)
            }
            OutOfRangeRference { index } => write!(f, "Reference index {} is out-of-range", index),
            NonZeroTimeZone { offset } => {
                write!(f, "Non zero time zone offset {} is unsupported", offset)
            }
            InvalidDate { millis } => write!(f, "Invalid date value {}", millis),
            ExternalizableType { ref name } => {
                write!(f, "Externalizable type {:?} is unsupported", name)
            }
        }
    }
}
impl From<io::Error> for DecodeError {
    fn from(f: io::Error) -> Self {
        DecodeError::Io(f)
    }
}
impl From<string::FromUtf8Error> for DecodeError {
    fn from(f: string::FromUtf8Error) -> Self {
        DecodeError::String(f)
    }
}
