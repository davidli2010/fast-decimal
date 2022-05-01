//! Decimal parsing utilities.

use crate::decimal::{DEC_DIGITS, DEC_NEG, DEC_POS, MAX_PRECISION};
use crate::{Decimal, DecimalParseError};
use stack_buf::StackVec;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
#[repr(u8)]
enum Sign {
    Positive = DEC_POS,
    Negative = DEC_NEG,
}

/// The interesting parts of a decimal string.
// #[derive(Debug)]
struct Parts<'a> {
    pub sign: Sign,
    pub integral: &'a [u8],
    pub fractional: &'a [u8],
    pub exp: i16,
}

/// Splits a decimal string bytes into sign and the rest, without inspecting or validating the rest.
#[inline]
fn extract_sign(s: &[u8]) -> (Sign, &[u8]) {
    match s.first() {
        Some(b'+') => (Sign::Positive, &s[1..]),
        Some(b'-') => (Sign::Negative, &s[1..]),
        _ => (Sign::Positive, s),
    }
}

/// Carves off decimal digits up to the first non-digit character.
#[inline]
fn eat_digits(s: &[u8]) -> (&[u8], &[u8]) {
    let i = s.iter().take_while(|&i| i.is_ascii_digit()).count();
    (&s[..i], &s[i..])
}

/// Carves off whitespaces up to the first non-whitespace character.
#[inline]
fn eat_whitespaces(s: &[u8]) -> &[u8] {
    let i = s.iter().take_while(|&i| i.is_ascii_whitespace()).count();
    &s[i..]
}

/// Extracts `NaN` value.
#[inline]
fn extract_nan(s: &[u8]) -> (bool, &[u8]) {
    if s.len() < 3 {
        (false, s)
    } else {
        let mut buf: [u8; 3] = s[0..3].try_into().unwrap();
        buf.make_ascii_lowercase();
        if &buf == b"nan" {
            (true, &s[3..])
        } else {
            (false, s)
        }
    }
}

/// Extracts exponent, if any.
fn extract_exponent(s: &[u8], decimal_is_zero: bool) -> Result<(i16, &[u8]), DecimalParseError> {
    let (sign, s) = extract_sign(s);
    let (mut number, s) = eat_digits(s);

    if number.is_empty() {
        return Err(DecimalParseError::Invalid);
    }

    if decimal_is_zero {
        return Ok((0, s));
    }

    while number.first() == Some(&b'0') {
        number = &number[1..];
    }

    if number.len() > 3 {
        return match sign {
            Sign::Positive => Err(DecimalParseError::Overflow),
            Sign::Negative => Err(DecimalParseError::Underflow),
        };
    }

    let exp = {
        let mut result: i16 = 0;
        for &n in number {
            result = result * 10 + (n - b'0') as i16;
        }
        match sign {
            Sign::Positive => result,
            Sign::Negative => -result,
        }
    };

    Ok((exp, s))
}

/// Checks if the input string is a valid decimal and if so, locate the integral
/// part, the fractional part, and the exponent in it.
fn parse_decimal(s: &[u8]) -> Result<(Parts, &[u8]), DecimalParseError> {
    let (sign, s) = extract_sign(s);

    if s.is_empty() {
        return Err(DecimalParseError::Invalid);
    }

    let (mut integral, s) = eat_digits(s);

    while integral.first() == Some(&b'0') && integral.len() > 1 {
        integral = &integral[1..];
    }

    let (fractional, exp, s) = match s.first() {
        Some(&b'e') | Some(&b'E') => {
            if integral.is_empty() {
                return Err(DecimalParseError::Invalid);
            }

            let decimal_is_zero = integral[0] == b'0';
            let (exp, s) = extract_exponent(&s[1..], decimal_is_zero)?;
            (&b""[..], exp, s)
        }
        Some(&b'.') => {
            let (mut fractional, s) = eat_digits(&s[1..]);
            if integral.is_empty() && fractional.is_empty() {
                return Err(DecimalParseError::Invalid);
            }

            while fractional.last() == Some(&b'0') {
                fractional = &fractional[0..fractional.len() - 1];
            }

            match s.first() {
                Some(&b'e') | Some(&b'E') => {
                    let decimal_is_zero = (integral.is_empty() || integral[0] == b'0') && fractional.is_empty();
                    let (exp, s) = extract_exponent(&s[1..], decimal_is_zero)?;
                    (fractional, exp, s)
                }
                _ => (fractional, 0, s),
            }
        }
        _ => {
            if integral.is_empty() {
                return Err(DecimalParseError::Invalid);
            }

            (&b""[..], 0, s)
        }
    };

    Ok((
        Parts {
            sign,
            integral,
            fractional,
            exp,
        },
        s,
    ))
}

/// Reads a decimal digit from `&[u8]`.
#[inline]
fn read_decimal_digit(s: &[u8]) -> u32 {
    debug_assert!(s.len() == DEC_DIGITS as usize);

    let mut digit = 0;

    for &i in s {
        digit = digit * 10 + i as u32;
    }

    digit
}

/// Parses a string bytes and put the number into this variable.
///
/// This function does not handle leading or trailing spaces, and it doesn't
/// accept `NaN` either. It returns the remaining string bytes so that caller can
/// check for trailing spaces/garbage if deemed necessary.
#[inline]
fn parse_str(s: &[u8]) -> Result<(Decimal, &[u8]), DecimalParseError> {
    let (
        Parts {
            sign,
            integral,
            fractional,
            exp,
        },
        s,
    ) = parse_decimal(s)?;

    if (integral.is_empty() || integral[0] == b'0') && fractional.is_empty() {
        return Ok((Decimal::ZERO, s));
    }

    if integral.len() + fractional.len() > MAX_PRECISION as usize {
        return Err(DecimalParseError::Overflow);
    }

    let dec_weight = integral.len() as i32 + exp as i32 - 1;
    let dec_scale = {
        let scale = fractional.len() as i32 - exp as i32;
        if scale < 0 {
            0
        } else {
            scale
        }
    };

    let weight = if dec_weight >= 0 {
        (dec_weight + 1 + DEC_DIGITS - 1) / DEC_DIGITS - 1
    } else {
        -((-dec_weight - 1) / DEC_DIGITS + 1)
    };

    let offset = (weight + 1) * DEC_DIGITS - (dec_weight + 1);
    let ndigits = (integral.len() as i32 + fractional.len() as i32 + offset + DEC_DIGITS - 1) / DEC_DIGITS;

    let mut dec_digits = StackVec::<u8, 64>::new();
    // leading padding for digit alignment later
    dec_digits.extend_from_slice([0; DEC_DIGITS as usize].as_ref());
    dec_digits.extend(integral.iter().map(|&i| i - b'0'));
    dec_digits.extend(fractional.iter().map(|&i| i - b'0'));
    // trailing padding for digit alignment later
    dec_digits.extend_from_slice([0; DEC_DIGITS as usize].as_ref());

    let mut digits = [0u32; 5];

    let iter = (&dec_digits[(DEC_DIGITS - offset) as usize..])
        .chunks_exact(DEC_DIGITS as usize)
        .take(ndigits as usize);
    for (i, chunk) in iter.enumerate() {
        let digit = read_decimal_digit(chunk);
        digits[i] = digit;
    }

    let dec = unsafe { Decimal::from_raw_parts(sign as u8, weight as i8, dec_scale as i8, ndigits as u8, digits) };
    Ok((dec, s))
}

/// Parses a string slice and creates a decimal.
///
/// This function handles leading or trailing spaces, and it
/// accepts `NaN` either.
#[inline]
fn from_str(s: &str) -> Result<Decimal, DecimalParseError> {
    let s = s.as_bytes();
    let s = eat_whitespaces(s);
    if s.is_empty() {
        return Err(DecimalParseError::Empty);
    }

    let (is_nan, s) = extract_nan(s);

    if is_nan {
        if s.iter().any(|n| !n.is_ascii_whitespace()) {
            return Err(DecimalParseError::Invalid);
        }

        Ok(Decimal::NAN)
    } else {
        let (n, s) = parse_str(s)?;

        if s.iter().any(|n| !n.is_ascii_whitespace()) {
            return Err(DecimalParseError::Invalid);
        }

        Ok(n)
    }
}

impl FromStr for Decimal {
    type Err = DecimalParseError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_parse_empty<S: AsRef<str>>(s: S) {
        let result = s.as_ref().parse::<Decimal>();
        assert_eq!(result.unwrap_err(), DecimalParseError::Empty);
    }

    fn assert_parse_invalid<S: AsRef<str>>(s: S) {
        let result = s.as_ref().parse::<Decimal>();
        assert_eq!(result.unwrap_err(), DecimalParseError::Invalid);
    }

    fn assert_parse_overflow<S: AsRef<str>>(s: S) {
        let result = s.as_ref().parse::<Decimal>();
        assert_eq!(result.unwrap_err(), DecimalParseError::Overflow);
    }

    fn assert_parse_underflow<S: AsRef<str>>(s: S) {
        let result = s.as_ref().parse::<Decimal>();
        assert_eq!(result.unwrap_err(), DecimalParseError::Underflow);
    }

    #[test]
    fn parse_error() {
        assert_parse_empty("");
        assert_parse_empty("   ");
        assert_parse_invalid("-");
        assert_parse_invalid("   -   ");
        assert_parse_invalid("-.");
        assert_parse_invalid("- 1");
        assert_parse_invalid("-NaN");
        assert_parse_invalid("NaN.");
        assert_parse_invalid("NaN1");
        assert_parse_invalid("   NaN   .   ");
        assert_parse_invalid("   NaN   1   ");
        assert_parse_invalid(".");
        assert_parse_invalid("   .   ");
        assert_parse_invalid("e");
        assert_parse_invalid("   e   ");
        assert_parse_invalid("-e");
        assert_parse_invalid("-1e");
        assert_parse_invalid("1e1.1");
        assert_parse_invalid("-1 e1");
        assert_parse_invalid("   x   ");
        assert_parse_overflow("1e10000000000");
        assert_parse_overflow("1e2147483648");
        assert_parse_underflow("1e-2147483648");
    }

    fn assert_parse<S: AsRef<str>, V: AsRef<str>>(s: S, expected: V) {
        let decimal = s.as_ref().parse::<Decimal>().unwrap();
        println!("{}: {:?}", decimal, decimal);
        assert_eq!(decimal.to_string(), expected.as_ref());
    }

    #[test]
    fn parse_valid() {
        // NaN
        assert_parse("NaN", "NaN");
        assert_parse("Nan", "NaN");
        assert_parse("NAN", "NaN");
        assert_parse("NAn", "NaN");
        assert_parse("naN", "NaN");
        assert_parse("nan", "NaN");
        assert_parse("nAN", "NaN");
        assert_parse("nAn", "NaN");
        assert_parse("   NaN   ", "NaN");

        // Integer
        assert_parse("0", "0");
        assert_parse("-0", "0");
        assert_parse("   -0   ", "0");
        assert_parse("00000.", "0");
        assert_parse("-00000.", "0");
        assert_parse("128", "128");
        assert_parse("-128", "-128");
        assert_parse("65536", "65536");
        assert_parse("-65536", "-65536");
        assert_parse("4294967296", "4294967296");
        assert_parse("-4294967296", "-4294967296");
        assert_parse("18446744073709551616", "18446744073709551616");
        assert_parse("-18446744073709551616", "-18446744073709551616");
        // assert_parse(
        //     "340282366920938463463374607431768211456",
        //     "340282366920938463463374607431768211456",
        // );
        // assert_parse(
        //     "-340282366920938463463374607431768211456",
        //     "-340282366920938463463374607431768211456",
        // );
        assert_parse("000000000123", "123");
        assert_parse("-000000000123", "-123");

        // Floating-point number
        assert_parse("0.0", "0");
        assert_parse("-0.0", "0");
        assert_parse("   -0.0   ", "0");
        assert_parse(".0", "0");
        assert_parse(".00000", "0");
        assert_parse("-.0", "0");
        assert_parse("-.00000", "0");
        assert_parse("128.128", "128.128");
        assert_parse("-128.128", "-128.128");
        assert_parse("65536.65536", "65536.65536");
        assert_parse("-65536.65536", "-65536.65536");
        assert_parse("4294967296.4294967296", "4294967296.4294967296");
        assert_parse("-4294967296.4294967296", "-4294967296.4294967296");
        // assert_parse(
        //     "18446744073709551616.18446744073709551616",
        //     "18446744073709551616.18446744073709551616",
        // );
        // assert_parse(
        //     "-18446744073709551616.18446744073709551616",
        //     "-18446744073709551616.18446744073709551616",
        // );
        // assert_parse(
        //     "340282366920938463463374607431768211456.340282366920938463463374607431768211456",
        //     "340282366920938463463374607431768211456.340282366920938463463374607431768211456",
        // );
        // assert_parse(
        //     "-340282366920938463463374607431768211456.340282366920938463463374607431768211456",
        //     "-340282366920938463463374607431768211456.340282366920938463463374607431768211456",
        // );
        assert_parse("000000000123.000000000123", "123.000000000123");
        assert_parse("-000000000123.000000000123", "-123.000000000123");

        // Scientific notation
        assert_parse("0e0", "0");
        assert_parse("-0E-0", "0");
        assert_parse("0000000000E0000000000", "0");
        assert_parse("-0000000000E-0000000000", "0");
        assert_parse("00000000001e0000000000", "1");
        assert_parse("-00000000001e-0000000000", "-1");
        assert_parse("00000000001e00000000001", "10");
        assert_parse("-00000000001e-00000000001", "-0.1");
        assert_parse("1e10", "10000000000");
        assert_parse("-1e-10", "-0.0000000001");
        assert_parse("0000001.23456000e3", "1234.56");
        assert_parse("-0000001.23456000E-3", "-0.00123456");
    }
}
