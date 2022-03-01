#[derive(Debug, PartialEq, Eq)]
/// The error type for decode operations.
pub struct DecodeError {
    pub(crate) kind: ErrorKind,
}

impl DecodeError {
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl std::error::Error for DecodeError {}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// A list specifying general decode error categories.
/// 
/// Used with [`DecodeError`] type.
pub enum ErrorKind {
    /// An unexpected byte was read.
    UnexpectedByte(u8),
    /// Returned when a decode operation could not be completed because an
    /// "end of buffer" was reached prematurely.
    UnexpectedEndOfBuffer,
    /// Encoded dictionary is unsorted.
    UnsortedDictionary,
    /// All dictionary keys must be byte strings.
    InvalidDictionaryKey,
    /// Integers with leading zeros are not allowed.
    LeadingZeros,
    /// Negative zero is not allowed.
    NegativeZero,
    /// Data not valid for the operation were encountered.
    InvalidData,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::UnexpectedByte(b) => write!(f, "unexpected byte `{}`", b),
            ErrorKind::UnexpectedEndOfBuffer => write!(f, "unexpected end of buffer"),
            ErrorKind::UnsortedDictionary => write!(f, "unsorted dictionary"),
            ErrorKind::InvalidDictionaryKey => write!(f, "invalid dictionary key"),
            ErrorKind::LeadingZeros => write!(f, "leading zeros"),
            ErrorKind::NegativeZero => write!(f, "negative zero"),
            ErrorKind::InvalidData => write!(f, "invalid data"),
        }
    }
}
