//! Implementing operators for decimal.

use crate::decimal::Decimal;
use std::cmp::Ordering;

impl PartialEq for Decimal {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.cmp_common(other) == Ordering::Equal
    }
}

impl Eq for Decimal {}

impl Ord for Decimal {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_common(other)
    }
}

impl PartialOrd for Decimal {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp_common(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmp() {
        macro_rules! assert_cmp {
            ($left: expr, $cmp: tt, $right: expr) => {{
                let l = $left.parse::<Decimal>().unwrap();
                let r = $right.parse::<Decimal>().unwrap();
                assert!(l $cmp r, "{} {} {}", l, stringify!($cmp),r);
            }};
        }

        assert_cmp!("0", ==, "0");

        assert_cmp!("-1", <, "1");
        assert_cmp!("1", >, "-1");

        assert_cmp!("1.1", ==, "1.1");
        assert_cmp!("1.2", >, "1.1");
        assert_cmp!("-1.2", <, "1.1");
        assert_cmp!("1.1", >, "-1.2");

        assert_cmp!("1", <, "1e39");
        assert_cmp!("1", >, "1e-39");
        assert_cmp!("1.0e-100", >=, "1.0e-101");
        assert_cmp!("1.0e-101", <=, "1.0e-100");
        assert_cmp!("1.0e-100", !=, "1.0e-101");

        assert_cmp!("1.12", <, "1.2");
        assert_cmp!("1.2", >, "1.12");
        assert_cmp!("-1.2", <, "-1.12");
        assert_cmp!("-1.12", >, "-1.2");
        assert_cmp!("-1.12", <, "1.2");
        assert_cmp!("1.12", >, "-1.2");

        assert_cmp!("0.000000001", <,"100000000");
        assert_cmp!("100000000", >, "0.000000001");

        assert_cmp!("123456789.987654321", ==, "123456789.987654321");
        assert_cmp!("987654321.123456789", ==, "987654321.123456789");
        assert_cmp!("123456789.987654321", <, "987654321.123456789");
        assert_cmp!("987654321.123456789", >, "123456789.987654321");

        assert_cmp!(
            "99999999999999999999999999999999999.9", >, "9.99999999999999999999999999999999999"
        );
        assert_cmp!(
            "9.99999999999999999999999999999999999", >, "0"
        );
        assert_cmp!(
            "9.99999999999999999999999999999999999", >, "1"
        );
        assert_cmp!(
            "-99999999999999999999999999999999999.9", <, "-9.99999999999999999999999999999999999"
        );
        assert_cmp!(
            "-9.99999999999999999999999999999999999", <, "0"
        );
        assert_cmp!(
            "-9.99999999999999999999999999999999999", <, "1"
        );
        assert_cmp!("4703178999618078116505370421100e36", >, "0");
        assert_cmp!("4703178999618078116505370421100e-36", >, "0");
        assert_cmp!("-4703178999618078116505370421100e36", <, "0");
        assert_cmp!("-4703178999618078116505370421100e-36", <, "0");
        assert_cmp!("0", <, "4703178999618078116505370421100e36");
        assert_cmp!("0", <, "4703178999618078116505370421100e-36");
        assert_cmp!("0", >, "-4703178999618078116505370421100e36");
        assert_cmp!("0", >, "-4703178999618078116505370421100e-36");
    }
}
