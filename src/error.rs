//! AMF error.
use std::io;
use std::fmt;
use std::error;
use std::string;

/// AMF Decoding Error.
#[derive(Debug)]
pub enum DecodeError {
    /// I/O error.
    Io(io::Error),

    /// Invalid UTF-8 error.
    String(string::FromUtf8Error),

    /// Unknown marker.
    Unknown {
        /// Unknown marker.
        marker: u8,
    },

    /// Unsupported type.
    Unsupported {
        /// The marker of the unsupported type.
        marker: u8,
    },

    /// Unexpected object end marker (only AMF0).
    UnexpectedObjectEnd,

    /// Circular reference.
    ///
    /// Note that circular references are allowed in the specification,
    /// but limited by the current implementation.
    CircularReference {
        /// Circular reference index.
        index: usize,
    },

    /// Out-of-range reference index.
    OutOfRangeReference {
        /// Out-of-range index.
        index: usize,
    },

    /// Unsupported non-zero time zone (only AMF0).
    NonZeroTimeZone {
        /// Time zone offset (non zero).
        offset: i16,
    },

    /// Invalid unix-time.
    InvalidDate {
        /// Invalid unix-time (e.g., infiniy, minus).
        millis: f64,
    },

    /// Unsupported externalizable type.
    ExternalizableType {
        /// The name of the externalizable type.
        name: String,
    },
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
            OutOfRangeReference { .. } => "Out-of-range reference index",
            NonZeroTimeZone { .. } => "Non zero time zone",
            InvalidDate { .. } => "Invalid date",
            ExternalizableType { .. } => "Unsupported externalizable type",
        }
    }
    fn cause(&self) -> Option<&dyn error::Error> {
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
            OutOfRangeReference { index } => write!(f, "Reference index {} is out-of-range", index),
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
impl PartialEq for DecodeError {
    fn eq(&self, other: &Self) -> bool {
        use self::DecodeError::*;
        match (self, other) {
            (&Unknown { marker: x }, &Unknown { marker: y }) => x == y,
            (&Unsupported { marker: x }, &Unsupported { marker: y }) => x == y,
            (&UnexpectedObjectEnd, &UnexpectedObjectEnd) => true,
            (&CircularReference { index: x }, &CircularReference { index: y }) => x == y,
            (&OutOfRangeReference { index: x }, &OutOfRangeReference { index: y }) => x == y,
            (&NonZeroTimeZone { offset: x }, &NonZeroTimeZone { offset: y }) => x == y,
            (&InvalidDate { millis: x }, &InvalidDate { millis: y }) => x == y,
            (&ExternalizableType { name: ref x }, &ExternalizableType { name: ref y }) => x == y,
            _ => false,
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
