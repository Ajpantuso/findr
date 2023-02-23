// SPDX-FileCopyrightText: 2023 Andrew Pantuso <ajpantuso@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use parse_size::parse_size as _parse_size;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub enum SizeFilter {
    Less(u64),
    Greater(u64),
    Equal(u64),
}

impl SizeFilter {
    pub fn matches(&self, size: impl PartialOrd<u64>) -> bool {
        match self {
            Self::Equal(u) => size == *u,
            Self::Less(u) => size < *u,
            Self::Greater(u) => size > *u,
        }
    }
}

impl FromStr for SizeFilter {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if let Some(maybe_int) = s.strip_prefix('-') {
            Self::Less(parse_size(maybe_int)?)
        } else if let Some(maybe_int) = s.strip_prefix('+') {
            Self::Greater(parse_size(maybe_int)?)
        } else {
            Self::Equal(parse_size(s)?)
        })
    }
}

fn parse_size(s: &str) -> anyhow::Result<u64> {
    _parse_size(s).map_err(|e| anyhow::anyhow!(e))
}

#[cfg(test)]
mod test {
    use super::SizeFilter;
    use crate::filter::testing::*;
    use anyhow::{anyhow, Result};
    use test_case::test_case;

    #[test_case("1", Ok(SizeFilter::Equal(1)) ; "equals 1 byte")]
    #[test_case("+1", Ok(SizeFilter::Greater(1)) ; "greater than 1 byte")]
    #[test_case("-1", Ok(SizeFilter::Less(1)) ; "less than 1 byte")]
    #[test_case("1M", Ok(SizeFilter::Equal(1_000_000)) ; "equals 1 megabytes")]
    #[test_case("1MiB", Ok(SizeFilter::Equal(2u64.pow(20))) ; "equals 1 mebibytes")]
    #[test_case("3jb", Err(anyhow!("")) ; "invalid suffix")]
    fn from_str(s: &str, expected: Result<SizeFilter>) {
        assert_from_str(s, expected)
    }
    #[test_case(SizeFilter::Equal(1), 1, true ; "equals 1 byte")]
    #[test_case(SizeFilter::Less(1), 0, true ; "less than 1 byte")]
    #[test_case(SizeFilter::Greater(1), 10, true ; "greater than 1 byte")]
    #[test_case(SizeFilter::Equal(1), 2, false ; "not equals 1 byte")]
    #[test_case(SizeFilter::Less(1), 2, false ; "not less than 1 byte")]
    #[test_case(SizeFilter::Greater(1), 0, false ; "not greater than 1 byte")]
    fn matches(f: SizeFilter, size: u64, expected: bool) {
        assert_eq!(expected, f.matches(size))
    }
}
