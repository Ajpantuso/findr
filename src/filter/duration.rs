// SPDX-FileCopyrightText: 2023 Andrew Pantuso <ajpantuso@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0

use humantime::Duration;
use std::str::FromStr;
use std::time;

#[derive(Clone, Debug, PartialEq)]
pub enum DurationFilter {
    Less(time::Duration),
    Greater(time::Duration),
}

impl DurationFilter {
    pub fn matches(&self, instant: u64) -> anyhow::Result<bool> {
        let now = time::SystemTime::now().duration_since(time::UNIX_EPOCH)?;
        let instant = time::Duration::from_secs(instant);

        Ok(match self {
            Self::Less(d) => now - *d < instant,
            Self::Greater(d) => now - *d > instant,
        })
    }
}

impl FromStr for DurationFilter {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if let Some(maybe_dur) = s.strip_prefix('-') {
            Self::Less(maybe_dur.parse::<Duration>()?.into())
        } else if let Some(maybe_dur) = s.strip_prefix('+') {
            Self::Greater(maybe_dur.parse::<Duration>()?.into())
        } else {
            Self::Greater(s.parse::<Duration>()?.into())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::DurationFilter;
    use crate::filter::testing::*;
    use anyhow;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use test_case::test_case;

    const DAY: Duration = Duration::from_secs(24 * 60 * 60);

    #[test_case("1d", Ok(DurationFilter::Greater(DAY)) ; "greater than one day")]
    #[test_case("+1d", Ok(DurationFilter::Greater(DAY)) ; "greater than one day '+' prefix")]
    #[test_case("-1d", Ok(DurationFilter::Less(DAY)) ; "less than one day")]
    #[test_case("!1d", Err(anyhow::anyhow!("")) ; "invalid prefix")]
    #[test_case("3x", Err(anyhow::anyhow!("")) ; "invalid duration format")]
    fn from_str(s: &str, expected: anyhow::Result<DurationFilter>) {
        assert_from_str(s, expected)
    }
    #[test_case(DurationFilter::Greater(DAY), days_ago(2), Ok(true) ; "greater than one day")]
    #[test_case(DurationFilter::Less(DAY), days_ago(0), Ok(true) ; "less than one day")]
    #[test_case(DurationFilter::Greater(DAY), days_ago(0), Ok(false) ; "not greater than one day")]
    #[test_case(DurationFilter::Less(DAY), days_ago(2), Ok(false) ; "not less than one day")]
    fn matches(f: DurationFilter, instant: u64, expected: anyhow::Result<bool>) {
        let result = f.matches(instant);

        match expected {
            Ok(b) => assert_eq!(b, result.unwrap()),
            Err(_) => assert!(result.is_err()),
        }
    }
    fn days_ago(num: u32) -> u64 {
        SystemTime::now()
            .checked_sub(num * DAY)
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}
