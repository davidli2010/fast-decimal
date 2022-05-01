//! Fast high precision decimal.

mod decimal;
mod error;
mod ops;
mod parse;

pub use crate::decimal::Decimal;
pub use crate::error::DecimalParseError;
