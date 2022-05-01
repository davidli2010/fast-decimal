//! Fast high precision decimal.

mod decimal;
mod error;
mod parse;

pub use crate::decimal::Decimal;
pub use crate::error::DecimalParseError;
