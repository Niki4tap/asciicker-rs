use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

/// Error used to implement [`TryInto`] traits for packets.
#[derive(Debug, Clone)]
pub enum PacketParseError {
    /// This error variant is raised if length of the packet in bytes
    /// doesn't match the appropriate constant length.
    SizeMismatch(usize, usize),
    /// This error variant is raised if C-style string doesn't have terminating null byte
    NoNullByte(Vec<u8>),
}

impl Display for PacketParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PacketParseError::SizeMismatch(expected, got) => {
                write!(
                    f,
                    "Size mismatch during parsing: expected: {}, got: {}",
                    expected, got
                )
            }
            PacketParseError::NoNullByte(bytes) => {
                write!(
                    f,
                    "No null byte found while parsing following bytes: {:?}",
                    bytes
                )
            }
        }
    }
}

impl Error for PacketParseError {}

/// Generic runtime error for all of the high level computation of this library.
#[derive(Debug, Clone)]
pub struct RuntimeError {
    /// String describing what error had happened
    pub what: String,
}

impl RuntimeError {
    /// Create error from [`String`]
    pub fn from_string(s: String) -> Self {
        Self { what: s }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Runtime error: {}", self.what)
    }
}

impl Error for RuntimeError {}

/// Something like strlen from C
#[doc(hidden)]
pub(crate) fn first_nul(bytes: &[u8]) -> Option<usize> {
    let mut size = 0;
    for byte in bytes {
        if *byte == b'\0' {
            return Some(size);
        }
        size += 1;
    }
    None
}
