// SPDX-FileCopyrightText: 2023 Andrew Pantuso <ajpantuso@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub enum OctalFilter {
    Contains(u32),
    Equal(u32),
    NotContains(u32),
}

impl OctalFilter {
    pub fn matches(&self, o: u32) -> bool {
        match self {
            Self::Contains(m) => *m == (o & *m),
            Self::Equal(m) => *m == (o & 0o777),
            Self::NotContains(m) => 0 == (o & *m),
        }
    }
}

impl FromStr for OctalFilter {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if let Some(maybe_octal) = s.strip_prefix('+') {
            Self::Contains(u32::from_str_radix(maybe_octal, 8)?)
        } else if let Some(maybe_octal) = s.strip_prefix('~') {
            Self::NotContains(u32::from_str_radix(maybe_octal, 8)?)
        } else {
            Self::Equal(u32::from_str_radix(s, 8)?)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::OctalFilter;
    use crate::filter::testing::*;
    use anyhow::{anyhow, Result};
    use test_case::test_case;

    #[test_case("0777", Ok(OctalFilter::Equal(0o777)) ; "equal rwx all")]
    #[test_case("+0440", Ok(OctalFilter::Contains(0o440)) ; "contains r ug")]
    #[test_case("~0006", Ok(OctalFilter::NotContains(0o006)) ; "not contains rw o")]
    #[test_case("-0000", Err(anyhow!("")) ; "bad prefix")]
    #[test_case("0999", Err(anyhow!("")) ; "bad octal integer")]
    fn from_str(s: &str, expected: Result<OctalFilter>) {
        assert_from_str(s, expected)
    }

    #[test_case(OctalFilter::Equal(0o600), 0o600, true ; "equal matches")]
    #[test_case(OctalFilter::Equal(0o700), 0o600, false ; "equal does not match")]
    #[test_case(OctalFilter::Equal(0o600), 0o66600, true ; "equal extended perms matches")]
    #[test_case(OctalFilter::Contains(0o400), 0o600, true ; "contains matches")]
    #[test_case(OctalFilter::Contains(0o700), 0o600, false ; "contains does not match")]
    #[test_case(OctalFilter::NotContains(0o040), 0o600, true ; "not_contains matches")]
    #[test_case(OctalFilter::NotContains(0o700), 0o600, false ; "not_contains does not match")]
    fn matches(f: OctalFilter, o: u32, expected: bool) {
        assert_eq!(expected, f.matches(o))
    }
}
