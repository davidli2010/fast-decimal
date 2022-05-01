//! Decimal implementation.

use std::fmt;

pub const MAX_PRECISION: u32 = 36;

// pub const NBASE: i32 = 10_0000_0000;
pub const DEC_DIGITS: i32 = 9;

pub const DEC_POS: u8 = 0x00;
pub const DEC_NEG: u8 = 0x80;
pub const DEC_NAN: u8 = 0x40;

#[derive(Debug)]
pub struct Decimal {
    sign: u8,
    weight: i8,
    dscale: i8,
    ndigits: u8,
    digits: [u32; 5],
}

impl Decimal {
    pub const ZERO: Decimal = unsafe { Decimal::from_raw_parts(DEC_POS, 0, 0, 0, [0; 5]) };
    pub const NAN: Decimal = unsafe { Decimal::from_raw_parts(DEC_NAN, 0, 0, 0, [0; 5]) };

    #[inline]
    pub(crate) const unsafe fn from_raw_parts(sign: u8, weight: i8, dscale: i8, ndigits: u8, digits: [u32; 5]) -> Self {
        debug_assert!(ndigits <= 5);
        Decimal {
            sign,
            weight,
            dscale,
            ndigits,
            digits,
        }
    }

    /// Checks if `self` is `NaN`.
    #[inline]
    pub const fn is_nan(&self) -> bool {
        self.sign == DEC_NAN
    }

    /// Checks if `self` is positive.
    #[inline]
    pub const fn is_sign_positive(&self) -> bool {
        self.sign == DEC_POS
    }

    /// Checks if `self` is negative.
    #[inline]
    pub const fn is_sign_negative(&self) -> bool {
        self.sign == DEC_NEG
    }

    /// Checks if `self` is zero.
    #[inline]
    pub const fn is_zero(&self) -> bool {
        self.ndigits == 0 && self.is_sign_positive()
    }

    #[inline]
    fn digits(&self) -> &[u32] {
        &self.digits[0..self.ndigits as usize]
    }

    /// Convert `self` to text representation.
    /// `self` is displayed to the number of digits indicated by its dscale.
    fn write<W: fmt::Write>(&self, f: &mut W) -> Result<(), fmt::Error> {
        if self.is_nan() {
            return f.write_str("NaN");
        }

        if self.is_zero() {
            return f.write_str("0");
        }

        // Output a dash for negative values.
        if self.sign == DEC_NEG {
            f.write_char('-')?;
        }

        // Output all digits before the decimal point.
        if self.weight < 0 {
            f.write_char('0')?;
        } else {
            let digits = self.digits();

            #[allow(clippy::needless_range_loop)]
            for d in 0..=self.weight as usize {
                let dig = if d < self.ndigits as usize { digits[d] } else { 0 };

                // In the first digit, suppress extra leading decimal zeroes.
                if d > 0 {
                    write!(f, "{:>0width$}", dig, width = DEC_DIGITS as usize)?;
                } else {
                    write!(f, "{}", dig)?;
                }
            }
        }

        // If requested, output a decimal point and all the digits that follow it.
        if self.dscale > 0 {
            f.write_char('.')?;

            let digits = self.digits();

            let mut d = self.weight as i32 + 1;

            for scale in (0..self.dscale as i32).step_by(DEC_DIGITS as usize) {
                let dig = if d >= 0 && d < self.ndigits as i32 {
                    digits[d as usize]
                } else {
                    0
                };

                if scale + DEC_DIGITS <= self.dscale as i32 {
                    write!(f, "{:>0width$}", dig, width = DEC_DIGITS as usize)?;
                } else {
                    // truncate the last digit
                    let width = (self.dscale as i32 - scale) as usize;
                    let dig = (0..DEC_DIGITS as usize - width).fold(dig, |acc, _| acc / 10);
                    write!(f, "{:>0width$}", dig, width = width)?;
                }

                d += 1;
            }
        }

        Ok(())
    }
}

impl fmt::Display for Decimal {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.write(f)
    }
}
