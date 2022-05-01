//! Decimal error definitions.

use std::fmt;

/// An error which can be returned when parsing a decimal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecimalParseError {
    /// Empty string.
    Empty,
    /// Invalid decimal.
    Invalid,
    /// Decimal is overflowed.
    Overflow,
    /// Decimal is underflow.
    Underflow,
}

impl std::error::Error for DecimalParseError {}

impl fmt::Display for DecimalParseError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecimalParseError::Empty => f.write_str("cannot parse number from empty string"),
            DecimalParseError::Invalid => f.write_str("invalid number"),
            DecimalParseError::Overflow => f.write_str("numeric overflow"),
            DecimalParseError::Underflow => f.write_str("numeric underflow"),
        }
    }
}
